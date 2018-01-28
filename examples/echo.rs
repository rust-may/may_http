extern crate env_logger;
extern crate may;
extern crate may_http;

use std::io::{Read, Write};
use may_http::server::*;

// test with: curl -v POST -d "asdfasdfasf" "http://127.0.0.1:8080/"
// test with: curl -v POST  --header "Transfer-Encoding: chunked" -d "hello chunk" "http://127.0.0.1:8080/"
fn hello(mut req: Request, mut rsp: Response) {
    let mut s = String::new();
    req.read_to_string(&mut s).unwrap();
    write!(rsp, "got data: {}", s).unwrap();
}

fn main() {
    may::config().set_io_workers(1);
    env_logger::init().unwrap();
    let server = HttpServer(hello).start("127.0.0.1:8080").unwrap();
    server.wait();
    std::thread::sleep(std::time::Duration::from_secs(10));
}
