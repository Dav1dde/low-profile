use super::FromRequestParts;
use crate::{Headers, Parts};

impl<'a, S> FromRequestParts<'a, S> for Headers<'a> {
    async fn from_request_parts(parts: &mut Parts<'a>, _state: &S) -> Headers<'a> {
        parts.headers
    }
}

pub struct State<S>(pub S);

impl<'a, S, T> FromRequestParts<'a, S> for State<T>
where
    T: FromRef<S>,
{
    async fn from_request_parts(_parts: &mut Parts<'a>, state: &S) -> Self {
        State(T::from_ref(state))
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
