use std::cmp;
use std::io::{self, BufRead, Read};

use bytes::{BufMut, BytesMut};

#[derive(Debug)]
pub struct BufReader<R> {
    inner: R,
    buf: BytesMut,
}

const INIT_BUFFER_SIZE: usize = 4096;

impl<R: Read> BufReader<R> {
    #[inline]
    pub fn new(r: R) -> BufReader<R> {
        BufReader::with_capacity(r, INIT_BUFFER_SIZE)
    }

    /// read some data into internal buffer
    #[inline]
    pub fn bump_read(&mut self) -> io::Result<usize> {
        if self.buf.remaining_mut() == 0 {
            self.buf.reserve(INIT_BUFFER_SIZE);
        }

        let n = self.inner.read(unsafe { self.buf.bytes_mut() })?;
        unsafe { self.buf.advance_mut(n) };
        Ok(n)
    }

    #[inline]
    pub fn with_capacity(r: R, cap: usize) -> BufReader<R> {
        BufReader {
            inner: r,
            buf: BytesMut::with_capacity(cap),
        }
    }

    /// return the intneral buffer
    #[inline]
    pub fn get_buf(&mut self) -> &mut BytesMut {
        &mut self.buf
    }

    // /// convert into inner reader
    // #[inline]
    // pub fn into_inner(self) -> R {
    //     self.inner
    // }
}

impl<R: Read> Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use std::ptr;
        if self.buf.remaining_mut() == 0 {
            self.bump_read()?;
        }

        let len = unsafe {
            let len = cmp::min(buf.len(), self.buf.remaining_mut());
            ptr::copy_nonoverlapping(self.buf.bytes_mut().as_ptr(), buf.as_mut_ptr(), len);
            len
        };

        self.buf.advance(len);
        Ok(len)
    }
}

impl<R: Read> BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.bump_read()?;
        Ok(unsafe { self.buf.bytes_mut() })
    }

    #[inline]
    fn consume(&mut self, amt: usize) {
        self.buf.advance(amt)
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
        let mut rdr = BufReader::new(SlowRead(0));
        rdr.bump_read().unwrap();
        rdr.consume(1);
        assert_eq!(rdr.get_buf().as_ref(), b"oo");
        rdr.bump_read().unwrap();
        rdr.bump_read().unwrap();
        assert_eq!(rdr.get_buf().as_ref(), b"oobarbaz");
        rdr.consume(5);
        assert_eq!(rdr.get_buf().as_ref(), b"baz");
        rdr.consume(3);
        assert_eq!(rdr.get_buf().as_ref(), b"");
    }

    #[test]
    // the minimum size is 31
    fn test_resize() {
        let raw = vec![1u8; 100];
        let mut rdr = BufReader::with_capacity(&raw[..], 65);
        rdr.bump_read().unwrap();
        assert_eq!(rdr.get_buf().len(), 65);
        rdr.bump_read().unwrap();
        assert_eq!(rdr.get_buf().len(), 100);
    }
}
