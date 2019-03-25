use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use attohttpc::ErrorKind;
use lazy_static::lazy_static;
use rouille::Response;

lazy_static! {
    static ref STARTED: bool = {
        thread::spawn(move || {
            rouille::start_server("localhost:55123", move |_| Response::redirect_301("/"));
        });

        // Wait until server is ready
        while TcpStream::connect(("localhost", 55123)).is_err() {
            thread::sleep(Duration::from_millis(100));
        }

        true
    };
}

#[test]
fn test_redirection_default() {
    let _ = *STARTED;

    match attohttpc::get("http://localhost:55123/").send() {
        Err(err) => match err.kind() {
            ErrorKind::TooManyRedirections => (),
            _ => panic!(),
        },
        _ => panic!(),
    }
}

#[test]
fn test_redirection_0() {
    let _ = *STARTED;

    match attohttpc::get("http://localhost:55123/").max_redirections(0).send() {
        Err(err) => match err.kind() {
            ErrorKind::TooManyRedirections => (),
            _ => panic!(),
        },
        _ => panic!(),
    }
}

#[test]
fn test_redirection_disallowed() {
    let _ = *STARTED;

    let resp = attohttpc::get("http://localhost:55123/")
        .follow_redirects(false)
        .send()
        .unwrap();

    assert!(resp.status().is_redirection());
}
