use core::future::Future;

use crate::{Read, Write};

pub enum ServiceError<E, E2> {
    Io(E),
    Lol(E2),
}

pub trait Service {
    type Error;

    fn serve<R: Read, W: Write<Error = R::Error>>(
        &self,
        reader: R,
        writer: W,
    ) -> impl Future<Output = Result<(), ServiceError<R::Error, Self::Error>>>;
}
