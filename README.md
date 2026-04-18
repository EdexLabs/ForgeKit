
# ForgeKit

ForgeKit is a Rust library for working with ForgeScript.

It currently provides:

- A high-performance ForgeScript parser that produces an AST
- Utilities for traversing and analyzing the AST
- A metadata manager for loading and querying function/event metadata

## Crate

- Package: `forge-kit`
- Library crate: `forge_kit`

## Usage (Rust)

```rust
use forge_kit::parser::Parser;

let input = "Hello $foo[bar]";
let (ast, errors) = Parser::new(input).parse();
assert!(errors.is_empty());
```

## Validation (optional)

Enable the `validation` feature to validate parsed function calls against metadata.

```bash
cargo build --features validation
```

## WebAssembly (WASM)

This crate can be built to WebAssembly using `wasm-bindgen` / `wasm-pack`.

```bash
wasm-pack build --release --target web --features wasm
```

The WASM bindings are available behind the `wasm` cargo feature.

## License

GPL-3.0 (see `LICENSE`).
