use crate::{request::Parts, Read, Request};

mod request;
mod request_parts;

pub use request_parts::{FromRef, State};

mod private {
    #[derive(Debug, Clone, Copy)]
    pub enum ViaParts {}

    #[derive(Debug, Clone, Copy)]
    pub enum ViaRequest {}
}

pub trait FromRequestParts<'a, S>: Sized {
    async fn from_request_parts(parts: &mut Parts<'a>, state: &S) -> Self;
}

pub trait FromRequest<'a, S, M = private::ViaRequest>: Sized {
    async fn from_request<R: Read>(req: Request<'a, R>, state: &S) -> Self;
}

impl<'a, S, T> FromRequest<'a, S, private::ViaParts> for T
where
    T: FromRequestParts<'a, S>,
{
    async fn from_request<R: Read>(req: Request<'a, R>, state: &S) -> Self {
        let (mut parts, _) = req.into_parts();
        Self::from_request_parts(&mut parts, state).await
    }
}
