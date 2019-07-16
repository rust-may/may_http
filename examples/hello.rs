use http::header::*;
use may_http::server::*;

fn hello(_req: Request, rsp: &mut Response) {
    rsp.headers_mut()
        .append(CONTENT_TYPE, "text/plain; charset=utf-8".parse().unwrap());
    rsp.send(b"Hello World!").unwrap();
}

fn main() {
    may::config().set_io_workers(1);
    env_logger::init();

    let server = HttpServer::new(hello).start("127.0.0.1:8080").unwrap();
    server.wait();
}
