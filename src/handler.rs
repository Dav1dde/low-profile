use core::{future::Future, marker::PhantomData};

use crate::{route::Route, FromRequest, FromRequestParts, IntoResponse, Read, Request};

pub trait Handler<S> {
    async fn call<Body: Read>(&self, req: Request<'_, Body>, state: &S) -> impl IntoResponse;
}

pub trait HandlerFunction<S, Params> {
    async fn call<Body: Read>(&self, req: Request<'_, Body>, state: &S) -> impl IntoResponse;
}

impl<S, Fut, F, Ret> HandlerFunction<S, ()> for F
where
    F: Fn() -> Fut,
    Fut: Future<Output = Ret>,
    Ret: IntoResponse,
{
    async fn call<Body: Read>(&self, _req: Request<'_, Body>, _state: &S) -> impl IntoResponse {
        self().await
    }
}

macro_rules! impl_handler_func {
    (
        [$($ty:ident),*], $last:ident
    ) => {
        #[allow(non_snake_case, unused_mut)]
        impl<S, Fut, F, Ret, M, $($ty,)* $last> HandlerFunction<S, (M, $($ty,)* $last,)> for F
        where
            F: Fn($($ty,)* $last,) -> Fut,
            Fut: Future<Output = Ret>,
            Ret: IntoResponse,
            $($ty: for<'a> FromRequestParts<'a, S>,)*
            $last: for<'a> FromRequest<'a, S, M>,
        {
            #[allow(unused_variables)]
            async fn call<Body: Read>(&self, req: Request<'_, Body>, state: &S) -> impl IntoResponse {
                let (mut parts, body) = req.into_parts();

                $(let $ty = $ty::from_request_parts(&mut parts, state).await;)*

                let req = Request::from_parts(parts, body);
                let $last = $last::from_request(req, state).await;

                self($($ty,)* $last).await
            }
        }
    };
}

#[rustfmt::skip]
macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!([], T1);
        $name!([T1], T2);
        $name!([T1, T2], T3);
        $name!([T1, T2, T3], T4);
        $name!([T1, T2, T3, T4], T5);
        $name!([T1, T2, T3, T4, T5], T6);
        $name!([T1, T2, T3, T4, T5, T6], T7);
        $name!([T1, T2, T3, T4, T5, T6, T7], T8);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8], T9);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], T10);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], T11);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], T12);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], T13);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], T14);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], T15);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], T16);
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
    async fn match_request<'a, Body: Read>(
        &self,
        req: Request<'a, Body>,
        state: &S,
    ) -> crate::route::Decision<'a, impl IntoResponse, Body> {
        crate::route::Decision::Match(self.handler.call(req, state).await)
    }
}
