use std::io::{Read, Write};

use may_http::server::*;

// test with: curl -v POST -d "asdfasdfasf" "http://127.0.0.1:8080/"
// test with: curl -v POST  --header "Transfer-Encoding: chunked" -d "hello chunk" "http://127.0.0.1:8080/"
fn hello(mut req: Request, rsp: &mut Response) {
    let mut s = String::new();
    req.read_to_string(&mut s).unwrap();
    write!(rsp, "got data: {}", s).unwrap();
}

fn main() {
    may::config().set_workers(1).set_stack_size(0x10000);
    env_logger::init();
    let server = HttpServer::new(hello).start("127.0.0.1:8080").unwrap();
    server.wait();
    std::thread::sleep(std::time::Duration::from_secs(10));
}
