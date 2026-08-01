use {
    exports::demo::demo::line_count::LineCount,
    url::Url,
    wasi::http::{
        client,
        types::{ErrorCode, Headers, Request, Response, Scheme},
    },
    wit_bindgen::StreamResult,
};

wit_bindgen::generate!({
    world: "demo:demo/middle",
    path: "../wit",
    generate_all,
    pub_export_macro: true,
    merge_structurally_equal_types: true,
});

pub async fn count_lines(url_string: String, retriever: &str) -> Result<LineCount, ErrorCode> {
    let url = Url::parse(&url_string).map_err(|v| {
        ErrorCode::InternalError(Some(format!("error parsing URL `{url_string}`: {v:?}")))
    })?;

    let request = Request::new(Headers::new(), None, wit_future::new(|| Ok(None)).1, None).0;

    request
        .set_scheme(Some(&match url.scheme() {
            "https" => Scheme::Https,
            "http" => Scheme::Http,
            scheme => Scheme::Other(scheme.into()),
        }))
        .map_err(|()| {
            ErrorCode::InternalError(Some(format!("unsupported scheme for URL `{url_string}`")))
        })?;

    request
        .set_path_with_query(Some(url.path()))
        .map_err(|()| {
            ErrorCode::InternalError(Some(format!("unsupported path for URL `{url_string}`")))
        })?;

    request.set_authority(Some(url.authority())).map_err(|()| {
        ErrorCode::InternalError(Some(format!(
            "unsupported authority for URL `{url_string}`"
        )))
    })?;

    let response = client::send(request).await?;
    let status = response.get_status_code();
    if !(200..300).contains(&status) {
        return Err(ErrorCode::InternalError(Some(format!(
            "unexpected response status for URL `{url_string}`: {status}"
        ))));
    }

    let (mut body, trailers) = Response::consume_body(response, wit_future::new(|| Ok(())).1);

    let mut count = 0;
    let mut status = StreamResult::Complete(0);
    let mut chunk = Vec::with_capacity(64 * 1024);

    while let StreamResult::Complete(_) = status {
        (status, chunk) = body.read(chunk).await;
        count += u64::try_from(chunk.iter().filter(|&&v| v == b'\n').count()).unwrap();
        chunk.clear();
    }

    _ = trailers.await?;

    Ok(LineCount {
        url: url_string,
        count,
        retriever: retriever.into(),
        deferrers: Vec::new(),
    })
}
