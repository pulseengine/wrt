# wrt-parser

Unified WebAssembly binary parser for the WRT runtime system.

## Overview

`wrt-parser` provides a streaming, memory-efficient parser for WebAssembly binaries that processes sections one at a time without loading entire binaries into memory. It supports both Core WebAssembly and Component Model formats with ASIL-D compliant memory management.

## Features

- **Streaming parsing**: Process WebAssembly binaries section-by-section with minimal memory usage
- **Component Model support**: Built-in parsing for WebAssembly Component Model (not feature-gated)
- **ASIL-D compliance**: Memory-safe parsing with bounded collections and static allocation
- **No_std support**: Works in embedded environments without standard library
- **Type safety**: Comprehensive type system with validation during parsing
- **Error handling**: Detailed error reporting with category classification

## Usage

### Basic WebAssembly Module Parsing

```rust
use wrt_parser::prelude::*;

// Parse a WebAssembly binary
let wasm_bytes = std::fs::read("module.wasm")?;
let module = parse_wasm(&wasm_bytes)?;

// Access parsed sections
println!("Functions: {}", module.functions.len());
println!("Types: {}", module.types.len());
```

### Streaming Parser

```rust
use wrt_parser::prelude::*;

// Create a streaming parser
let mut parser = StreamingParser::new()?;

// Parse a binary
let module = parser.parse(&wasm_bytes)?;
```

### Component Model Parsing

```rust
use wrt_parser::prelude::*;

// Parse a WebAssembly Component
let component_bytes = std::fs::read("component.wasm")?;
let component = parse_component(&component_bytes)?;
```

### Header Validation

```rust
use wrt_parser::prelude::*;

// Validate WebAssembly header without full parsing
validate_header(&wasm_bytes)?;
```

## Memory Management

The parser uses bounded collections with a standardized memory provider (`NoStdProvider<8192>`) to ensure deterministic memory usage suitable for safety-critical environments.

## ASIL Compliance

- **ASIL-D**: Static memory allocation, no dynamic allocation after initialization
- **ASIL-C**: Memory budget enforcement with runtime bounds checking
- **ASIL-B**: Bounded collections with capacity limits
- **ASIL-A**: Basic memory safety features

## Feature Flags

- `std` (default off): Enable standard library support
- `alloc`: Enable allocation support for no_std environments
- `asil-d`: Enable ASIL-D safety features
- `asil-c`: Enable ASIL-C safety features
- `asil-b`: Enable ASIL-B safety features

## Architecture

The parser is designed around these core principles:

1. **Streaming-first**: Process data as it arrives without buffering
2. **Memory-bounded**: All collections have compile-time capacity limits
3. **Type-safe**: Strong typing throughout the parsing pipeline
4. **Validation-enabled**: Built-in validation during parsing
5. **Component-ready**: Native Component Model support

## Error Handling

The parser provides detailed error information with categories:

- `Parse`: Binary format errors
- `Validation`: WebAssembly specification violations  
- `Resource`: Memory or capacity limit exceeded
- `Runtime`: Internal parser errors

## Performance

- Minimal memory allocations
- Section-by-section processing
- Zero-copy where possible
- Optimized for embedded systems

## Safety

- `#![forbid(unsafe_code)]` - No unsafe code
- Bounded collections prevent buffer overflows
- Validation prevents malformed input processing
- ASIL compliance for safety-critical systems