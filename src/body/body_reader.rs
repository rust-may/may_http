use std::rc::Rc;
use std::io::{self, Read};
use std::fmt;

use self::BodyReader::*;

pub enum BodyReader {
    SizedReader(Rc<Read>, usize),
    ChunkReader(Rc<Read>),
    EmptyReader,
}

impl fmt::Debug for BodyReader {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let name = match *self {
            SizedReader(..) => "SizedReader",
            ChunkReader(_) => "ChunkReader",
            EmptyReader => "EmptyReader",
        };
        write!(f, "BodyReader {}", name)
    }
}

impl Read for BodyReader {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use std::cmp;
        match *self {
            SizedReader(ref r, ref mut remain) => {
                let len = cmp::min(*remain, buf.len());
                if len == 0 {
                    return Ok(0);
                }
                let r = unsafe { &mut *(r.as_ref() as *const _ as *mut Read) };
                let n = r.read(&mut buf[0..len])?;
                *remain -= n;
                Ok(n)
            }
            ChunkReader(ref _r) => unimplemented!(),
            EmptyReader => Ok(0),
        }
    }
}

impl Drop for BodyReader {
    fn drop(&mut self) {
        match *self {
            SizedReader(ref r, remain) => {
                // read enough data
                if remain > 0 {
                    let r = unsafe { &mut *(r.as_ref() as *const _ as *mut Read) };
                    let mut buf = vec![0u8; remain];
                    r.read_exact(&mut buf).ok();
                }
            }
            ChunkReader(ref _r) => unimplemented!(),
            EmptyReader => {}
        }
    }
}
