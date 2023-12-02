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

#[cfg(feature = "alloc")]
pub mod alloc;
pub(crate) mod either;
mod error;
pub mod extract;
mod handler;
#[cfg(feature = "heapless")]
pub mod heapless;
pub mod http;
mod io;
pub(crate) mod macros;
mod method;
mod parse;
mod path;
pub mod request;
pub mod response;
mod route;
mod router;
mod service;
mod utils;

pub use extract::{FromRef, FromRequest, FromRequestParts};
pub use io::{ErrorType, Read, Write};
pub use method::Method;
pub use path::*;
pub use request::{Headers, Parts, Request};
pub use response::{IntoResponse, Response};
pub use route::{connect, delete, get, head, options, patch, post, put, trace};
pub use router::Router;
pub use service::Service;
