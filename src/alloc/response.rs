extern crate alloc;

use alloc::{string::String, vec::Vec};

use crate::{http::StatusCode, io::Cursor, IntoResponse, Response};

impl IntoResponse for Vec<u8> {
    type Body = Cursor<Self>;

    fn into_response(self) -> Response<Self::Body> {
        Response {
            status_code: StatusCode::OK,
            body: Cursor::new(self),
        }
    }
}

impl IntoResponse for String {
    type Body = Cursor<Self>;

    fn into_response(self) -> Response<Self::Body> {
        Response {
            status_code: StatusCode::OK,
            body: Cursor::new(self),
        }
    }
}
