# WRT Reorganization Plan

## Background

The WRT (WebAssembly Runtime) project consists of several crates that handle different aspects of WebAssembly:

- `wrt-error`: Error handling shared across all crates
- `wrt-types`: Core and runtime types shared across all crates
- `wrt-format`: Binary format specifications
- `wrt-decoder`: Parsing and decoding WebAssembly binaries
- `wrt-instructions`: WebAssembly instruction encoding/decoding
- `wrt-component`: Component Model implementation
- `wrt-host`: Host functions and interface for WebAssembly-host interactions
- `wrt-intercept`: Implements function interception for WebAssembly functions
- `wrt-sync`: Synchronization primitives for concurrent WebAssembly execution
- `wrt-runtime`: Runtime execution engine
- `wrt`: Main library that combines all components
- `wrt-test-registry`: Unified testing framework for integration tests

The current structure has issues with inconsistent support for std and no_std configurations, code duplication, and multiple warnings and errors that prevent successful builds.

## Current Issues

1. **Inconsistent No_Std Support**:
   - Some crates properly support no_std configurations, others don't
   - Missing imports for no_std environments (e.g., alloc::format, alloc::boxed::Box)
   - Improper use of std:: paths in no_std builds

2. **Type Mismatches Between Crates**:
   - Type system inconsistencies between `ValType` and `FormatValType`
   - Redundant type definitions across crates

3. **Error Handling Inconsistencies**:
   - Different error types used across crates
   - Inconsistent error conversion mechanisms

4. **Documentation and Linting Issues**:
   - Missing documentation on public items
   - Unused imports, variables, and dead code
   - Improper macro usage in no_std environments

## Implementation Plan

### Phase 1: Fix Core Dependencies

1. Ensure `wrt-error` and `wrt-types` fully support no_std:
   
   ```rust
   // In wrt-error/src/lib.rs
   #![cfg_attr(not(feature = "std"), no_std)]
   #![cfg_attr(feature = "alloc", feature(alloc))]
   
   #[cfg(feature = "alloc")]
   extern crate alloc;
   
   // Proper imports for each environment
   #[cfg(feature = "std")]
   use std::fmt;
   
   #[cfg(all(feature = "alloc", not(feature = "std")))]
   use alloc::fmt;
   
   #[cfg(not(any(feature = "std", feature = "alloc")))]
   use core::fmt;
   ```

2. Create proper prelude modules in base crates:
   
   ```rust
   // In wrt-types/src/prelude.rs
   // Re-export commonly used types with appropriate conditional compilation
   
   #[cfg(feature = "std")]
   pub use std::string::String;
   #[cfg(feature = "std")]
   pub use std::vec::Vec;
   #[cfg(feature = "std")]
   pub use std::boxed::Box;
   
   #[cfg(all(feature = "alloc", not(feature = "std")))]
   pub use alloc::string::String;
   #[cfg(all(feature = "alloc", not(feature = "std")))]
   pub use alloc::vec::Vec;
   #[cfg(all(feature = "alloc", not(feature = "std")))]
   pub use alloc::boxed::Box;
   
   // Common format macros
   #[cfg(feature = "std")]
   pub use std::format;
   #[cfg(all(feature = "alloc", not(feature = "std")))]
   pub use alloc::format;
   ```

3. Update Cargo.toml files for all crates to properly specify features and dependencies:
   
   ```toml
   [features]
   default = ["std"]
   std = [
       "wrt-error/std",
       "wrt-types/std"
   ]
   alloc = [
       "wrt-error/alloc",
       "wrt-types/alloc"
   ]
   no_std = [
       "wrt-error/no_std",
       "wrt-types/no_std",
       "alloc"
   ]
   ```

### Phase 2: Standardize Error Handling

1. Establish a clear error conversion hierarchy across all crates:
   
   ```rust
   // In wrt-error/src/lib.rs
   
   #[derive(Debug)]
   pub enum ErrorCategory {
       Core,
       Component,
       Resource,
       Memory,
       Validation,
       Type,
       Runtime,
       System,
       Parse,
   }
   
   #[derive(Debug)]
   pub struct Error {
       category: ErrorCategory,
       code: u16,
       message: Option<&'static str>,
       #[cfg(feature = "std")]
       source: Option<Box<dyn std::error::Error + Send + Sync>>,
   }
   
   // Error conversion traits
   pub trait FromError<E> {
       fn from_error(error: E) -> Self;
   }
   ```

2. Create consistent error constructors for all crates:
   
   ```rust
   // In wrt-types/src/error.rs
   
   pub fn parse_error(message: &str) -> Error {
       Error::new(ErrorCategory::Parse, 0, Some(message))
   }
   
   pub fn validation_error(message: &str) -> Error {
       Error::new(ErrorCategory::Validation, 0, Some(message))
   }
   ```

3. Establish proper error conversion between crates:
   
   ```rust
   // In wrt-format/src/error.rs
   
   impl From<wrt_error::Error> for Error {
       fn from(err: wrt_error::Error) -> Self {
           // Convert error category and code appropriately
           // ...
       }
   }
   
   impl From<Error> for wrt_error::Error {
       fn from(err: Error) -> Self {
           // Convert back to base error type
           // ...
       }
   }
   ```

### Phase 3: Resolve Type System Inconsistencies

