extern crate alloc;

use alloc::{string::String, vec::Vec};

use crate::{
    extract::{FromRequest, InvalidUtf8, StringRejection, UnknownBodyError, VecRejection},
    Read, Request,
};

impl<'a, S, P> FromRequest<'a, S, P> for Vec<u8> {
    type Rejection = VecRejection;

    async fn from_request<R: Read>(
        mut req: Request<'a, R, P>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let mut buf = Self::new();
        read_to_end(req.body_mut(), &mut buf)
            .await
            .map_err(|_| UnknownBodyError)?;
        Ok(buf)
    }
}

impl<'a, S, P> FromRequest<'a, S, P> for String {
    type Rejection = StringRejection;

    async fn from_request<R: Read>(
        req: Request<'a, R, P>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let data = Vec::<u8>::from_request(req, &()).await?;
        Ok(Self::from_utf8(data).map_err(|_| InvalidUtf8)?)
    }
}

async fn read_to_end<R: Read>(r: &mut R, buf: &mut Vec<u8>) -> Result<usize, R::Error> {
    let start_len = buf.len();

    let mut current = start_len;
    loop {
        if buf.len() == buf.capacity() {
            buf.reserve(32);
            buf.resize(buf.capacity(), 0);
        }

        // TODO: there is an optimization here when we have a perfect fit, we dont have to resize
        // the buffer.
        let read = r.read(&mut buf[current..]).await?;
        current += read;

        if read == 0 {
            break;
        }
    }

    buf.truncate(current);
    Ok(current - start_len)
}
