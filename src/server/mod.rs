mod request;
mod response;
mod server_impl;

use std::rc::Rc;
use std::cell::RefCell;
use std::io::{self, Read, Write};

pub use self::request::Request;
pub use self::response::Response;
pub use self::server_impl::HttpServer;

/// the http service trait
/// user code should supply a type that impl the `handle` method for the http server
///
pub trait HttpService {
    /// Receives a `Request`/`Response` pair, and should perform some action on them.
    ///
    /// This could reading from the request, and writing to the response.
    fn handle(&self, request: Request, response: &mut Response);
}

impl<F> HttpService for F
where
    F: Fn(Request, &mut Response),
    F: Sync + Send,
{
    fn handle(&self, req: Request, res: &mut Response) {
        self(req, res)
    }
}

// when client has expect header, we need to write CONTINUE rsp first
// return true if need to close the connection
#[inline]
fn handle_expect(req: &Request, raw_rsp: &mut Write) -> io::Result<bool> {
    use http::header::*;
    use http::{StatusCode, Version};
    let expect = match req.headers().get(EXPECT) {
        Some(v) => v.as_bytes(),
        None => return Ok(false),
    };
    if req.version() == Version::HTTP_11 && expect == b"100-continue" {
        write!(
            raw_rsp,
            "{:?} {}\r\n\r\n",
            Version::HTTP_11,
            StatusCode::CONTINUE
        )?;
        raw_rsp.flush()?;
        return Ok(false);
    }

    // don't support expect continue, close the connection
    Ok(true)
}

// return ture if need to close the connection
#[inline]
fn process_request<S: Read + Write + 'static, T: HttpService>(
    server: &T,
    mut req: Request,
    stream: Rc<RefCell<S>>,
) -> bool {
    req.set_reader(stream.clone());
    let mut rsp = Response::new(stream.clone());
    server.handle(req, &mut rsp);
    false
}
