# WRTD (WebAssembly Runtime Daemon)

A WebAssembly runtime daemon with three optimized binary variants for different deployment environments: servers, embedded systems, and bare metal.

## Features

- **Three Specialized Binaries**: Optimized builds for different runtime environments
- **Environment-Specific Optimization**: Each binary contains only the features it needs
- **Resource Management**: Automatic resource limits appropriate for each environment
- **Component Model Support**: WebAssembly Component Model implementation (level varies by mode)
- **Memory Strategies**: Multiple memory management strategies per environment
- **WASI Integration**: Full WASI support in std mode, limited/none in constrained modes
- **Cross-Platform**: Supports x86_64, ARM, and embedded targets
- **No Runtime Overhead**: Compile-time feature selection for maximum performance

## Installation

```bash
cargo install --path .
```

## Usage

### Binary Selection

Choose the appropriate binary for your deployment environment:

```bash
# Server/desktop environments (unlimited resources)
wrtd-std module.wasm --call function --fuel 1000000 --stats

# Embedded Linux systems (limited resources)  
wrtd-alloc module.wasm --call function --fuel 100000

# Bare metal/microcontrollers (minimal resources)
# wrtd-nostd is typically embedded in firmware
```

### Standard Library Mode (wrtd-std)

Full featured runtime for servers and desktop applications:

```bash
# Basic execution with full std support
wrtd-std module.wasm

# Execute specific function with unlimited resources
wrtd-std module.wasm --call function_name --fuel 1000000

# Use different memory strategies
wrtd-std module.wasm --memory-strategy zero-copy --stats

# Analyze component interfaces
wrtd-std module.wasm --analyze-component-interfaces

# Full WASI and file system support
wrtd-std server.wasm --call handle_request --interceptors logging,stats
```

### Allocation Mode (wrtd-alloc)

Heap allocation without std, suitable for embedded Linux:

```bash
# Embedded execution with memory limits
wrtd-alloc sensor.wasm --call process_data --fuel 100000

# Note: No command line arguments in alloc mode
# Configuration typically embedded in binary or read from fixed locations
```

### No Standard Library Mode (wrtd-nostd)

Minimal stack-only execution for bare metal systems:

```bash
# Typically used as embedded firmware, not command line
# Configuration and WASM data embedded at compile time
# Used in microcontrollers, safety-critical systems
```

## Binary Variants

WRTD provides three optimized binary variants for different deployment environments:

### wrtd-std (Standard Library Binary)
- **Target**: Server applications, desktop applications, development/testing
- **Features**: Full standard library support, unlimited resources, WASI integration
- **Memory**: Unlimited (system-dependent)
- **Fuel**: Unlimited (configurable)
- **Heap Allocation**: ✅ Available
- **WASI Support**: ✅ Full support
- **File System**: ✅ Available
- **Networking**: ✅ Available
- **Binary Size**: ~4-6MB

### wrtd-alloc (Allocation Binary)
- **Target**: Embedded Linux systems, IoT devices, resource-constrained environments
- **Features**: Heap allocation without std, automatic resource limits
- **Memory**: Limited to 16MB
- **Fuel**: Limited to 1,000,000
- **Heap Allocation**: ✅ Available
- **WASI Support**: ❌ Not available
- **File System**: ❌ Not available
- **Networking**: ❌ Not available
- **Binary Size**: ~2-3MB

### wrtd-nostd (No Standard Library Binary)
- **Target**: Bare metal systems, microcontrollers, safety-critical systems
- **Features**: Minimal runtime, stack-only operations, ultra-low resource usage
- **Memory**: Limited to 1MB
- **Fuel**: Limited to 100,000
- **Heap Allocation**: ❌ Not available
- **WASI Support**: ❌ Not available
- **File System**: ❌ Not available
- **Networking**: ❌ Not available
- **Binary Size**: ~500KB-1MB

### Binary Selection Guide

```bash
# Choose based on your deployment target:

# Server/Desktop (unlimited resources)
cargo build --bin wrtd-std --features std-runtime

# Embedded Linux (limited resources) 
cargo build --bin wrtd-alloc --features alloc-runtime

# Bare Metal/MCU (minimal resources)
cargo build --bin wrtd-nostd --features nostd-runtime
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
# Build all binary variants
cargo xtask wrtd-build-all

# Build specific binaries
cargo xtask wrtd-build --binary wrtd-std
cargo xtask wrtd-build --binary wrtd-alloc  
cargo xtask wrtd-build --binary wrtd-nostd

# Build in release mode with summary
cargo xtask wrtd-build-all --release --show-summary

# Build with cross-compilation for embedded targets
cargo xtask wrtd-build-all --cross-compile

# Test WRTD runtime modes
cargo xtask wrtd-test

# Alternative: Build directly with cargo
cargo build --bin wrtd-std --features std-runtime -p wrtd
cargo build --bin wrtd-alloc --features alloc-runtime -p wrtd
cargo build --bin wrtd-nostd --features nostd-runtime -p wrtd

# Build for embedded targets
cargo build --bin wrtd-alloc --features alloc-runtime --target armv7-unknown-linux-gnueabihf -p wrtd
cargo build --bin wrtd-nostd --features nostd-runtime --target thumbv7em-none-eabihf -p wrtd
```

## Configuration

WRTD uses environment variables for configuration:

- `RUST_LOG`: Controls log level (error, warn, info, debug, trace)
- `RUST_LOG_FORMAT`: Log output format (pretty, json, compact)

## License

This project is licensed under the MIT License - see the LICENSE file for details. 