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
async fn no_accepts_encoding() {
    let mut app = tide::new();
    app.middleware(tide_compress::CompressMiddleware::with_threshold(16));
    app.at("/").get(|_| async {
        let res = Response::new(StatusCode::Ok)
            .body_string(TEXT.to_owned())
            .set_mime("text/plain; charset=utf-8".parse().unwrap());
        Ok(res)
    });

    let req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    let res: http_types::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(res.header(headers::CONTENT_LENGTH).is_none());
    assert!(res.header(headers::CONTENT_ENCODING).is_none());
    assert_eq!(res.body_string().await.unwrap(), TEXT);
}

#[async_std::test]
async fn invalid_accepts_encoding() {
    let mut app = tide::new();
    app.middleware(tide_compress::CompressMiddleware::with_threshold(16));
    app.at("/").get(|_| async {
        let res = Response::new(StatusCode::Ok)
            .body_string(TEXT.to_owned())
            .set_mime("text/plain; charset=utf-8".parse().unwrap());
        Ok(res)
    });

    let mut req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "not_an_encoding");
    let res: http_types::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(res.header(headers::CONTENT_LENGTH).is_none());
    assert!(res.header(headers::CONTENT_ENCODING).is_none());
    assert_eq!(res.body_string().await.unwrap(), TEXT);
}

#[async_std::test]
async fn head_request() {
    let mut app = tide::new();
    app.middleware(tide_compress::CompressMiddleware::with_threshold(16));
    app.at("/").get(|_| async {
        let res = Response::new(StatusCode::Ok)
            .body_string(TEXT.to_owned())
            .set_mime("text/plain; charset=utf-8".parse().unwrap());
        Ok(res)
    });

    let mut req = Request::new(Method::Head, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "gzip");
    let res: http_types::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(res.header(headers::CONTENT_LENGTH).is_none());
    assert!(res.header(headers::CONTENT_ENCODING).is_none());
    // XXX(Jeremiah): seems like async-h1 or tide may mishandle HEAD requests.
    // HEAD requests should never have a body.
    //
    // assert_eq!(res.body_string().await.unwrap(), "");
}

#[async_std::test]
async fn below_threshold_request() {
    let mut app = tide::new();
    app.middleware(tide_compress::CompressMiddleware::new());
    app.at("/").get(|_| async {
        let res = Response::new(StatusCode::Ok)
            .body_string(TEXT.to_owned())
            .set_mime("text/plain; charset=utf-8".parse().unwrap());
        Ok(res)
    });

    let mut req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "gzip");
    let res: http_types::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(res.header(headers::TRANSFER_ENCODING).is_none());
    // XXX(Jeremiah): Content-Length should be set ...?
    //
    // assert_eq!(res[headers::CONTENT_LENGTH], "64");
    assert!(res.header(headers::CONTENT_ENCODING).is_none());
    assert_eq!(res.body_string().await.unwrap(), TEXT);
}

#[async_std::test]
async fn cache_control() {
    let mut app = tide::new();
    app.middleware(tide_compress::CompressMiddleware::with_threshold(16));
    app.at("/").get(|_| async {
        let res = Response::new(StatusCode::Ok)
            .body_string(TEXT.to_owned())
            .set_mime("text/plain; charset=utf-8".parse().unwrap())
            .set_header(headers::CACHE_CONTROL, "no-transform");
        Ok(res)
    });

    let mut req = Request::new(Method::Get, Url::parse("http://_/").unwrap());
    req.insert_header(headers::ACCEPT_ENCODING, "gzip");
    let res: http_types::Response = app.respond(req).await.unwrap();

    assert_eq!(res.status(), 200);
    assert!(res.header(headers::TRANSFER_ENCODING).is_none());
    assert!(res.header(headers::CONTENT_ENCODING).is_none());
    assert_eq!(res.body_string().await.unwrap(), TEXT);
}
