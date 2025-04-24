# wrt-error

Error handling for the WRT WebAssembly runtime.

This crate provides a lightweight, no_std compatible error handling system that supports error chaining, context, and specific error types for WebAssembly operations.

## Features

- **no_std compatible**: Works in embedded environments with or without the `alloc` feature.
- **Flexible error handling**: Similar to `anyhow` but designed for WebAssembly runtimes.
- **Error chaining**: Add context to errors with the `context()` method.
- **Predefined error types**: Common WebAssembly error types like memory access errors, stack underflow, etc.
- **Customizable**: Implement the `ErrorSource` trait for your own error types.
- **Formally verified**: Critical error handling components are verified using the Kani verifier.

## Usage

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
wrt-error = { version = "0.2.0", features = ["std"] }
```

### Basic example

```rust
use wrt_error::{Error, Result, ResultExt};

// Create a basic error
fn may_fail() -> Result<()> {
    Err(Error::division_by_zero())
}

// Add context to errors
fn with_context() -> Result<()> {
    may_fail().context("Failed during calculation")
}

// Convert from other error types
fn from_io_error() -> Result<()> {
    std::fs::File::open("non_existent_file.txt")
        .map(|_| ())
        .map_err(Error::from)
        .context("Failed to open configuration file")
}
```

## Features

- `std`: Enables integration with the standard library (recommended for most use cases)
- `alloc`: Enables features that require heap allocation (enabled by default)
- `minimal`: Minimal feature set for basic functionality with `alloc`
- `no_std`: For embedded environments without standard library or allocator
- `kani`: Enables formal verification with the Kani verifier
- Integration features:
  - `wasmparser`: Allows working with `wasmparser::BinaryReaderError`
  - `serde_json`: Adds support for `serde_json::Error`
  - `bincode`: Adds support for `bincode::Error`
  - `wat`: Adds support for `wat::Error`
  - `wasi`: Adds support for WASI errors

## Formal Verification

The `wrt-error` crate includes formal verification using the [Kani Verifier](https://github.com/model-checking/kani), which applies model checking to Rust code. This helps guarantee the correctness of critical error handling components.

To run the verification:

```bash
# Install Kani
cargo install --locked kani-verifier

# Run verification on all proofs
cd wrt-error
cargo kani --features kani

# Run verification on a specific proof
cargo kani --features kani --harness verify_error_context

# Run with increased unwinding limits for complex proofs
cargo kani --features kani --unwind 3
```

Verified properties include:
- Error creation and display formatting
- Context chaining and preservation
- Factory method correctness
- Error type conversion
- Result type behavior

## License

MIT 