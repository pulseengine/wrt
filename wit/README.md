# WebAssembly Interface Types (WIT) Directory

This directory contains the WebAssembly Interface Type (WIT) definitions used by the project. 

## Directory Structure

```
wit/
├── deps/               # WIT dependencies for direct component use
│   └── wasi/           
│       ├── logging/    # WASI logging interface
│       ├── cli/        # WASI CLI interface
│       └── io/         # WASI I/O interface
├── wasi/               # WASI interfaces (direct import paths)
│   └── logging/        # WASI logging interface
└── example.wit         # Example component interface
```

## Interface Descriptions

- `wasi/logging/logging.wit`: Defines the WASI logging interface (v0.2.0-preview2) used by components
- `example.wit`: Defines the hello-world component that uses the WASI logging interface

## Usage

Components reference these WIT definitions in their `Cargo.toml` files:

```toml
[package.metadata.component.target]
path = "../wit/deps"  # Base path for dependencies
world = "hello-world" # World defined in wit/example.wit

[package.metadata.component.target.dependencies]
"wasi:logging" = { version = "0.2.0-preview2" }
```

## Maintaining WIT Files

When adding or updating components, ensure that:

1. All required WIT interfaces are in place
2. Interface versions are consistent across components
3. Component build scripts properly reference these WIT files 