# Code Consistency Report

This report identifies inconsistencies in the PulseEngine codebase and provides recommendations for standardization.

## 1. Test Organization Inconsistencies

### Current State
- **Mixed test locations**: 
  - Some modules use `#[cfg(test)] mod tests {}` within source files
  - Others have separate test files like `canonical_abi_tests.rs` in src/
  - Integration tests exist in both `tests/` directories and `src/`

### Examples
```rust
// In wrt-runtime/src/memory.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_memory_grow() { ... }
}

// But in wrt-component/src/
canonical_abi_tests.rs  // Separate test file
simple_instantiation_test.rs  // Another separate test file
```

### Recommendation
```rust
// STANDARD: Use #[cfg(test)] mod tests for unit tests in source files
// Keep integration tests in tests/ directory
// No test files in src/ directory
```

## 2. Import Organization Inconsistencies

### Current State
Different import patterns across files:

```rust
// wrt-runtime/src/memory.rs - Mixed ordering
extern crate alloc;
use core::alloc::Layout;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
#[cfg(not(feature = "std"))]
use core::borrow::BorrowMut;
#[cfg(feature = "std")]
use std::borrow::BorrowMut;

// wrt-component/src/adapter.rs - Better organization
#[cfg(not(feature = "std"))]
use core::{fmt, mem};
#[cfg(feature = "std")]
use std::{fmt, mem};

use wrt_foundation::{bounded::BoundedVec, component::ComponentType, prelude::*};
use crate::execution_engine::ComponentExecutionEngine;
```

### Recommendation
```rust
// STANDARD Import Order:
// 1. extern crate declarations
// 2. std/core imports (grouped by feature flags)
// 3. Third-party crates
// 4. Internal crates (wrt_*)
// 5. Module imports (crate::)
// 6. Each group separated by blank line
```

## 3. Error Handling Inconsistencies

### Current State
- Production code uses `.unwrap()` in some places (27 files in wrt-runtime)
- Mixed use of custom error types vs generic Error
- No consistent error conversion patterns

### Examples
```rust
// Some modules carefully avoid unwrap
pub fn new(memory_type: MemoryType) -> Result<Self> {
    // Proper error handling
}

// Others use unwrap in non-test code
let memory_adapter = create_platform_memory_adapter(64 * 1024 * 1024)
    .unwrap_or_else(|_| panic!("Failed to create memory adapter"));
```

### Recommendation
```rust
// STANDARD: No unwrap() in production code except:
// 1. Constants/static initialization
// 2. Documented infallible operations
// All unwrap() must have comment explaining why it's safe
```

## 4. Documentation Inconsistencies

### Current State
- Some modules use `//!` for module docs, others use `//`
- Mixed use of `///` and `//` for item documentation
- Many public items lack documentation

### Examples
```rust
// Good - wrt-runtime/src/memory.rs
//! WebAssembly memory implementation.
//!
//! This module provides a comprehensive implementation...

// Poor - many files lack module documentation entirely
// No //! documentation at top of file
```

### Recommendation
```rust
// STANDARD:
//! Module-level documentation (required for all modules)
/// Item documentation (required for all public items)
// Implementation comments (as needed)
```

## 5. Module Organization Inconsistencies

### Current State
- Some use `mod.rs`, others don't
- Inconsistent re-export patterns
- Mixed approaches to module visibility

### Recommendation
```rust
// STANDARD: Use mod.rs for modules with submodules
// Parent module should selectively re-export public API
// Keep implementation details private
```

## 6. Feature Flag Inconsistencies

### Current State
```rust
// Redundant checks
#[cfg(all(not(feature = "std"), feature = "alloc"))]
#[cfg(not(feature = "std"))]  // This makes previous line redundant

// Inconsistent patterns
#[cfg(not(feature = "std"))]
vs
#[cfg(not(any(feature = "std", )))]  // Empty any clause
```

### Recommendation
```rust
// STANDARD: Simple, clear feature flags
#[cfg(feature = "std")]
#[cfg(not(feature = "std"))]
#[cfg(all(not(feature = "std"), feature = "alloc"))]
```

## 7. Type Definition Inconsistencies

### Current State
- `Result_` type with trailing underscore
- Inconsistent derive macros
- Mixed naming conventions

### Recommendation
```rust
// STANDARD: All types should derive these when possible:
#[derive(Debug, Clone, PartialEq, Eq)]
// Add Hash, Ord when semantically appropriate
// Document why if any are omitted
```

## 8. Const/Static Naming

### Current State
- Mixed SCREAMING_SNAKE_CASE and regular naming
- Some constants not marked const

### Recommendation
```rust
// STANDARD:
const MAX_MEMORY_PAGES: u32 = 65536;  // SCREAMING_SNAKE_CASE for constants
static INSTANCE_COUNT: AtomicU32 = AtomicU32::new(0);  // Same for statics
```

## Proposed Coding Standards

### 1. File Structure Template
```rust
//! Module documentation (required)
//!
//! Detailed description...

// 1. Attributes
#![cfg_attr(not(feature = "std"), no_std)]

// 2. Extern crate (if needed)
extern crate alloc;

// 3. Imports (grouped and ordered)
use core::mem;

use external_crate::Type;

use wrt_foundation::prelude::*;

use crate::module::Type;

// 4. Constants
const MAX_SIZE: usize = 1024;

// 5. Type definitions
/// Type documentation (required for public)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MyType {
    // fields
}

// 6. Implementations
impl MyType {
    /// Constructor documentation
    pub fn new() -> Result<Self> {
        // No unwrap without safety comment
    }
}

// 7. Tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new() {
        // Test implementation
    }
}
```

### 2. Error Handling Standards
```rust
// Define crate-specific error type
#[derive(Debug, thiserror::Error)]
pub enum CrateError {
    #[error("Memory allocation failed: {0}")]
    MemoryError(String),
    // Other variants
}

// Use Result<T, CrateError> consistently
pub type Result<T> = core::result::Result<T, CrateError>;
```

### 3. Testing Standards
- Unit tests: `#[cfg(test)] mod tests` in source file
- Integration tests: `tests/` directory only
- Test names: `test_<functionality>` or `<functionality>_<condition>`
- Use consistent assertion style

### 4. Documentation Standards
- All public items must have `///` documentation
- All modules must have `//!` documentation
- Examples in documentation should be tested
- Use `# Safety` sections for unsafe code

## Implementation Plan

1. **Phase 1**: Update CLAUDE.md with coding standards
2. **Phase 2**: Create rustfmt.toml and clippy.toml for automation
3. **Phase 3**: Gradually update existing code during normal development
4. **Phase 4**: Add CI checks for new code

## Automation Opportunities

### rustfmt.toml
```toml
# Enforce consistent formatting
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
format_code_in_doc_comments = true
```

### clippy.toml
```toml
# Enforce documentation
missing-docs-in-private-items = "warn"
```

### CI Checks
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- Custom script to check import ordering
- Documentation coverage tool

This report provides a path to more consistent, maintainable code while respecting the project's existing patterns and constraints.