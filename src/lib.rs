//! Outgoing HTTP body compression middleware for Tide.

mod middleware;

pub use middleware::CompressMiddleware;

#[derive(PartialEq)]
pub enum Encoding {
    BROTLI,
    GZIP,
    DEFLATE,
}
