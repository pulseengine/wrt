# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# WRT (WebAssembly Runtime) Project Guidelines

## Important Rules
- NEVER create hardcoded examples in the runtime code. Real implementations should parse or process actual external files.
- NEVER add dummy or simulated implementations except in dedicated test modules.
- Any example code MUST be in the `example/` directory or test files, not in the runtime implementation.
- The runtime should be able to execute real WebAssembly modules without special-casing specific files.
- Only use placeholders when absolutely necessary and clearly document them.

## Build Commands
- Build all: `just build` or `cargo build`
- Build specific crate: `cargo build -p wrt|wrtd|example`
- Clean: `just clean` or `cargo clean`
- Run tests: `just ci-test` or `cargo test`
- Run single test: `cargo test -p wrt -- test_name --nocapture`
- Format code: `just fmt` or `cargo fmt`
- Format check: `just fmt-check`
- Main CI checks: `just ci-main`
- Full CI suite: `just ci-full`
- Typecheck: `cargo check`

## Code Style Guidelines
- Use 4-space indentation
- Follow Rust naming conventions: snake_case for functions/variables, CamelCase for types
- Organize imports in the following order:
  1. Standard library imports (std, core, alloc)
  2. External crates/third-party dependencies
  3. Internal modules (crate:: imports)
  4. Each group should be separated by a blank line
- Always derive Debug, Clone, PartialEq for data structures
- Use thiserror for error definitions
- Handle no_std environments with appropriate macro imports
- Write tests for new functionality
- Fix type consistency:
  - WebAssembly spec uses u32 for sizes
  - Convert between u32 and usize explicitly when working with Rust memory
- Break cyclic references with Box<T> for recursive types
- Use conventional commits: `type(scope): short summary`
- Thoroughly document public API with doc comments