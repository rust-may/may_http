//! http server implementation on top of `MAY`

use std::io::{self, Read, Write};
use std::net::ToSocketAddrs;
use std::rc::Rc;
use std::sync::Arc;

use bytes::{BufMut, BytesMut};
use may::coroutine;
use may::net::TcpListener;

use super::{HttpService, Request, Response};

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
                    let mut stream = match stream {
                        Ok(s) => s,
                        Err(e) => {
                            error!("incoming stream err = {}", e);
                            continue;
                        }
                    };

                    let writer = match stream.try_clone() {
                        Ok(s) => s,
                        Err(e) => {
                            error!("clone stream err = {}", e);
                            continue;
                        }
                    };

                    let server = server.clone();
                    go!(move || {
                        let reader = Rc::new(stream);
                        let writer = Rc::new(writer);
                        let mut buf = BytesMut::with_capacity(512);
                        loop {
                            match super::request::decode(&mut buf) {
                                Ok(None) => {
                                    // need more data
                                    buf.reserve(512);
                                    match (&*reader).read(unsafe { buf.bytes_mut() }) {
                                        Ok(0) => return, // connection was closed
                                        Ok(n) => unsafe { buf.advance_mut(n) },
                                        Err(err) => {
                                            match err.kind() {
                                                io::ErrorKind::UnexpectedEof
                                                | io::ErrorKind::ConnectionReset => {
                                                    info!("http server read req: connection closed")
                                                }
                                                _ => {
                                                    error!("http server read req: err = {:?}", err)
                                                }
                                            }
                                            return;
                                        }
                                    }
                                }
                                Ok(Some(req)) => {
                                    // req need a read stream composed from buf and reader
                                    // req.set_reader(reader.clone());
                                    let rsp = Response::new(writer.clone());
                                    server.0.handle(req, rsp);
                                }
                                Err(ref e) => {
                                    error!("error decode req: err = {:?}", e);
                                    // exit the coroutine
                                    return;
                                }
                            }
                        }
                    });
                }
            }
        )
    }
}
