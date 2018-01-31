//! Server Responses
//!
//! These are responses sent by a `may_http::Server` to clients, after
//! receiving a request.
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::{self, Write};
use std::ops::{Deref, DerefMut};

use http::header::*;
use body::BodyWriter;
use http::{self, StatusCode};

/// The outgoing half for a Stream, created by a `Server` and given to a `HttpService`.
///
/// There is a `Drop` implementation for `Response` that will automatically
/// write the head and flush the body, if the handler has not already done so,
/// so that the server doesn't accidentally leave dangling requests.
///
/// it's a thin wraper to http::Response
/// impl Write for writing http response body
pub struct Response {
    // the Raw http rsponse
    raw_rsp: http::Response<BodyWriter>,
    // the underline write stream
    writer: Rc<RefCell<Write>>,
    // the cached response size
    body_size: Option<usize>,
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<HTTP Response {}>", self.status())
    }
}

impl Response {
    /// Creates a new Response that can be used to write to a network stream.
    #[inline]
    pub fn new(stream: Rc<RefCell<Write>>) -> Response {
        Response {
            raw_rsp: http::Response::new(BodyWriter::InvalidWriter),
            writer: stream,
            body_size: None,
        }
    }

    // actual write head to stream
    fn write_head_impl(&mut self) -> io::Result<()> {
        let mut writer = self.writer.borrow_mut();

        if let Some(len) = self.body_size {
            write!(
                writer,
                "{:?} {}\r\nDate: {}\r\nContent-Length: {}\r\n",
                self.version(),
                self.status(),
                ::date::now(),
                len
            )?;
        } else {
            write!(
                writer,
                "{:?} {}\r\nDate: {}\r\n",
                self.version(),
                self.status(),
                ::date::now()
            )?;
        }

        for (key, value) in self.headers().iter() {
            // we can use writev here?
            writer.write_all(key.as_str().as_bytes())?;
            writer.write_all(b": ")?;
            writer.write_all(value.as_bytes())?;
            writer.write_all(b"\r\n")?;
        }
        writer.write_all(b"\r\n")
    }

    // write head to stream
    fn write_head(&mut self) -> io::Result<BodyWriter> {
        let body = match self.status() {
            StatusCode::NO_CONTENT | StatusCode::NOT_MODIFIED => {
                BodyWriter::EmptyWriter(self.writer.clone())
            }
            c if c.is_informational() => BodyWriter::EmptyWriter(self.writer.clone()),
            _ => if let Some(size) = self.body_size {
                BodyWriter::SizedWriter(self.writer.clone(), size)
            } else {
                self.headers_mut()
                    .append(TRANSFER_ENCODING, "chunked".parse().unwrap());
                BodyWriter::ChunkWriter(self.writer.clone())
            },
        };
        // TODO: sanity check the headers, overwrite content-length header

        self.write_head_impl()?;
        Ok(body)
    }

    /// Writes the body and ends the response.
    ///
    /// This is a shortcut method for when you have a response with a fixed
    /// size, and would only need a single `write` call normally.
    /// successive write would return Ok(0) or write error becuase the writer
    /// is closed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use may_http::server::Response;
    /// fn handler(res: &mut Response) {
    ///     res.send(b"Hello World!").unwrap();
    /// }
    /// ```
    ///
    /// The above is the same, but shorter, than the longer:
    ///
    /// ```no_run
    /// # use may_http::server::Response;
    /// use std::io::Write;
    /// fn handler(mut res: Response) {
    ///     let body = b"Hello World!";
    ///     res.set_content_length(body.len());
    ///     res.write_all(body).unwrap();
    /// }
    /// ```
    #[inline]
    pub fn send(&mut self, body: &[u8]) -> io::Result<()> {
        self.body_size = Some(body.len());
        self.write_all(body)
    }

    /// set the content-length
    ///
    /// if you don't call `send()`, should call this before write the response
    #[inline]
    pub fn set_content_length(&mut self, len: usize) {
        self.body_size = Some(len);
    }
}

impl Deref for Response {
    type Target = http::Response<BodyWriter>;

    /// deref to the http::Response
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.raw_rsp
    }
}

impl DerefMut for Response {
    /// deref_mut to the http::Response
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw_rsp
    }
}

impl Write for Response {
    #[inline]
    fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
        if let BodyWriter::InvalidWriter = *self.body() {
            *self.body_mut() = self.write_head()?;
        }
        self.body_mut().write(msg)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Response {
    fn drop(&mut self) {
        use std::thread;

        if thread::panicking() {
            *self.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            self.write_all(
                b"sorry, the server paniced inside!\n\
                  please contact the service provider!",
            ).ok();
            return;
        }

        // make sure we write every thing
        if let BodyWriter::InvalidWriter = *self.body() {
            *self.body_mut() = self.write_head()
                .unwrap_or(BodyWriter::EmptyWriter(self.writer.clone()));
        }
    }
}
