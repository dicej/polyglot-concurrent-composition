use {
    demo::demo::line_count, exports::wasi::cli::run::Guest, wasi::cli::environment,
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
            eprintln!("please specify one or more URLs");
            return Err(());
        }

        let (mut stream, future) =
            line_count::count_lines(arguments.into_iter().skip(1).collect()).await;

        while let (StreamResult::Complete(_), value) = stream.read(Vec::with_capacity(1)).await {
            if let [value] = value.as_slice() {
                println!(
                    "{} line count: {}; retriever: {}{}",
                    value.url,
                    value.count,
                    value.retriever,
                    if value.deferrers.is_empty() {
                        String::new()
                    } else {
                        format!("; deferrers: {}", value.deferrers.join(", "))
                    }
                );
            }
        }

        if let Err(error) = future.await {
            eprintln!("error retrieving line counts: {error:?}");
            return Err(());
        }

        Ok(())
    }
}

export!(Component);
