use core::future::Future;

use crate::{request::Parts, IntoResponse, Read, Request};

mod rejections;
mod request_parts;

pub use rejections::*;
pub use request_parts::*;

mod private {
    #[derive(Debug, Clone, Copy)]
    pub enum ViaParts {}

    #[derive(Debug, Clone, Copy)]
    pub enum ViaRequest {}
}

pub trait FromRequestParts<'a, S, P>: Sized {
    type Rejection: IntoResponse;

    fn from_request_parts(
        parts: &mut Parts<'a, P>,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>>;
}

pub trait FromRequest<'a, S, P, M = private::ViaRequest>: Sized {
    type Rejection: IntoResponse;

    fn from_request<R: Read>(
        req: Request<'a, R, P>,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>>;
}

impl<'a, S, P, T> FromRequest<'a, S, P, private::ViaParts> for T
where
    T: FromRequestParts<'a, S, P>,
{
    type Rejection = T::Rejection;

    fn from_request<R: Read>(
        req: Request<'a, R, P>,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> {
        let (mut parts, _) = req.into_parts();
        async move { Self::from_request_parts(&mut parts, state).await }
    }
}
