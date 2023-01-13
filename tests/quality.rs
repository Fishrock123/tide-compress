use tide::http::{headers, Method, Request, StatusCode, Url};
use tide::Response;

const TEXT: &str = concat![
    "Chunk one\n",
    "data data\n",
    "\n",
    "Chunk two\n",
    "data data\n",
    "\n",
    "Chunk three\n",
    "data data\n",
];

const BR_COMPRESSED: &[u8] = &[
    27, 63, 0, 248, 157, 9, 118, 12, 101, 50, 101, 248, 252, 26, 229, 16, 90, 93, 43, 144, 189,
    209, 105, 5, 16, 55, 58, 200, 132, 35, 141, 117, 16, 5, 199, 247, 22, 131, 0, 51, 145, 60, 128,
    132, 79, 166, 110, 169, 162, 169, 129, 224, 63, 191, 0,
];

#[async_std::test]
async fn brotli_compressed_quality_best() {
    let mut app = tide::new();
    app.with(
        tide_compress::CompressMiddleware::builder()
            .threshold(16)
            .brotli_quality(async_compression::Level::Best)
            .build(),
    );
    app.at("/").get(|_| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(TEXT.to_owned());
        Ok(res)
    });

    let mut req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "br");
    let mut res: tide::http::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(res.header(headers::CONTENT_LENGTH).is_none());
    assert_eq!(res[headers::CONTENT_ENCODING], "br");
    assert_eq!(res[headers::VARY], "accept-encoding");
    assert_eq!(res.body_bytes().await.unwrap(), BR_COMPRESSED);
}

const GZIPPED: &[u8] = &[
    // It should be this but miniz_oxide's gzip compression doesn't always pick the best huffman codes.
    //
    // See https://github.com/Frommi/miniz_oxide/issues/77
    //
    //     0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // gzip header
    //     0xff, // OS type
    //     // Same as DEFLATE
    //     0x73, 0xce, 0x28, 0xcd, 0xcb, 0x56, 0xc8, 0xcf, 0x4b, 0xe5, 0x4a, 0x49, 0x2c, 0x49, 0x54, 0x00, 0x11,
    //     0x5c, 0x5c, 0xce, 0x60, 0xc1, 0x92, 0xf2, 0x7c, 0x2c, 0x82, 0x19, 0x45, 0xa9, 0xc8, 0x6a, 0x01,
    //     0xde, 0xf2, 0xd7, 0x81, // crc32
    //     0x40, 0x00, 0x00, 0x00 // input size
    //
    0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, // gzip header.
    // Last byte is quality level.
    0xff, // OS type
    // Same as DEFLATE
    //
    // Deflate header?
    109, 202, 177, 9, 0, 48, 8, 5, 209, 254, 79, 225, 46, //
    // Deflate data
    153, 68, 136, 32, 4, 20, 130, 33, 235, 135, 96, 99, 97, 115, 197, 227, 134, 30, 91, 228, 38,
    152, 28, 76, 63, 64, 98, 92, 111, 80, 183, 212, 247, 1,
    // ..........................
    0xde, 0xf2, 0xd7, 0x81, // crc32
    0x40, 0x00, 0x00, 0x00, // input size
];

#[async_std::test]
async fn gzip_compressed_quality_fastest() {
    let mut app = tide::new();
    app.with(
        tide_compress::CompressMiddleware::builder()
            .threshold(16)
            .deflate_quality(async_compression::Level::Fastest)
            .build(),
    );
    app.at("/").get(|_| async move {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(TEXT.to_owned());
        res.insert_header(headers::CONTENT_ENCODING, "identity");
        Ok(res)
    });

    let mut req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "gzip");
    let mut res: tide::http::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(res.header(headers::CONTENT_LENGTH).is_none());
    assert_eq!(res[headers::CONTENT_ENCODING], "gzip");
    assert_eq!(res[headers::VARY], "accept-encoding");
    assert_eq!(res.body_bytes().await.unwrap(), GZIPPED);
}

#[cfg(feature = "deflate")]
const DEFLATED: &[u8] = &[
    // Deflate header?
    109, 202, 177, 9, 0, 48, 8, 5, 209, 254, 79, 225, 46, //
    // Deflate data
    78, 34, 68, 16, 2, 10, 193, 144, 245, 67, 82, 89, 216, 92, 241, 56, 182, 237, 147, 194, 21, 67,
    82, 232, 5, 224, 143, 121, 162, 65, 91, 90, 223, 11,
];

#[cfg(feature = "deflate")]
#[async_std::test]
async fn deflate_compressed_quality_best() {
    let mut app = tide::new();
    app.with(
        tide_compress::CompressMiddleware::builder()
            .threshold(16)
            .deflate_quality(async_compression::Level::Best)
            .build(),
    );
    app.at("/").get(|_| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(TEXT.to_owned());
        Ok(res)
    });

    let mut req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "deflate");
    let mut res: tide::http::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(res.header(headers::CONTENT_LENGTH).is_none());
    assert_eq!(res[headers::CONTENT_ENCODING], "deflate");
    assert_eq!(res[headers::VARY], "accept-encoding");
    assert_eq!(res.body_bytes().await.unwrap(), DEFLATED);
}
