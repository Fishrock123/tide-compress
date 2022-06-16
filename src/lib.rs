#![forbid(unsafe_code, future_incompatible)]
#![warn(
    missing_debug_implementations,
    rust_2018_idioms,
    trivial_casts,
    unused_qualifications
)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]

//! Outgoing HTTP body compression middleware for Tide.

mod middleware;

pub use middleware::{CompressMiddleware, CompressMiddlewareBuilder};
