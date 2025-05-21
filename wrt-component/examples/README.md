# WebAssembly Component Examples

This directory contains examples for working with WebAssembly components using the wrt-component library.

## Examples

### 1. Component Info Simple

A simple command-line tool that prints detailed information about a WebAssembly component.

```
cargo run --example component_info_simple -- <component-file>
```

#### Arguments
- `<component-file>`: Path to the WebAssembly component to analyze

#### Output Information
- Component summary (name, counts)
- Core modules details
- Core instances details
- Aliases
- Component-level imports
- Component-level exports
- Module-level imports
- Module-level exports
- Producers information

## Example Usage

```
# Print component information to the console
cargo run --example component_info_simple -- ./target/wasm32-wasip2/debug/example.wasm 