mod test_utils;

use std::time::Duration;

use async_std::io::Cursor;
use async_std::prelude::*;
use async_std::task;
use http_types::headers::HeaderValue;
use http_types::{headers, StatusCode};
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

const GZIPPED: &'static [u8] = &[
    // It should be this but miniz / miniz_oxide's gzip compression is rather lacking.
    //
    //     0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // gzip header
    //     0x13, // OS type
    //     0x73, 0xce, 0x28, 0xcd, 0xcb, 0x56,
    //     0xc8, 0xcf, 0x4b, 0xe5, 0x4a, 0x49, 0x2c, 0x49, 0x54, 0x00, 0x11, 0x5c, 0x5c, 0xce, 0x60, 0xc1,
    //     0x92, 0xf2, 0x7c, 0x2c, 0x82, 0x19, 0x45, 0xa9, 0xc8, 0x6a, 0x01, 0xde, 0xf2, 0xd7, 0x81, 0x40,
    //     0x00, 0x00, 0x00
    //
    0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // gzip header
    0xff, // OS type
    0x6d, 0xca, 0xb1, 0x09, 0x00, 0x30, 0x08, 0x05, 0xd1, 0xfe, 0x4f, 0xe1,
    0x2e, 0x4e, 0x22, 0x44, 0x10, 0x02, 0x0a, 0xc1, 0x90, 0xf5, 0x43, 0x52, 
    0x59, 0xd8, 0x5c, 0xf1, 0x38, 0xb6, 0xed, 0x93, 0xc2, 0x15, 0x43, 0x52, 
    0xe8, 0x05, 0xe0, 0x8f, 0x79, 0xa2, 0x41, 0x5b, 0x5a, 0xdf, 0x0b,
    0xde, 0xf2, 0xd7, 0x81, // crc32
    0x40, 0x00, 0x00, 0x00, // input size    
];

#[async_std::test]
async fn chunked_large() -> Result<(), http_types::Error> {
    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = tide::new();
        app.middleware(tide_compress::CompressMiddleware);
        app.at("/").get(|mut _req: tide::Request<()>| async move {
            let body = Cursor::new(TEXT.to_owned());
            let res = Response::new(StatusCode::Ok)
                .body(body)
                .set_header(headers::CONTENT_TYPE, "text/plain; charset=utf-8");
            Ok(res)
        });
        app.listen(&port).await?;
        Result::<(), http_types::Error>::Ok(())
    });

    let client = task::spawn(async move {
        task::sleep(Duration::from_millis(100)).await;
        let mut res = 
            surf::get(format!("http://{}", port))
            .set_header("Accept-Encoding".parse().unwrap(), "gzip")
            .await?;
        assert_eq!(res.status(), 200);
        let bytes = res.body_bytes().await?;
        assert_eq!(bytes.as_slice(), GZIPPED);
        assert_eq!(res.header(&"Content-Length".parse().unwrap()), None);
        assert_eq!(
            res.header(&"Content-Encoding".parse().unwrap()),
            Some(&vec![HeaderValue::from_ascii(
                b"gzip"
            ).unwrap()])
        );
        Ok(())
    });

    server.race(client).await
}
