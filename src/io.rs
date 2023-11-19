use core::convert::Infallible;

pub use embedded_io_async::{ErrorType, Read, Write};

pub struct Cursor<T> {
    inner: T,
    pos: usize,
}

impl<T> Cursor<T> {
    pub fn new(inner: T) -> Self {
        Self { inner, pos: 0 }
    }
}

impl<T: AsRef<[u8]>> Cursor<T> {
    pub fn remaining_slice(&self) -> &[u8] {
        let len = self.pos.min(self.inner.as_ref().len());
        &self.inner.as_ref()[len..]
    }
}

impl<T> ErrorType for Cursor<T>
where
    T: AsRef<[u8]>,
{
    type Error = Infallible;
}

impl<T> Read for Cursor<T>
where
    T: AsRef<[u8]>,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let n = Read::read(&mut self.remaining_slice(), buf).await?;
        self.pos += n;
        Ok(n)
    }
}
