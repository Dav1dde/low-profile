use core::{fmt, fmt::Debug};

use heapless::Vec;

use crate::Write;

/// Adapter to use the `write!()` macro with the async `Write` trait.
///
/// This carries an internal buffer size of 128 bytes. Attempting to write
/// more than this will always fail.
pub(crate) trait WriteExt: Write {
    async fn write_fmt(
        &mut self,
        fmt: fmt::Arguments<'_>,
    ) -> Result<(), WriteFmtError<Self::Error>> {
        let mut buf = Vec::<u8, 128>::new();
        fmt::write(&mut buf, fmt).map_err(|_| WriteFmtError::FmtError)?;
        self.write_all(&buf).await?;

        Ok(())
    }
}

impl<W: Write> WriteExt for W {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum WriteFmtError<E> {
    /// An error was encountered while formatting.
    FmtError,
    /// Error returned by the inner Write.
    Other(E),
}

impl<E> From<E> for WriteFmtError<E> {
    fn from(err: E) -> Self {
        Self::Other(err)
    }
}

impl<E: fmt::Debug> fmt::Display for WriteFmtError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
