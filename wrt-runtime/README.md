# wrt-runtime

Core runtime implementation for the WebAssembly Runtime (WRT).

This crate provides the execution engine and runtime environment for WebAssembly modules, supporting both standard and stackless execution models.

## Features

- WebAssembly module instantiation and execution
- Memory management with safety features
- Table management
- Global variable support
- Function imports and exports
- Stackless execution engine for constrained environments
- Control Flow Integrity (CFI) protection
- Support for both `std` and `no_std` environments

## Architecture

### Core Components

- **Module**: Represents a parsed WebAssembly module
- **ModuleInstance**: Runtime instance of a module with its own memory, tables, and globals
- **Memory**: Linear memory with bounds checking and safety features
- **Table**: Function and element tables
- **Global**: Global variables
- **Execution**: Instruction execution engine

### Stackless Execution

The stackless execution engine allows running WebAssembly in environments with limited stack space:

```rust
use wrt_runtime::stackless::StacklessEngine;

// Create a stackless engine with limited stack frames
let engine = StacklessEngine::new(max_frames);
```

### Control Flow Integrity

Built-in CFI protection guards against control flow hijacking:

```rust
use wrt_runtime::cfi_engine::CfiEngine;

// Create an engine with CFI protection
let engine = CfiEngine::new(policy);
```

## no_std Support

This crate supports `no_std` environments with the `alloc` feature. Without `alloc`, bounded alternatives from `wrt-foundation` are used.

## Usage

```rust
use wrt_runtime::prelude::*;

// Load and instantiate a module
let module = Module::new(wasm_bytes)?;
let instance = ModuleInstance::new(module, imports)?;

// Execute an exported function
let result = instance.invoke("function_name", &args)?;
```

## License

Licensed under the MIT license. See LICENSE file in the project root for details.