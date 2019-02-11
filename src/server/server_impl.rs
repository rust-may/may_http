//! http server implementation on top of `MAY`
//!
use std::cell::RefCell;
use std::io;
use std::net::ToSocketAddrs;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use buffer::BufferIo;
use may::coroutine;
use may::net::TcpListener;
use server::HttpService;

macro_rules! t {
    ($e: expr) => {
        match $e {
            Ok(val) => val,
            Err(ref err)
                if err.kind() == io::ErrorKind::ConnectionReset
                    || err.kind() == io::ErrorKind::UnexpectedEof =>
            {
                // info!("http server read req: connection closed");
                return;
            }
            Err(err) => {
                error!("call = {:?}\nerr = {:?}", stringify!($e), err);
                return;
            }
        }
    };
}

macro_rules! t_c {
    ($e: expr) => {
        match $e {
            Ok(val) => val,
            Err(err) => {
                error!("call = {:?}\nerr = {:?}", stringify!($e), err);
                continue;
            }
        }
    };
}

/// this is the generic type http server
/// with a type parameter that impl `HttpService` trait
///
pub struct HttpServer<T: HttpService> {
    inner: T,
    name: String,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl<T: HttpService + Send + Sync + 'static> HttpServer<T> {
    /// create a http server with default configuration
    pub fn new(server: T) -> Self {
        HttpServer {
            inner: server,
            name: String::from("Example"),
            read_timeout: None,
            write_timeout: None,
        }
    }

    /// set read timeout
    pub fn set_read_timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
        self.read_timeout = timeout;
        self
    }

    /// set write timeout
    pub fn set_write_timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
        self.write_timeout = timeout;
        self
    }

    /// set the serer name
    pub fn set_server_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

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
                    t_c!(stream.set_read_timeout(server.read_timeout));
                    t_c!(stream.set_write_timeout(server.write_timeout));
                    let server = server.clone();
                    go!(move || {
                        let mut stream = BufferIo::new(stream);
                        loop {
                            match t!(super::request::decode(stream.get_reader_buf())) {
                                None => {
                                    // need more data
                                    if t!(stream.bump_read()) == 0 {
                                        // break the connection
                                        return;
                                    };
                                }
                                Some(req) => {
                                    if !t!(super::handle_expect(&req, &mut stream)) {
                                        // close the connection
                                        return;
                                    };
                                    let io = Rc::new(RefCell::new(stream));
                                    if !super::process_request(
                                        &server.inner,
                                        &server.name,
                                        req,
                                        io.clone(),
                                    ) {
                                        // close the connection
                                        return;
                                    }
                                    // since handle is done, the reader should be released
                                    stream = Rc::try_unwrap(io).expect("no reader").into_inner();
                                }
                            }
                        }
                    });
                }
            }
        )
    }
}

// TODO: pub struct HttpsServer<T>(pub T);
// TODO: support web socket
// TODO: support pipeline server
