# PulseEngine (WRT Edition)

A pure Rust implementation of WebAssembly infrastructure designed for safety-critical systems. Provides foundational components for WebAssembly execution with emphasis on memory safety, deterministic behavior, and formal verification capabilities.

## Features

- **Memory Operations**: Complete WebAssembly memory management with bounds checking
- **Arithmetic Instructions**: Full implementation of WebAssembly numeric operations
- **Type System**: Complete WebAssembly value types and validation infrastructure
- **`no_std` Compatible**: Works in embedded and bare-metal environments
- **Safety-Critical Design**: ASIL compliance framework and formal verification support
- **Development Status**: Core execution engine and Component Model under development

## Quick Start

### Prerequisites

- Rust 1.86.0 or newer
- [just](https://github.com/casey/just) command runner

### Building

```bash
# Build everything
just build
# Or directly: cargo build --workspace

# Run tests
cargo xtask run-tests
# Or via just: just ci-test

# Run example
just test-wrtd-example
```

### Usage

Add PulseEngine to your `Cargo.toml`:

```toml
[dependencies]
wrt = { path = "wrt" }  # Use appropriate version/path
```

Basic usage:

```rust
use wrt::prelude::*;

// Note: Core execution engine under development
// Current example shows memory and arithmetic operations
let memory = WrtMemory::new(1024)?;
let value = Value::I32(42);
let result = ArithmeticOp::I32Add.execute(&[value, Value::I32(8)])?;
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
cargo xtask publish-docs-dagger --output-dir docs_output

# Preview documentation locally
cargo xtask preview-docs --open-browser

# API documentation only  
cargo doc --workspace --open

# Generate changelog (requires git-cliff)
cargo xtask generate-changelog

# Deploy to SFTP hosting (shared hosting, VPS, etc.)
cargo xtask deploy-docs-sftp --build-docs

# Validate documentation structure
cargo xtask validate-docs-comprehensive
```

## Development

See the [Developer Guide](docs/source/development/) for detailed development instructions.

Common commands:

```bash
just --list          # Show all available commands
just fmt            # Format code
just ci-main        # Run main CI checks
just ci-full        # Run complete CI suite

# Xtask commands for development
cargo xtask --help                # Show all xtask commands
cargo xtask verify-no-std         # Verify no_std compatibility
cargo xtask fmt-check             # Check code formatting
cargo xtask coverage              # Generate code coverage
```

## License

MIT License - see [LICENSE](LICENSE) file for details.
