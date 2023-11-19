use core::fmt::Debug;

use embedded_io_async::{Error, ErrorKind};

use crate::{ErrorType, IntoResponse, Read, Response};

pub enum Either<L, R> {
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

impl<L: ErrorType, R: ErrorType> ErrorType for Either<L, R> {
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

impl<L: IntoResponse, R: IntoResponse> IntoResponse for Either<L, R> {
    type Body = Either<L::Body, R::Body>;

    fn into_response(self) -> Response<Self::Body> {
        match self {
            Either::Left(left) => left.into_response().map_body(Either::Left),
            Either::Right(right) => right.into_response().map_body(Either::Right),
        }
    }
}
