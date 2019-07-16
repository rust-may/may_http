mod request;
mod response;
mod server_impl;

use std::cell::RefCell;
use std::io::{self, Read, Write};
use std::rc::Rc;

use http::header::*;
use http::Version;

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
// return false if need to close the connection
#[inline]
fn handle_expect(req: &Request, raw_rsp: &mut dyn Write) -> io::Result<bool> {
    use http::header::*;
    use http::StatusCode;
    let expect = match req.headers().get(EXPECT) {
        Some(v) => v.as_bytes(),
        None => return Ok(true),
    };
    if req.version() == Version::HTTP_11 && expect == b"100-continue" {
        write!(
            raw_rsp,
            "{:?} {}\r\n\r\n",
            Version::HTTP_11,
            StatusCode::CONTINUE
        )?;
        raw_rsp.flush()?;
        return Ok(true);
    }

    // don't support expect continue, close the connection
    Ok(false)
}

// return false if need to close the connection
#[inline]
fn process_request<S: Read + Write + 'static, T: HttpService>(
    server: &T,
    name: &str,
    mut req: Request,
    stream: Rc<RefCell<S>>,
) -> bool {
    req.set_reader(stream.clone());
    let version = req.version();
    let mut rsp = Response::new(stream);
    let mut keep_alive = should_keep_alive(version, req.headers());
    if !keep_alive {
        rsp.headers_mut()
            .append(CONNECTION, "close".parse().unwrap());
    }
    rsp.headers_mut().append(SERVER, name.parse().unwrap());
    server.handle(req, &mut rsp);
    if keep_alive {
        keep_alive = should_keep_alive(version, rsp.headers());
    }
    keep_alive
}

#[inline]
pub fn should_keep_alive(version: Version, headers: &HeaderMap) -> bool {
    let conn = headers.get_all(CONNECTION);
    match version {
        Version::HTTP_10 => {
            for v in conn {
                if v.as_bytes() == b"keep-alive" {
                    return true;
                }
            }
            false
        }
        Version::HTTP_11 => {
            for v in conn {
                if v.as_bytes() == b"close" {
                    return false;
                }
            }
            true
        }
        _ => true,
    }
}
