use {
    exports::demo::demo::line_count::{Guest, LineCount},
    futures::stream::{FuturesUnordered, TryStreamExt},
    url::Url,
    wasi::http::{
        client,
        types::{ErrorCode, Headers, Request, Response, Scheme},
    },
    wit_bindgen::{FutureReader, StreamReader, StreamResult},
};

wit_bindgen::generate!({
    world: "demo:demo/bottom",
    path: "../wit",
    generate_all,
});

struct Component;

impl Guest for Component {
    async fn count_lines(
        urls: Vec<String>,
    ) -> (StreamReader<LineCount>, FutureReader<Result<(), ErrorCode>>) {
        let (mut stream_tx, stream_rx) = wit_stream::new();
        let (future_tx, future_rx) = wit_future::new(|| Ok(()));

        let mut results = urls
            .into_iter()
            .map(count_lines)
            .collect::<FuturesUnordered<_>>();

        wit_bindgen::spawn_local(async move {
            _ = future_tx
                .write(
                    async move {
                        while let Some(result) = results.try_next().await? {
                            if stream_tx.write_one(result).await.is_some() {
                                break;
                            }
                        }
                        Ok(())
                    }
                    .await,
                )
                .await;
        });

        (stream_rx, future_rx)
    }
}

export!(Component);

async fn count_lines(url_string: String) -> Result<LineCount, ErrorCode> {
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
        retriever: "bottom".into(),
        deferrers: Vec::new(),
    })
}