1. Create a comprehensive type mapping system:
   
   ```rust
   // In wrt-types/src/conversion.rs
   
   /// Convert from ValType to FormatValType
   pub fn val_type_to_format_val_type(val_type: ValType) -> FormatValType {
       match val_type {
           ValType::I32 => FormatValType::I32,
           ValType::I64 => FormatValType::I64,
           // Additional cases for all type variants...
       }
   }
   
   /// Convert from FormatValType to ValType
   pub fn format_val_type_to_val_type(format_val_type: FormatValType) -> ValType {
       match format_val_type {
           FormatValType::I32 => ValType::I32,
           FormatValType::I64 => ValType::I64,
           // Additional cases for all type variants...
       }
   }
   ```

2. Move type definitions to appropriate locations:
   
   - Core types should be in `wrt-types`
   - Format-specific types should be in `wrt-format`
   - Decoder-specific types should be in `wrt-decoder`

3. Update all imports to use the proper types:
   
   ```rust
   // In wrt-decoder/src/component/parse.rs
   
   use wrt_types::{ValType, prelude::*};
   use wrt_format::{FormatValType, conversion::format_val_type_to_val_type};
   ```

### Phase 4: Implement No_Std Support

1. Add proper conditional compilation in all crates:
   
   ```rust
   // In all crate root lib.rs files
   #![cfg_attr(not(feature = "std"), no_std)]
   
   #[cfg(feature = "alloc")]
   extern crate alloc;
   
   #[cfg(all(feature = "alloc", not(feature = "std")))]
   use alloc::{string::String, vec::Vec, boxed::Box, format};
   ```

2. Fix the format macro usage in no_std environments:
   
   ```rust
   // Direct replacement in files
   #[cfg(feature = "std")]
   let message = format!("Error at position {}", pos);
   
   #[cfg(all(feature = "alloc", not(feature = "std")))]
   let message = alloc::format!("Error at position {}", pos);
   
   #[cfg(not(any(feature = "std", feature = "alloc")))]
   // Use alternative for no alloc environments
   ```

3. Update all type imports for Box, Vec, and String:
   
   ```rust
   // In files using these types
   #[cfg(feature = "std")]
   use std::{boxed::Box, vec::Vec, string::String};
   
   #[cfg(all(feature = "alloc", not(feature = "std")))]
   use alloc::{boxed::Box, vec::Vec, string::String};
   ```

### Phase 5: Fix Documentation and Lints

1. Add missing documentation for public items:
   
   ```rust
   /// Result type alias for functions returning WRT errors
   pub type Result<T> = core::result::Result<T, Error>;
   
   /// Error category for classifying different error types
   #[derive(Debug)]
   pub enum ErrorCategory {
       /// Core WebAssembly errors
       Core,
       /// Component Model errors
       Component,
       // ...
   }
   ```

2. Fix unused imports and variables:
   
   - Remove unused imports
   - Prefix unused variables with underscore
   - Address dead code warnings

3. Fix clippy warnings:
   
   - Run `cargo clippy -- -D warnings` on each crate
   - Address all reported issues

### Phase 6: Final Integration and Testing

1. Update the main WRT crate to properly integrate all subcrates:
   
   ```rust
   // In wrt/src/lib.rs
   
   // Re-export all public functionality
   pub use wrt_types::*;
   pub use wrt_decoder::*;
   pub use wrt_format::*;
   pub use wrt_host::*;
   pub use wrt_intercept::*;
   pub use wrt_sync::*;
   // ...
   
   // Main WRT functionality
   // ...
   ```

2. Create comprehensive integration tests using the wrt-test-registry:
   
   ```rust
   // In wrt-test-registry/src/tests/compatibility.rs
   
   use wrt_test_registry::{test_case, TestRegistry};
   
   #[test]
   fn test_std_and_no_std_compatibility() {
       let registry = TestRegistry::new();
       
       registry.register(test_case!(
           name: "basic_wasm_execution",
           features: ["std", "no_std"],
           test_fn: |config| {
               // Test functionality that should work in both environments
               // ...
           }
       ));
       
       registry.run_all();
   }
   ```

3. Final validation of the entire workspace:
   
   - Build all crates with std and no_std features
   - Run all tests through the test registry: `cargo test --package wrt-test-registry`
   - Run individual crate tests: `cargo test --all`
   - Check for remaining clippy warnings
   - Verify documentation is complete

## Validation Criteria

After each phase and at completion, the following criteria must be met:

1. **Build Verification**:
   - Standard build succeeds: `cargo build --features std`
   - No-std build succeeds: `cargo build --no-default-features --features no_std,alloc`

2. **Test Verification**:
   - All tests pass: `cargo test --features std`
   - Core functionality tests pass
   - Component Model tests pass

3. **Lint Verification**:
   - No clippy warnings: `cargo clippy -- -D warnings`
   - No build warnings

4. **Documentation Verification**:
   - All public items have documentation: `cargo doc --no-deps`
   - Documentation builds without warnings

## Implementation Sequence

The implementation will proceed in order of dependency, starting from the most fundamental crates:

1. `wrt-error`: Error handling foundation
6. `wrt-sync`: Synchronization primitives
2. `wrt-types`: Core and runtime type definitions 
3. `wrt-format`: Format specifications
4. `wrt-decoder`: Binary parsing
5. `wrt-instructions`: Instruction encoding/decoding
7. `wrt-intercept`: Function interception
8. `wrt-host`: Host interface
9. `wrt-component`: Component model
10. `wrt-runtime`: Runtime execution
11. `wrt-test-registry`: Test framework
12. `wrt`: Main library integration

This ensures that fixes at the foundation level propagate properly through the dependency chain.

## Success Metrics

The reorganization will be considered successful if:

1. All crates build successfully with both std and no_std features
2. All tests pass
3. No clippy warnings are present
4. Documentation is complete and builds without warnings
5. Code duplication is eliminated
6. Type system is consistent across all crates
7. Error handling is standardized 