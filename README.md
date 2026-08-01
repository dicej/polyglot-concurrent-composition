# Polyglot Concurrency Using the WebAssembly Component Model

This is a demonstration of how to compose WebAssembly components written in a
variety of programming languages using concurrent inter-component function calls
and streaming.

This example is a CLI application which accepts one or more URLs as arguments,
retrieves the URLs concurrently, and prints the number of lines received in the
response body for each URL.  The application is a composition of several
components written in various languages:

- `top` (Rust): exports `wasi:cli/run` and imports `line-count`, an interface
  which contains an asynchronous function for retrieving the line counts for a
  list of URLs.  This component calls that function and prints the stream of
  results as they arrive.

- `middle-rust` (Rust): exports _and_ imports `line-count`, and imports
  `wasi:http/client`.  When called, it retrieves and counts the lines for any
  URLs with `rust-lang.org` as the authority, deferring any other URLs to the
  `line-count` import.  All the results (direct and deferred) are returned via a
  `stream` of `line-count` records.
  
- `middle-python` (Python): Like `middle-rust`, except handles any
  `www.python.org` URLs itself, deferring other URLs to the `line-count` import.
  Note that, as of this writing `www.python.org` always returns gzipped
  responses, so the "line count" is really just a count of byte values equal to
  10 (the ASCII newline character) that happen to appear in the compressed
  stream.
  
- `middle-javascript` (JavaScript): Like `middle-rust`, except handles any
  `tc39.es` URLs itself, deferring other URLs to the `line-count` import.
    
- `middle-go` (JavaScript): Like `middle-rust`, except handles any
  `go.dev` URLs itself, deferring other URLs to the `line-count` import.
  
- `bottom` (Rust): exports `line-count` and imports `wasi:http/client`.  When
  called, it retrieves and counts the lines for all the URLs it receives.
  
Whenever a component retrieves a result itself, it includes its name in the
result, and whenever it defers to the next component in line it adds itself to
the lest of deferrers.

## Building and Running

### Prerequisites

- Make and Bash
- Rust 1.97 or later, including the `wasm32-wasip2` target
    - e.g. `rustup target add wasm32-wasip2`
- Python 3.14 or later
- Go 1.25 or later
- Wasmtime 47.0.2
- wac 0.10.1
- WASI-SDK 30
- componentize-js Git commit `bdd7c3d5` 

After installing WASI-SDK, point `WASI_SDK_PATH` to wherever you installed it.
For example, follow the steps below, replacing `arm64-linux` with
`x86_64-linux`, `arm64-macos`, `x86_64-macos`, `arm64-windows`, or
`x86_64-windows` depending on your architecture and OS, if necessary.

```shell
curl -LO https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-30/wasi-sdk-30.0-arm64-linux.tar.gz
tar xf wasi-sdk-30.0-arm64-linux.tar.gz
export WASI_SDK_PATH=$(pwd)/wasi-sdk-30.0-arm64-linux
```

Once you have Rust and WASI-SDK, you can get Wasmtime, wac, and componentize-js using `cargo`:

```
cargo install --version 47.0.2 wasmtime-cli
cargo install --version 0.10.1 wac-cli
cargo install --git https://github.com/dicej/componentize-js --rev bdd7c3d5
```

Once you have all the prereqs, you should be able to build and run using:

```
make run
```

If all goes well, the output should end something like this:

```
https://www.python.org/ line count: 35; retriever: python; deferrers: rust
https://bytecodealliance.org/ line count: 461; retriever: bottom; deferrers: go, javascript, python, rust
https://go.dev line count: 1402; retriever: go; deferrers: javascript, python, rust
https://tc39.es line count: 937; retriever: javascript; deferrers: python, rust
https://rust-lang.org/ line count: 390; retriever: rust
```

If you have any trouble, please open an issue!
