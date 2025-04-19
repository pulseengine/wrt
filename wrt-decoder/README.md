# wrt-decoder

High-level WebAssembly module decoder for wrt runtime.

## Overview

The `wrt-decoder` crate is responsible for handling all WebAssembly binary reading and writing operations in the wrt ecosystem. It provides a clean high-level API for decoding WebAssembly modules from binary format, validating them, and encoding them back to binary when needed.

This crate sits between the low-level format handling in `wrt-format` and the runtime execution in `wrt`.

## Key Features

- Decoding WebAssembly modules from binary format
- Encoding WebAssembly modules back to binary format
- Validating WebAssembly modules against the specification
- Providing memory-efficient zero-copy access to module data
- Supporting WebAssembly Component Model (planned)

## Architecture

The architecture follows a clean separation of concerns:

- `wrt-format`: Low-level binary format operations and structures
- `wrt-decoder`: High-level module processing and validation (this crate)
- `wrt`: Runtime execution of WebAssembly modules

## Usage

```rust
use wrt_decoder::{decode, validate, encode, Module};
use wrt_error::Result;

// Decode a WebAssembly binary
fn process_wasm(bytes: &[u8]) -> Result<()> {
    // Decode the module
    let module = decode(bytes)?;
    
    // Validate the module
    validate(bytes)?;
    
    // Access module components
    println!("Module has {} functions", module.functions.len());
    
    // Encode the module back to binary
    let binary = encode(&module)?;
    
    Ok(())
}
```

## Zero-Copy Operations

The decoder is designed to allow for zero-copy operations where possible, reducing memory usage when passing data between the decoder and the runtime:

```rust
// Get a view of the binary data without copying
let binary_view = module.get_binary_view();

// Get a view of data segments without copying
let data_view = module.get_data_view(0);
```

## WebAssembly Features Supported

- WebAssembly Core 1.0 specification
- Reference types
- SIMD operations
- Multi-value returns
- Component Model (planned) 