extern crate env_logger;
extern crate may_http;

use may_http::server::*;

fn hello(_req: Request, rsp: Response) {
    rsp.send(b"hello world").unwrap();
}

fn main() {
    env_logger::init().unwrap();
    let server = HttpServer(hello).start("127.0.0.1:8080").unwrap();
    server.wait();
}
