use std::rc::Rc;
use std::io::{self, Write};
use std::fmt;

use self::BodyWriter::*;

pub enum BodyWriter {
    SizedWriter(Rc<Write>, usize),
    ChunkWriter(Rc<Write>),
    // this is used to write all the data out when get drop
    EmptyWriter(Rc<Write>),
    // this is used as a invalid place holder
    InvalidWriter,
}

impl fmt::Debug for BodyWriter {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let name = match *self {
            SizedWriter(..) => "SizedWriter",
            ChunkWriter(_) => "ChunkWriter",
            EmptyWriter(_) => "EmptyWriter",
            InvalidWriter => "Invalid",
        };
        write!(f, "BodyWriter {}", name)
    }
}

impl Write for BodyWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use std::cmp;
        match *self {
            SizedWriter(ref w, ref mut remain) => {
                let len = cmp::min(*remain, buf.len());
                let w = unsafe { &mut *(w.as_ref() as *const _ as *mut Write) };
                let n = w.write(&buf[0..len])?;
                *remain -= n;
                Ok(n)
            }
            ChunkWriter(ref _w) => unimplemented!(),
            EmptyWriter(_) => Ok(0),
            InvalidWriter => unreachable!(),
        }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        match *self {
            SizedWriter(ref w, _) => {
                let w = unsafe { &mut *(w.as_ref() as *const _ as *mut Write) };
                w.flush()
            }
            ChunkWriter(ref w) => {
                let w = unsafe { &mut *(w.as_ref() as *const _ as *mut Write) };
                w.flush()
            }
            EmptyWriter(ref w) => {
                let w = unsafe { &mut *(w.as_ref() as *const _ as *mut Write) };
                w.flush()
            }
            InvalidWriter => unreachable!(),
        }
    }
}

impl Drop for BodyWriter {
    fn drop(&mut self) {
        match *self {
            SizedWriter(ref w, ref remain) => {
                // TODO: write enough data to avoid dangling response
                assert_eq!(*remain, 0);
                let w = unsafe { &mut *(w.as_ref() as *const _ as *mut Write) };
                w.flush().ok();
            }
            ChunkWriter(ref _w) => unimplemented!(),
            EmptyWriter(ref w) => {
                let w = unsafe { &mut *(w.as_ref() as *const _ as *mut Write) };
                w.flush().ok();
            }
            InvalidWriter => {}
        }
    }
}
