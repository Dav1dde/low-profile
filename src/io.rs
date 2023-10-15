use core::{convert::Infallible, fmt::Debug};

pub use embedded_io::{
    asynch::{Read, Write},
    Io,
};
use embedded_io::{Error, ErrorKind};

pub(crate) enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L: Debug, R: Debug> Debug for Either<L, R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Either::Left(left) => left.fmt(f),
            Either::Right(right) => right.fmt(f),
        }
    }
}

impl<L: Error, R: Error> Error for Either<L, R> {
    fn kind(&self) -> ErrorKind {
        match self {
            Either::Left(left) => left.kind(),
            Either::Right(right) => right.kind(),
        }
    }
}

impl<L: Io, R: Io> Io for Either<L, R> {
    type Error = Either<L::Error, R::Error>;
}

impl<L: Read, R: Read> Read for Either<L, R> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        match self {
            Either::Left(left) => left.read(buf).await.map_err(Either::Left),
            Either::Right(right) => right.read(buf).await.map_err(Either::Right),
        }
    }
}

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

impl<T> Io for Cursor<T>
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
