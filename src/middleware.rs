use std::future::Future;
use std::pin::Pin;

use async_std::io::Read;

use async_compression::futures::bufread::GzipEncoder;
use futures::io::BufReader;
use http_types::Body;
use http_types::headers::HeaderValue;
use tide::{Middleware, Next, Request, Response};
use tide::http::Method;

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
            let encoding = accepts_encoding(&req);

            // Propagate to route
            let mut res: Response = next.run(req).await?;

            if is_head || encoding.is_none() {
                return Ok(res)
            }

            let body = res.take_body();

            let encoder = get_encoder(body, encoding.unwrap());
            let reader = BufReader::new(encoder);

            let body = Body::from_reader(reader, None);
            res.set_body(Body::from_reader(body, None));

            res.remove_header(&"Content-Length".parse().unwrap());
            let res = res.set_header("Content-Encoding".parse().unwrap(), "gzip");

            Ok(res)
        })
    }
}


fn accepts_encoding<State: Send + Sync + 'static>(req: &Request<State>) -> Option<Encoding> {
    let header_value = req.header(&"Accept-Encoding".parse().unwrap()).cloned();

    if header_value.is_some() && header_value.unwrap().contains(&HeaderValue::from_ascii(b"gzip").unwrap()) {
        return Some(Encoding::GZIP);
    } else {
        return None;
    }
}

fn get_encoder(body: Body, _: Encoding) -> impl Read {
    GzipEncoder::new(body)
}
