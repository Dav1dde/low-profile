use heapless::Vec;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    either::Either,
    extract::{JsonError, JsonRejection},
    http::StatusCode,
    io::Cursor,
    FromRequest, IntoResponse, Read, Request, Response,
};

pub struct Json<T, const N: usize = 1024>(pub T);

impl<T> Json<T, 0> {
    pub fn new<const N: usize>(value: T) -> Json<T, N> {
        Json(value)
    }
}

impl<'a, S, P, T, const N: usize> FromRequest<'a, S, P> for Json<T, N>
where
    T: DeserializeOwned,
{
    type Rejection = JsonRejection;

    async fn from_request<R: Read>(
        req: Request<'a, R, P>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // TODO: check headers
        let data = heapless::Vec::<u8, N>::from_request(req, state).await?;
        let (res, _) = serde_json_core::from_slice(&data).map_err(|_| JsonError)?;
        Ok(Json(res))
    }
}

impl<T, const N: usize> IntoResponse for Json<T, N>
where
    T: Serialize,
{
    type Body = Either<Cursor<heapless::Vec<u8, N>>, &'static [u8]>;

    fn into_response(self) -> Response<Self::Body> {
        let mut buffer = Vec::<u8, N>::new();
        buffer.resize_default(N).unwrap();
        let len = match serde_json_core::to_slice(&self.0, &mut buffer) {
            Ok(len) => len,
            Err(_err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to serialize JSON",
                )
                    .into_response()
                    .map_body(Either::Right);
            }
        };
        buffer.truncate(len);
        // TODO: send headers
        (StatusCode::OK, buffer)
            .into_response()
            .map_body(Either::Left)
    }
}
