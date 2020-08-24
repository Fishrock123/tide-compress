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

#[async_std::test]
async fn existing_encoding() {
    let mut app = tide::new();
    app.with(tide_compress::CompressMiddleware::with_threshold(16));
    app.at("/").get(|_| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(TEXT.to_owned());
        res.insert_header(headers::CONTENT_ENCODING, "deflate");
        Ok(res)
    });

    let mut req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "gzip");
    let mut res: tide::http::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(res.header(headers::CONTENT_LENGTH).is_none());
    assert_eq!(res[headers::CONTENT_ENCODING], "deflate");
    assert_eq!(res.body_string().await.unwrap(), TEXT);
}

#[async_std::test]
async fn multi_existing_encoding() {
    let mut app = tide::new();
    app.with(tide_compress::CompressMiddleware::with_threshold(16));
    app.at("/").get(|_| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(TEXT.to_owned());
        res.append_header(headers::CONTENT_ENCODING, "identity");
        res.append_header(headers::CONTENT_ENCODING, "gzip");
        Ok(res)
    });

    let mut req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "gzip");
    let mut res: tide::http::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(res.header(headers::CONTENT_LENGTH).is_none());
    assert_eq!(res[headers::CONTENT_ENCODING][0].as_str(), "identity");
    assert_eq!(res[headers::CONTENT_ENCODING][1].as_str(), "gzip");
    assert_eq!(res.body_string().await.unwrap(), TEXT);
}
