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
    139, 31, 128, 67, 104, 117, 110, 107, 32, 111, 110, 101, 10, 100, 97, 116, 97, 32, 100, 97,
    116, 97, 10, 10, 67, 104, 117, 110, 107, 32, 116, 119, 111, 10, 100, 97, 116, 97, 32, 100, 97,
    116, 97, 10, 10, 67, 104, 117, 110, 107, 32, 116, 104, 114, 101, 101, 10, 100, 97, 116, 97, 32,
    100, 97, 116, 97, 10, 3,
];

#[async_std::test]
async fn raw_bytes_uncompressed_default() {
    let mut app = tide::new();
    app.with(
        tide_compress::CompressMiddleware::builder()
            .threshold(16)
            .build(),
    );
    app.at("/").get(|_| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(TEXT.as_bytes().to_owned());
        Ok(res)
    });

    let mut req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "br");
    let mut res: tide::http::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert_eq!(
        res[headers::CONTENT_TYPE],
        http_types::mime::BYTE_STREAM.to_string()
    );
    assert!(res.header(headers::TRANSFER_ENCODING).is_none());
    // XXX(Jeremiah): Content-Length should be set ...?
    //
    // assert_eq!(res[headers::CONTENT_LENGTH], "64");
    assert!(res.header(headers::CONTENT_ENCODING).is_none());
    assert_eq!(res[headers::VARY], "accept-encoding");
    assert_eq!(res.body_string().await.unwrap(), TEXT);
}

#[async_std::test]
async fn raw_bytes_compressed_no_filter() {
    let mut app = tide::new();
    app.with(
        tide_compress::CompressMiddleware::builder()
            .threshold(16)
            .content_type_check(None)
            .build(),
    );
    app.at("/").get(|_| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(TEXT.as_bytes().to_owned());
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

#[async_std::test]
async fn compressible_content_type() {
    let mut app = tide::new();
    app.with(
        tide_compress::CompressMiddleware::builder()
            .threshold(16)
            .build(),
    );
    app.at("/").get(|_| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(TEXT.as_bytes().to_owned());
        res.set_content_type("font/ttf".parse::<tide::http::Mime>()?);
        Ok(res)
    });

    let mut req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "br");
    let mut res: tide::http::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert_eq!(res[headers::CONTENT_TYPE], "font/ttf");
    assert!(res.header(headers::CONTENT_LENGTH).is_none());
    assert_eq!(res[headers::CONTENT_ENCODING], "br");
    assert_eq!(res[headers::VARY], "accept-encoding");
    assert_eq!(res.body_bytes().await.unwrap(), BR_COMPRESSED);
}
