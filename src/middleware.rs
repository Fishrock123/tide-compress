use futures_lite::io::BufReader;
use tide::http::cache::{CacheControl, CacheDirective};
use tide::http::conditional::Vary;
use tide::http::content::{AcceptEncoding, ContentEncoding, Encoding};
use tide::http::{headers, Body, Method};
use tide::{Middleware, Next, Request, Response};

#[cfg(feature = "brotli")]
use async_compression::futures::bufread::BrotliEncoder;
#[cfg(feature = "deflate")]
use async_compression::futures::bufread::DeflateEncoder;
#[cfg(feature = "gzip")]
use async_compression::futures::bufread::GzipEncoder;

#[cfg(feature = "regex-check")]
use http_types::content::ContentType;
#[cfg(feature = "regex-check")]
use regex::{Regex, RegexBuilder};

const THRESHOLD: usize = 1024;

// These regular expressions ere taken from jshttp/compressible
// Used under terms of the MIT license.
// https://github.com/jshttp/compressible/blob/89b61014fb82f0c64b42acef12d161dee48fb58e/index.js#L23-L24
#[cfg(feature = "regex-check")]
const CONTENT_TYPE_CHECK_PATTERN: &str = r"^text/|\+(?:json|text|xml)$";
#[cfg(feature = "regex-check")]
const EXTRACT_TYPE_PATTERN: &str = r"^\s*([^;\s]*)(?:;|\s|$)";

/// A middleware for compressing response body data.
///
/// ## Example
/// ```rust
/// # async_std::task::block_on(async {
/// let mut app = tide::new();
///
/// app.with(tide_compress::CompressMiddleware::new());
/// # })
/// ```
#[derive(Clone, Debug)]
pub struct CompressMiddleware {
    threshold: usize,
    #[cfg(feature = "regex-check")]
    content_type_check: Option<Regex>,
    #[cfg(feature = "regex-check")]
    extract_type_regex: Regex,
}

impl Default for CompressMiddleware {
    fn default() -> Self {
        CompressMiddlewareBuilder::default().into()
    }
}

impl CompressMiddleware {
    /// Creates a new CompressMiddleware.
    ///
    /// Uses the defaults:
    /// - Minimum body size threshold (1024 bytes).
    /// - Check for `Content-Type` header match `^text/|\+(?:json|text|xml)$` (case insensitive).
    ///
    /// ## Example
    /// ```rust
    /// # async_std::task::block_on(async {
    /// let mut app = tide::new();
    ///
    /// app.with(tide_compress::CompressMiddleware::new());
    /// # })
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Used to create a new CompressMiddleware with custom settings.
    ///
    /// See [`CompressMiddlewareBuilder`]
    pub fn builder() -> CompressMiddlewareBuilder {
        CompressMiddlewareBuilder::new()
    }

    /// Sets the minimum body size threshold value.
    pub fn set_threshold(&mut self, threshold: usize) {
        self.threshold = threshold
    }

    /// Gets the existing minimum body size threshold value.
    pub fn threshold(&self) -> usize {
        self.threshold
    }

    #[cfg(feature = "regex-check")]
    /// Sets the `Content-Type` header (MIME) check regular expression.
    pub fn set_content_type_check(&mut self, content_type_check: Option<Regex>) {
        self.content_type_check = content_type_check
    }

