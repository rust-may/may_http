use std::rc::Rc;
use std::io::{self, Write};
use std::fmt;

pub enum BodyWriter {
    SizedWriter(Rc<Write>),
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
    fn write(&mut self, _msg: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        unimplemented!()
    }
}
