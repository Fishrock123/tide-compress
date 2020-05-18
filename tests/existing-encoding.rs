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
async fn existing_encoding() -> Result<(), http_types::Error> {
    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = tide::new();
        app.middleware(tide_compress::CompressMiddleware);
        app.at("/").get(|mut _req: tide::Request<()>| async move {
            let body = Cursor::new(TEXT.to_owned());
            let res = Response::new(StatusCode::Ok)
                .body(body)
                .set_header(headers::CONTENT_TYPE, "text/plain; charset=utf-8")
                .set_header("Content-Encoding".parse().unwrap(), "some-format");
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

        let res = client::connect(stream.clone(), req).await?;

        assert_eq!(res.status(), 200);
        assert_eq!(res.header(&"Content-Length".parse().unwrap()), None);
        assert_eq!(
            res.header(&"Content-Encoding".parse().unwrap()),
            Some(&vec!(
                headers::HeaderValue::from_ascii(b"some-format").unwrap()
            ))
        );
        let str = res.body_string().await?;
        assert_eq!(str, TEXT);
        Ok(())
    });

    server.race(client).await
}

#[async_std::test]
async fn multi_existing_encoding() -> Result<(), http_types::Error> {
    let port = test_utils::find_port().await;
    let server = task::spawn(async move {
        let mut app = tide::new();
        app.middleware(tide_compress::CompressMiddleware);
        app.at("/").get(|mut _req: tide::Request<()>| async move {
            let body = Cursor::new(TEXT.to_owned());
            let res = Response::new(StatusCode::Ok)
                .body(body)
                .set_header(headers::CONTENT_TYPE, "text/plain; charset=utf-8")
                .set_header("Content-Encoding".parse().unwrap(), "gzip, identity");
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

        let res = client::connect(stream.clone(), req).await?;

        assert_eq!(res.status(), 200);
        assert_eq!(res.header(&"Content-Length".parse().unwrap()), None);
        assert_eq!(
            res.header(&"Content-Encoding".parse().unwrap()),
            Some(&vec!(
                headers::HeaderValue::from_ascii(b"gzip, identity").unwrap()
            ))
        );
        let str = res.body_string().await?;
        assert_eq!(str, TEXT);
        Ok(())
    });

    server.race(client).await
}
