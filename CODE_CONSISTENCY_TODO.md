# Code Consistency Immediate Actions Todo

This document outlines the immediate actions needed to improve code consistency in the PulseEngine codebase.

## Phase 1: Move Test Files from src/ to Test Modules

### 1.1 wrt-component Test Files Migration
**Priority: HIGH** - These violate standard Rust project structure

- [ ] **canonical_abi_tests.rs** (26KB)
  - Move contents to `canonical_abi.rs` under `#[cfg(test)] mod tests`
  - Update any imports/references
  - Delete the separate test file

- [ ] **component_instantiation_tests.rs** (25KB)
  - Move to `component_instantiation.rs` or relevant module
  - Consolidate with existing tests if any
  - Delete the separate test file

- [ ] **resource_management_tests.rs** (39KB) 
  - Move to `resources/mod.rs` under `#[cfg(test)] mod tests`
  - May need to split across multiple resource-related modules
  - Delete the separate test file

- [ ] **simple_instantiation_test.rs** (8KB)
  - Move to appropriate instantiation module
  - Merge with component_instantiation tests if similar
  - Delete the separate test file

### 1.2 Other Crates Test File Audit
- [ ] **wrt-instructions/src/arithmetic_test.rs**
  - Move to `arithmetic_ops.rs` test module
  
- [ ] **wrt-debug/src/test.rs**
  - Move to lib.rs or appropriate module

- [ ] **wrt-component/src/type_conversion/** test files:
  - [ ] minimal_test.rs → type_conversion module
  - [ ] integration_test.rs → tests/ directory (if integration test)
  - [ ] simple_test.rs → type_conversion module
  - [ ] registry_test.rs → registry module

## Phase 2: Add Module Documentation

### 2.1 Critical Missing Module Documentation
**Files lacking `//!` module-level documentation:**

- [ ] **wrt-runtime modules**:
  ```rust
  //! Module purpose and functionality
  //! 
  //! Brief description of what this module provides
  ```
  - [ ] stackless/frame.rs
  - [ ] stackless/engine.rs
  - [ ] stackless/tail_call.rs
  - [ ] platform_runtime.rs
  - [ ] component_unified.rs
  - [ ] unified_types.rs

- [ ] **wrt-component modules**:
  - [ ] adapter.rs (has //, needs //!)
  - [ ] agent_registry.rs
  - [ ] bounded_component_infra.rs
  - [ ] type_conversion/registry_conversions.rs

- [ ] **wrt-host modules**:
  - [ ] bounded_host_infra.rs
  - [ ] bounded_host_integration.rs

### 2.2 Documentation Template
```rust
//! Module name and primary purpose.
//!
//! This module provides [functionality] for [use case].
//! 
//! # Key Features
//! 
//! - Feature 1
//! - Feature 2
//!
//! # Usage
//! 
//! ```no_run
//! // Example if applicable
//! ```
```

## Phase 3: Remove .unwrap() from Production Code

### 3.1 High-Priority unwrap() Removal
**Files with unwrap() in non-test code that need immediate attention:**

- [ ] **component_unified.rs**
  ```rust
  // Current:
  let memory_adapter = create_platform_memory_adapter(64 * 1024 * 1024)
      .unwrap_or_else(|_| panic!("Failed to create memory adapter"));
  
  // Should be:
  let memory_adapter = create_platform_memory_adapter(64 * 1024 * 1024)?;
  // Or handle the error appropriately
  ```

- [ ] **platform_runtime.rs**
  - Multiple unwrap() calls in non-test code
  - Need proper error propagation

- [ ] **memory.rs**
  - Check for unwrap() in initialization
  - Replace with proper Result handling

### 3.2 Safe unwrap() Patterns
**When unwrap() is acceptable (with safety comment):**

```rust
// SAFETY: This unwrap is safe because we just checked the condition
if some_vec.len() > index {
    let value = some_vec.get(index).unwrap();
}

// Or better - avoid unwrap entirely:
if let Some(value) = some_vec.get(index) {
    // use value
}
```

## Implementation Strategy

### Week 1: Test File Migration
1. Start with wrt-component test files (largest impact)
2. Run tests after each migration to ensure nothing breaks
3. Update CI if any test paths change

### Week 2: Module Documentation
1. Add documentation to all runtime modules first (core functionality)
2. Then component modules
3. Finally host and other modules

### Week 3: unwrap() Removal
1. Start with initialization code (highest risk)
2. Replace with proper error propagation
3. Add safety comments where unwrap() must remain

### Validation Steps After Each Change
1. `cargo test --all` - Ensure tests still pass
2. `cargo check --all-features` - Check all feature combinations
3. `cargo clippy` - Check for new warnings
4. `cargo-wrt verify-matrix --report` - Ensure ASIL compliance

## Success Criteria

- [ ] No test files in any `src/` directory (only `#[cfg(test)]` modules)
- [ ] All public modules have `//!` documentation
- [ ] No unwrap() in production code without safety documentation
- [ ] All changes pass CI and build matrix verification

## Notes

- Each change should be a separate commit for easy review
- Run `cargo fmt` after moving code to ensure consistent formatting
- Consider creating a lint rule to prevent future test files in src/
- Document any exceptions with clear rationale