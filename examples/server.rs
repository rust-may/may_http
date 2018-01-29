extern crate env_logger;
extern crate http;
extern crate may;
extern crate may_http;

use std::time::Duration;

use http::header::*;
use may_http::server::*;

fn hello(_req: Request, rsp: &mut Response) {
    rsp.headers_mut()
        .append(CONTENT_TYPE, "text/plain; charset=utf-8".parse().unwrap());
    rsp.send(b"Hello World!").unwrap();
}

fn main() {
    may::config().set_io_workers(1);
    env_logger::init().unwrap();
    // config the timeout would hurt the performance here
    let mut server = HttpServer::new(hello);
    server
        .set_read_timeout(Some(Duration::from_secs(10)))
        .set_write_timeout(Some(Duration::from_secs(10)));

    let server = server.start("127.0.0.1:8080").unwrap();
    server.wait();
}
