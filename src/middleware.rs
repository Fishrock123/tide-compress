#[cfg(feature = "brotli")]
use async_compression::futures::bufread::BrotliEncoder;
#[cfg(feature = "deflate")]
use async_compression::futures::bufread::DeflateEncoder;
#[cfg(feature = "gzip")]
use async_compression::futures::bufread::GzipEncoder;
use futures_util::io::BufReader;
use regex::Regex;
use tide::http::{headers, Body, Method};
use tide::{Middleware, Next, Request, Response};

use crate::Encoding;

const THRESHOLD: usize = 1024;

/// A middleware for compressing response body data.
#[derive(Clone, Debug)]
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

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for CompressMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        // Incoming Request data
        // Need to grab these things before the request is consumed by `next.run()`.
        let is_head = req.method() == Method::Head;
        let accepts_encoding = accepts_encoding(&req);

        // Propagate to route
        let mut res: Response = next.run(req).await;

        // Head requests should have no body to compress.
        // Can't tell if we can compress if there is no Accepts-Encoding header.
        if is_head || accepts_encoding.is_none() {
            return Ok(res);
        }

        // Should we transform?
        if let Some(cache_control) = res.header(headers::CACHE_CONTROL) {
            // No compression for `Cache-Control: no-transform`
            // https://tools.ietf.org/html/rfc7234#section-5.2.2.4
            let regex = Regex::new(r"(?:^|,)\s*?no-transform\s*?(?:,|$)").unwrap();
            if regex.is_match(cache_control.as_str()) {
                return Ok(res);
            }
        }

        // Check if an encoding may already exist.
        // Can't tell if we should compress if an encoding set.
        if let Some(previous_encoding) = res.header(headers::CONTENT_ENCODING) {
            if previous_encoding.iter().any(|v| v.as_str() != "identity") {
                return Ok(res);
            }
        }

        // Check body length against threshold.
        if let Some(body_len) = res.len() {
            if body_len < self.threshold {
                return Ok(res);
            }
        }

        let body = res.take_body();
        let encoding = accepts_encoding.unwrap();

        // Get a new Body backed by an appropriate encoder, if one is available.
        res.set_body(get_encoder(body, &encoding));
        res.insert_header(headers::CONTENT_ENCODING, get_encoding_name(&encoding));

        // End size no longer matches body size, so any existing Content-Length is useless.
        res.remove_header(headers::CONTENT_LENGTH);

        Ok(res)
    }
}

/// Gets an `Encoding` that matches up to the Accept-Encoding value.
fn accepts_encoding<State: Send + Sync + 'static>(req: &Request<State>) -> Option<Encoding> {
    let header = req.header(headers::ACCEPT_ENCODING)?;

    #[cfg(feature = "brotli")]
    {
        if header.iter().any(|v| v.as_str() == "br") {
            return Some(Encoding::BROTLI);
        }
    }

    #[cfg(feature = "gzip")]
    {
        if header.iter().any(|v| v.as_str() == "gzip") {
            return Some(Encoding::GZIP);
        }
    }

    #[cfg(feature = "deflate")]
    {
        if header.iter().any(|v| v.as_str() == "deflate") {
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
fn get_encoding_name(encoding: &Encoding) -> String {
    (match *encoding {
        Encoding::BROTLI => "br",
        Encoding::GZIP => "gzip",
        Encoding::DEFLATE => "deflate",
    })
    .to_string()
}
