use {
    exports::demo::demo::transformer::Guest,
    wit_bindgen::{StreamReader, StreamResult},
};

wit_bindgen::generate!({
    world: "demo:demo/bottom",
    path: "../wit",
    generate_all,
});

struct Component;

impl Guest for Component {
    async fn transform(mut stream: StreamReader<String>) -> StreamReader<String> {
        // Note that we can't just return `stream` here; we need to create a new
        // one and pipe to it so that the caller doesn't end up reading from its
        // own stream, because intra-component non-unit stream reads and writes
        // are not currently allowed in the Component Model.
        let (mut tx, rx) = wit_stream::new();

        wit_bindgen::spawn_local(async move {
            while let (StreamResult::Complete(_), values) = stream.read(Vec::with_capacity(8)).await
            {
                tx.write_all(values).await;
            }
        });

        rx
    }
}

export!(Component);
