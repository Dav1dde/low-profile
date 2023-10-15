#![no_std]
#![allow(stable_features)]
#![feature(
    async_fn_in_trait,
    type_alias_impl_trait,
    return_position_impl_trait_in_trait,
    maybe_uninit_slice
)]

pub mod extract;
mod handler;
mod io;
mod method;
pub mod request;
pub mod response;
mod route;
mod router;
mod utils;

pub use extract::{FromRef, FromRequest, FromRequestParts};
pub use io::{Io, Read, Write};
pub use method::Method;
pub use request::{Headers, Parts, Request};
pub use response::{IntoResponse, Response};
pub use route::{connect, delete, get, head, options, patch, post, put, trace};
pub use router::Router;
