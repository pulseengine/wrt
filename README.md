# WRT - WebAssembly Runtime

A pure Rust implementation of a WebAssembly runtime supporting both the core WebAssembly specification and the WebAssembly Component Model.

## Features

- **Core WebAssembly Support**: Full WebAssembly 1.0 specification implementation
- **Component Model**: WebAssembly Component Model for language-agnostic interoperability  
- **`no_std` Compatible**: Works in embedded and bare-metal environments
- **Memory Safety**: Safe memory management with ASIL-B compliance features
- **Stackless Execution**: Configurable execution engine for constrained environments
- **Control Flow Integrity**: Hardware and software CFI protection

## Quick Start

### Prerequisites

- Rust 1.86.0 or newer
- [just](https://github.com/casey/just) command runner

### Building

```bash
# Build everything
just build

# Run tests
just ci-test

# Run example
just test-wrtd-example
```

### Usage

Add WRT to your `Cargo.toml`:

```toml
[dependencies]
wrt = { path = "wrt" }  # Use appropriate version/path
```

Basic usage:

```rust
use wrt::prelude::*;

// Load and run a WebAssembly module
let module = Module::from_bytes(&wasm_bytes)?;
let mut instance = ModuleInstance::new(module, imports)?;
let result = instance.invoke("function_name", &args)?;
```

## Project Structure

This is a multi-crate workspace:

- **`wrt/`** - Main library facade
- **`wrt-foundation/`** - Core types and bounded collections  
- **`wrt-runtime/`** - Execution engine
- **`wrt-component/`** - Component Model implementation
- **`wrt-decoder/`** - Binary format parsing
- **`wrtd/`** - Standalone runtime daemon
- **`example/`** - Example WebAssembly component

## Documentation

- **[API Documentation](docs/source/)** - Complete API reference and specifications
- **[Architecture Guide](docs/source/architecture/)** - System design and components
- **[Developer Guide](docs/source/development/)** - Contributing and development setup

Generate documentation:

```bash
# Build comprehensive documentation
just docs

# API documentation only  
cargo doc --workspace --open
```

## Development

See the [Developer Guide](docs/source/development/) for detailed development instructions.

Common commands:

```bash
just --list          # Show all available commands
just fmt            # Format code
just ci-main        # Run main CI checks
just ci-full        # Run complete CI suite
```

## License

MIT License - see [LICENSE](LICENSE) file for details.
