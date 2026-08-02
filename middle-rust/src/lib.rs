use {
    demo::demo::transformer,
    exports::demo::demo::transformer::Guest,
    wit_bindgen::{StreamReader, StreamResult},
};

wit_bindgen::generate!({
    world: "demo:demo/middle",
    path: "../wit",
    generate_all,
});

struct Component;

impl Guest for Component {
    async fn transform(stream: StreamReader<String>) -> StreamReader<String> {
        map(
            transformer::transform(map(stream, |v| format!("🦀{v}"))).await,
            |v| format!("{v}🦀"),
        )
    }
}

export!(Component);

fn map(
    mut stream: StreamReader<String>,
    fun: impl Fn(String) -> String + Copy + 'static,
) -> StreamReader<String> {
    let (mut tx, rx) = wit_stream::new();

    wit_bindgen::spawn_local(async move {
        while let (StreamResult::Complete(_), values) = stream.read(Vec::with_capacity(8)).await {
            tx.write_all(values.into_iter().map(fun).collect()).await;
        }
    });

    rx
}
