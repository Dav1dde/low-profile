use core::{
    fmt,
    fmt::Debug,
    future::Future,
    task::{Context, Poll},
};

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
        struct ImmediateAsyncWrite<'a, T: Write> {
            sink: &'a mut T,
            spill: Vec<u8, 128>,
            error: Option<T::Error>,
        }

        impl<'a, T: Write> fmt::Write for ImmediateAsyncWrite<'a, T> {
            fn write_str(&mut self, data: &str) -> fmt::Result {
                let mut data = data.as_bytes();

                // Previously already encountered an error, just yield immediately.
                if self.error.is_some() {
                    return Err(fmt::Error);
                }

                if self.spill.is_empty() {
                    loop {
                        let fut = self.sink.write(data);
                        // See if data can be immediately written,
                        // if it fails (future returns pending), use the spill buffer instead.
                        match now_or_never(fut) {
                            Some(Err(err)) => {
                                self.error = Some(err);
                                return Err(fmt::Error);
                            }
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
                    error: None,
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

            /// Consumes a saved, previously encountered error in the write call.
            ///
            /// An error needs to be consumed before an additional write call
            /// can succeed.
            ///
            /// This allows access to the full details of the write error.
            fn consume_error(&mut self) -> Result<(), T::Error> {
                self.error.take().map(Err).unwrap_or(Ok(()))
            }
        }

        let mut iw = ImmediateAsyncWrite::new(self);
        if fmt::write(&mut iw, fmt).is_err() {
            iw.consume_error()?;
            return Err(WriteFmtError::FmtError);
        }
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

/// Re-Implementation of [`futures::FuturesExt::now_or_never`].
///
/// Evaluates and consumes the future, returning the resulting output
/// after the first call to [`Future::poll`].
///
/// Copied here to minimize dependencies.
fn now_or_never<F>(future: F) -> Option<F::Output>
where
    F: Future,
{
    let noop_waker = noop::noop_waker();
    let mut cx = Context::from_waker(&noop_waker);

    let future = core::pin::pin!(future);
    match future.poll(&mut cx) {
        Poll::Ready(x) => Some(x),
        _ => None,
    }
}

mod noop {
    use core::task::{RawWaker, RawWakerVTable, Waker};

    #[inline]
    pub const fn noop_waker() -> Waker {
        unsafe { Waker::from_raw(noop_raw_waker()) }
    }

    unsafe fn noop(_data: *const ()) {}
    unsafe fn noop_clone(_data: *const ()) -> RawWaker {
        noop_raw_waker()
    }

    const NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);

    const fn noop_raw_waker() -> RawWaker {
        RawWaker::new(core::ptr::null(), &NOOP_WAKER_VTABLE)
    }
}
