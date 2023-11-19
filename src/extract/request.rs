use super::{
    BodyTooLarge, FromRequest, InvalidUtf8, StringRejection, UnknownBodyError, VecRejection,
};
use crate::{Read, Request};

impl<'a, const SIZE: usize, S> FromRequest<'a, S> for heapless::Vec<u8, SIZE> {
    type Rejection = VecRejection;

    async fn from_request<R: Read>(
        mut req: Request<'a, R>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let mut data = Self::default();
        data.resize_default(data.capacity()).unwrap();

        let mut current = 0;
        loop {
            let read = req
                .body_mut()
                .read(&mut data[current..])
                .await
                .map_err(|_| UnknownBodyError)?;
            if read == 0 {
                break;
            }

            current += read;
            if current == data.len() {
                // buffer is completely full, read one more byte to check for EoF
                let mut eof = [0u8; 1];
                let read = req
                    .body_mut()
                    .read(&mut eof)
                    .await
                    .map_err(|_| UnknownBodyError)?;
                if read > 0 {
                    return Err(BodyTooLarge.into());
                }
                break;
            }
        }

        data.truncate(current);
        Ok(data)
    }
}

impl<'a, const SIZE: usize, S> FromRequest<'a, S> for heapless::String<SIZE> {
    type Rejection = StringRejection;

    async fn from_request<R: Read>(
        req: Request<'a, R>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let data = heapless::Vec::<u8, SIZE>::from_request(req, &()).await?;
        Ok(Self::from_utf8(data).map_err(|_| InvalidUtf8)?)
    }
}
