use std::fmt;
use std::rc::Rc;
use std::io::{self, Read};
use std::ops::{Deref, DerefMut};

use httparse;
use http::header::*;
use bytes::BytesMut;
use body::BodyReader;
use http::{self, Method, Version};

pub(crate) fn decode(buf: &mut BytesMut) -> io::Result<Option<Request>> {
    let (req, amt) = {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut r = httparse::Request::new(&mut headers);
        let status = r.parse(buf).map_err(|e| {
            let msg = format!("failed to parse http request: {:?}", e);
            io::Error::new(io::ErrorKind::Other, msg)
        })?;

        let amt = match status {
            httparse::Status::Complete(amt) => amt,
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
            .uri(r.path.unwrap())
            .version(version);

        for header in r.headers.iter() {
            req_builder.header(header.name, header.value);
        }

        let req = req_builder
            .body(BodyReader::EmptyReader)
            .map(|req| Some(Request(req)))
            .map_err(|e| {
                let msg = format!("failed to build http request: {:?}", e);
                io::Error::new(io::ErrorKind::Other, msg)
            });

        (req, amt)
    };

    buf.advance(amt);
    req
}

pub struct Request(http::Request<BodyReader>);

impl Request {
    // set the body reader
    // this function would be called by the server to
    // set a proper `BodyReader` according to the request
    pub(crate) fn set_reader(&mut self, reader: Rc<Read>) {
        if self.method() == &Method::GET || self.method() == &Method::HEAD {
            return;
        }

        let size = self.headers().get(CONTENT_LENGTH).map(|v| {
            let s = v.to_str().expect("failed to get content length");
            s.parse().expect("failed to parse content length")
        });

        let body_reader = match size {
            Some(n) => BodyReader::SizedReader(reader, n),
            None => BodyReader::ChunkReader(reader),
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