    #[cfg(feature = "regex-check")]
    /// Gets a reference to the existing `Content-Type` header (MIME) check regular expression.
    pub fn content_type_check(&self) -> Option<&Regex> {
        self.content_type_check.as_ref()
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for CompressMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        // Incoming Request data
        // Need to grab these things before the request is consumed by `next.run()`.
        let is_head = req.method() == Method::Head;
        let accepts = AcceptEncoding::from_headers(&req)?;

        // Propagate to route
        let mut res: Response = next.run(req).await;

        // Head requests should have no body to compress.
        // Can't tell if we can compress if there is no Accepts-Encoding header.
        if is_head || accepts.is_none() {
            return Ok(res);
        }
        let mut accepts = accepts.unwrap();

        // Should we transform?
        if let Some(cache_control) = CacheControl::from_headers(&res)? {
            // No compression for `Cache-Control: no-transform`
            // https://tools.ietf.org/html/rfc7234#section-5.2.2.4
            if cache_control
                .iter()
                .any(|directive| directive == &CacheDirective::NoTransform)
            {
                return Ok(res);
            }
        }

        // Set the Vary header, similar to how https://www.npmjs.com/package/compression does it.
        let mut vary = Vary::new();
        vary.push(headers::ACCEPT_ENCODING)?;
        vary.apply(&mut res);

        // Check if an encoding may already exist.
        // Can't tell if we should compress if an encoding set.
        if let Some(previous_encoding) = ContentEncoding::from_headers(&res)? {
            if previous_encoding != Encoding::Identity {
                return Ok(res);
            }
        }

        // Check body length against threshold.
        if let Some(body_len) = res.len() {
            if body_len < self.threshold {
                return Ok(res);
            }
        }

        #[cfg(feature = "regex-check")]
        // Check if the `Content-Type` header indicates a compressible body.
        if let Some(ref content_type_check) = self.content_type_check {
            if let Some(content_type) = ContentType::from_headers(&res)? {
                if let Some(extension_match) = self
                    .extract_type_regex
                    .captures(content_type.value().as_str())
                    .and_then(|captures| captures.get(1))
                {
                    if !content_type_check.is_match(extension_match.as_str()) {
                        return Ok(res);
                    }
                }
            }
        }

        let body = res.take_body();
        let encoding = accepts.negotiate(&[
            #[cfg(feature = "brotli")]
            Encoding::Brotli,
            #[cfg(feature = "gzip")]
            Encoding::Gzip,
            #[cfg(feature = "deflate")]
            Encoding::Deflate,
        ])?;

        // Get a new Body backed by an appropriate encoder, if one is available.
        res.set_body(get_encoder(body, &encoding));
        encoding.apply(&mut res);

        // End size no longer matches body size, so any existing Content-Length is useless.
        res.remove_header(headers::CONTENT_LENGTH);

        Ok(res)
    }
}

/// Returns a `Body` made from an encoder chosen from the `Encoding`.
fn get_encoder(body: Body, encoding: &ContentEncoding) -> Body {
    #[cfg(feature = "brotli")]
    {
        if *encoding == Encoding::Brotli {
            return Body::from_reader(BufReader::new(BrotliEncoder::new(body)), None);
        }
    }

    #[cfg(feature = "gzip")]
    {
        if *encoding == Encoding::Gzip {
            return Body::from_reader(BufReader::new(GzipEncoder::new(body)), None);
        }
    }

    #[cfg(feature = "deflate")]
    {
        if *encoding == Encoding::Deflate {
            return Body::from_reader(BufReader::new(DeflateEncoder::new(body)), None);
        }
    }

    body
}

#[derive(Clone, Debug)]
/// Used to create a new CompressMiddleware with custom settings.
///
/// Uses the defaults:
/// - Minimum body size threshold (1024 bytes).
/// - Check for `Content-Type` header match `^text/|\+(?:json|text|xml)$` (case insensitive).
///
/// ## Example
/// ```rust
/// # async_std::task::block_on(async {
/// let mut app = tide::new();
///
/// let check_regex = regex::Regex::new(r"^text/|\+(?:json|text|xml)$").expect("regular expression defined in source code");
///
/// let compress_middleware = tide_compress::CompressMiddleware::builder()
///     .threshold(1024)
///     .content_type_check(Some(check_regex))
///     .build();
///
/// app.with(compress_middleware);
/// # })
/// ```
pub struct CompressMiddlewareBuilder {
    pub threshold: usize,
    #[cfg(feature = "regex-check")]
    pub content_type_check: Option<Regex>,
}

impl Default for CompressMiddlewareBuilder {
    fn default() -> Self {
        Self {
            threshold: THRESHOLD,
            #[cfg(feature = "regex-check")]
            content_type_check: Some(
                RegexBuilder::new(CONTENT_TYPE_CHECK_PATTERN)
                    .case_insensitive(true)
                    .build()
                    .expect("Constant regular expression defined in Tide-Compress's source code"),
            ),
        }
    }
}

impl CompressMiddlewareBuilder {
    /// Make a new builder.
    /// Identical to `CompressMiddleware::builder()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the minimum body size threshold value.
    pub fn threshold(mut self, threshold: usize) -> Self {
        self.threshold = threshold;
        self
    }

    #[cfg(feature = "regex-check")]
    /// Sets the `Content-Type` header (MIME) check regular expression.
    pub fn content_type_check(mut self, content_type_check: Option<Regex>) -> Self {
        self.content_type_check = content_type_check;
        self
    }

    /// Construct a middleware instance from this builder.
    pub fn build(self) -> CompressMiddleware {
        self.into()
    }
}

impl From<CompressMiddlewareBuilder> for CompressMiddleware {
    fn from(builder: CompressMiddlewareBuilder) -> Self {
        Self {
            threshold: builder.threshold,
            #[cfg(feature = "regex-check")]
            content_type_check: builder.content_type_check,
            #[cfg(feature = "regex-check")]
            extract_type_regex: Regex::new(EXTRACT_TYPE_PATTERN)
                .expect("Constant regular expression defined in Tide-Compress's source code"),
        }
    }
}
