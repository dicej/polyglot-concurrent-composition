.PHONY: build
build: top bottom
	wac plug top/target/wasm32-wasip2/release/top.wasm --plug bottom/target/wasm32-wasip2/release/bottom.wasm -o composed.wasm

.PHONY: run
run: build
	wasmtime run -Shttp composed.wasm "https://bytecodealliance.org/"

.PHONY: top
top:
	cargo build --manifest-path top/Cargo.toml --release --target wasm32-wasip2

.PHONY: bottom
bottom:
	cargo build --manifest-path bottom/Cargo.toml --release --target wasm32-wasip2
