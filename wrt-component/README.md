# wrt-component

Component Model implementation for the WebAssembly Runtime (WRT).

This crate provides an implementation of the WebAssembly Component Model, enabling composition and interoperability between WebAssembly modules with shared-nothing linking.

## Features

- `std` (default): Enables standard library features
- `alloc`: Enables allocation features for no_std environments with an allocator
- `no_std`: Builds without the standard library for embedded environments
- `kani`: Enables formal verification using Kani
- `wat-parsing`: Enables parsing of WebAssembly Text Format

## Usage

```rust
use wrt_component::{Component, ComponentType, Import, Export};
use wrt_host::CallbackRegistry;
use wrt_logging::LoggingExt;

// Create a component
let component_type = ComponentType::new();
let mut component = Component::new(component_type);

// Instantiate the component
component.instantiate(imports)?;

// Call a component function
let result = component.execute_function("function_name", args)?;
```

## Component Model

This implementation follows the [WebAssembly Component Model specification](https://github.com/WebAssembly/component-model), providing:

- Shared-nothing linking between components as described in the [Linking specification](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Linking.md)
- Component imports and exports
- Component instantiation and execution
- Host function integration through the [Canonical ABI](https://github.com/WebAssembly/component-model/blob/main/design/mvp/CanonicalABI.md)

## no_std Support

This crate supports no_std environments with the `no_std` feature. When used without `std`,
you must enable the `alloc` feature and provide an allocator.

## Verification

This crate supports formal verification using [Kani](https://github.com/model-checking/kani).
Enable the `kani` feature to include verification harnesses. 