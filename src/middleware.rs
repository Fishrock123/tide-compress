use std::future::Future;
use std::pin::Pin;

use async_compression::futures::bufread::GzipEncoder;
use futures::io::BufReader;
use http_types::Body;
use http_types::headers::HeaderValue;
use tide::{Middleware, Next, Request, Response};
use tide::http::Method;

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
            let is_head = req.method() == Method::Head;

            let header_value = req.header(&"Accept-Encoding".parse().unwrap()).cloned();

            let mut res: Response = next.run(req).await?;

            if is_head || header_value.is_none() {
                return Ok(res)
            }

            let accept_encoding = header_value.unwrap();

            if !accept_encoding.contains(&HeaderValue::from_ascii(b"gzip").unwrap()) {
                return Ok(res) 
            }

            let body = res.take_body();

            let encoder = GzipEncoder::new(body);
            let reader = BufReader::new(encoder);

            let body = Body::from_reader(reader, None);
            res.set_body(Body::from_reader(body, None));

            res.remove_header(&"Content-Length".parse().unwrap());
            let res = res.set_header("Content-Encoding".parse().unwrap(), "gzip");

            Ok(res)
        })
    }
}
