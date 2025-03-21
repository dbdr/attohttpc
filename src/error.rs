use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io;
use std::result;

/// Errors than can occur while parsing the response from the server.
#[derive(Debug)]
pub enum InvalidResponseKind {
    /// Invalid or missing Location header in redirection
    LocationHeader,
    /// Invalid redirection URL
    RedirectionUrl,
    /// Status line
    StatusLine,
    /// Status code
    StatusCode,
    /// Error parsing header
    Header,
    /// Error decoding chunk size
    ChunkSize,
    /// Error decoding chunk
    Chunk,
    /// Invalid Content-Length header
    ContentLength,
}

impl Display for InvalidResponseKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InvalidResponseKind::*;

        match self {
            LocationHeader => write!(f, "missing or invalid location header"),
            RedirectionUrl => write!(f, "invalid redirection url"),
            StatusLine => write!(f, "invalid status line"),
            StatusCode => write!(f, "invalid status code"),
            Header => write!(f, "invalid header"),
            ChunkSize => write!(f, "invalid chunk size"),
            Chunk => write!(f, "invalid chunk"),
            ContentLength => write!(f, "invalid content length"),
        }
    }
}

/// Common errors that can occur during HTTP requests.
#[derive(Debug)]
pub enum ErrorKind {
    /// CONNECT is not supported.
    ConnectNotSupported,
    /// Error generated by the `http` crate.
    Http(http::Error),
    /// IO Error
    Io(io::Error),
    /// Invalid base URL given to the Request.
    InvalidBaseUrl,
    /// An URL with an invalid host was found while processing the request.
    InvalidUrlHost,
    /// The URL scheme is unknown and the port is missing.
    InvalidUrlPort,
    /// Server sent an invalid response.
    InvalidResponse(InvalidResponseKind),
    /// Too many redirections
    TooManyRedirections,
    /// JSON decoding/encoding error.
    #[cfg(feature = "json")]
    Json(serde_json::Error),
    /// TLS error encountered while connecting to an https server.
    #[cfg(feature = "tls")]
    Tls(native_tls::Error),
}

/// A type that contains all the errors that can possibly occur while accessing an HTTP server.
#[derive(Debug)]
pub struct Error(Box<ErrorKind>);

impl Error {
    /// Get a reference to the `ErrorKind` inside.
    pub fn kind(&self) -> &ErrorKind {
        &self.0
    }

    /// Comsume this `Error` and get the `ErrorKind` inside.
    pub fn into_kind(self) -> ErrorKind {
        *self.0
    }
}

impl Display for Error {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        use ErrorKind::*;

        match *self.0 {
            ConnectNotSupported => write!(w, "CONNECT is not supported"),
            Http(ref e) => write!(w, "Http Error: {}", e),
            Io(ref e) => write!(w, "Io Error: {}", e),
            InvalidBaseUrl => write!(w, "Invalid base URL"),
            InvalidUrlHost => write!(w, "URL is missing a host"),
            InvalidUrlPort => write!(w, "URL is missing a port"),
            InvalidResponse(ref k) => write!(w, "InvalidResponse: {}", k),
            TooManyRedirections => write!(w, "Too many redirections"),
            #[cfg(feature = "json")]
            Json(ref e) => write!(w, "Json Error: {}", e),
            #[cfg(feature = "tls")]
            Tls(ref e) => write!(w, "Tls Error: {}", e),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        use ErrorKind::*;

        match *self.0 {
            ConnectNotSupported => "CONNECT is not supported",
            Http(ref e) => e.description(),
            Io(ref e) => e.description(),
            InvalidBaseUrl => "invalid base url",
            InvalidUrlHost => "url has no host",
            InvalidUrlPort => "url has no port",
            InvalidResponse(_) => "invalid response",
            TooManyRedirections => "too many redirections",
            #[cfg(feature = "json")]
            Json(ref e) => e.description(),
            #[cfg(feature = "tls")]
            Tls(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        use ErrorKind::*;

        match *self.0 {
            Io(ref e) => Some(e),
            Http(ref e) => Some(e),
            #[cfg(feature = "json")]
            Json(ref e) => Some(e),
            #[cfg(feature = "tls")]
            Tls(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error(Box::new(ErrorKind::Io(err)))
    }
}

impl From<http::Error> for Error {
    fn from(err: http::Error) -> Error {
        Error(Box::new(ErrorKind::Http(err)))
    }
}

#[cfg(feature = "tls")]
impl From<native_tls::Error> for Error {
    fn from(err: native_tls::Error) -> Error {
        Error(Box::new(ErrorKind::Tls(err)))
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error(Box::new(ErrorKind::Json(err)))
    }
}

impl From<ErrorKind> for Error {
    fn from(err: ErrorKind) -> Error {
        Error(Box::new(err))
    }
}

impl From<InvalidResponseKind> for Error {
    fn from(kind: InvalidResponseKind) -> Error {
        ErrorKind::InvalidResponse(kind).into()
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err)
    }
}

impl From<InvalidResponseKind> for io::Error {
    fn from(kind: InvalidResponseKind) -> io::Error {
        io::Error::new(io::ErrorKind::Other, Error(Box::new(ErrorKind::InvalidResponse(kind))))
    }
}

/// Wrapper for the `Result` type with an `Error`.
pub type Result<T = ()> = result::Result<T, Error>;
