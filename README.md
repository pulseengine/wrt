# WRT - WebAssembly Runtime

A Rust implementation of a WebAssembly runtime focusing on the Component Model and WASI Preview 2. Designed with safety-critical systems in mind.

## Current Status

**Early Development** - Basic WebAssembly component execution is working:

```bash
# Run a Rust-compiled WASI Preview 2 component
./target/debug/wrtd hello_rust.wasm --component
# Output: Hello wasm component world from Rust!
```

### What Works

- WebAssembly Component Model parsing and instantiation
- WASI Preview 2 stdout/stderr output (`wasi:cli/stdout`, `wasi:io/streams`)
- Core WebAssembly module execution
- Basic memory management with bounds checking
- `no_std` compatible foundation (for embedded use cases)

### In Progress

- Additional WASI Preview 2 interfaces (filesystem, environment, etc.)
- Cross-component function calls
- Full Component Model linking

## Quick Start

```bash
# Clone and build
git clone https://github.com/pulseengine/wrt
cd wrt
cargo build --bin wrtd --features "std,wrt-execution"

# Run a WebAssembly component
./target/debug/wrtd your_component.wasm --component
```

## Project Structure

- **`wrtd/`** - Runtime daemon (main executable)
- **`wrt-runtime/`** - Execution engine
- **`wrt-component/`** - Component Model support
- **`wrt-decoder/`** - Binary format parsing
- **`wrt-foundation/`** - Core types and bounded collections
- **`cargo-wrt/`** - Build tooling

## Building

```bash
# Install build tool (optional but recommended)
cargo install --path cargo-wrt

# Build runtime
cargo build --bin wrtd --features "std,wrt-execution"

# Run tests
cargo test --workspace
```

## Usage

```bash
# Basic component execution
wrtd component.wasm --component

# With WASI support
wrtd component.wasm --component --wasi

# Set resource limits
wrtd component.wasm --component --fuel 100000 --memory 1048576
```

## Design Goals

- **WASI Preview 2 focus** - Targeting the modern component-based WASI
- **Safety-critical awareness** - Bounded allocations, deterministic behavior
- **`no_std` support** - Usable in embedded/constrained environments

## License

MIT License - see [LICENSE](LICENSE) file.
