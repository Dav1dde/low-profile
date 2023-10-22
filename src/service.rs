use core::future::Future;

use crate::{Read, Write};

pub trait Service {
    fn serve<R: Read, W: Write<Error = R::Error>>(
        &self,
        reader: R,
        writer: W,
    ) -> impl Future<Output = ()>;
}
