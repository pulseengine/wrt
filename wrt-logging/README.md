# WRT Logging

Logging infrastructure for the WebAssembly Runtime (WRT) with support for both standard and `no_std` environments.

This crate provides logging functionality for WebAssembly components, allowing them to log messages to the host environment. It's designed to work seamlessly with the WRT ecosystem and extends the `wrt-host` crate with logging-specific capabilities.

## Features

- **Component Logging** - Enable WebAssembly components to log messages to the host
- **Log Levels** - Support for different log levels (Debug, Info, Warning, Error)
- **Custom Handlers** - Extensible architecture for custom log handlers
- **Std/No-std Support** - Works in both standard and `no_std` environments
- **Integration** - Seamless integration with the WRT component model

## Usage

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
wrt-logging = "0.2.0"
```

### Example

```rust
use wrt_logging::{LogHandler, LogLevel, LogOperation};
use wrt_host::CallbackRegistry;

// Create a custom log handler
struct MyLogHandler;

impl LogHandler for MyLogHandler {
    fn handle_log(&self, level: LogLevel, message: &str) -> wrt_logging::Result<()> {
        match level {
            LogLevel::Debug => println!("DEBUG: {}", message),
            LogLevel::Info => println!("INFO: {}", message),
            LogLevel::Warning => println!("WARN: {}", message),
            LogLevel::Error => println!("ERROR: {}", message),
        }
        Ok(())
    }
}

// Register the log handler with a component
fn register_logging(registry: &mut CallbackRegistry) {
    let handler = Box::new(MyLogHandler);
    registry.register_log_handler(handler);
}
```

## Feature Flags

- `std` (default): Use the standard library
- `alloc`: Enable allocation support without std
- `no_std`: Enable complete no_std support
- `kani`: Enable formal verification support using Kani

## No-std Usage

To use this crate in a `no_std` environment:

```toml
[dependencies]
wrt-logging = { version = "0.2.0", default-features = false, features = ["no_std", "alloc"] }
```

## License

This project is licensed under the MIT License. 