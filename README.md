# attohttpc
[Documentation](https://docs.rs/attohttpc) | [Crates.io](https://crates.io/crates/attohttpc) | [Repository](https://github.com/sbstp/attohttpc)

## Why attohttpc?
This project's goal is to provide a lightweight and simple HTTP client for the Rust ecosystem. The intended use is for
projects that have HTTP needs where performance is not critical or when HTTP is not the main purpose of the application.
Note that the project still tries to perform well and avoid allocation where possible, but stays away from Rust's
asynchronous stack to provide a crate that's as small as possible. Features are provided behind feature flags when
possible to allow users to get just what they need. Here are the goals of the project:

* Lightweight
* Secure
* Easy to use
* Modular
* HTTP/1.1, eventually HTTP/2.0
* Use quality crates from the ecosystem (`http`, `url`, `encoding_rs`), not reinventing the wheel.

## Features
* `charsets` support for decoding more text encodings than just UTF-8
* `compress` support for decompressing response bodies (**default**)
* `json` support for serialization and deserialization
* `tls` support for tls connections (**default**)

## Usage
See the `examples/` folder in the repository for more use cases.
```rust
let (status, headers, body) = attohttpc::post("https://my-api.com/do/something").json(&request)?.send()?;
if status.is_success() {
    let response = body.json()?;
    // ...
}
```

## Current feature set
* Query parameters
* Request headers
* Tls
* Automatic redirection
* Streaming response body
* Text encoding support
* Gzip, deflate support
* Transfer-Encoding: chunked
* `serde` support behind a feature flag

## Features being worked on
* File upload, form data
* Thorough test suite
* Connection: keep-alive
* Authentication

## License
This project is licensed under the `MPL-2.0`.
