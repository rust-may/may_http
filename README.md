# may_http

[![Travis Build Status](https://travis-ci.org/rust-may/may_http.svg?branch=master)](https://travis-ci.org/rust-may/may_http)
<!-- refs
[![crates.io](http://meritbadge.herokuapp.com/may_http)](https://crates.io/crates/may_http)
-->

Coroutine based HTTP library for Rust.


<!-- refs

### Documentation

- [Docs](http://rust-may.github.io/may_http)
-->

## Overview

may_http is a fast, coroutine based HTTP implementation that can be used as lower level layer for http servers and clients.

Some of the implementation logic comes from [hyper](https://github.com/hyperium/hyper) 0.10.x branch

But most of the logic are re-written for ergonomical usage and performance consideration.

Thanks to the [httparse](https://github.com/seanmonstar/httparse) and [http](https://github.com/hyperium/http) crates, they make may_http only focus on the transportation logic.

## Example

### Hello World Server:

```rust
use http::header::*;
use may_http::server::*;

fn hello(_req: Request, rsp: &mut Response) {
    rsp.headers_mut()
        .append(CONTENT_TYPE, "text/plain; charset=utf-8".parse().unwrap());
    rsp.send(b"Hello World!").unwrap();
}

fn main() {
    let server = HttpServer::new(hello).start("127.0.0.1:8080").unwrap();
    server.wait();
}
```

### Simple Client
for a more complicated client example, you can ref [wrk-rs](https://github.com/Xudong-Huang/wrk-rs)
```rust
use http::Uri;
use std::io::{self, Read};
use may_http::client::*;

fn client_get(uri: Uri) -> io::Result<()> {
    let mut client = {
        let host = uri.host().unwrap_or("127.0.0.1");
        let port = uri.port().unwrap_or(80);
        HttpClient::connect((host, port))?
    };

    let mut s = String::new();
    for _ in 0..100 {
        let uri = uri.clone();
        let mut rsp = client.get(uri)?;
        rsp.read_to_string(&mut s)?;
        println!("get rsp={}", s);
        s.clear();
    }
    Ok(())
}

fn main() {
    let uri: Uri = "http://127.0.0.1:8080/".parse().unwrap();
    client_get(uri).unwrap();
}
```

## Performance

The data only benched on one thread hello server, compared with hyper master branch which is power by tokio and future.
### hyper
```sh
$ wrk http://127.0.0.1:3000 -d 10 -t 1 -c 200     
Running 10s test @ http://127.0.0.1:3000
  1 threads and 200 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.95ms  272.35us   8.49ms   94.67%
    Req/Sec    68.13k     1.99k   70.36k    87.00%
  677836 requests in 10.05s, 83.39MB read
Requests/sec:  67466.95
Transfer/sec:      8.30MB
wrk http://127.0.0.1:3000 -d 10 -t 1 -c 200  2.79s user 5.97s system 87% cpu 10.051 total
```

### may_http
```sh
$ wrk http://127.0.0.1:8080 -d 10 -t 1 -c 200
Running 10s test @ http://127.0.0.1:8080
  1 threads and 200 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     2.45ms  330.62us  10.91ms   96.82%
    Req/Sec    81.79k     2.83k   84.95k    81.00%
  814731 requests in 10.03s, 111.11MB read
Requests/sec:  81253.47
Transfer/sec:     11.08MB
wrk http://127.0.0.1:8080 -d 10 -t 1 -c 200  2.75s user 6.89s system 96% cpu 10.030 total
```

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
