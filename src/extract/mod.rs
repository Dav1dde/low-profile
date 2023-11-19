use core::future::Future;

use crate::{request::Parts, IntoResponse, Read, Request};

mod request;
mod request_parts;
mod utils;

pub use request_parts::{FromRef, State};

mod private {
    #[derive(Debug, Clone, Copy)]
    pub enum ViaParts {}

    #[derive(Debug, Clone, Copy)]
    pub enum ViaRequest {}
}

pub trait FromRequestParts<'a, S>: Sized {
    type Rejection: IntoResponse;

    fn from_request_parts(
        parts: &mut Parts<'a>,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>>;
}

pub trait FromRequest<'a, S, M = private::ViaRequest>: Sized {
    type Rejection: IntoResponse;

    fn from_request<R: Read>(
        req: Request<'a, R>,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>>;
}

impl<'a, S, T> FromRequest<'a, S, private::ViaParts> for T
where
    T: FromRequestParts<'a, S>,
{
    type Rejection = T::Rejection;

    fn from_request<R: Read>(
        req: Request<'a, R>,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> {
        let (mut parts, _) = req.into_parts();
        async move { Self::from_request_parts(&mut parts, state).await }
    }
}
