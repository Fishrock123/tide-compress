mod test_utils;

use std::time::Duration;

use async_std::io::Cursor;
use async_std::prelude::*;
use async_std::task;
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

#[async_std::test]
async fn no_accepts_encoding() -> Result<(), http_types::Error> {
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
        let mut res = surf::get(format!("http://{}", port)).await?;
        assert_eq!(res.status(), 200);
        assert_eq!(res.header(&"Content-Length".parse().unwrap()), None);
        assert_eq!(res.header(&"Content-Encoding".parse().unwrap()), None);
        let str = res.body_string().await?;
        assert_eq!(str, TEXT);
        Ok(())
    });

    server.race(client).await
}

#[async_std::test]
async fn accepts_non_gzip_encopding() -> Result<(), http_types::Error> {
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
            .set_header("Accept-Encoding".parse().unwrap(), "br")
            .await?;
        assert_eq!(res.status(), 200);
        assert_eq!(res.header(&"Content-Length".parse().unwrap()), None);
        assert_eq!(res.header(&"Content-Encoding".parse().unwrap()), None);
        let str = res.body_string().await?;
        assert_eq!(str, TEXT);
        Ok(())
    });

    server.race(client).await
}

#[async_std::test]
async fn head_request() -> Result<(), http_types::Error> {
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
            surf::head(format!("http://{}", port))
            .set_header("Accept-Encoding".parse().unwrap(), "gzip")
            .await?;
        assert_eq!(res.status(), 200);
        assert_eq!(res.header(&"Content-Length".parse().unwrap()), None);
        assert_eq!(res.header(&"Content-Encoding".parse().unwrap()), None);
        let str = res.body_string().await?;
        assert_eq!(str, "");
        Ok(())
    });

    server.race(client).await
}
