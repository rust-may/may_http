use std::cmp;
use std::fmt;
use std::io::{self, BufRead, Read, Write};

use bytes::{BufMut, BytesMut};

#[derive(Debug)]
pub struct BufferIo<T> {
    inner: T,
    reader_buf: BytesMut,
    writer_buf: BytesMut,
}

const INIT_BUFFER_SIZE: usize = 1024;

impl<T> BufferIo<T> {
    #[inline]
    pub fn new(io: T) -> Self {
        BufferIo::with_capacity(io, INIT_BUFFER_SIZE)
    }

    #[inline]
    pub fn with_capacity(io: T, cap: usize) -> Self {
        BufferIo {
            inner: io,
            reader_buf: BytesMut::with_capacity(cap),
            writer_buf: BytesMut::with_capacity(cap),
        }
    }
}

impl<T: Read> BufferIo<T> {
    /// read some data into internal buffer
    #[inline]
    pub fn bump_read(&mut self) -> io::Result<usize> {
        if self.reader_buf.remaining_mut() < 32 {
            self.reader_buf.reserve(INIT_BUFFER_SIZE);
        }

        let n = self.inner.read(unsafe { self.reader_buf.bytes_mut() })?;
        unsafe { self.reader_buf.advance_mut(n) };
        Ok(n)
    }

    /// return the intneral buffer
    #[inline]
    pub fn get_reader_buf(&mut self) -> &mut BytesMut {
        &mut self.reader_buf
    }

    // /// convert into inner reader
    // #[inline]
    // pub fn into_inner(self) -> R {
    //     self.inner
    // }
}

impl<T: Read> Read for BufferIo<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use std::ptr;
        if self.reader_buf.remaining_mut() == 0 {
            self.bump_read()?;
        }

        let len = unsafe {
            let len = cmp::min(buf.len(), self.reader_buf.remaining_mut());
            ptr::copy_nonoverlapping(self.reader_buf.bytes_mut().as_ptr(), buf.as_mut_ptr(), len);
            len
        };

        self.reader_buf.advance(len);
        Ok(len)
    }
}

impl<T: Write> Write for BufferIo<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.writer_buf.remaining_mut() == 0 {
            self.flush()?;
        }

        let remain = self.writer_buf.remaining_mut();
        let len = cmp::min(remain, buf.len());
        self.writer_buf.put_slice(&buf[0..len]);
        Ok(len)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        use std::fmt::Write;
        self.writer_buf.write_fmt(fmt).expect("format err");
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.inner.write_all(self.writer_buf.as_ref())?;
        self.writer_buf.clear();
        Ok(())
    }
}

impl<T: Write> fmt::Write for BufferIo<T> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.writer_buf.write_str(s)
    }

    #[inline]
    fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        self.writer_buf.write_fmt(args)
    }
}

impl<T: Read> BufRead for BufferIo<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.bump_read()?;
        Ok(unsafe { self.reader_buf.bytes_mut() })
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        self.reader_buf.advance(amt)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, BufRead, Read};
    use super::*;

    struct SlowRead(u8);

    impl Read for SlowRead {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let state = self.0;
            self.0 += 1;
            (&match state % 3 {
                0 => b"foo",
                1 => b"bar",
                _ => b"baz",
            }[..])
                .read(buf)
        }
    }

    #[test]
    fn test_consume_and_get_buf() {
        let mut rdr = BufferIo::new(SlowRead(0));
        rdr.bump_read().unwrap();
        rdr.consume(1);
        assert_eq!(rdr.get_reader_buf().as_ref(), b"oo");
        rdr.bump_read().unwrap();
        rdr.bump_read().unwrap();
        assert_eq!(rdr.get_reader_buf().as_ref(), b"oobarbaz");
        rdr.consume(5);
        assert_eq!(rdr.get_reader_buf().as_ref(), b"baz");
        rdr.consume(3);
        assert_eq!(rdr.get_reader_buf().as_ref(), b"");
    }

    #[test]
    // the minimum size is 31
    fn test_resize() {
        let raw = vec![1u8; 100];
        let mut rdr = BufferIo::with_capacity(&raw[..], 65);
        rdr.bump_read().unwrap();
        assert_eq!(rdr.get_reader_buf().len(), 65);
        rdr.bump_read().unwrap();
        assert_eq!(rdr.get_reader_buf().len(), 100);
    }


    #[test]
    fn test_write() {
        let data = vec![0u8; 100];
        let mut wrt = BufferIo::with_capacity(io::sink(), 40);
        let n = wrt.write(&data).unwrap();
        assert_eq!(n, 40);
        let n = wrt.write(&[0u8; 6]).unwrap();
        assert_eq!(n, 6);
        let n = wrt.write(&data).unwrap();
        assert_eq!(n, 34);
        let n = wrt.write(&data).unwrap();
        assert_eq!(n, 40);
    }
}
