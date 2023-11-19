use core::future::Future;

use crate::{error::ProtocolError, Read, Write};

#[derive(Debug)]
pub enum ServiceError<IO, BODY> {
    ProtocolError(ProtocolError),
    Io(IO),
    Body(BODY),
}

impl<IO: embedded_io_async::Error, BODY: embedded_io_async::Error> embedded_io_async::Error
    for ServiceError<IO, BODY>
{
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            Self::Io(err) => err.kind(),
            Self::Body(err) => err.kind(),
            Self::ProtocolError(..) => embedded_io_async::ErrorKind::Other,
        }
    }
}

pub trait Service {
    // TODO: this should come from crate::io or somewhere else
    type BodyError: embedded_io_async::Error;

    fn serve<R: Read, W: Write<Error = R::Error>>(
        &self,
        reader: R,
        writer: W,
    ) -> impl Future<Output = Result<(), ServiceError<R::Error, Self::BodyError>>>;
}
