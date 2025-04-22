# wrt-logging

Logging infrastructure for the WebAssembly Runtime (WRT).

This crate provides logging functionality for WebAssembly components, allowing components to log messages to the host environment. It extends the wrt-host crate with logging-specific capabilities.

## Features

- `std` (default): Enables standard library features
- `alloc`: Enables allocation features for no_std environments with an allocator
- `no_std`: Builds without the standard library for embedded environments
- `kani`: Enables formal verification using Kani

## Usage

```rust
use wrt_host::CallbackRegistry;
use wrt_logging::{LogLevel, LogOperation, LoggingExt};

// Create a callback registry
let mut registry = CallbackRegistry::new();

// Register a log handler
registry.register_log_handler(|log_op| {
    println!("[{}] {}: {}", 
        log_op.component_id.unwrap_or_default(),
        log_op.level.as_str(),
        log_op.message
    );
});

// Log a message
registry.handle_log(LogOperation::new(
    LogLevel::Info,
    "Hello from component".to_string(),
));
```

## Component Model Integration

This crate provides a standardized way for components to log information to the host environment, 
following the patterns established in the [WebAssembly Component Model](https://github.com/WebAssembly/component-model).

## no_std Support

This crate supports no_std environments with the `no_std` feature. When used without `std`,
you must enable the `alloc` feature and provide an allocator.

## Verification

This crate supports formal verification using [Kani](https://github.com/model-checking/kani).
Enable the `kani` feature to include verification harnesses. 