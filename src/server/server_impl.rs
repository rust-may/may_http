//! http server implementation on top of `MAY`

use std::io;
use std::net::ToSocketAddrs;
use std::rc::Rc;
use std::sync::Arc;

// use bytes::BytesMut};
use may::coroutine;
use may::net::TcpListener;

use buffer::BufferIo;
use server::{HttpService, Response};

macro_rules! t {
    ($e: expr) => (match $e {
        Ok(val) => val,
        Err(ref err) if err.kind() == io::ErrorKind::ConnectionReset ||
                        err.kind() == io::ErrorKind::UnexpectedEof=> {
            // info!("http server read req: connection closed");
            return;
        }
        Err(err) => {
            error!("call = {:?}\nerr = {:?}", stringify!($e), err);
            return;
        }
    })
}

macro_rules! t_c {
    ($e: expr) => (match $e {
        Ok(val) => val,
        Err(err) => {
            error!("call = {:?}\nerr = {:?}", stringify!($e), err);
            continue;
        }
    })
}

/// this is the generic type http server
/// with a type parameter that impl `HttpService` trait
///
pub struct HttpServer<T>(pub T);

impl<T: HttpService + Send + Sync + 'static> HttpServer<T> {
    /// Spawns the http service, binding to the given address
    /// return a coroutine that you can cancel it when need to stop the service
    pub fn start<L: ToSocketAddrs>(self, addr: L) -> io::Result<coroutine::JoinHandle<()>> {
        let listener = TcpListener::bind(addr)?;
        go!(
            coroutine::Builder::new().name("TcpServer".to_owned()),
            move || {
                let server = Arc::new(self);
                for stream in listener.incoming() {
                    let mut stream = t_c!(stream);
                    // let writer = t_c!(stream.try_clone());
                    let server = server.clone();
                    go!(move || {
                        let mut stream = BufferIo::new(stream);
                        // let writer = Rc::new(BufWriter::with_capacity(1024, writer));
                        // first try to read some data
                        // t!(reader.bump_read());
                        loop {
                            match t!(super::request::decode(stream.get_reader_buf())) {
                                None => {
                                    // need more data
                                    if t!(stream.bump_read()) == 0 {
                                        // break the connection
                                        return;
                                    };
                                }
                                Some(mut req) => {
                                    let io = Rc::new(stream);
                                    req.set_reader(io.clone());
                                    let rsp = Response::new(io.clone());
                                    server.0.handle(req, rsp);
                                    // since handle is done, the reader should be released
                                    stream = Rc::try_unwrap(io).expect("no reader");
                                }
                            }
                        }
                    });
                }
            }
        )
    }
}
