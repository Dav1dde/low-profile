use core::convert::Infallible;

use super::FromRequest;
use crate::{Read, Request};

impl<'a, const SIZE: usize, S> FromRequest<'a, S> for heapless::Vec<u8, SIZE> {
    type Rejection = Infallible;

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
                .expect("TODO");
            if read == 0 {
                break;
            }

            current += read;
            if current == data.len() {
                // buffer is completely full, read one more byte to check for EoF
                let mut eof = [0u8; 1];
                let read = req.body_mut().read(&mut eof).await.unwrap();
                if read > 0 {
                    todo!("error not eof");
                }
                break;
            }
        }

        data.truncate(current);
        Ok(data)
    }
}

impl<'a, const SIZE: usize, S> FromRequest<'a, S> for heapless::String<SIZE> {
    type Rejection = Infallible;

    async fn from_request<R: Read>(
        req: Request<'a, R>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let data = heapless::Vec::<u8, SIZE>::from_request(req, &()).await?;
        Ok(Self::from_utf8(data).expect("TODO"))
    }
}
