use std::io;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Duration;
use std::net::ToSocketAddrs;
// use std::io::{self, Read, Write};

use http::Uri;
use buffer::BufferIo;
use may::net::TcpStream;

use client::{Request, Response};

/// this is just a simple client connector
#[derive(Debug)]
pub struct HttpClient {
    conn: Rc<RefCell<BufferIo<TcpStream>>>,
}

impl HttpClient {
    pub fn connect<A: ToSocketAddrs>(remote: A) -> io::Result<Self> {
        // TODO: use async dns resolve
        let stream = TcpStream::connect(remote)?;
        let stream = BufferIo::new(stream);
        Ok(HttpClient {
            conn: Rc::new(RefCell::new(stream)),
        })
    }

    pub fn set_timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
        {
            let mut s = self.conn.borrow_mut();
            let s = s.inner_mut();
            s.set_read_timeout(timeout).unwrap();
            s.set_write_timeout(timeout).unwrap();
        }
        self
    }

    pub fn get(&mut self, uri: Uri) -> io::Result<Response> {
        let mut req = Request::new(self.conn.clone());
        *req.uri_mut() = uri;
        // send out the request by drop the req
        drop(req);
        self.get_rsp()
    }

    #[inline]
    fn get_rsp(&mut self) -> io::Result<Response> {
        let mut stream = self.conn.borrow_mut();
        loop {
            match super::response::decode(stream.get_reader_buf())? {
                None => {
                    // need more data
                    if stream.bump_read()? == 0 {
                        // break the connection
                        return Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "connection breaked",
                        ));
                    }
                }
                Some(mut rsp) => {
                    rsp.set_reader(self.conn.clone());
                    return Ok(rsp);
                }
            }
        }
    }
}
