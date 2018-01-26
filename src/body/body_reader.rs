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
    fn read(&mut self, _msg: &mut [u8]) -> io::Result<usize> {
        unimplemented!()
    }
}

impl Drop for BodyReader {
    fn drop(&mut self) {
        // unimplemented!()
    }
}
