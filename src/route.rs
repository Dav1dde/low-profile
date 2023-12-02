use core::future::Future;

use crate::{either::Either, handler, http, IntoResponse, PathSegments, Read, Request, Response};

macro_rules! impl_handler_func {
    ($name:ident, $method:ident) => {
        pub fn $name<H, S, P, FuncParams>(handler: H) -> impl Route<S, P>
        where
            H: handler::HandlerFunction<S, P, FuncParams>,
        {
            Method {
                method: $crate::http::Method::$method,
                route: $crate::handler::HandlerFunctionHandlerAdapter {
                    handler,
                    _params: Default::default(),
                },
            }
        }
    };
}

impl_handler_func!(get, GET);
impl_handler_func!(post, POST);
impl_handler_func!(put, PUT);
impl_handler_func!(delete, DELETE);
impl_handler_func!(head, HEAD);
impl_handler_func!(options, OPTIONS);
impl_handler_func!(connect, CONNECT);
impl_handler_func!(patch, PATCH);
impl_handler_func!(trace, TRACE);

pub enum Decision<'a, T, R, P> {
    Match(T),
    NoMatch(Request<'a, R, P>),
}

impl<'a, T, R, P> Decision<'a, T, R, P> {
    fn erase(self) -> Decision<'a, T, R, ()> {
        match self {
            Decision::Match(t) => Decision::Match(t),
            Decision::NoMatch(req) => Decision::NoMatch(req.with_extracted_path(())),
        }
    }

    fn map<F, U>(self, f: F) -> Decision<'a, U, R, P>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Match(t) => Decision::Match(f(t)),
            Self::NoMatch(req) => Decision::NoMatch(req),
        }
    }

    #[track_caller]
    pub(crate) fn unwrap(self) -> T {
        match self {
            Decision::Match(t) => t,
            Decision::NoMatch(..) => panic!("unwrap"),
        }
    }
}

pub trait Route<S, P = ()> {
    type Response: IntoResponse;

    fn match_request<'a, Body: Read>(
        &'a self,
        req: Request<'a, Body, P>,
        state: &'a S,
    ) -> impl Future<Output = Decision<'a, Self::Response, Body, P>>;
}

// impl<S, T: Handler<S>> Route<S> for T {
//     async fn match_request<'a, Body: Read>(
//         &self,
//         req: Request<'a, Body>,
//         state: S,
//     ) -> Decision<'a, impl IntoResponse, Body, S> {
//         Decision::Match(self.call(req, state).await)
//     }
// }

pub struct NotFound;

impl<S, P> Route<S, P> for NotFound {
    type Response = Response<&'static [u8]>;

    async fn match_request<'a, Body: Read>(
        &'a self,
        _req: Request<'a, Body, P>,
        _state: &'a S,
    ) -> Decision<'a, Self::Response, Body, P> {
        Decision::Match((http::StatusCode::NOT_FOUND, "Not Found").into_response())
    }
}

pub struct Path<P, R> {
    pub(crate) path: P,
    pub(crate) route: R,
}

impl<S, P: PathSegments, R: Route<S, P::Output>> Route<S, ()> for Path<P, R> {
    type Response = R::Response;

    async fn match_request<'a, Body: Read>(
        &'a self,
        req: Request<'a, Body, ()>,
        state: &'a S,
    ) -> Decision<'a, Self::Response, Body, ()> {
        if let Some(path) = self.path.parse(req.path()) {
            self.route
                .match_request(req.with_extracted_path(path), state)
                .await
                .erase()
        } else {
            Decision::NoMatch(req)
        }
    }
}

pub struct Method<R> {
    pub(crate) method: http::Method<'static>,
    pub(crate) route: R,
}

impl<S, P, R: Route<S, P>> Route<S, P> for Method<R> {
    type Response = R::Response;

    async fn match_request<'a, Body: Read>(
        &'a self,
        req: Request<'a, Body, P>,
        state: &'a S,
    ) -> Decision<'a, Self::Response, Body, P> {
        if self.method == req.method() {
            self.route.match_request(req, state).await
        } else {
            Decision::NoMatch(req)
        }
    }
}

pub struct Fallback<T, S> {
    pub(crate) route: T,
    pub(crate) fallback: S,
}

impl<S, P, R1: Route<S, P>, R2: Route<S, P>> Route<S, P> for Fallback<R1, R2> {
    type Response = Either<R1::Response, R2::Response>;

    async fn match_request<'a, Body: Read>(
        &'a self,
        req: Request<'a, Body, P>,
        state: &'a S,
    ) -> Decision<'a, Self::Response, Body, P> {
        match self.route.match_request(req, state).await {
            Decision::Match(t) => return Decision::Match(Either::Left(t)),
            Decision::NoMatch(req) => self
                .fallback
                .match_request(req, state)
                .await
                .map(Either::Right),
        }
    }
}
