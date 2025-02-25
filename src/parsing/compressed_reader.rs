#[cfg(feature = "compress")]
use std::io::BufReader;
use std::io::{self, Read};

use http::header::HeaderMap;
#[cfg(feature = "compress")]
use http::header::{CONTENT_ENCODING, TRANSFER_ENCODING};
#[cfg(feature = "compress")]
use http::Method;
#[cfg(feature = "compress")]
use libflate::{deflate, gzip};

use crate::error::Result;
use crate::parsing::body_reader::BodyReader;
use crate::request::PreparedRequest;

pub enum CompressedReader {
    Plain(BodyReader),
    #[cfg(feature = "compress")]
    // The BodyReader needs to be wrapped in a BufReader because libflate reads one byte at a time.
    Deflate(deflate::Decoder<BufReader<BodyReader>>),
    #[cfg(feature = "compress")]
    // The BodyReader needs to be wrapped in a BufReader because libflate reads one byte at a time.
    Gzip(gzip::Decoder<BufReader<BodyReader>>),
}

#[cfg(feature = "compress")]
fn have_encoding_item(value: &str, enc: &str) -> bool {
    value.split(",").map(|s| s.trim()).any(|s| s.eq_ignore_ascii_case(enc))
}

#[cfg(feature = "compress")]
fn have_encoding_content_encoding(headers: &HeaderMap, enc: &str) -> bool {
    headers
        .get_all(CONTENT_ENCODING)
        .into_iter()
        .filter_map(|val| val.to_str().ok())
        .any(|val| have_encoding_item(val, enc))
}

#[cfg(feature = "compress")]
fn have_encoding_transfer_encoding(headers: &HeaderMap, enc: &str) -> bool {
    headers
        .get_all(TRANSFER_ENCODING)
        .into_iter()
        .filter_map(|val| val.to_str().ok())
        .any(|val| have_encoding_item(val, enc))
}

#[cfg(feature = "compress")]
fn have_encoding(headers: &HeaderMap, enc: &str) -> bool {
    have_encoding_content_encoding(headers, enc) || have_encoding_transfer_encoding(headers, enc)
}

impl CompressedReader {
    #[cfg(feature = "compress")]
    pub fn new(headers: &HeaderMap, request: &PreparedRequest, reader: BodyReader) -> Result<CompressedReader> {
        if request.method() != Method::HEAD {
            if have_encoding(headers, "gzip") {
                // There's an issue when a Content-Encoding of Transfer-Encoding header are present and the body
                // is empty, because the gzip decoder tries to read the header eagerly.
                debug!("creating gzip decoder");
                return Ok(CompressedReader::Gzip(gzip::Decoder::new(BufReader::new(reader))?));
            }

            if have_encoding(headers, "deflate") {
                debug!("creating deflate decoder");
                return Ok(CompressedReader::Deflate(deflate::Decoder::new(BufReader::new(reader))));
            }
        }
        debug!("creating plain reader");
        return Ok(CompressedReader::Plain(reader));
    }

    #[cfg(not(feature = "compress"))]
    pub fn new(_: &HeaderMap, _: &PreparedRequest, reader: BodyReader) -> Result<CompressedReader> {
        Ok(CompressedReader::Plain(reader))
    }
}

