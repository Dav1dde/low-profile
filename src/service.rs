use core::future::Future;

use crate::{Read, Write};

pub trait Service<R, W>
where
    R: Read,
    W: Write<Error = R::Error>,
{
    fn serve(&self, reader: R, writer: W) -> impl Future<Output = ()>;
}
