//! HTTP cookies.

mod encoding;
mod middleware;

pub use encoding::Encoding;
pub use middleware::CompressMiddleware;
