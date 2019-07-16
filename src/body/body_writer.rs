use std::cell::RefCell;
use std::fmt;
use std::io::{self, Write};
use std::rc::Rc;

use self::BodyWriter::*;

pub enum BodyWriter {
    SizedWriter(Rc<RefCell<dyn Write>>, usize),
    ChunkWriter(Rc<RefCell<dyn Write>>),
    // this is used to write all the data out when get drop
    EmptyWriter(Rc<RefCell<dyn Write>>),
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
                let mut w = w.borrow_mut();
                let n = w.write(&buf[0..len])?;
                *remain -= n;
                Ok(n)
            }
            ChunkWriter(ref w) => {
                let chunk_size = buf.len();
                let mut w = w.borrow_mut();
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
                let mut w = w.borrow_mut();
                w.flush()
            }
            ChunkWriter(ref w) => {
                let mut w = w.borrow_mut();
                w.flush()
            }
            EmptyWriter(ref w) => {
                let mut w = w.borrow_mut();
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
                let mut w = w.borrow_mut();
                if remain > 0 {
                    // write enough data when drop
                    let buf = vec![0u8; remain];
                    w.write_all(&buf).ok();
                }
                w.flush().ok();
            }
            ChunkWriter(ref w) => {
                // write the chunk end and flush
                let mut w = w.borrow_mut();
                w.write_all(b"0\r\n\r\n").ok();
                w.flush().ok();
            }
            EmptyWriter(ref w) => {
                let mut w = w.borrow_mut();
                w.flush().ok();
            }
            InvalidWriter => {}
        }
    }
}
