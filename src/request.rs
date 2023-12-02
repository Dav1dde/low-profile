use core::{fmt, mem::MaybeUninit, str::Utf8Error};

use crate::{ErrorType, Method, Read};

pub struct Request<'a, R, P> {
    pub(crate) parts: Parts<'a, P>,
    pub(crate) body: Body<'a, R>,
}

impl<'a, R, P> Request<'a, R, P> {
    pub fn from_parts(parts: Parts<'a, P>, body: Body<'a, R>) -> Self {
        Self { parts, body }
    }

    pub fn into_parts(self) -> (Parts<'a, P>, Body<'a, R>) {
        (self.parts, self.body)
    }

    pub fn method(&self) -> Method<'a> {
        self.parts.method
    }

    pub fn path(&self) -> &'a str {
        self.parts.path
    }

    pub fn body(&self) -> &Body<'a, R> {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut Body<'a, R> {
        &mut self.body
    }

    pub(crate) fn with_extracted_path<P2>(self, extracted_path: P2) -> Request<'a, R, P2> {
        Request {
            parts: self.parts.with_extracted_path(extracted_path),
            body: self.body,
        }
    }
}

impl<'a, R, P> fmt::Debug for Request<'a, R, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Request")
            .field("method", &self.parts.method)
            .field("path", &self.parts.path)
            .field("query", &self.parts.query)
            .finish_non_exhaustive()
    }
}

pub struct Parts<'a, P> {
    pub method: Method<'a>,
    pub path: &'a str,
    pub query: Option<&'a str>,
    pub headers: Headers<'a>,
    pub(crate) extracted_path: P,
}

impl<'a, P> Parts<'a, P> {
    fn with_extracted_path<P2>(self, extracted_path: P2) -> Parts<'a, P2> {
        Parts {
            method: self.method,
            path: self.path,
            query: self.query,
            headers: self.headers,
            extracted_path,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Headers<'a> {
    pub(crate) buf: &'a [u8],
    pub(crate) headers: &'a [HeaderIndices],
}

impl<'a> Headers<'a> {
    fn try_iter(&self) -> impl Iterator<Item = Result<(&'a str, &'a str), Utf8Error>> {
        self.headers.iter().map(|indices| {
            Ok((
                // SAFETY: we converted from str to indices, so we can convert back to str
                unsafe {
                    core::str::from_utf8_unchecked(&self.buf[indices.name.0..indices.name.1])
                },
                core::str::from_utf8(&self.buf[indices.value.0..indices.value.1])?,
            ))
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'a str, &'a str)> + 'a {
        self.try_iter().flatten()
    }

    pub fn get_first(&self, key: &str) -> Option<&'a str> {
        self.iter()
            .find_map(|(header_key, value)| key.eq_ignore_ascii_case(header_key).then_some(value))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct HeaderIndices {
    pub name: (usize, usize),
    pub value: (usize, usize),
}

// Shamelessly stolen from Hyper
pub(crate) fn record_header_indices(
    bytes: &[u8],
    headers: &[httparse::Header<'_>],
    indices: &mut [MaybeUninit<HeaderIndices>],
) {
    let bytes_ptr = bytes.as_ptr() as usize;

    for (header, indices) in headers.iter().zip(indices.iter_mut()) {
        if header.name.len() >= (1 << 16) {
            todo!("return error too large");
        }
        let name_start = header.name.as_ptr() as usize - bytes_ptr;
        let name_end = name_start + header.name.len();
        let value_start = header.value.as_ptr() as usize - bytes_ptr;
        let value_end = value_start + header.value.len();

        indices.write(HeaderIndices {
            name: (name_start, name_end),
            value: (value_start, value_end),
        });
    }
}

pub struct Body<'a, R> {
    content_length: usize,
    buf: &'a [u8],
    reader: R,
}

impl<'a, R: Read> Body<'a, R> {
    pub(crate) fn new(content_length: usize, buf: &'a [u8], reader: R) -> Self {
        Self {
            content_length,
            buf,
            reader,
        }
    }
}

impl<'a, R: ErrorType> ErrorType for Body<'a, R> {
    type Error = R::Error;
}

impl<'a, R: Read> Read for Body<'a, R> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, R::Error> {
        if self.content_length == 0 {
            return Ok(0);
        }

        let read = if !self.buf.is_empty() {
            let mut reader = &self.buf[..self.content_length.min(self.buf.len())];
            reader.read(buf).await.unwrap_or_else(|x| match x {})
        } else {
            self.reader.read(buf).await?
        };

        self.content_length -= read;
        Ok(read)
    }
}
