use core::{future::Future, marker::PhantomData};

use crate::{
    either::Either, route::Route, FromRequest, FromRequestParts, IntoResponse, Read, Request,
};

pub trait Handler<S> {
    type Response: IntoResponse;

    fn call<Body: Read>(
        &self,
        req: Request<'_, Body>,
        state: &S,
    ) -> impl Future<Output = Self::Response>;
}

pub trait HandlerFunction<S, Params> {
    type Response: IntoResponse;

    fn call<Body: Read>(
        &self,
        req: Request<'_, Body>,
        state: &S,
    ) -> impl Future<Output = Self::Response>;
}

impl<S, Fut, F, Ret> HandlerFunction<S, ()> for F
where
    F: Fn() -> Fut,
    Fut: Future<Output = Ret>,
    Ret: IntoResponse,
{
    type Response = Ret;

    async fn call<Body: Read>(&self, _req: Request<'_, Body>, _state: &S) -> Self::Response {
        self().await
    }
}

// TODO: clean this up, should be able to get rid of quite a bit duplicated code
macro_rules! impl_handler_func_inner_extract {
    ($parts:ident, $state:ident,) => {
    };
    ($parts:ident, $state:ident, $v:ident) => {
        let $v = match $v::from_request_parts(&mut $parts, $state).await {
            Ok(o) => o,
            Err(err) => return impl_handler_func_inner_extract!(@builderr-right, ($v), err),
        };
    };
    ($parts:ident, $state:ident, $v:ident, $($x:tt)*) => {
        let $v = match $v::from_request_parts(&mut $parts, $state).await {
            Ok(o) => o,
            Err(err) => return impl_handler_func_inner_extract!(@builderr-right, ($v, $($x)*), err),
        };

        impl_handler_func_inner_extract!(@cont, $parts, $state, $($x)*)
    };

    (@cont, $parts:ident, $state:ident, $v:ident) => {
        let $v = match $v::from_request_parts(&mut $parts, $state).await {
            Ok(o) => o,
            Err(err) => return impl_handler_func_inner_extract!(@builderr, ($v), err),
        };
    };
    (@cont, $parts:ident, $state:ident, $v:ident, $($x:tt)*) => {
        let $v = match $v::from_request_parts(&mut $parts, $state).await {
            Ok(o) => o,
            Err(err) => return impl_handler_func_inner_extract!(@builderr, ($v, $($x)*), err),
        };

        impl_handler_func_inner_extract!(@cont, $parts, $state, $($x)*)
    };

    (@builderr-right, ($v:ident), $err:expr) => { Either::Right(Either::Right($err)) };
    (@builderr-right, ($v:ident, $($x:tt)*), $err:expr) => { Either::Right(
        impl_handler_func_inner_extract!(@builderr-right, ($($x)*), $err)
    )};

    (@builderr, ($v:ident), $err:expr) => { Either::Right(Either::Right(Either::Left($err))) };
    (@builderr, ($v:ident, $($x:tt)*), $err:expr) => { Either::Right(
        impl_handler_func_inner_extract!(@builderr, ($($x)*), $err)
    )};

    (@builderr-last, (), $err:expr) => { Either::Right($err) };
    (@builderr-last, ($($x:tt)*), $err:expr) => { Either::Right(Either::Left($err))};

    (@buildty, $ret:ident, (), $lol:ident) => {
        Either<$ret, $lol>
    };
    (@buildty, $ret:ident, ($($x:tt,)*), $lol:ident) => {
        Either<$ret, Either<$lol, impl_handler_func_inner_extract!(@build-reverse-either, [$($x),*])>>
    };

    // build an either from tokens
    (@build-either, $v:ident, $($x:tt),*) => {
        Either<$v, impl_handler_func_inner_extract!(@build-either, $($x),*)>
    };
    (@build-either, $v:ident) => { $v };

    // build a reverse either from tokens contained within []
    (@build-reverse-either, [] reversed: [$($x:ident),*]) => {
        impl_handler_func_inner_extract!(@build-either, $($x),*)
    };
    (@build-reverse-either, [$($x:ident),* $(,)?]) => {
        impl_handler_func_inner_extract!(@build-reverse-either, [$($x),*] reversed: [])
    };
    (@build-reverse-either, [$head:ident $(, $tail:ident)*] reversed: [$($reversed:ident),*]) => {
        impl_handler_func_inner_extract!(@build-reverse-either, [$($tail),*] reversed: [$head $(, $reversed)*])
    };
}

