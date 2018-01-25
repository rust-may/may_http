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
use http::header::*;

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
    fn fmt(&self, _f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        unimplemented!()
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
        // use std::str;
        // debug!("writing head: {:?} {:?}", self.version, self.status);
        let writer = unsafe {&mut *(self.writer.as_ref() as *const _ as *mut Write)};
        // TODO: don't use std write!
        // write!(writer, "{:?} {:?}\r\n", self.version, self.status)?;
        // writer.write(b"HTTP/1.1 200 OK\r\n")?;
        // writer.write(b"Server: Example\r\n")?;
        // // writer.write(b"Date: Wed, 24 Jan 2018 22:53:21\r\n")?;
        // write!(writer, "Date: {}\r\n", ::date::now())?;
        // writer.write(b"Content-Length: 13\r\n")?;

        write!(
            writer,
            "\
             HTTP/1.1 200 OK\r\n\
             Server: Example\r\n\
             Content-Length: 13\r\n\
             Date: {}\r\n\r\n\
             ",
            ::date::now()
        )?;

        // if !self.headers.contains_key(header::DATE) {
        //     // don't write in the header but write to stream direclty
        //     // so that we can save some write
        //     self.headers.insert(head)
        //     self.headers.set(header::Date(header::HttpDate(now_utc())));
        // }

        // for (key, value) in self.headers.iter() {
        //     writer.write(key.as_str().as_bytes())?;
        //     writer.write(b": ")?;
        //     writer.write(value.as_bytes())?;
        //     writer.write(b"\r\n")?;
        // }

        // writer.write(b"\r\n")?;

        // let body = match self.status {
        //     StatusCode::NO_CONTENT | StatusCode::NOT_MODIFIED => BodyWriter::EmptyWriter,
        //     c if c.is_informational() => BodyWriter::EmptyWriter,
        //     // _ => if let Some(size) = self.headers().get(CONTENT_LENGTH).map(|v| unsafe {
        //     //     str::from_utf8_unchecked(v.as_bytes())
        //     //         .parse()
        //     //         .expect("failed to parse content length")
        //     // }) {
        //     //     BodyWriter::SizedWriter(self.writer.clone(), size)
        //     // } else {
        //     //     BodyWriter::ChunkWriter(self.writer.clone())
        //     // },
        //     _ => BodyWriter::SizedWriter(self.writer.clone(), 13),
        // };

        let body = BodyWriter::SizedWriter(self.writer.clone(), 13);

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

        // debug!("headers [\n{:?}]", self.headers);
        // try!(write!(&mut self.body, "{}", self.headers));
        // try!(write!(&mut self.body, "{}", LINE_ENDING));

        // Ok(body_type)
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
    /// # use may_http::http::header::*;
    /// use std::io::Write;
    /// fn handler(mut res: Response) {
    ///     let body = b"Hello World!";
    ///     res.headers_mut().insert(CONTENT_LENGTH, HeaderValue::from_static("64"));
    ///     res.write_all(body).unwrap();
    /// }
    /// ```
    #[inline]
    pub fn send(self, body: &[u8]) -> io::Result<()> {
        let mut me = self;
        // let len = body.len().to_string();
        // let len = HeaderValue::from_bytes(len.as_bytes()).expect("invalide length value");
        // me.headers_mut().insert(CONTENT_LENGTH, len);
        me.write(body)?;
        Ok(())
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
        // debug!("write {:?} bytes", msg.len());
        if self.state == ResponseState::Init {
            self.body = self.write_head()?;
            self.state = ResponseState::WriteHeadDone;
        }
        self.body.write(msg)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        // self.body.flush()
        Ok(())
    }
}

// impl Drop for Response {
//     fn drop(&mut self) {
//         // unimplemented!()
//         // if TypeId::of::<T>() == TypeId::of::<Fresh>() {
//         //     if thread::panicking() {
//         //         self.status = status::StatusCode::InternalServerError;
//         //     }

//         //     let mut body = match self.write_head() {
//         //         Ok(Body::Chunked) => ChunkedWriter(self.body.get_mut()),
//         //         Ok(Body::Sized(len)) => SizedWriter(self.body.get_mut(), len),
//         //         Ok(Body::Empty) => EmptyWriter(self.body.get_mut()),
//         //         Err(e) => {
//         //             debug!("error dropping request: {:?}", e);
//         //             return;
//         //         }
//         //     };
//         //     end(&mut body);
//         // } else {
//         //     end(&mut self.body);
//         // };

//         // #[inline]
//         // fn end<W: Write>(w: &mut W) {
//         //     match w.write(&[]) {
//         //         Ok(_) => match w.flush() {
//         //             Ok(_) => debug!("drop successful"),
//         //             Err(e) => debug!("error dropping request: {:?}", e)
//         //         },
//         //         Err(e) => debug!("error dropping request: {:?}", e)
//         //     }
//         // }
//     }
// }
