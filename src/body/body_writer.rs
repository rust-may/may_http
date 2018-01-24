use std::rc::Rc;
use std::io::{self, Write};
use std::fmt;

use mut_io::MutIo;
use self::BodyWriter::*;

pub enum BodyWriter {
    SizedWriter(Rc<Write>, usize),
    ChunkWriter(Rc<Write>),
    EmptyWriter,
}

impl fmt::Debug for BodyWriter {
    fn fmt(&self, _f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        unimplemented!()
    }
}

impl Write for BodyWriter {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use std::cmp;
        match *self {
            SizedWriter(ref w, ref mut remain) => {
                let len = cmp::min(*remain, buf.len());
                let mut w = MutIo::new(w.as_ref());
                let n = w.write(&buf[0..len])?;
                *remain -= n;
                Ok(n)
            }
            ChunkWriter(ref _w) => unimplemented!(),
            EmptyWriter => Ok(0),
        }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        match *self {
            SizedWriter(ref w, _) => {
                let mut w = MutIo::new(w.as_ref());
                w.flush()
            }
            ChunkWriter(ref _w) => unimplemented!(),
            EmptyWriter => Ok(()),
        }
    }
}

impl Drop for BodyWriter {
    fn drop(&mut self) {
        match *self {
            SizedWriter(ref w, ref remain) => {
                // TODO: write enough data
                assert_eq!(*remain, 0);
                let mut w = MutIo::new(w.as_ref());
                w.flush().ok();
            }
            ChunkWriter(ref _w) => unimplemented!(),
            EmptyWriter => {}
        }
    }
}
