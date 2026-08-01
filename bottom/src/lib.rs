use {
    futures::stream::{FuturesUnordered, TryStreamExt},
    line_count::{
        exports::demo::demo::line_count::{Guest, LineCount},
        wasi::http::types::ErrorCode,
        wit_future, wit_stream,
    },
    wit_bindgen::{FutureReader, StreamReader},
};

struct Component;

impl Guest for Component {
    async fn count_lines(
        urls: Vec<String>,
    ) -> (StreamReader<LineCount>, FutureReader<Result<(), ErrorCode>>) {
        let (mut stream_tx, stream_rx) = wit_stream::new();
        let (future_tx, future_rx) = wit_future::new(|| Ok(()));

        let mut results = urls
            .into_iter()
            .map(|url| line_count::count_lines(url, "bottom"))
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

line_count::export!(Component with_types_in line_count);
