extern crate may_http;

use std::io::Write;

use may_http::server::*;


fn hello(_req: Request, mut rsp: Response) {
    rsp.write_all(b"hello world").unwrap();
}

fn main() {
    let server = HttpServer(hello).start("127.0.0.1:8080").unwrap();
    server.wait();
}
