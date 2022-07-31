#![forbid(unsafe_code)]
#![deny(future_incompatible)]
#![warn(
    meta_variable_misuse,
    missing_debug_implementations,
    noop_method_call,
    rust_2018_idioms,
    trivial_casts,
    unused_lifetimes,
    unused_qualifications,
    unused_macro_rules,
    variant_size_differences
)]
#![doc(test(attr(deny(future_incompatible, rust_2018_idioms, warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]
#![deny(
    clippy::allow_attributes_without_reason,
    clippy::default_union_representation,
    clippy::exit,
    clippy::lossy_float_literal,
    clippy::mem_forget,
    clippy::multiple_inherent_impl,
    clippy::mut_mut,
    clippy::ptr_as_ptr,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::wildcard_dependencies
)]
#![warn(
    clippy::dbg_macro,
    clippy::empty_drop,
    clippy::fallible_impl_from,
    clippy::inefficient_to_string,
    clippy::macro_use_imports,
    clippy::match_same_arms,
    clippy::multiple_crate_versions,
    clippy::no_effect_underscore_binding,
    clippy::panic,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::same_name_method,
    clippy::single_char_lifetime_names,
    clippy::string_to_string,
    clippy::trait_duplication_in_bounds,
    clippy::type_repetition_in_bounds,
    clippy::unimplemented,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::used_underscore_binding
)]

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
//!   - Prioritizes Brotli if available.
//!   - Only pulls in the necessary dependencies for the desired configuration.
//!   - Defaults to Brotli & Gzip.
//!   - Also handles the `"identity"` encoding directive [as per RFC 9110][Identity].
//! - [`Accept-Encoding`][] header checking including priority.
//! - Minimum body size threshold (Default: 1024 bytes, configurable).
//! - Does not compress responses with a [`Cache-Control: no-transform`][] header.
//! - Sets the [`Vary`][] header.
//! - Checks the [`Content-Type`][] header (MIME).
//!   - Checks against [jshttp's comprehensive database][jshttp mime-db], which is compiled to a [perfect hash function][].
//!   - The database can be regenerated in the crate's repository by running `cargo run generate-database`.
//!   - If not in the database, checks against a regular expression.
//!     - Default: `^text/|\+(?:json|text|xml)$` (case insensitive).
//!     - Fully override-able to any custom [`Regex`][], with `None` as an option.
//!   - Functionality can be excluded in crate features if the `regex` crate poses build issues.
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
//! [Identity]: https://www.rfc-editor.org/rfc/rfc9110.html#name-accept-encoding
//! [Tide]: https://github.com/http-rs/tide

#[cfg(feature = "db-check")]
mod codegen_database;

mod middleware;

pub use middleware::{CompressMiddleware, CompressMiddlewareBuilder};
