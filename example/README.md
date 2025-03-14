# WRT Example with Component Model

This is a WebAssembly Component that implements a simple logging interface. It exports a `hello` function that logs a message and returns an integer.

## Component Model

This example demonstrates basic WebAssembly Component Model functionality:

1. Defining an interface with WIT (WebAssembly Interface Types)
2. Implementing a component using wit-bindgen
3. Building for the WASI Preview 2 target
4. Generating valid component model WebAssembly

## Prerequisites

You'll need:

1. Rust with the `wasm32-wasip2` target
2. wasm-tools for verifying components
3. `just` command runner (optional)

## Quick Start

The easiest way to get started is to use the `just` command runner:

```bash
# Install just (if you haven't already)
cargo install just

# Install required tools
rustup target add wasm32-wasip2
cargo install wasm-tools

# Build the example
just build-example
```

## Manual Build

If you prefer to build manually:

```bash
# Install required tools
rustup target add wasm32-wasip2

# Build the module
cargo build -p example --target wasm32-wasip2

# Copy the component
cp target/wasm32-wasip2/debug/example.wasm example/hello-world.wasm
```

## Verification

You can verify the component's structure using wasm-tools:

```bash
# Install wasm-tools
cargo install wasm-tools

# Validate the component
wasm-tools validate --features=component-model example/hello-world.wasm

# View the interface definitions
wasm-tools component wit example/hello-world.wasm
```

## Implementation Details

This example defines a WIT interface:

```wit
interface example {
    // Log levels
    enum level {
        trace, debug, info, warn, error, critical,
    }

    // Mock logging function
    log: func(level: level, message: string);

    // Main function
    hello: func() -> s32;
}
```

And implements it with wit-bindgen:

```rust
impl exports::example::hello::example::Guest for HelloComponent {
    // Log a message
    fn log(level: exports::example::hello::example::Level, message: String) {
        println!("[{:?}] {}", level, message);
    }

    // Main hello function
    fn hello() -> i32 {
        Self::log(
            exports::example::hello::example::Level::Info,
            "Hello from WebAssembly!".to_string()
        );
        
        // Return value
        42
    }
}
```

## Using with WRT

This example can be loaded into the WRT runtime:

```rust
use wrt::{Module, Engine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new WRT engine with component model support
    let mut engine = wrt::new_engine();
    
    // Load the WebAssembly component
    let wasm_bytes = std::fs::read("example/hello-world.wasm")?;
    let module = Module::from_bytes(&wasm_bytes)?;
    
    // Instantiate the component
    let instance = engine.instantiate(&module)?;
    
    // Call the "hello" function from the example interface
    let results = engine.invoke_export(&instance, "example:hello/example", "hello", &[])?;
    
    // Print the results - should be 42
    println!("Result: {:?}", results);
    
    Ok(())
}
```

## Development

The project includes a `justfile` with various development commands:

- `just build` - Build all crates
- `just test` - Run all tests
- `just fmt` - Format all Rust code
- `just check` - Run code style checks
- `just clean` - Clean all build artifacts

See `just --list` for all available commands.
