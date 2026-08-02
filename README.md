# Polyglot Concurrency Using the WebAssembly Component Model

This is a demonstration of how to compose WebAssembly components written in a
variety of programming languages using concurrent inter-component function calls
and streaming.

This example is a CLI application which accepts one or more strings as arguments
and transforms them, printing the results to standard output.  It's built as a
composition of the following components:

- `top`: exports `wasi:cli/run` and imports `transformer`, an interface
  which contains an asynchronous function that accepts a stream of strings and
  returns a stream of transformed strings.  This component calls that function
  and prints the results as they arrive.

- `middle-rust`: exports _and_ imports `transformer`.  When called, forwards the
  stream to the imported function, but also transforms each string in the input
  and output streams before forwarding them to the callee and the caller,
  respectively.
  
- `middle-python`: As above, but implemented in Python.
  
- `middle-javascript`: As above, but implemented in JavaScript.
    
- `middle-go`: As above, but implemented in Go.
  
- `bottom`: exports `transformer` and returns the input stream unmodified as the
  output stream.

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

> **Note:** As of this writing, the `componentize-js` build fails on MacOS,
> apparently due to `https://bugzilla.mozilla.org/show_bug.cgi?id=1844694` :(

```
cargo install --version 47.0.2 wasmtime-cli
cargo install --version 0.10.1 wac-cli
cargo install --git https://github.com/dicej/componentize-js --rev bdd7c3d5
```

Once you have all the prereqs, you should be able to build and run using e.g.:

```
make build
wasmtime run composed.wasm foo bar baz
```

If all goes well, the output should look like this:

```
ʕ◔ϖ◔ʔ🐒🐍🦀fooʕ◔ϖ◔ʔ🐒🐍🦀
ʕ◔ϖ◔ʔ🐒🐍🦀barʕ◔ϖ◔ʔ🐒🐍🦀
ʕ◔ϖ◔ʔ🐒🐍🦀bazʕ◔ϖ◔ʔ🐒🐍🦀
```

If you have any trouble, please open an issue!
