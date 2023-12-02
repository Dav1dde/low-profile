use crate::{http::StatusCode, io::Cursor, IntoResponse, Response};

impl<const SIZE: usize> IntoResponse for heapless::Vec<u8, SIZE> {
    type Body = Cursor<Self>;

    fn into_response(self) -> Response<Self::Body> {
        Response {
            status_code: StatusCode::OK,
            body: Cursor::new(self),
        }
    }
}

impl<const SIZE: usize> IntoResponse for heapless::String<SIZE> {
    type Body = Cursor<Self>;

    fn into_response(self) -> Response<Self::Body> {
        Response {
            status_code: StatusCode::OK,
            body: Cursor::new(self),
        }
    }
}
