use core::{convert::Infallible, future::Future};

use super::FromRequestParts;
use crate::{Headers, Parts};

impl<'a, S> FromRequestParts<'a, S> for Headers<'a> {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts<'a>,
        _state: &S,
    ) -> Result<Headers<'a>, Self::Rejection> {
        Ok(parts.headers)
    }
}

pub struct State<S>(pub S);

impl<'a, S, T> FromRequestParts<'a, S> for State<T>
where
    T: FromRef<S>,
{
    type Rejection = Infallible;

    fn from_request_parts(
        _parts: &mut Parts<'a>,
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
