# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# WRT (WebAssembly Runtime) Project Guidelines

## Quick Setup

```bash
# Install the unified build tool
cargo install --path cargo-wrt

# Verify installation
cargo-wrt --help
```

## Important Rules
- NEVER create hardcoded examples in the runtime code. Real implementations should parse or process actual external files.
- NEVER add dummy or simulated implementations except in dedicated test modules.
- Any example code MUST be in the `example/` directory or test files, not in the runtime implementation.
- The runtime should be able to execute real WebAssembly modules without special-casing specific files.
- Only use placeholders when absolutely necessary and clearly document them.

## Build Commands
- Build all: `cargo-wrt build` or `cargo build`
- Build specific crate: `cargo-wrt build --package wrt|wrtd|example`
- Clean: `cargo-wrt clean` or `cargo clean`
- Run tests: `cargo-wrt test` or `cargo test`
- Run single test: `cargo test -p wrt -- test_name --nocapture`
- Format code: `cargo-wrt check` or `cargo fmt`
- Format check: `cargo-wrt check --strict`
- Main CI checks: `cargo-wrt ci`
- Full CI suite: `cargo-wrt verify --asil d`
- Typecheck: `cargo check`

## Build Matrix Verification
- **Comprehensive verification**: `cargo-wrt verify-matrix --report`
  - Tests all ASIL-D, ASIL-C, ASIL-B, development, and server configurations
  - Performs architectural analysis to identify root causes of failures
  - Generates detailed reports on ASIL compliance issues
  - CRITICAL: Run this before any architectural changes or feature additions
  - Required for all PRs that modify core runtime or safety-critical components

When build failures occur, the script will:
1. Analyze the root cause (not just symptoms)
2. Identify if issues are architectural problems affecting ASIL levels
3. Generate reports with specific remediation steps
4. Classify failures by their impact on safety compliance

## Code Style Guidelines

### General Formatting
- Use 4-space indentation
- Follow Rust naming conventions: snake_case for functions/variables, CamelCase for types
- Constants and statics use SCREAMING_SNAKE_CASE
- Use conventional commits: `type(scope): short summary`

### Import Organization
Organize imports in the following order:
1. Module attributes (`#![no_std]`, etc.)
2. `extern crate` declarations
3. Standard library imports (std, core, alloc) - grouped by feature flags
4. External crates/third-party dependencies
5. Internal crates (wrt_* imports)
6. Module imports (crate:: imports)
7. Each group should be separated by a blank line

Example:
```rust
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

use thiserror::Error;

use wrt_foundation::prelude::*;

use crate::types::Value;
```

### Type Definitions
- Always derive Debug, Clone, PartialEq, Eq for data structures
- Add Hash, Ord when semantically appropriate
- Document why if any standard derives are omitted
- Use thiserror for error definitions

### Documentation Standards
- All modules MUST have `//!` module-level documentation
- All public items MUST have `///` documentation
- Use `//` for implementation comments
- Include `# Safety` sections for unsafe code
- Examples in docs should be tested (use `no_run` if needed)

### Error Handling
- NO `.unwrap()` in production code except:
  - Constants/static initialization
  - Documented infallible operations (with safety comment)
- Define crate-specific error types using thiserror
- Use `Result<T, CrateError>` consistently

### Testing Standards
- Unit tests: Use `#[cfg(test)] mod tests {}` in source files
- Integration tests: Place in `tests/` directory only
- No test files in `src/` directory (except `#[cfg(test)]` modules)
- Test naming: `test_<functionality>` or `<functionality>_<condition>`

### Feature Flags
- Use simple, clear feature flag patterns
- Group feature-gated imports together
- Avoid redundant feature checks

### Type Consistency
- WebAssembly spec uses u32 for sizes
- Convert between u32 and usize explicitly when working with Rust memory
- Break cyclic references with Box<T> for recursive types
- Handle no_std environments with appropriate macro imports

## ASIL Compliance Requirements
When working on safety-critical components (ASIL-D, ASIL-C, ASIL-B):
1. **Always run `cargo-wrt verify-matrix --report` before committing**
2. **No unsafe code** in safety-critical configurations
3. **Deterministic compilation** - all feature combinations must build consistently
4. **Memory budget compliance** - no dynamic allocation after initialization for ASIL-D
5. **Module boundaries** - clear separation between safety-critical and non-critical code
6. **Formal verification** - Kani proofs must pass for safety-critical paths

## Architectural Analysis
The build matrix verification performs deep architectural analysis:
- **Dependency cycles** that violate ASIL modular design
- **Feature flag interactions** that create compilation conflicts
- **Memory allocation patterns** incompatible with no_std requirements
- **Trait coherence issues** indicating poor abstraction boundaries
- **Import/visibility problems** breaking deterministic builds

If architectural issues are detected, they must be resolved before merging, as they directly impact ASIL compliance and safety certification.

## Memories
- can you build and test it
- Use `cargo-wrt` for all build operations instead of just/xtask
# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.