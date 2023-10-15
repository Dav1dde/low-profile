use crate::{handler, io::Either, IntoResponse, Read, Request};

macro_rules! impl_handler_func {
    ($name:ident, $method:ident) => {
        pub fn $name<H, S, FuncParams>(handler: H) -> impl Route<S>
        where
            H: handler::HandlerFunction<S, FuncParams>,
        {
            Method {
                method: crate::Method::$method,
                route: crate::handler::HandlerFunctionHandlerAdapter {
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

pub enum Decision<'a, T, R> {
    Match(T),
    NoMatch(Request<'a, R>),
}

impl<'a, T, R> Decision<'a, T, R> {
    fn map<F, U>(self, f: F) -> Decision<'a, U, R>
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

pub trait Route<S> {
    async fn match_request<'a, Body: Read>(
        &self,
        req: Request<'a, Body>,
        state: &S,
    ) -> Decision<'a, impl IntoResponse, Body>;
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

impl<S> Route<S> for NotFound {
    async fn match_request<'a, Body: Read>(
        &self,
        _req: Request<'a, Body>,
        _state: &S,
    ) -> Decision<'a, impl IntoResponse, Body> {
        Decision::Match((404, "Not Found"))
    }
}

pub struct Path<R> {
    pub(crate) path: &'static str,
    pub(crate) route: R,
}

impl<S, R: Route<S>> Route<S> for Path<R> {
    async fn match_request<'a, Body: Read>(
        &self,
        req: Request<'a, Body>,
        state: &S,
    ) -> Decision<'a, impl IntoResponse, Body> {
        if self.path == req.path() {
            self.route.match_request(req, state).await
        } else {
            Decision::NoMatch(req)
        }
    }
}

pub struct Method<R> {
    pub(crate) method: crate::Method<'static>,
    pub(crate) route: R,
}

impl<S, R: Route<S>> Route<S> for Method<R> {
    async fn match_request<'a, Body: Read>(
        &self,
        req: Request<'a, Body>,
        state: &S,
    ) -> Decision<'a, impl IntoResponse, Body> {
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

impl<S, R1: Route<S>, R2: Route<S>> Route<S> for Fallback<R1, R2> {
    async fn match_request<'a, Body: Read>(
        &self,
        req: Request<'a, Body>,
        state: &S,
    ) -> Decision<'a, impl IntoResponse, Body> {
        match self.route.match_request(req, state).await {
            Decision::Match(t) => return Decision::Match(t.into_response().map_body(Either::Left)),
            Decision::NoMatch(req) => self
                .fallback
                .match_request(req, state)
                .await
                .map(|response| response.into_response().map_body(Either::Right)),
        }
    }
}
