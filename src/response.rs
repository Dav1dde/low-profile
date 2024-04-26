use crate::{http::StatusCode, Read};

pub struct Response<Body> {
    // TODO: headers
    pub status_code: StatusCode,
    pub body: Body,
}

impl<Body> Response<Body> {
    pub fn status_code(&self) -> StatusCode {
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
    type Body: Read; // TODO: this should probably be a Body trait (with content-length?)

    fn into_response(self) -> Response<Self::Body>;
}

impl IntoResponse for core::convert::Infallible {
    type Body = &'static [u8];

    fn into_response(self) -> Response<Self::Body> {
        match self {}
    }
}

impl<Body: Read> IntoResponse for Response<Body>
where
    Body: 'static,
{
    type Body = Body;

    fn into_response(self) -> Response<Self::Body> {
        self
    }
}

impl IntoResponse for &'static str {
    type Body = &'static [u8];

    fn into_response(self) -> Response<Self::Body> {
        Response {
            status_code: StatusCode::OK,
            body: self.as_bytes(),
        }
    }
}

impl<T: IntoResponse> IntoResponse for (StatusCode, T) {
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
        (StatusCode::OK, "").into_response()
    }
}
