use {
    demo::demo::transformer, exports::wasi::cli::run::Guest, wasi::cli::environment,
    wit_bindgen::StreamResult,
};

wit_bindgen::generate!({
    world: "demo:demo/top",
    path: "../wit",
    generate_all,
});

struct Component;

impl Guest for Component {
    async fn run() -> Result<(), ()> {
        let arguments = environment::get_arguments();
        if arguments.len() == 1 {
            eprintln!("please specify one or more arguments");
            return Err(());
        }

        let (mut input_tx, input_rx) = wit_stream::new();
        wit_bindgen::spawn_local(async move {
            for argument in arguments.into_iter().skip(1) {
                if input_tx.write_one(argument).await.is_some() {
                    break;
                }
            }
        });

        let mut output_rx = transformer::transform(input_rx).await;
        while let (StreamResult::Complete(_), values) = output_rx.read(Vec::with_capacity(1)).await
        {
            for value in values {
                println!("{value}");
            }
        }

        Ok(())
    }
}

export!(Component);
