extern crate alloc;

use alloc::vec::Vec;

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    either::Either,
    extract::{JsonError, JsonRejection},
    http::StatusCode,
    io::Cursor,
    FromRequest, IntoResponse, Read, Request, Response,
};

pub struct Json<T>(pub T);

impl<'a, S, P, T> FromRequest<'a, S, P> for Json<T>
where
    T: DeserializeOwned,
{
    type Rejection = JsonRejection;

    async fn from_request<R: Read>(
        req: Request<'a, R, P>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // TODO: check headers
        let data = Vec::from_request(req, state).await?;
        Ok(Json(serde_json::from_slice(&data).map_err(|_| JsonError)?))
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    type Body = Either<Cursor<Vec<u8>>, &'static [u8]>;

    fn into_response(self) -> Response<Self::Body> {
        let buf = match serde_json::to_vec(&self.0) {
            Ok(buf) => buf,
            Err(_err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to serialize JSON",
                )
                    .into_response()
                    .map_body(Either::Right);
            }
        };
        (StatusCode::OK, buf).into_response().map_body(Either::Left)
    }
}
