# wrt-format

WebAssembly format handling for the WRT runtime.

This crate provides utilities for handling WebAssembly formats, including serialization and deserialization of modules and state, binary format parsing, and compression.

## Features

- **Module handling**: Parse and manipulate WebAssembly modules
- **Section management**: Work with individual WebAssembly sections
- **Binary format**: Low-level binary format utilities
- **State serialization**: Save and load WebAssembly runtime state
- **Compression**: Efficient compression of WebAssembly modules and state
- **Version management**: Handle different versions of the WebAssembly specification
- **Formally verified**: Critical components are verified using the Kani verifier

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
wrt-format = { version = "0.1.0" }
```

### Basic example

```rust
use wrt_format::{module::Module, binary::Binary};

// Parse a WebAssembly module from binary
fn parse_module(wasm_bytes: &[u8]) -> Result<Module, wrt_error::Error> {
    let binary = Binary::from_bytes(wasm_bytes)?;
    Module::from_binary(binary)
}

// Use the module
fn use_module() -> Result<(), wrt_error::Error> {
    let wasm_bytes = include_bytes!("path/to/module.wasm");
    let module = parse_module(wasm_bytes)?;
    
    // Work with the module...
    
    Ok(())
}
```

## Features

- `std`: Enables integration with the standard library (default)
- `kani`: Enables formal verification with the Kani verifier

## Formal Verification

The `wrt-format` crate includes formal verification using the [Kani Verifier](https://github.com/model-checking/kani) for critical components.

To run the verification:

```bash
# Install Kani
cargo install --locked kani-verifier

# Run verification on all proofs
cd wrt-format
cargo kani --features kani

# Run verification on a specific proof
cargo kani --features kani --harness verify_binary_format
```

Verified properties include:
- Binary format parsing correctness
- Module serialization and deserialization
- State consistency
- Section handling

## License

MIT 