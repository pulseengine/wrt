# WRTD (WebAssembly Runtime Daemon)

A WebAssembly runtime daemon that executes WebAssembly components with WASI logging support.

## Features

- Execute WebAssembly components with Component Model support
- WASI logging integration via tracing framework
- Structured logging and diagnostics
- Runtime monitoring and debugging support

## Installation

```bash
cargo install --path .
```

## Usage

```bash
wrtd <wasm-file>
```

### Example

```bash
wrtd example/hello.wasm
```

## Logging

WRTD uses the tracing framework for structured logging and diagnostics. All WASI logging calls from WebAssembly components are captured and emitted through tracing.

Log levels are mapped from WebAssembly to tracing levels as follows:

| WebAssembly Level | Tracing Level |
|------------------|---------------|
| 0                | ERROR         |
| 1                | WARN          |
| 2                | INFO          |
| 3                | DEBUG         |
| 4                | TRACE         |

## Building

### Prerequisites

- Rust 1.75 or later
- Cargo

### Build Commands

```bash
cargo build
```

## Configuration

WRTD uses environment variables for configuration:

- `RUST_LOG`: Controls log level (error, warn, info, debug, trace)
- `RUST_LOG_FORMAT`: Log output format (pretty, json, compact)

## License

This project is licensed under the MIT License - see the LICENSE file for details. 