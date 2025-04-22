# WebAssembly Component Examples

This directory contains examples for working with WebAssembly components using the wrt-component library.

## Examples

### 1. Component Graph View

An interactive visualization tool for WebAssembly components that displays various aspects of component structure using a graph-based UI.

```
cargo run --example component_graph_view -- <component-file> [--debug]
```

#### Arguments
- `<component-file>`: Path to the WebAssembly component to visualize
- `--debug`: (Optional) Enable debug mode with more detailed information

#### Interactive Navigation
- Navigate between views using number keys:
  - `1`: Overview - Shows main component sections
  - `2`: Modules - Shows core modules and their structure
  - `3`: Imports - Shows component imports
  - `4`: Exports - Shows component exports 
  - `5`: Producers - Shows producer information
  - `6`: Details - Shows detailed component structure
  - `7`: Debug - Shows raw data for debugging
- Arrow keys: Navigate within a view
- `f`: Focus on a selected node
- `Esc`: Return to normal view
- `q`: Quit the application

### 2. Component Info Simple

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
# Run the interactive graph visualization
cargo run --example component_graph_view -- ./target/wasm32-wasip2/debug/example.wasm

# Print component information to the console
cargo run --example component_info_simple -- ./target/wasm32-wasip2/debug/example.wasm 