impl Read for CompressedReader {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // TODO: gzip does not read until EOF, leaving some data in the buffer.
        match self {
            CompressedReader::Plain(s) => s.read(buf),
            #[cfg(feature = "compress")]
            CompressedReader::Deflate(s) => s.read(buf),
            #[cfg(feature = "compress")]
            CompressedReader::Gzip(s) => s.read(buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;

    #[cfg(feature = "compress")]
    use http::header::{HeaderMap, HeaderValue};
    use http::Method;
    #[cfg(feature = "compress")]
    use libflate::{deflate, gzip};

    #[cfg(feature = "compress")]
    use super::have_encoding;
    use crate::parsing::response::parse_response;
    use crate::streams::BaseStream;
    use crate::PreparedRequest;

    #[test]
    #[cfg(feature = "compress")]
    fn test_have_encoding_none() {
        let mut headers = HeaderMap::new();
        headers.insert("content-encoding", HeaderValue::from_static("gzip"));
        assert!(!have_encoding(&headers, "deflate"));
    }

    #[test]
    #[cfg(feature = "compress")]
    fn test_have_encoding_content_encoding_simple() {
        let mut headers = HeaderMap::new();
        headers.insert("content-encoding", HeaderValue::from_static("gzip"));
        assert!(have_encoding(&headers, "gzip"));
    }

    #[test]
    #[cfg(feature = "compress")]
    fn test_have_encoding_content_encoding_multi() {
        let mut headers = HeaderMap::new();
        headers.insert("content-encoding", HeaderValue::from_static("identity, deflate"));
        assert!(have_encoding(&headers, "deflate"));
    }

    #[test]
    #[cfg(feature = "compress")]
    fn test_have_encoding_transfer_encoding_simple() {
        let mut headers = HeaderMap::new();
        headers.insert("transfer-encoding", HeaderValue::from_static("deflate"));
        assert!(have_encoding(&headers, "deflate"));
    }

    #[test]
    #[cfg(feature = "compress")]
    fn test_have_encoding_transfer_encoding_multi() {
        let mut headers = HeaderMap::new();
        headers.insert("transfer-encoding", HeaderValue::from_static("gzip, chunked"));
        assert!(have_encoding(&headers, "gzip"));
    }

    #[test]
    fn test_stream_plain() {
        let payload = b"Hello world!!!!!!!!";

        let mut buf: Vec<u8> = Vec::new();
        let _ = write!(buf, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", payload.len());
        buf.extend(payload);

        let req = PreparedRequest::new(Method::GET, "http://google.ca");

        let sock = BaseStream::mock(buf);
        let response = parse_response(sock, &req).unwrap();
        assert_eq!(response.text().unwrap(), "Hello world!!!!!!!!");
    }

    #[test]
    #[cfg(feature = "compress")]
    fn test_stream_deflate() {
        let mut payload = Vec::new();
        let mut enc = deflate::Encoder::new(&mut payload);
        enc.write_all(b"Hello world!!!!!!!!").unwrap();
        enc.finish();

        let mut buf: Vec<u8> = Vec::new();
        let _ = write!(
            buf,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Encoding: deflate\r\n\r\n",
            payload.len()
        );
        buf.extend(payload);

        let req = PreparedRequest::new(Method::GET, "http://google.ca");

        let sock = BaseStream::mock(buf);
        let response = parse_response(sock, &req).unwrap();
        assert_eq!(response.text().unwrap(), "Hello world!!!!!!!!");
    }

    #[test]
    #[cfg(feature = "compress")]
    fn test_stream_gzip() {
        let mut payload = Vec::new();
        let mut enc = gzip::Encoder::new(&mut payload).unwrap();
        enc.write_all(b"Hello world!!!!!!!!").unwrap();
        enc.finish();

        let mut buf: Vec<u8> = Vec::new();
        let _ = write!(
            buf,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Encoding: gzip\r\n\r\n",
            payload.len()
        );
        buf.extend(payload);

        let req = PreparedRequest::new(Method::GET, "http://google.ca");

        let sock = BaseStream::mock(buf);
        let response = parse_response(sock, &req).unwrap();

        assert_eq!(response.text().unwrap(), "Hello world!!!!!!!!");
    }

    #[test]
    #[cfg(feature = "compress")]
    fn test_no_body_with_gzip() {
        let buf = b"HTTP/1.1 200 OK\r\ncontent-encoding: gzip\r\n\r\n";

        let req = PreparedRequest::new(Method::GET, "http://google.ca");
        let sock = BaseStream::mock(buf.to_vec());
        assert!(parse_response(sock, &req).is_err());
    }

    #[test]
    #[cfg(feature = "compress")]
    fn test_no_body_with_gzip_head() {
        let buf = b"HTTP/1.1 200 OK\r\ncontent-encoding: gzip\r\n\r\n";

        let req = PreparedRequest::new(Method::HEAD, "http://google.ca");
        let sock = BaseStream::mock(buf.to_vec());
        assert!(parse_response(sock, &req).is_ok());
    }
}
