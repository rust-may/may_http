use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::{self, Read};
use std::ops::{Deref, DerefMut};

use httparse;
use http::header::*;
use bytes::BytesMut;
use body::BodyReader;
use http::{self, Version};

pub(crate) fn decode(buf: &mut BytesMut) -> io::Result<Option<Response>> {
    let (req, amt) = {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut r = httparse::Response::new(&mut headers);
        let status = r.parse(buf).map_err(|e| {
            let msg = format!("failed to parse http Response: {:?}", e);
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

        // build the Response from the parsing result
        // this is not a zero memory copy solution
        // but convinient to be used by framework

        let mut req_builder = http::Response::builder();
        req_builder.status(r.code.unwrap()).version(version);

        for header in r.headers.iter() {
            req_builder.header(header.name, header.value);
        }

        let req = req_builder
            .body(BodyReader::EmptyReader)
            .map(|req| Some(Response(req)))
            .map_err(|e| {
                let msg = format!("failed to build http Response: {:?}", e);
                io::Error::new(io::ErrorKind::Other, msg)
            });

        (req, amt)
    };

    buf.advance(amt);
    req
}

/// http server Response
/// a thin wraper to http::Response
/// impl Read for reading http Response body
pub struct Response(http::Response<BodyReader>);

impl Response {
    // set the body reader
    // this function would be called by the client to
    // set a proper `BodyReader` according to the Response
    pub(crate) fn set_reader(&mut self, reader: Rc<RefCell<Read>>) {
        use std::str;

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

impl Deref for Response {
    type Target = http::Response<BodyReader>;

    /// deref to the http::Response
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Response {
    /// deref_mut to the http::Response
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Read for Response {
    #[inline]
    fn read(&mut self, msg: &mut [u8]) -> io::Result<usize> {
        self.body_mut().read(msg)
    }
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<HTTP Response {} {:?}>", self.status(), self.version())
    }
}
