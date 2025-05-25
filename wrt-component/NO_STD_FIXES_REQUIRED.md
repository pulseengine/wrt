# No_std Fixes Required for wrt-component

## Summary
To achieve full WebAssembly Component Model MVP compliance across all build configurations (std, no_std+alloc, pure no_std), the following fixes are required:

## 1. Immediate Build Fixes

### wrt-instructions (Dependency)
- [ ] Add `#[cfg(feature = "alloc")]` guards around Vec usage
- [ ] Define `BranchTarget` type
- [ ] Use `BoundedVec` for no_std configurations
- [ ] Fix imports for no_std mode

### wrt-format (Dependency)
- [ ] Complete ~200 remaining compilation errors
- [ ] Implement `ToBytes` trait for `Table`, `Memory`, `Element<P>`
- [ ] Fix generic parameter bounds
- [ ] Add missing `vec!` macro imports

### wrt-component
- [ ] Replace all `format!` usage with `error_format` module utilities
- [ ] Add conditional compilation for alloc-dependent features
- [ ] Fix unused import warnings
- [ ] Ensure all tests compile in no_std mode

## 2. Canonical ABI Implementation

### Required for MVP Compliance:
1. **String Operations**
   - UTF-8 validation without std
   - Bounded string support for no_std
   - Proper lifting/lowering

2. **List Operations**
   - Dynamic lists with BoundedVec
   - Proper memory layout
   - Size calculations

3. **Record/Struct Operations**
   - Field offset calculations
   - Alignment handling
   - Proper serialization

4. **Variant/Union Operations**
   - Discriminant handling
   - Payload serialization
   - Case validation

5. **Option/Result Types**
   - Proper representation
   - Null handling
   - Error propagation

## 3. Resource Management

### No_std Compatible Implementation:
- [ ] Bounded resource tables (BoundedMap)
- [ ] Reference counting without Arc
- [ ] Drop handler registration
- [ ] Borrow tracking

## 4. Type System

### Features Needed:
- [ ] Type equality checking
- [ ] Subtyping rules
- [ ] Recursive type support via indices
- [ ] Size and alignment calculations

## 5. Component Operations

### Core Functionality:
- [ ] Component instantiation
- [ ] Import/export resolution
- [ ] Type checking at boundaries
- [ ] Value marshaling

## 6. Testing Strategy

### Requirements:
1. **Shared Test Suite**
   ```rust
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_canonical_abi_primitives() {
           // Test on all configurations
       }
   }
   ```

2. **Configuration-Specific Tests**
   ```rust
   #[cfg(all(test, feature = "alloc"))]
   mod alloc_tests {
       // Alloc-specific tests
   }
   
   #[cfg(all(test, not(feature = "alloc")))]
   mod no_alloc_tests {
       // No-alloc specific tests
   }
   ```

## 7. Memory Limits for No_std

```rust
// Define in lib.rs or a constants module
pub const MAX_STRING_SIZE: usize = 4096;
pub const MAX_LIST_SIZE: usize = 1024;
pub const MAX_RECORD_FIELDS: usize = 64;
pub const MAX_VARIANT_CASES: usize = 256;
pub const MAX_TUPLE_SIZE: usize = 16;
pub const MAX_RESOURCES_PER_TYPE: usize = 256;
pub const MAX_COMPONENT_IMPORTS: usize = 128;
pub const MAX_COMPONENT_EXPORTS: usize = 128;
```

## 8. Feature Flags Structure

```toml
[features]
default = ["std"]
std = ["alloc", "wrt-format/std", "wrt-intercept/std", "wrt-instructions/std"]
alloc = ["wrt-format/alloc", "wrt-intercept/alloc", "wrt-instructions/alloc"]
# Component Model features
component-model-async = ["alloc"]  # Async requires alloc
component-model-threading = ["alloc"]  # Threading requires alloc
```

## 9. Error Handling Pattern

Replace all instances of:
```rust
format!("Error: {}", value)
```

With:
```rust
use crate::error_format::{format_error, CanonicalErrorContext};
format_error(ErrorCategory::Runtime, codes::OUT_OF_BOUNDS_ERROR, 
    CanonicalErrorContext::OutOfBounds { addr, size })
```

## 10. Clippy Configuration

Add to Cargo.toml:
```toml
[lints.clippy]
# Ensure no_std compatibility
std_instead_of_core = "deny"
std_instead_of_alloc = "deny"
alloc_instead_of_core = "deny"
```

## Priority Order

1. **Week 1**: Fix build errors in dependencies
2. **Week 2-3**: Implement core Canonical ABI operations
3. **Week 4**: Complete resource management
4. **Week 5**: Add component linking
6. **Week 6**: Comprehensive testing

## Success Metrics

- [ ] `cargo build --no-default-features` succeeds
- [ ] `cargo build --no-default-features --features alloc` succeeds
- [ ] `cargo build` succeeds
- [ ] `cargo clippy -- -D warnings` passes on all configurations
- [ ] All Component Model MVP features have tests
- [ ] Memory usage in no_std mode < 64KB static allocation