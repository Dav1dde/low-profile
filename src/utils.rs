use core::{fmt, fmt::Debug};

use futures_util::FutureExt;
use heapless::Vec;

use crate::Write;

/// Adapter to use the `write!()` macro with the async `Write` trait.
///
/// This carries an internal overflow buffer of 128 bytes.
/// Writes exceeding the buffer may fail.
pub(crate) trait WriteExt: Write
where
    Self: Sized,
{
    async fn write_fmt(
        &mut self,
        fmt: fmt::Arguments<'_>,
    ) -> Result<(), WriteFmtError<Self::Error>> {
        struct ImmediateAsyncWrite<'a, T> {
            sink: &'a mut T,
            spill: Vec<u8, 128>,
        }

        impl<'a, T: Write> fmt::Write for ImmediateAsyncWrite<'a, T> {
            fn write_str(&mut self, data: &str) -> fmt::Result {
                let mut data = data.as_bytes();

                if self.spill.is_empty() {
                    loop {
                        let fut = self.sink.write(data);
                        let fut = core::pin::pin!(fut);
                        // See if data can be immediately written,
                        // if it fails (future returns pending), use the spill buffer instead.
                        match fut.now_or_never() {
                            // TODO: carry error out somehow, probably through self.error
                            Some(Err(_todo)) => return Err(fmt::Error),
                            Some(Ok(size)) if size == data.len() => return Ok(()),
                            Some(Ok(size)) => data = &data[size..],
                            None => break,
                        }
                    }
                }

                self.spill.extend_from_slice(data).map_err(|_| fmt::Error)
            }
        }

        impl<'a, T: Write> ImmediateAsyncWrite<'a, T> {
            fn new(sink: &'a mut T) -> Self {
                Self {
                    sink,
                    spill: Vec::new(),
                }
            }

            /// Consumes the remaining data in the spill buffer
            /// and writes it to the sink.
            async fn consume(self) -> Result<(), T::Error> {
                if self.spill.is_empty() {
                    return Ok(());
                }

                self.sink.write_all(&self.spill).await.map(|_| ())
            }
        }

        let mut iw = ImmediateAsyncWrite::new(self);
        fmt::write(&mut iw, fmt).map_err(|_| WriteFmtError::FmtError)?;
        iw.consume().await?;

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
