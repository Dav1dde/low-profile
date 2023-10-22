use core::{marker::PhantomData, mem::MaybeUninit};

use crate::{
    handler,
    parse::PathAndQuery,
    request::{record_header_indices, Body, HeaderIndices, Headers, Parts},
    route::{self, Route},
    utils, IntoResponse, Method, Read, Request, Service, Write,
};

mod private {
    #[derive(Debug, Clone, Copy)]
    pub enum HasAnyState {}

    #[derive(Debug, Clone, Copy)]
    pub enum Untouched {}
}

pub struct Router<RS, B, R: Route<RS, B>, S = (), HasRoute = private::Untouched> {
    state: S,
    route: R,
    _priv: PhantomData<(RS, B, HasRoute)>,
}

impl<RS, B> Router<RS, B, route::NotFound> {
    pub fn new() -> Self {
        Self {
            state: (),
            route: route::NotFound,
            _priv: Default::default(),
        }
    }
}

impl<RS, B> Default for Router<RS, B, route::NotFound> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B, R, S> Router<(), B, R, S, private::Untouched>
where
    R: Route<(), B>,
{
    pub fn with_state<S2>(self, state: S2) -> Router<S2, B, R, S2, private::HasAnyState>
    where
        R: Route<S2, B>,
    {
        Router {
            route: self.route,
            state,
            _priv: Default::default(),
        }
    }
}

impl<RS, B, R, S> Router<RS, B, R, S, private::HasAnyState>
where
    R: Route<RS, B>,
{
    pub fn with_state<S2>(self, state: S2) -> Router<S2, B, R, S2, private::HasAnyState>
    where
        R: Route<S2, B>,
    {
        Router {
            route: self.route,
            state,
            _priv: Default::default(),
        }
    }
}

macro_rules! impl_method {
    ($method:ident) => {
        impl<RS, B, R, S, HasRoute> Router<RS, B, R, S, HasRoute>
        where
            R: Route<RS, B>,
        {
            pub fn $method<H, X>(
                self,
                path: &'static str,
                handler: H,
            ) -> Router<RS, B, impl Route<RS, B>, S, private::HasAnyState>
            where
                H: handler::HandlerFunction<RS, B, X>,
            {
                self.route(path, route::$method(handler))
            }
        }
    };
}

impl_method!(get);
impl_method!(post);
impl_method!(put);
impl_method!(delete);
impl_method!(head);
impl_method!(options);
impl_method!(connect);
impl_method!(patch);
impl_method!(trace);

impl<RS, B, R, S, HasRoute> Router<RS, B, R, S, HasRoute>
where
    R: Route<RS, B>,
{
    pub fn route<T: Route<RS, B>>(
        self,
        path: &'static str,
        route: T,
    ) -> Router<RS, B, impl Route<RS, B>, S, private::HasAnyState> {
        Router {
            route: route::Fallback {
                route: route::Path { path, route },
                fallback: self.route,
            },
            state: self.state,
            _priv: Default::default(),
        }
    }
}

impl<B, W, R: Route<S, B>, S, HasRoute> Service<B, W> for Router<S, B, R, S, HasRoute>
where
    B: Read,
    W: Write<Error = B::Error>,
{
    async fn serve(&self, mut reader: B, mut writer: W) {
        // TODO: buf size, optinally make the buffer an arg
        let mut buf = [0u8; 2048];

        const MAX_HEADERS: usize = 100;

        let mut headers_indices: [MaybeUninit<HeaderIndices>; MAX_HEADERS] = unsafe {
            // SAFETY: We can go safely from MaybeUninit array to array of MaybeUninit
            MaybeUninit::uninit().assume_init()
        };

        let mut pos = 0;
        let (method, path, headers, body_start) = loop {
            // TODO check if buffer is full first
            let read = reader.read(&mut buf[pos..]).await.unwrap();
            if read == 0 {
                // TODO
                return;
            }
            pos += read;

            let mut headers: [MaybeUninit<httparse::Header<'_>>; MAX_HEADERS] =
                unsafe { MaybeUninit::uninit().assume_init() };
            let mut req = httparse::Request::new(&mut []);

            match req.parse_with_uninit_headers(&buf, &mut headers) {
                Ok(httparse::Status::Complete(len)) => {
                    record_header_indices(&buf, req.headers, &mut headers_indices);

                    let headers = unsafe {
                        MaybeUninit::slice_assume_init_ref(&headers_indices[..req.headers.len()])
                    };

                    break (req.method.unwrap(), req.path.unwrap(), headers, len);
                }
                Ok(httparse::Status::Partial) => {
                    continue;
                }
                Err(err) => panic!("parse error: {err:?}"),
            }
        };

        let paq = PathAndQuery::parse(path).unwrap();
        let parts = Parts {
            method: Method::new(method).unwrap(),
            path: paq.path(),
            query: paq.query(),
            headers: Headers { headers, buf: &buf },
        };

        let content_length = parts
            .headers
            .get_first("Content-Length")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);

        let body = Body::new(content_length, &buf[body_start..pos], reader);
        let request = Request::from_parts(parts, body);

        // It is safe to unwrap here, we always have a `NotFound` fallback handler.
        let response = self
            .route
            .match_request(request, &self.state)
            .await
            .unwrap()
            .into_response();

        use utils::WriteExt;
        write!(writer, "HTTP/1.1 {}\r\n", response.status_code())
            .await
            .unwrap();
        writer.write_all(b"\r\n").await.unwrap();

        let mut body = response.into_body();
        loop {
            let mut buf = [0; 1024];
            let len = body.read(&mut buf).await.unwrap();
            if len == 0 {
                break;
            }
            writer.write_all(&buf[..len]).await.unwrap();
        }
    }
}
