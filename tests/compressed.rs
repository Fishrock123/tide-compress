mod test_utils;

use std::time::Duration;

use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task;

use async_h1::client;
use http_types::headers::HeaderValue;
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
async fn brotli_compressed() -> Result<(), http_types::Error> {
    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = tide::new();
        app.middleware(tide_compress::CompressMiddleware::with_threshold(16));
        app.at("/").get(|mut _req: tide::Request<()>| async move {
            let res = Response::new(StatusCode::Ok)
                .body_string(TEXT.to_owned())
                .set_header(headers::CONTENT_TYPE, "text/plain; charset=utf-8");
            Ok(res)
        });
        app.listen(&port).await?;
        Result::<(), http_types::Error>::Ok(())
    });

    let client = task::spawn(async move {
        task::sleep(Duration::from_millis(100)).await;

        let stream = TcpStream::connect(port).await?;
        let peer_addr = stream.peer_addr()?;
        let url = Url::parse(&format!("http://{}", peer_addr))?;
        let mut req = Request::new(Method::Get, url);
        req.insert_header("Accept-Encoding", "br")?;

        let mut res = client::connect(stream.clone(), req).await?;

        assert_eq!(res.header(&"Content-Length".parse().unwrap()), None);
        assert_eq!(
            res.header(&"Content-Encoding".parse().unwrap()),
            Some(&vec![HeaderValue::from_ascii(b"br").unwrap()])
        );
        let mut bytes = Vec::with_capacity(1024);
        res.read_to_end(&mut bytes).await?;
        assert_eq!(bytes.as_slice(), BR_COMPRESSED);

        Result::<(), http_types::Error>::Ok(())
    });

    server.race(client).await
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
async fn gzip_compressed() -> Result<(), http_types::Error> {
    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = tide::new();
        app.middleware(tide_compress::CompressMiddleware::with_threshold(16));
        app.at("/").get(|mut _req: tide::Request<()>| async move {
            let res = Response::new(StatusCode::Ok)
                .body_string(TEXT.to_owned())
                .set_header(headers::CONTENT_TYPE, "text/plain; charset=utf-8")
                .set_header("Content-Encoding".parse().unwrap(), "identity");
            Ok(res)
        });
        app.listen(&port).await?;
        Result::<(), http_types::Error>::Ok(())
    });

    let client = task::spawn(async move {
        task::sleep(Duration::from_millis(100)).await;

        let stream = TcpStream::connect(port).await?;
        let peer_addr = stream.peer_addr()?;
        let url = Url::parse(&format!("http://{}", peer_addr))?;
        let mut req = Request::new(Method::Get, url);
        req.insert_header("Accept-Encoding", "gzip")?;

        let mut res = client::connect(stream.clone(), req).await?;

        assert_eq!(res.header(&"Content-Length".parse().unwrap()), None);
        assert_eq!(
            res.header(&"Content-Encoding".parse().unwrap()),
            Some(&vec![HeaderValue::from_ascii(b"gzip").unwrap()])
        );
        let mut bytes = Vec::with_capacity(1024);
        res.read_to_end(&mut bytes).await?;
        assert_eq!(bytes.as_slice(), GZIPPED);

        Result::<(), http_types::Error>::Ok(())
    });

    server.race(client).await
}

#[cfg(feature = "deflate")]
const DEFLATED: &'static [u8] = &[
    0x6d, 0xca, 0xb1, 0x09, 0x00, 0x30, 0x08, 0x05, 0xd1, 0xfe, 0x4f, 0xe1, 0x2e, 0x4e, 0x22, 0x44,
    0x10, 0x02, 0x0a, 0xc1, 0x90, 0xf5, 0x43, 0x52, 0x59, 0xd8, 0x5c, 0xf1, 0x38, 0xb6, 0xed, 0x93,
    0xc2, 0x15, 0x43, 0x52, 0xe8, 0x05, 0xe0, 0x8f, 0x79, 0xa2, 0x41, 0x5b, 0x5a, 0xdf, 0x0b,
];

#[cfg(feature = "deflate")]
#[async_std::test]
async fn deflate_compressed() -> Result<(), http_types::Error> {
    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = tide::new();
        app.middleware(tide_compress::CompressMiddleware::with_threshold(16));
        app.at("/").get(|mut _req: tide::Request<()>| async move {
            let res = Response::new(StatusCode::Ok)
                .body_string(TEXT.to_owned())
                .set_header(headers::CONTENT_TYPE, "text/plain; charset=utf-8");
            Ok(res)
        });
        app.listen(&port).await?;
        Result::<(), http_types::Error>::Ok(())
    });

    let client = task::spawn(async move {
        task::sleep(Duration::from_millis(100)).await;

        let stream = TcpStream::connect(port).await?;
        let peer_addr = stream.peer_addr()?;
        let url = Url::parse(&format!("http://{}", peer_addr))?;
        let mut req = Request::new(Method::Get, url);
        req.insert_header("Accept-Encoding", "deflate")?;

        let mut res = client::connect(stream.clone(), req).await?;

        assert_eq!(res.header(&"Content-Length".parse().unwrap()), None);
        assert_eq!(
            res.header(&"Content-Encoding".parse().unwrap()),
            Some(&vec![HeaderValue::from_ascii(b"deflate").unwrap()])
        );
        let mut bytes = Vec::with_capacity(1024);
        res.read_to_end(&mut bytes).await?;
        assert_eq!(bytes.as_slice(), DEFLATED);

        Result::<(), http_types::Error>::Ok(())
    });

    server.race(client).await
}
