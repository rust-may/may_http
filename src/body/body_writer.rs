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
            ChunkWriter(ref w) => {
                let chunk_size = buf.len();
                let w = unsafe { &mut *(w.as_ref() as *const _ as *mut Write) };
                write!(w, "{:X}\r\n", chunk_size)?;
                w.write_all(buf)?;
                w.write_all(b"\r\n")?;
                Ok(chunk_size)
            }
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
            SizedWriter(ref w, remain) => {
                let w = unsafe { &mut *(w.as_ref() as *const _ as *mut Write) };
                if remain > 0 {
                    // write enough data when drop
                    let buf = vec![0u8; remain];
                    w.write_all(&buf).ok();
                }
                w.flush().ok();
            }
            ChunkWriter(ref w) => {
                // write the chunk end and flush
                let w = unsafe { &mut *(w.as_ref() as *const _ as *mut Write) };
                w.write_all(b"0\r\n\r\n").ok();
                w.flush().ok();
            }
            EmptyWriter(ref w) => {
                let w = unsafe { &mut *(w.as_ref() as *const _ as *mut Write) };
                w.flush().ok();
            }
            InvalidWriter => {}
        }
    }
}
