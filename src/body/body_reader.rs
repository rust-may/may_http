use std::rc::Rc;
use std::io::{self, Read};
use std::fmt;

pub enum BodyReader {
    SizedReader(Rc<Read>, usize),
    ChunkReader(Rc<Read>),
    EmptyReader,
}

impl fmt::Debug for BodyReader {
    fn fmt(&self, _f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        unimplemented!()
    }
}

impl Read for BodyReader {
    #[inline]
    fn read(&mut self, _msg: &mut [u8]) -> io::Result<usize> {
        unimplemented!()
    }
}

impl Drop for BodyReader {
    fn drop(&mut self) {
        // unimplemented!()
    }
}
