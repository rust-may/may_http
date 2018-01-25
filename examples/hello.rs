extern crate env_logger;
extern crate may;
extern crate may_http;

use may_http::server::*;
use std::io::Write;

fn hello<T: Write>(_req: Request, rsp: Response<T>) {
    rsp.send(b"hello, world!").unwrap();
}

fn main() {
    may::config().set_io_workers(1);
    env_logger::init().unwrap();
    let server = HttpServer(hello).start("127.0.0.1:8080").unwrap();
    server.wait();
    std::thread::sleep(std::time::Duration::from_secs(10));
}
