mod test_utils;

use std::time::Duration;

use async_std::io::Cursor;
use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::task;

use async_h1::client;
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

#[async_std::test]
async fn no_accepts_encoding() -> Result<(), http_types::Error> {
    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = tide::new();
        app.middleware(tide_compress::CompressMiddleware::new());
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

        let stream = TcpStream::connect(port).await?;
        let peer_addr = stream.peer_addr()?;
        let url = Url::parse(&format!("http://{}", peer_addr))?;
        let req = Request::new(Method::Get, url);
        let res = client::connect(stream.clone(), req).await?;

        assert_eq!(res.status(), 200);
        assert!(res.header(headers::CONTENT_LENGTH).is_none());
        assert!(res.header(headers::CONTENT_ENCODING).is_none());
        let str = res.body_string().await?;
        assert_eq!(str, TEXT);
        Ok(())
    });

    server.race(client).await
}

#[async_std::test]
async fn invalid_accepts_encoding() -> Result<(), http_types::Error> {
    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = tide::new();
        app.middleware(tide_compress::CompressMiddleware::new());
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

        let stream = TcpStream::connect(port).await?;
        let peer_addr = stream.peer_addr()?;
        let url = Url::parse(&format!("http://{}", peer_addr))?;
        let mut req = Request::new(Method::Get, url);
        req.insert_header(headers::ACCEPT_ENCODING, "not_an_encoding");

        let res = client::connect(stream.clone(), req).await?;

        assert_eq!(res.status(), 200);
        assert!(res.header(headers::CONTENT_LENGTH).is_none());
        assert!(res.header(headers::CONTENT_ENCODING).is_none());
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
        app.middleware(tide_compress::CompressMiddleware::new());
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

        let stream = TcpStream::connect(port).await?;
        let peer_addr = stream.peer_addr()?;
        let url = Url::parse(&format!("http://{}", peer_addr))?;
        let mut req = Request::new(Method::Head, url);
        req.insert_header(headers::ACCEPT_ENCODING, "gzip");

        let res = client::connect(stream.clone(), req).await?;

        assert_eq!(res.status(), 200);
        assert!(res.header(headers::CONTENT_LENGTH).is_none());
        assert!(res.header(headers::CONTENT_ENCODING).is_none());
        // XXX(Jeremiah): seems like async-h1 or tide may mishandle HEAD requests.
        // HEAD requests should never have a body.
        //
        // let str = res.body_string().await?;
        // assert_eq!(str, "");
        Ok(())
    });

    server.race(client).await
}

#[async_std::test]
async fn below_threshold_request() -> Result<(), http_types::Error> {
    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = tide::new();
        app.middleware(tide_compress::CompressMiddleware::new());
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
        req.insert_header(headers::ACCEPT_ENCODING, "gzip");

        let res = client::connect(stream.clone(), req).await?;

        assert_eq!(res.status(), 200);
        assert!(res.header(headers::TRANSFER_ENCODING).is_none());
        assert_eq!(res[headers::CONTENT_LENGTH], "64");
        assert!(res.header(headers::CONTENT_ENCODING).is_none());
        let str = res.body_string().await?;
        assert_eq!(str, TEXT);
        Ok(())
    });

    server.race(client).await
}
