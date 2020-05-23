use std::future::Future;
use std::pin::Pin;

#[cfg(feature = "brotli")]
use async_compression::futures::bufread::BrotliEncoder;
#[cfg(feature = "deflate")]
use async_compression::futures::bufread::DeflateEncoder;
#[cfg(feature = "gzip")]
use async_compression::futures::bufread::GzipEncoder;
use futures::io::BufReader;
use http_types::headers::HeaderName;
use http_types::{headers, Body};
use tide::http::Method;
use tide::{Middleware, Next, Request, Response};

use crate::Encoding;

const THRESHOLD: usize = 1024;

/// A middleware for compressing response body data.
///
/// Currently, it compresses unconditionally, and only with gzip.
#[derive(Debug, Clone)]
pub struct CompressMiddleware {
    threshold: usize,
}

impl Default for CompressMiddleware {
    fn default() -> Self {
        CompressMiddleware {
            threshold: THRESHOLD,
        }
    }
}

impl CompressMiddleware {
    /// Creates a new CompressMiddleware.
    ///
    /// Uses the default minimum body size threshold (1024 bytes).
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new CompressMiddleware with a custom minimum body size threshold.
    ///
    /// # Arguments
    ///
    /// * `threshold` - minimum body size in bytes.
    pub fn with_threshold(threshold: usize) -> Self {
        CompressMiddleware { threshold }
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for CompressMiddleware {
    fn handle<'a>(
        &'a self,
        req: Request<State>,
        next: Next<'a, State>,
    ) -> Pin<Box<dyn Future<Output = tide::Result> + Send + 'a>> {
        Box::pin(async move {
            // Incoming Request data
            // Need to grab these things before the request is consumed by `next.run()`.
            let is_head = req.method() == Method::Head;
            let accepts_header = accepts_encoding(&req);

            // Propagate to route
            let mut res: Response = next.run(req).await?;

            // Head requests should have no body.
            // Can't tell if we can compress if there is no Accepts-Encoding header.
            if is_head || accepts_header.is_none() {
                return Ok(res);
            }

            // Check if an encoding may already exist.
            // Can't tell if we should compress if an encoding set.
            let encoding_header: HeaderName = "Content-Encoding".parse().unwrap();
            let previous_encoding = res.header(&encoding_header);
            if previous_encoding.is_some() {
                let previous_encoding = previous_encoding.unwrap();
                if previous_encoding.iter().count() > 1
                    || previous_encoding.iter().any(|v| v.as_str() != "identity")
                {
                    return Ok(res);
                }
            }

            let encoding = accepts_header.unwrap();

            let body = res.take_body();

            // Check body length against threshold.
            let body_len = body.len();
            if body_len.is_some() && body_len.unwrap() < self.threshold {
                res.set_body(body);
                return Ok(res);
            }

            let body = get_encoder(body, &encoding);
            res.set_body(Body::from_reader(body, None));

            // End size no longer matches body size, so Content-Length is useless.
            res.remove_header(&headers::CONTENT_LENGTH);
            let res = res.set_header(encoding_header, get_encoding_name(encoding));

            Ok(res)
        })
    }
}

/// Gets an `Encoding` that matches up to the Accept-Encoding value.
fn accepts_encoding<State: Send + Sync + 'static>(req: &Request<State>) -> Option<Encoding> {
    let header = req.header(headers::ACCEPT_ENCODING);

    if header.is_none() {
        return None;
    }

    let header_values = header.unwrap();

    #[cfg(feature = "brotli")]
    {
        if header_values.iter().any(|v| v.as_str() == "br") {
            return Some(Encoding::BROTLI);
        }
    }

    #[cfg(feature = "gzip")]
    {
        if header_values.iter().any(|v| v.as_str() == "gzip") {
            return Some(Encoding::GZIP);
        }
    }

    #[cfg(feature = "deflate")]
    {
        if header_values.iter().any(|v| v.as_str() == "deflate") {
            return Some(Encoding::DEFLATE);
        }
    }

    None
}

/// Returns a `Body` made from an encoder chosen from the `Encoding`.
fn get_encoder(body: Body, encoding: &Encoding) -> Body {
    #[cfg(feature = "brotli")]
    {
        if *encoding == Encoding::BROTLI {
            return Body::from_reader(BufReader::new(BrotliEncoder::new(body)), None);
        }
    }

    #[cfg(feature = "gzip")]
    {
        if *encoding == Encoding::GZIP {
            return Body::from_reader(BufReader::new(GzipEncoder::new(body)), None);
        }
    }

    #[cfg(feature = "deflate")]
    {
        if *encoding == Encoding::DEFLATE {
            return Body::from_reader(BufReader::new(DeflateEncoder::new(body)), None);
        }
    }

    body
}

/// Maps an `Encoding` to a Content-Encoding string.
fn get_encoding_name(encoding: Encoding) -> String {
    (match encoding {
        Encoding::BROTLI => "br",
        Encoding::GZIP => "gzip",
        Encoding::DEFLATE => "deflate",
    })
    .to_string()
}
