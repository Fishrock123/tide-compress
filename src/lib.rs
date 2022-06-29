#![forbid(unsafe_code, future_incompatible)]
#![warn(
    missing_debug_implementations,
    rust_2018_idioms,
    trivial_casts,
    unused_qualifications
)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]

//! Outgoing body compression middleware for the [Tide][] server framework.
//!
//! ```rust
//! #[async_std::main]
//! async fn main() {
//!     let mut app = tide::new();
//!     app.with(tide_compress::CompressMiddleware::new());
//! }
//! ```
//!
//! ## Features
//!
//! - Support for [Brotli][], [Gzip][], and [Deflate][] encodings, compile-time configurable through cargo feature flags.
//! - Prioritizes Brotli if available.
//! - Only pulls in the necessary dependencies for the desired configuration.
//! - Defaults to Brotli & Gzip.
//! - [`Accept-Encoding`][] header checking including priority.
//! - Minimum body size threshold (Default: 1024 bytes, configurable).
//! - Does not compress responses with a [`Cache-Control: no-transform`][] header.
//! - Sets the [`Vary`][] header.
//! - Checks the [`Content-Type`][] header (MIME).
//! - Checks against [jshttp's comprehensive database][jshttp mime-db], which is compiled to a [perfect hash function][] at build time.
//! - If not in the database, checks against a regular expression.
//!     - Default: `^text/|\+(?:json|text|xml)$` (case insensitive).
//!     - Fully override-able to any custom [`Regex`][], with `None` as an option.
//! - Functionality can be excluded in crate features if the `regex` crate or codegen poses build issues.
//!
//! ## Note
//!
//! This crate, in its current set up with the `db-check` feature enabled (which is by default),
//! pulls down [a json MIME database][jshttp mime-db] from the network at build time.
//!
//! [`Accept-Encoding`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Encoding
//! [`Cache-Control: no-transform`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control
//! [`Content-Type`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type
//! [`Regex`]: https://docs.rs/regex/1/regex/struct.Regex.html
//! [`Vary`]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Vary
//! [jshttp mime-db]: https://github.com/jshttp/mime-db/blob/master/db.json
//! [perfect hash function]: https://github.com/rust-phf/rust-phf
//! [Brotli]: https://en.wikipedia.org/wiki/Brotli
//! [Deflate]: https://en.wikipedia.org/wiki/Deflate
//! [Gzip]: https://en.wikipedia.org/wiki/Gzip
//! [Tide]: https://github.com/http-rs/tide

mod middleware;

pub use middleware::{CompressMiddleware, CompressMiddlewareBuilder};
