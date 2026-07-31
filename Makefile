.PHONY: build
build: top bottom middle-rust middle-python/component.wasm
	wac plug \
		top/target/wasm32-wasip2/release/top.wasm \
		--plug middle-rust/target/wasm32-wasip2/release/middle.wasm \
		-o composed0.wasm
	wac plug \
		composed0.wasm \
		--plug middle-python/component.wasm \
		-o composed1.wasm
	wac plug \
		composed1.wasm \
		--plug bottom/target/wasm32-wasip2/release/bottom.wasm \
		-o composed.wasm

.PHONY: run
run: build
	wasmtime run -Shttp composed.wasm \
		"https://bytecodealliance.org/" \
		"https://rust-lang.org/" \
		"https://www.python.org/"

.PHONY: top
top:
	cargo build --manifest-path top/Cargo.toml --release --target wasm32-wasip2

.PHONY: middle-rust
middle-rust:
	cargo build --manifest-path middle-rust/Cargo.toml --release --target wasm32-wasip2

middle-python/component.wasm:
	bash middle-python/build.sh

.PHONY: bottom
bottom:
	cargo build --manifest-path bottom/Cargo.toml --release --target wasm32-wasip2
