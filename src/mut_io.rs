//! Wrap a io ref so that we can use the Read/Write without mut ref
use std::fmt;
use std::io::{self, Read, Write};

#[derive(Debug)]
pub struct MutIo<'a, T: 'a + ?Sized> {
    inner: &'a T,
}

impl<'a, T: ?Sized> MutIo<'a, T> {
    pub fn new(io: &'a T) -> Self {
        MutIo { inner: io }
    }
}

impl<'a, T: Read + ?Sized> Read for MutIo<'a, T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let io = unsafe { &mut *(self.inner as *const _ as *mut T) };
        io.read(buf)
    }
}

impl<'a, T: Write + ?Sized> Write for MutIo<'a, T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let io = unsafe { &mut *(self.inner as *const _ as *mut T) };
        io.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        let io = unsafe { &mut *(self.inner as *const _ as *mut T) };
        io.flush()
    }
}

impl<'a, T: fmt::Write> fmt::Write for MutIo<'a, T> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let io = unsafe { &mut *(self.inner as *const _ as *mut T) };
        io.write_str(s)
    }

    #[inline]
    fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        let io = unsafe { &mut *(self.inner as *const _ as *mut T) };
        fmt::write(io, args)
    }
}
