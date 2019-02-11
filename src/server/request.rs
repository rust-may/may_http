use std::cell::RefCell;
use std::fmt;
use std::io::{self, Read};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use body::BodyReader;
use bytes::{Bytes, BytesMut};
use http::header::*;
use http::{self, Method, Version};
use httparse;

pub(crate) fn decode(buf: &mut BytesMut) -> io::Result<Option<Request>> {
    #[inline]
    fn get_slice(buf: &Bytes, data: &[u8]) -> Bytes {
        let begin = data.as_ptr() as usize - buf.as_ptr() as usize;
        buf.slice(begin, begin + data.len())
    }

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut r = httparse::Request::new(&mut headers);
    let status = r.parse(buf).map_err(|e| {
        let msg = format!("failed to parse http request: {:?}", e);
        io::Error::new(io::ErrorKind::Other, msg)
    })?;

    let bytes = match status {
        httparse::Status::Complete(amt) => {
            let buf = unsafe { &mut *(buf as *const _ as *mut BytesMut) };
            buf.split_to(amt).freeze()
        }
        httparse::Status::Partial => return Ok(None),
    };

    let version = match r.version {
        Some(v) => {
            if v == 0 {
                Version::HTTP_10
            } else {
                Version::HTTP_11
            }
        }
        None => Version::HTTP_11,
    };

    // build the request from the parsing result
    // this is not a zero memory copy solution
    // but convinient to be used by framework

    let mut req_builder = http::Request::builder();
    req_builder
        .method(r.method.unwrap())
        .uri(get_slice(&bytes, r.path.unwrap().as_bytes())) // can be optimized with Bytes
        .version(version);

    for header in r.headers.iter() {
        let value = unsafe { HeaderValue::from_shared_unchecked(get_slice(&bytes, header.value)) };
        req_builder.header(header.name, value);
    }

    req_builder
        .body(BodyReader::EmptyReader)
        .map(|req| Some(Request(req)))
        .map_err(|e| {
            let msg = format!("failed to build http request: {:?}", e);
            io::Error::new(io::ErrorKind::Other, msg)
        })
}

/// http server request
/// a thin wraper to http::Request
/// impl Read for reading http request body
pub struct Request(http::Request<BodyReader>);

impl Request {
    // set the body reader
    // this function would be called by the server to
    // set a proper `BodyReader` according to the request
    pub(crate) fn set_reader(&mut self, reader: Rc<RefCell<Read>>) {
        use std::str;

        if self.method() == &Method::GET || self.method() == &Method::HEAD {
            return;
        }

        let size = self.headers().get(CONTENT_LENGTH).map(|v| {
            let s = unsafe { str::from_utf8_unchecked(v.as_bytes()) };
            s.parse().expect("failed to parse content length")
        });

        let body_reader = match size {
            Some(n) => BodyReader::SizedReader(reader, n),
            None => BodyReader::ChunkReader(reader, None),
        };

        *self.body_mut() = body_reader;
    }
}

impl Deref for Request {
    type Target = http::Request<BodyReader>;

    /// deref to the http::Request
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Request {
    /// deref_mut to the http::Request
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Read for Request {
    #[inline]
    fn read(&mut self, msg: &mut [u8]) -> io::Result<usize> {
        self.body_mut().read(msg)
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<HTTP Request {} {}>", self.method(), self.uri())
    }
}
