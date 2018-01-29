extern crate env_logger;
extern crate may;
extern crate may_http;

use std::io::Write;
use may_http::server::*;

fn server(req: Request, rsp: &mut Response) {
    println!("req = {:?}", req);
    println!("uri = {}", req.uri());
    println!("method = {}", req.method());
    println!("version = {:?}", req.version());
    let headers = req.headers();

    for (name, value) in headers {
        println!("{:?}: {:?}", name, value);
    }

    let msg = "this is simple server";
    rsp.set_content_length(msg.len());
    rsp.write_all(msg.as_bytes()).unwrap();
}

fn main() {
    may::config().set_io_workers(1);
    env_logger::init().unwrap();
    let server = HttpServer(server).start("127.0.0.1:8080").unwrap();
    server.wait();
}
