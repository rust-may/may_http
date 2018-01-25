use std::rc::Rc;
use std::io::{self, Write};
use std::fmt;

use mut_io::MutIo;
use self::BodyWriter::*;

pub enum BodyWriter<T: Write> {
    SizedWriter(Rc<T>, usize),
    ChunkWriter(Rc<T>),
    EmptyWriter,
}

impl<T: Write> fmt::Debug for BodyWriter<T> {
    fn fmt(&self, _f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        unimplemented!()
    }
}

impl<T: Write> Write for BodyWriter<T> {
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
        Ok(())
    }
}

impl<T: Write> Drop for BodyWriter<T> {
    fn drop(&mut self) {
        match *self {
            SizedWriter(ref w, ref remain) => {
                // TODO: write enough data to avoid dangling response
                assert_eq!(*remain, 0);
                let mut w = MutIo::new(w.as_ref());
                w.flush().ok();
            }
            ChunkWriter(ref _w) => unimplemented!(),
            EmptyWriter => {}
        }
    }
}
