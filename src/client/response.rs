use std::cell::RefCell;
use std::fmt;
use std::io::{self, Read};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::body::BodyReader;
use bytes::{Bytes, BytesMut};
use http::header::*;
use http::{self, Version};
use httparse;

pub(crate) fn decode(buf: &mut BytesMut) -> io::Result<Option<Response>> {
    #[inline]
    fn get_slice(buf: &Bytes, data: &[u8]) -> Bytes {
        let begin = data.as_ptr() as usize - buf.as_ptr() as usize;
        buf.slice(begin..begin + data.len())
    }

    let mut headers: [httparse::Header; 64] =
        unsafe { std::mem::MaybeUninit::uninit().assume_init() };
    let mut r = httparse::Response::new(&mut headers);
    let status = r.parse(buf).map_err(|e| {
        let msg = format!("failed to parse http Response: {:?}", e);
        io::Error::new(io::ErrorKind::Other, msg)
    })?;

    let bytes = match status {
        httparse::Status::Complete(amt) => {
            #[allow(clippy::cast_ref_to_mut)]
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

    // build the Response from the parsing result
    // this is not a zero memory copy solution
    // but convinient to be used by framework

    let mut rsp_builder = http::Response::builder();
    rsp_builder = rsp_builder.status(r.code.unwrap()).version(version);

    for header in r.headers.iter() {
        let value =
            unsafe { HeaderValue::from_maybe_shared_unchecked(get_slice(&bytes, header.value)) };
        rsp_builder = rsp_builder.header(header.name, value);
    }

    rsp_builder
        .body(BodyReader::EmptyReader)
        .map(|req| Some(Response(req)))
        .map_err(|e| {
            let msg = format!("failed to build http Response: {:?}", e);
            io::Error::new(io::ErrorKind::Other, msg)
        })
}

/// http server Response
/// a thin wraper to http::Response
/// impl Read for reading http Response body
pub struct Response(http::Response<BodyReader>);

impl Response {
    // set the body reader
    // this function would be called by the client to
    // set a proper `BodyReader` according to the Response
    pub(crate) fn set_reader(&mut self, reader: Rc<RefCell<dyn Read>>) {
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
