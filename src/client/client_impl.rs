use std::cell::RefCell;
use std::io;
use std::net::ToSocketAddrs;
use std::rc::Rc;
use std::time::Duration;

use bytes::Buf;
use http::{Method, Uri};
use may::net::TcpStream;

use crate::buffer::BufferIo;
use crate::client::{Request, Response};

/// this is just a simple client connector
#[derive(Debug)]
pub struct HttpClient {
    conn: Rc<RefCell<BufferIo<TcpStream>>>,
}

impl HttpClient {
    /// create HttpClient connect to the given address
    pub fn connect<A: ToSocketAddrs>(remote: A) -> io::Result<Self> {
        // TODO: use async dns resolve
        let stream = TcpStream::connect(remote)?;
        let stream = BufferIo::new(stream);
        Ok(HttpClient {
            conn: Rc::new(RefCell::new(stream)),
        })
    }

    /// set both read/write timeout for the connection
    pub fn set_timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
        {
            let mut s = self.conn.borrow_mut();
            let s = s.inner_mut();
            s.set_read_timeout(timeout).unwrap();
            s.set_write_timeout(timeout).unwrap();
        }
        self
    }

    /// create a GET request to the specified uri and return the response
    ///
    /// this is a shortcut for
    /// ```no_doc
    /// client.send_request(client.new_request(GET, uri))
    /// ```
    pub fn get(&mut self, uri: Uri) -> io::Result<Response> {
        let mut req = Request::new(self.conn.clone());
        *req.uri_mut() = uri;
        // send out the request by drop the req
        drop(req);
        self.get_rsp()
    }

    /// create a post request with the uri and data, return the response
    ///
    /// this is a shortcut for
    /// ```no_doc
    ///  let req = client.new_request(POST, uri);
    ///  req.send(data.bytes())?;
    ///  client.send_request()
    /// ```
    pub fn post<T: Buf>(&mut self, uri: Uri, data: T) -> io::Result<Response> {
        let mut req = Request::new(self.conn.clone());
        *req.method_mut() = Method::POST;
        *req.uri_mut() = uri;
        req.send(data.bytes())?;
        // send out the request by drop the req
        drop(req);
        self.get_rsp()
    }

    /// create a request with specified method and uri
    ///
    /// you can use the request to write any data and
    /// then send the request via `send_request` to get response
    #[inline]
    pub fn new_request(&self, method: Method, uri: Uri) -> Request {
        let mut req = Request::new(self.conn.clone());
        *req.method_mut() = method;
        *req.uri_mut() = uri;
        req
    }

    /// get response according to the request
    ///
    /// note that you can only send the request that created form the
    /// same client, or call this function will panic
    #[inline]
    pub fn send_request(&mut self, req: Request) -> io::Result<Response> {
        use std::io::Write;
        let conn: Rc<RefCell<dyn Write>> = self.conn.clone();
        assert_eq!(Rc::ptr_eq(&conn, req.conn()), true);
        drop(req);
        self.get_rsp()
    }

    // get response from the connection
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