macro_rules! impl_handler_func {
    (
        [$(($ty:ident, $ty_err:ident)),*], ($last:ident, $last_err:ident)
    ) => {
        #[allow(non_snake_case, unused_mut)]
        impl<
            S,
            Fut,
            F,
            Ret,
            M,
            $($ty, $ty_err,)*
            $last, $last_err,
        > HandlerFunction<S, (M, $($ty,)* $last)> for F
        where
            F: Fn($($ty,)* $last,) -> Fut,
            Fut: Future<Output = Ret>,
            Ret: IntoResponse,
            $($ty: for<'a> FromRequestParts<'a, S, Rejection = $ty_err>, $ty_err: IntoResponse,)*
            $last: for<'a> FromRequest<'a, S, M, Rejection = $last_err>,
            $last_err: IntoResponse,
        {
            type Response = impl_handler_func_inner_extract!(@buildty, Ret, ($($ty_err,)*), $last_err);

            #[allow(unused_variables)]
            async fn call<Body: Read>(&self, req: Request<'_, Body>, state: &S) -> Self::Response {
                let (mut parts, body) = req.into_parts();

                impl_handler_func_inner_extract!(parts, state, $($ty),*);

                let req = Request::from_parts(parts, body);
                let $last = match $last::from_request(req, state).await {
                    Ok(r) => r,
                    Err(err) => return impl_handler_func_inner_extract!(@builderr-last, ($($ty,)*), err),
                };

                Either::Left(self($($ty,)* $last).await)
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!([], (T1, T1E));
        $name!([(T1, T1E)], (T2, T2E));
        $name!([(T1, T1E), (T2, T2E)], (T3, T3E));
        $name!([(T1, T1E), (T2, T2E), (T3, T3E)], (T4, T4E));
        $name!([(T1, T1E), (T2, T2E), (T3, T3E), (T4, T4E)], (T5, T5E));
        $name!([(T1, T1E), (T2, T2E), (T3, T3E), (T4, T4E), (T5, T5E)], (T6, T6E));
        $name!([(T1, T1E), (T2, T2E), (T3, T3E), (T4, T4E), (T5, T5E), (T6, T6E)], (T7, T7E));
        $name!([(T1, T1E), (T2, T2E), (T3, T3E), (T4, T4E), (T5, T5E), (T6, T6E), (T7, T7E)], (T8, T8E));
        $name!([(T1, T1E), (T2, T2E), (T3, T3E), (T4, T4E), (T5, T5E), (T6, T6E), (T7, T7E), (T8, T8E)], (T9, T9E));
        $name!([(T1, T1E), (T2, T2E), (T3, T3E), (T4, T4E), (T5, T5E), (T6, T6E), (T7, T7E), (T8, T8E), (T9, T9E)], (T10, T10E));
        $name!([(T1, T1E), (T2, T2E), (T3, T3E), (T4, T4E), (T5, T5E), (T6, T6E), (T7, T7E), (T8, T8E), (T9, T9E), (T10, T10E)], (T11, T11E));
        $name!([(T1, T1E), (T2, T2E), (T3, T3E), (T4, T4E), (T5, T5E), (T6, T6E), (T7, T7E), (T8, T8E), (T9, T9E), (T10, T10E), (T11, T11E)], (T12, T12E));
        $name!([(T1, T1E), (T2, T2E), (T3, T3E), (T4, T4E), (T5, T5E), (T6, T6E), (T7, T7E), (T8, T8E), (T9, T9E), (T10, T10E), (T11, T11E), (T12, T12E)], (T13, T13E));
        $name!([(T1, T1E), (T2, T2E), (T3, T3E), (T4, T4E), (T5, T5E), (T6, T6E), (T7, T7E), (T8, T8E), (T9, T9E), (T10, T10E), (T11, T11E), (T12, T12E), (T13, T13E)], (T14, T14E));
        $name!([(T1, T1E), (T2, T2E), (T3, T3E), (T4, T4E), (T5, T5E), (T6, T6E), (T7, T7E), (T8, T8E), (T9, T9E), (T10, T10E), (T11, T11E), (T12, T12E), (T13, T13E), (T14, T14E)], (T15, T15E));
        $name!([(T1, T1E), (T2, T2E), (T3, T3E), (T4, T4E), (T5, T5E), (T6, T6E), (T7, T7E), (T8, T8E), (T9, T9E), (T10, T10E), (T11, T11E), (T12, T12E), (T13, T13E), (T14, T14E), (T15, T15E)], (T16, T16E));
    };
}

all_the_tuples!(impl_handler_func);

pub(crate) struct HandlerFunctionHandlerAdapter<FuncParams, Handler> {
    pub handler: Handler,
    pub _params: PhantomData<FuncParams>,
}

impl<S, FuncParams, H> Route<S> for HandlerFunctionHandlerAdapter<FuncParams, H>
where
    H: HandlerFunction<S, FuncParams>,
{
    type Response = H::Response;

    async fn match_request<'a, Body: Read>(
        &'a self,
        req: Request<'a, Body>,
        state: &'a S,
    ) -> crate::route::Decision<'a, Self::Response, Body> {
        crate::route::Decision::Match(self.handler.call(req, state).await)
    }
}
