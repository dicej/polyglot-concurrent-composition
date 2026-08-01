middles = \
	middle-rust/target/wasm32-wasip2/release/middle.wasm \
	middle-python/component.wasm \
	middle-javascript/component.wasm \
	middle-go/component.wasm \

.PHONY: build
build: top bottom middle-rust middle-python/component.wasm middle-javascript/component.wasm middle-go/component.wasm
	cp top/target/wasm32-wasip2/release/top.wasm composed.wasm
	for component in $(middles) bottom/target/wasm32-wasip2/release/bottom.wasm; do \
		wac plug composed.wasm --plug $$component -o composed.wasm; done

.PHONY: run
run: build
	WASMTIME_BACKTRACE_DETAILS=1 wasmtime run -Shttp composed.wasm \
		"https://bytecodealliance.org/" \
		"https://rust-lang.org/" \
		"https://www.python.org/" \
		"https://tc39.es" \
		"https://go.dev"

.PHONY: top
top:
	cargo build --manifest-path top/Cargo.toml --release --target wasm32-wasip2

.PHONY: middle-rust
middle-rust:
	cargo build --manifest-path middle-rust/Cargo.toml --release --target wasm32-wasip2

middle-python/component.wasm:
	bash middle-python/build.sh

middle-javascript/component.wasm:
	cd middle-javascript && \
		componentize-js -d ../wit -w "demo:demo/middle" componentize component.js -o component.wasm

middle-go/component.wasm:
	cd middle-go && \
		go mod tidy && \
		cp go.mod hide.go.mod && \
		go tool componentize-go -d ../wit -w "demo:demo/middle" bindings && \
		mv hide.go.mod go.mod && \
		go tool componentize-go -d ../wit -w "demo:demo/middle" build -o component.wasm

.PHONY: bottom
bottom:
	cargo build --manifest-path bottom/Cargo.toml --release --target wasm32-wasip2
