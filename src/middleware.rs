use std::future::Future;
use std::pin::Pin;

use async_compression::futures::bufread::GzipEncoder;
use futures::io::BufReader;
use http_types::Body;
use tide::{Middleware, Next, Request, Response};

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
            let header_value = req.header(&"Accept-Encoding".parse().unwrap()).cloned();

            let mut res: Response = next.run(req).await?;

            if header_value.is_none() {
                return Ok(res)
            }

            let accept_encoding = header_value.unwrap();

            if !accept_encoding.contains(&"gzip".parse().unwrap()) {
                return Ok(res) 
            }

            let body = res.take_body();

            let encoder = GzipEncoder::new(body);
            let reader = BufReader::new(encoder);

            let body = Body::from_reader(reader, None);
            let res = res.body(body);

            Ok(res)
        })
    }
}
