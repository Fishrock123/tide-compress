use std::future::Future;
use std::pin::Pin;

#[cfg(feature = "brotli")]
use async_compression::futures::bufread::BrotliEncoder;
#[cfg(feature = "deflate")]
use async_compression::futures::bufread::DeflateEncoder;
#[cfg(feature = "gzip")]
use async_compression::futures::bufread::GzipEncoder;
use futures::io::BufReader;
use http_types::headers::{HeaderName, HeaderValue};
use http_types::Body;
use tide::http::Method;
use tide::{Middleware, Next, Request, Response};

use crate::Encoding;

/// A middleware for compressing response body data.
///
/// Currently, it compresses unconditionally, and only with gzip.
#[derive(Debug, Clone, Default)]
pub struct CompressMiddleware;

impl CompressMiddleware {
    /// Creates a new CompressMiddleware.
    pub fn new() -> Self {
        Self::default()
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

            if is_head || accepts_header.is_none() {
                return Ok(res);
            }

            let encoding_header: HeaderName = "Content-Encoding".parse().unwrap();
            let previous_encoding = res.header(&encoding_header);

            if previous_encoding.is_some() {
                let previous_encoding = previous_encoding.unwrap();
                if previous_encoding.len() > 1 || 
                    previous_encoding.iter().any(|v| v.as_str() != "identity") {
                    return Ok(res);
                }
            }

            let encoding = accepts_header.unwrap();

            let body = res.take_body();
            let body = get_encoder(body, &encoding);
            res.set_body(Body::from_reader(body, None));

            res.remove_header(&"Content-Length".parse().unwrap());
            let res = res.set_header(
                encoding_header,
                get_encoding_name(encoding),
            );

            Ok(res)
        })
    }
}

fn accepts_encoding<State: Send + Sync + 'static>(req: &Request<State>) -> Option<Encoding> {
    let header = req.header(&"Accept-Encoding".parse().unwrap());

    if header.is_none() {
        return None;
    }

    let header_values = header.unwrap();

    #[cfg(feature = "brotli")]
    {
        if header_values.contains(&HeaderValue::from_ascii(b"br").unwrap()) {
            return Some(Encoding::BROTLI);
        }
    }

    #[cfg(feature = "gzip")]
    {
        if header_values.contains(&HeaderValue::from_ascii(b"gzip").unwrap()) {
            return Some(Encoding::GZIP);
        }
    }

    #[cfg(feature = "deflate")]
    {
        if header_values.contains(&HeaderValue::from_ascii(b"deflate").unwrap()) {
            return Some(Encoding::DEFLATE);
        }
    }

    None
}

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

    Body::from_reader(BufReader::new(body), None)
}

fn get_encoding_name(encoding: Encoding) -> String {
    (match encoding {
        Encoding::BROTLI => "br",
        Encoding::GZIP => "gzip",
        Encoding::DEFLATE => "deflate",
    })
    .to_string()
}
