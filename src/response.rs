use crate::{io::Cursor, Read};

pub struct Response<Body> {
    status_code: u16,
    body: Body,
}

impl<Body> Response<Body> {
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn into_body(self) -> Body {
        self.body
    }
}

impl<Body> Response<Body> {
    pub(crate) fn map_body<F, T>(self, map: F) -> Response<T>
    where
        F: FnOnce(Body) -> T,
    {
        Response {
            status_code: self.status_code,
            body: map(self.body),
        }
    }
}

pub trait IntoResponse {
    type Body: Read;

    fn into_response(self) -> Response<Self::Body>;
}

impl<Body: Read> IntoResponse for Response<Body> {
    type Body = Body;

    fn into_response(self) -> Response<Self::Body> {
        self
    }
}

impl IntoResponse for &'static str {
    type Body = &'static [u8];

    fn into_response(self) -> Response<Self::Body> {
        Response {
            status_code: 200,
            body: self.as_bytes(),
        }
    }
}

impl<T: IntoResponse> IntoResponse for (u16, T) {
    type Body = T::Body;

    fn into_response(self) -> Response<Self::Body> {
        let mut response = self.1.into_response();
        response.status_code = self.0;
        response
    }
}

impl IntoResponse for () {
    type Body = &'static [u8];

    fn into_response(self) -> Response<Self::Body> {
        (200, "").into_response()
    }
}

impl<const SIZE: usize> IntoResponse for heapless::Vec<u8, SIZE> {
    type Body = Cursor<Self>;

    fn into_response(self) -> Response<Self::Body> {
        Response {
            status_code: 200,
            body: Cursor::new(self),
        }
    }
}

impl<const SIZE: usize> IntoResponse for heapless::String<SIZE> {
    type Body = Cursor<Self>;

    fn into_response(self) -> Response<Self::Body> {
        Response {
            status_code: 200,
            body: Cursor::new(self),
        }
    }
}
