use core::{convert::Infallible, future::Future};

use super::FromRequestParts;
use crate::{Headers, Parts};

impl<'a, S, P> FromRequestParts<'a, S, P> for Headers<'a> {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts<'a, P>,
        _state: &S,
    ) -> Result<Headers<'a>, Self::Rejection> {
        Ok(parts.headers)
    }
}

pub struct State<S>(pub S);

impl<'a, S, P, T> FromRequestParts<'a, S, P> for State<T>
where
    T: FromRef<S>,
{
    type Rejection = Infallible;

    fn from_request_parts(
        _parts: &mut Parts<'a, P>,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> {
        let val = T::from_ref(state);
        async move { Ok(State(val)) }
    }
}

pub trait FromRef<T> {
    fn from_ref(input: &T) -> Self;
}

impl<T: Clone> FromRef<T> for T {
    fn from_ref(input: &T) -> Self {
        input.clone()
    }
}

pub struct Path<P>(pub P);

impl<'a, S, P> FromRequestParts<'a, S, P> for Path<P>
where
    P: Clone,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts<'a, P>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // TODO: can we pass a reference here, and or use serde? Can the extracted path keep
        // references into the original path?
        Ok(Self(parts.extracted_path.clone()))
    }
}
