//! Server Responses
//!
//! These are responses sent by a `may_http::Server` to clients, after
//! receiving a request.
use std::rc::Rc;
use std::io::{self, Write};
// use std::thread;
use std::fmt;

// use time::now_utc;

use http::{HeaderMap, StatusCode, Version};
// use http::header::*;

use body::BodyWriter;

/// response internal state
#[derive(Debug, PartialEq)]
enum ResponseState {
    // the fresh state
    Init,
    // head write done, need to write body
    WriteHeadDone,
    // the response is finished to write to the stream
    // Done,
}

/// The outgoing half for a Tcp connection, created by a `Server` and given to a `Handler`.
///
/// The default `StatusCode` for a `Response` is `200 OK`.
///
/// There is a `Drop` implementation for `Response` that will automatically
/// write the head and flush the body, if the handler has not already done so,
/// so that the server doesn't accidentally leave dangling requests.
pub struct Response {
    /// The HTTP version of this response.
    pub version: Version,
    // Stream the Response is writing to, not accessible through UnwrittenResponse
    body: BodyWriter,
    // The status code for the request.
    status: StatusCode,
    // The outgoing headers on this response.
    headers: HeaderMap,
    // the underline write stream
    writer: Rc<Write>,
    // the response current state
    state: ResponseState,
    // the cached response size
    body_size: Option<usize>,
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<HTTP Response {}>", self.status)
    }
}

impl Response {
    /// Creates a new Response that can be used to write to a network stream.
    #[inline]
    pub fn new(stream: Rc<Write>) -> Response {
        Response {
            status: StatusCode::OK,
            version: Version::HTTP_11,
            headers: HeaderMap::with_capacity(16),
            body: BodyWriter::EmptyWriter,
            writer: stream,
            state: ResponseState::Init,
            body_size: None,
        }
    }

    /// write head to stream
    fn write_head(&mut self) -> io::Result<BodyWriter> {
        let writer = unsafe { &mut *(self.writer.as_ref() as *const _ as *mut Write) };
        // TODO: don't use std write!
        write!(writer, "{:?} {}\r\n", self.version, self.status)?;

        // if !self.headers.contains_key(header::DATE) {
        //     // don't write in the header but write to stream direclty
        //     // so that we can save some write
        //     self.headers.insert(head)
        //     self.headers.set(header::Date(header::HttpDate(now_utc())));
        // }
        write!(writer, "Server: Example\r\nDate: {}\r\n", ::date::now())?;

        for (key, value) in self.headers.iter() {
            // TODO: filter out Content-Length and set body_size
            write!(
                writer,
                "{}: {}\r\n",
                key.as_str(),
                value.to_str().unwrap_or("")
            )?;
        }

        if let Some(len) = self.body_size {
            write!(writer, "Content-Length: {}\r\n", len)?
        }

        write!(writer, "\r\n")?;

        let body = match self.status {
            StatusCode::NO_CONTENT | StatusCode::NOT_MODIFIED => BodyWriter::EmptyWriter,
            c if c.is_informational() => BodyWriter::EmptyWriter,
            _ => if let Some(size) = self.body_size {
                BodyWriter::SizedWriter(self.writer.clone(), size)
            } else {
                BodyWriter::ChunkWriter(self.writer.clone())
            },
        };

        // // can't do in match above, thanks borrowck
        // if body_type == Body::Chunked {
        //     let encodings = match self.headers.get_mut::<header::TransferEncoding>() {
        //         Some(&mut header::TransferEncoding(ref mut encodings)) => {
        //             //TODO: check if chunked is already in encodings. use HashSet?
        //             encodings.push(header::Encoding::Chunked);
        //             false
        //         },
        //         None => true
        //     };

        //     if encodings {
        //         self.headers.set::<header::TransferEncoding>(
        //             header::TransferEncoding(vec![header::Encoding::Chunked]))
        //     }
        // }
        Ok(body)
    }

    /// Writes the body and ends the response.
    ///
    /// This is a shortcut method for when you have a response with a fixed
    /// size, and would only need a single `write` call normally.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use may_http::server::Response;
    /// fn handler(res: Response) {
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
    pub fn send(self, body: &[u8]) -> io::Result<()> {
        let mut me = self;
        me.body_size = Some(body.len());
        me.write_all(body)
    }

    /// set the content-length
    ///
    /// if you don't call `send()`, should call this before write the response
    pub fn set_content_length(&mut self, len: usize) {
        self.body_size = Some(len);
    }

    /// The status of this response.
    #[inline]
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// The headers of this response.
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get a mutable reference to the status.
    #[inline]
    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.status
    }

    /// Get a mutable reference to the Headers.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }
}

impl Write for Response {
    #[inline]
    fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
        if self.state == ResponseState::Init {
            self.body = self.write_head()?;
            self.state = ResponseState::WriteHeadDone;
        }
        self.body.write(msg)
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
            self.status = StatusCode::INTERNAL_SERVER_ERROR;
        }
        // make sure we write every thing
        if self.state == ResponseState::Init {
            self.body = self.write_head().unwrap_or(BodyWriter::EmptyWriter);
            self.state = ResponseState::WriteHeadDone;
        }
    }
}
