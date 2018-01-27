# may_http

[![Travis Build Status](https://travis-ci.org/rust-may/may_http.svg?branch=master)](https://travis-ci.org/rust-may/may_http)
<!-- refs
[![crates.io](http://meritbadge.herokuapp.com/may_http)](https://crates.io/crates/may_http)
-->

Coroutine based HTTP library for Rust.

(**Don't Use, Under Development**)

<!-- refs

### Documentation

- [Docs](http://rust-may.github.io/may_http)

## Overview

may_http is a fast, coroutine based HTTP implementation written in and for Rust.

may_http offers both an HTTP client and server which can be used in [may](https://github.com/Xudong-Huang/may) coroutine context.
## Example

### Hello World Server:

```rust
extern crate may_http;

use may_http::Server;
use may_http::server::Request;
use may_http::server::Response;

fn hello(_: Request, res: Response) {
    res.send(b"Hello World!").unwrap();
}

fn main() {
    Server::http("127.0.0.1:3000").unwrap()
        .handle(hello).unwrap();
}
```

### Client:

```rust
extern crate may_http;

use std::io::Read;

use may_http::Client;
use may_http::header::Connection;

fn main() {
    // Create a client.
    let client = Client::new();

    // Creating an outgoing request.
    let mut res = client.get("http://rust-lang.org/")
        // set a header
        .header(Connection::close())
        // let 'er go!
        .send().unwrap();

    // Read the Response.
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("Response: {}", body);
}
```
-->

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
