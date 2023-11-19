#![no_std]
#![allow(stable_features)]
#![feature(
    async_fn_in_trait,
    type_alias_impl_trait,
    return_position_impl_trait_in_trait,
    maybe_uninit_slice,
    impl_trait_projections,
    const_waker,
    impl_trait_in_assoc_type
)]

pub(crate) mod either;
mod error;
pub mod extract;
mod handler;
pub mod http;
mod io;
#[cfg(feature = "json")]
mod json;
pub(crate) mod macros;
mod method;
mod parse;
pub mod request;
pub mod response;
mod route;
mod router;
mod service;
mod utils;

pub use extract::{FromRef, FromRequest, FromRequestParts};
pub use io::{ErrorType, Read, Write};
#[cfg(feature = "json")]
pub use json::Json;
pub use method::Method;
pub use request::{Headers, Parts, Request};
pub use response::{IntoResponse, Response};
pub use route::{connect, delete, get, head, options, patch, post, put, trace};
pub use router::Router;
pub use service::Service;
