# WRT - WebAssembly Runtime

[![Rust CI](https://github.com/avrabe/wrt/actions/workflows/ci.yml/badge.svg)](https://github.com/avrabe/wrt/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/avrabe/wrt/graph/badge.svg?token=angh0LQdpK)](https://codecov.io/gh/avrabe/wrt)

WRT is a pure Rust implementation of a WebAssembly runtime that supports both the core WebAssembly specification and the WebAssembly Component Model. The project provides both a library (`wrt`) for embedding the runtime in Rust applications and a standalone daemon (`wrtd`) for executing WebAssembly modules.

## Features

- **Core WebAssembly Support**: Implements the WebAssembly 1.0 specification
- **Component Model**: Supports the WebAssembly Component Model for language-agnostic interoperability
- **`no_std` Compatible**: Can be used in environments without the standard library
- **Memory Safety**: Provides safe memory management for WebAssembly modules
- **Extensible**: Easily extendable architecture for adding new features

## Project Structure

- `wrt/` - Main WRT library (often a facade or re-exporting common types)
- `wrt-<feature>/` - Various crates implementing specific features and components of the WRT ecosystem (e.g., `wrt-runtime/`, `wrt-component/`, `wrt-types/`, etc.)
- `wrtd/` - WebAssembly Runtime Daemon for executing modules
- `example/` - Example Component Model implementation and usage
- `docs/` - Documentation including requirements and specifications
- `xtask/` - Contains build and development scripts
- `tests/` - Integration and end-to-end tests
- `justfile` - Command runner for development tasks

## Installation

### Prerequisites

- Rust 1.86.0 or newer
- For development: [just](https://github.com/casey/just) command runner
- Python 3 (for documentation and hooks, if still used - verify this separately)
- Java (optional, for PlantUML diagrams in documentation, if still used - verify this separately)

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install just command runner
cargo install just

# Setup project Rust targets
# Note: Other dependencies like Wasm tools, Python deps for docs, PlantUML/Java
# may need to be installed manually if required for specific tasks.
just setup-rust-targets

# Alternatively, install components individually if other setup commands are added back:
# just setup-wasm-tools
# just setup-python-deps
# just setup-plantuml # (Requires manual steps on Windows)
```

## Getting Started

### Building the Project

The easiest way to build the project is using the provided justfile:

```bash
# Build all crates and WAT files (includes wrt, wrtd, example, adapter)
just build

# Build specific components
just build-wrt      # Build only the core WRT library
just build-wrtd     # Build only the WRT daemon
just build-example  # Build the example component (debug)
just build-example-release # Build the example component (release)
just build-adapter  # Build the logging adapter component (debug)
just build-adapter-release # Build the logging adapter component (release)
```

### Running the Example

The project includes an example WebAssembly component that demonstrates basic functionality:

```bash
# Build the example (release) and run it using wrtd
# This command builds the example and then executes its 'hello' function via wrtd.
# Additional arguments can be passed to wrtd, e.g.:
# just test-wrtd-example "--fuel 10000 --verbose"
just test-wrtd-example
```

### Using WRT in Your Project

Add WRT to your Cargo.toml:

```toml
[dependencies]
wrt = { git = "https://github.com/avrabe/wrt" }
```

Basic usage example:

```rust
use wrt::{Module, Engine, Result, Value};

fn run_wasm_module(wasm_path: &str) -> Result<()> {
    // Load WebAssembly bytes from a file
    let wasm_bytes = std::fs::read(wasm_path)?;
    
    // Parse the WebAssembly module
    let module = Module::from_bytes(&wasm_bytes)?;

    // Create a new engine with the parsed module
    let mut engine = Engine::new(module);
    
    // Instantiate the module within the engine
    let instance_idx = engine.instantiate()?; // Module is already in the engine
    
    // Execute a function (assuming exported function "main" at index 0 exists)
    // The third argument is a Vec<Value> for function arguments.
    let results = engine.execute(instance_idx, 0, vec![])?; 
    
    println!("Execution completed with results: {:?}", results);
    
    Ok(())
}
```

For more advanced examples, see the [example directory](./example).

## Development

The project uses the `just` command runner to simplify development tasks.

### Available Commands

```bash
# List all available commands (defined in justfile)
just --list

# Build everything (default command)
just build

# Run all tests (via xtask)
just ci-test
# Note: The README previously listed 'just test'. 'just ci-test' is the specific command.

# Generate code coverage report (as part of advanced tests via xtask)
# This is typically part of 'just ci-advanced-tests'
# Report at target/coverage/tarpaulin-report.html
just ci-advanced-tests
# Note: The README previously listed 'just coverage'. Coverage is run via ci-advanced-tests.

# Code quality checks
just fmt          # Format Rust code using 'cargo fmt'
just fmt-check    # Check Rust code formatting (via xtask)
# 'just check' (generic) is not a direct justfile recipe, use specific checks like fmt-check.
just check-imports  # Check import organization (via xtask, e.g., 'just xtask check-imports')
just check-udeps    # Check for unused dependencies (via cargo-machete, likely via xtask)
# 'just check-all' is not a direct justfile recipe, likely an xtask aggregate.

# Run WRTD with its help output
just test-wrtd-help

# Documentation (likely via xtask)
# The justfile prepares Sphinx vars but actual build recipes might be in xtask.
# e.g., 'just xtask docs' or similar
just docs           # Build HTML documentation with diagrams (Default, assumed via xtask)
just docs-html      # Build basic HTML documentation (assumed via xtask)
just docs-pdf       # Build PDF documentation (requires LaTeX, assumed via xtask)

# Clean build artifacts
just clean
```

### Code Organization Standards

The project follows these guidelines:

1. Imports are organized in this order:
   - Standard library imports (std, core, alloc)
   - External crates/third-party dependencies
   - Internal modules (crate:: imports)

2. All public API should be documented following the format in CLAUDE.md.

3. Each module is organized by functionality with clear separation of concerns.

## Documentation

WRT uses two documentation systems:

1. **Rust API Documentation**: Generated with `cargo doc`

   ```bash
   # Build docs for all workspace members (excluding xtask)
   cargo doc --workspace --exclude xtask --all-features --open
   ```

2. **Requirements and Specifications**: Using Sphinx with sphinx-needs

   ```bash
   # Build Sphinx docs (HTML with diagrams by default)
   just docs
   # Documentation will be available in docs/_build/html
   ```

## CI/CD

This project includes GitHub Actions workflows that automatically run on pull requests and pushes to main:

- Build and test checks
- Code style enforcement
- Code coverage reporting (with Codecov integration)
- Documentation generation
- Security audit
- Unused dependency detection

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
