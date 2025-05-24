# wrt-instructions

WebAssembly instruction implementations for the WebAssembly Runtime (WRT).

This crate provides the core instruction set implementation for WebAssembly, including arithmetic operations, control flow, memory operations, and more.

## Features

- Complete WebAssembly instruction set implementation
- Type-safe instruction execution
- Support for both `std` and `no_std` environments
- Efficient instruction dispatch
- Control Flow Integrity (CFI) support

## Instruction Categories

- **Arithmetic Operations**: Addition, subtraction, multiplication, division for all numeric types
- **Comparison Operations**: Equality, ordering, and relational comparisons
- **Control Operations**: Branching, loops, function calls, returns
- **Memory Operations**: Load, store, memory growth
- **Variable Operations**: Local and global variable access
- **Conversion Operations**: Type conversions between numeric types
- **Table Operations**: Table access and manipulation

## no_std Support

This crate fully supports `no_std` environments without requiring `alloc`, using bounded collections from `wrt-foundation` for all dynamic data structures.

## Usage

```rust
use wrt_instructions::prelude::*;

// Instructions are typically executed within the context of a WRT runtime
// See wrt-runtime for execution examples
```

## License

Licensed under the MIT license. See LICENSE file in the project root for details.