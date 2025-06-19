# cargo-wrt

Unified build tool for WRT (WebAssembly Runtime) - the next-generation safety-critical WebAssembly runtime.

## Installation

### From crates.io (once published)
```bash
cargo install cargo-wrt
```

### From source
```bash
git clone https://github.com/pulseengine/wrt
cd wrt
cargo install --path cargo-wrt
```

### For development
```bash
cargo build -p cargo-wrt
# Binary will be available at ./target/debug/cargo-wrt
```

## Usage

```bash
cargo-wrt --help
```

### Available Commands

- `build` - Build all WRT components
- `test` - Run tests across the workspace  
- `verify` - Run safety verification and compliance checks
- `docs` - Generate documentation
- `coverage` - Run coverage analysis
- `check` - Run static analysis (clippy + formatting)
- `no-std` - Verify no_std compatibility
- `wrtd` - Build WRTD (WebAssembly Runtime Daemon) binaries
- `ci` - Run comprehensive CI checks
- `clean` - Clean build artifacts

### Examples

```bash
# Build all components
cargo-wrt build

# Run all tests
cargo-wrt test

# Run safety verification for ASIL-D level
cargo-wrt verify --asil d

# Generate and open documentation
cargo-wrt docs --open

# Run full CI pipeline
cargo-wrt ci
```

## Features

- **Safety-Critical Support**: ASIL-D compliance verification
- **Cross-Platform**: Supports std/no_std environments
- **Comprehensive Testing**: Unit, integration, and formal verification
- **AI-Friendly Architecture**: Clean, linear build process
- **Modern Rust**: Uses latest Rust features and best practices

## License

MIT - see LICENSE file for details.