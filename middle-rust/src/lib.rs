use {
    futures::{channel::mpsc, sink::SinkExt, stream::TryStreamExt},
    line_count::{
        demo::demo::line_count::{LineCount, count_lines as defer},
        exports::demo::demo::line_count::Guest,
        wasi::http::types::ErrorCode,
        wit_future, wit_stream,
    },
    wit_bindgen::{FutureReader, StreamReader, StreamResult},
};

struct Component;

impl Guest for Component {
    async fn count_lines(
        urls: Vec<String>,
    ) -> (StreamReader<LineCount>, FutureReader<Result<(), ErrorCode>>) {
        let (mut stream_tx, stream_rx) = wit_stream::new();
        let (future_tx, future_rx) = wit_future::new(|| Ok(()));

        let (to_retrieve, to_defer) = urls
            .into_iter()
            .partition::<Vec<_>, _>(|url| url.contains("://rust-lang.org"));

        let (mut mpsc_tx, mut mpsc_rx) = mpsc::channel(1);

        wit_bindgen::spawn_local({
            let mut mpsc_tx = mpsc_tx.clone();
            async move {
                for url in to_retrieve {
                    _ = mpsc_tx.send(line_count::count_lines(url).await).await;
                }
            }
        });

        wit_bindgen::spawn_local(async move {
            let (mut stream, future) = defer(to_defer).await;

            while let (StreamResult::Complete(_), values) = stream.read(Vec::with_capacity(1)).await
            {
                for mut value in values {
                    value.deferrers.push("rust".into());
                    _ = mpsc_tx.send(Ok(value)).await;
                }
            }

            if let Err(error) = future.await {
                _ = mpsc_tx.send(Err(error)).await;
            }
        });

        wit_bindgen::spawn_local(async move {
            _ = future_tx
                .write(
                    async move {
                        while let Some(result) = TryStreamExt::try_next(&mut mpsc_rx).await? {
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
