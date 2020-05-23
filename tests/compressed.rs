use http_types::{headers, Method, Request, StatusCode, Url};
use tide::Response;

const TEXT: &'static str = concat![
    "Chunk one\n",
    "data data\n",
    "\n",
    "Chunk two\n",
    "data data\n",
    "\n",
    "Chunk three\n",
    "data data\n",
];

const BR_COMPRESSED: &'static [u8] = &[
    27, 63, 0, 248, 157, 9, 118, 12, 101, 50, 101, 248, 252, 26, 229, 16, 90, 93, 43, 144, 189,
    209, 105, 5, 16, 55, 58, 200, 132, 35, 141, 117, 16, 5, 199, 247, 22, 131, 0, 51, 145, 60, 128,
    132, 79, 166, 110, 169, 162, 169, 129, 224, 63, 191, 0,
];

#[async_std::test]
async fn brotli_compressed() {
    let mut app = tide::new();
    app.middleware(tide_compress::CompressMiddleware::with_threshold(16));
    app.at("/").get(|_| async {
        let res = Response::new(StatusCode::Ok)
            .body_string(TEXT.to_owned())
            .set_mime("text/plain; charset=utf-8".parse().unwrap());
        Ok(res)
    });

    let mut req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "br");
    let res: http_types::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(res.header(headers::CONTENT_LENGTH).is_none());
    assert_eq!(res[headers::CONTENT_ENCODING], "br");
    assert_eq!(res.body_bytes().await.unwrap(), BR_COMPRESSED);
}

const GZIPPED: &'static [u8] = &[
    // It should be this but miniz / miniz_oxide's gzip compression is rather lacking.
    //
    //     0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // gzip header
    //     0xff, // OS type
    //     // Same as DEFLATE
    //     0x73, 0xce, 0x28, 0xcd, 0xcb, 0x56, 0xc8, 0xcf, 0x4b, 0xe5, 0x4a, 0x49, 0x2c, 0x49, 0x54, 0x00, 0x11,
    //     0x5c, 0x5c, 0xce, 0x60, 0xc1, 0x92, 0xf2, 0x7c, 0x2c, 0x82, 0x19, 0x45, 0xa9, 0xc8, 0x6a, 0x01,
    //     0xde, 0xf2, 0xd7, 0x81, // crc32
    //     0x40, 0x00, 0x00, 0x00 // input size
    //
    0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // gzip header
    0xff, // OS type
    // Same as DEFLATE
    // 109, 202, 177, 9, 0, 48, 8, 5, 209, 254, 79, 225, 46, 78, 34, 68, 16, 2, 10, 193, 144, 245, 67,
    // 82, 89, 216, 92, 241, 56, 182, 237, 147, 194, 21, 67, 82, 232, 5, 224, 143, 121, 162, 65, 91,
    // 90, 223, 11,
    0x6d, 0xca, 0xb1, 0x09, 0x00, 0x30, 0x08, 0x05, 0xd1, 0xfe, 0x4f, 0xe1, 0x2e, 0x4e, 0x22, 0x44,
    0x10, 0x02, 0x0a, 0xc1, 0x90, 0xf5, 0x43, 0x52, 0x59, 0xd8, 0x5c, 0xf1, 0x38, 0xb6, 0xed, 0x93,
    0xc2, 0x15, 0x43, 0x52, 0xe8, 0x05, 0xe0, 0x8f, 0x79, 0xa2, 0x41, 0x5b, 0x5a, 0xdf, 0x0b,
    //
    0xde, 0xf2, 0xd7, 0x81, // crc32
    0x40, 0x00, 0x00, 0x00, // input size
];

#[async_std::test]
async fn gzip_compressed() {
    let mut app = tide::new();
    app.middleware(tide_compress::CompressMiddleware::with_threshold(16));
    app.at("/").get(|_| async move {
        let res = Response::new(StatusCode::Ok)
            .body_string(TEXT.to_owned())
            .set_mime("text/plain; charset=utf-8".parse().unwrap())
            .set_header(headers::CONTENT_ENCODING, "identity");
        Ok(res)
    });

    let mut req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "gzip");
    let res: http_types::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(res.header(headers::CONTENT_LENGTH).is_none());
    assert_eq!(res[headers::CONTENT_ENCODING], "gzip");
    assert_eq!(res.body_bytes().await.unwrap(), GZIPPED);
}

#[cfg(feature = "deflate")]
const DEFLATED: &'static [u8] = &[
    0x6d, 0xca, 0xb1, 0x09, 0x00, 0x30, 0x08, 0x05, 0xd1, 0xfe, 0x4f, 0xe1, 0x2e, 0x4e, 0x22, 0x44,
    0x10, 0x02, 0x0a, 0xc1, 0x90, 0xf5, 0x43, 0x52, 0x59, 0xd8, 0x5c, 0xf1, 0x38, 0xb6, 0xed, 0x93,
    0xc2, 0x15, 0x43, 0x52, 0xe8, 0x05, 0xe0, 0x8f, 0x79, 0xa2, 0x41, 0x5b, 0x5a, 0xdf, 0x0b,
];

#[cfg(feature = "deflate")]
#[async_std::test]
async fn deflate_compressed() {
    let mut app = tide::new();
    app.middleware(tide_compress::CompressMiddleware::with_threshold(16));
    app.at("/").get(|_| async {
        let res = Response::new(StatusCode::Ok)
            .body_string(TEXT.to_owned())
            .set_mime("text/plain; charset=utf-8".parse().unwrap());
        Ok(res)
    });

    let mut req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "deflate");
    let res: http_types::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(res.header(headers::CONTENT_LENGTH).is_none());
    assert_eq!(res[headers::CONTENT_ENCODING], "deflate");
    assert_eq!(res.body_bytes().await.unwrap(), DEFLATED);
}
