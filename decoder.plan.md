# Detailed Implementation Plan for wrt-decoder

## Overview

This plan addresses the incomplete parts of wrt-decoder identified in the codebase, focusing on WebAssembly Core and Component Model specifications. Each step includes specific implementation tasks, references to relevant documentation, and verification criteria. The plan has been adjusted to better align with the wrt-decoder's primary responsibilities of decoding, analyzing, and validating WebAssembly modules, while moving encoding responsibilities to wrt-format.

## Prerequisites

1. Understand the WebAssembly Core Specification: https://webassembly.github.io/spec/core/bikeshed/
2. Review Component Model documentation:
   - Binary Format: https://github.com/WebAssembly/component-model/blob/main/design/mvp/Binary.md
   - Canonical ABI: https://github.com/WebAssembly/component-model/blob/main/design/mvp/CanonicalABI.md
   - Explainer: https://github.com/WebAssembly/component-model/blob/main/design/mvp/Explainer.md

## Phase 1: WebAssembly Instruction Decoding

### Step 1.1: Complete Control Instructions ✅
- Implement block, loop, if, br_table instructions in `instructions.rs`
- Reference: [WebAssembly Control Instructions](https://webassembly.github.io/spec/core/bikeshed/#control-instructions)
- Implementation:
  - Add parsing logic in `parse_instruction`
  - Handle block types and instruction sequences

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- parse_control_instructions
cargo llvm-cov --lib --test-dir=tests/control_instructions
```

### Step 1.2: Complete Call Indirect Implementation ✅
- Implement call_indirect instruction
- Reference: [WebAssembly Table Instructions](https://webassembly.github.io/spec/core/bikeshed/#table-instructions)
- Implementation:
  - Add type indices and table handling
  - Update the table segment of the instruction handling code

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- parse_call_indirect
cargo llvm-cov --lib --test-dir=tests/call_indirect
```

### Step 1.3: Implement Memory Instructions
- Add memory load/store operations
- Reference: [WebAssembly Memory Instructions](https://webassembly.github.io/spec/core/bikeshed/#memory-instructions)
- Implementation:
  - Add all memory load variants (with alignment and offset)
  - Add all memory store variants
  - Add memory.size and memory.grow instructions

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- parse_memory_instructions
cargo llvm-cov --lib --test-dir=tests/memory_instructions
```

### Step 1.4: Complete Numeric Instructions
- Implement remaining numeric operations
- Reference: [WebAssembly Numeric Instructions](https://webassembly.github.io/spec/core/bikeshed/#numeric-instructions)
- Implementation:
  - Add integer operations
  - Add floating-point operations
  - Add conversion operations

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- parse_numeric_instructions
cargo llvm-cov --lib --test-dir=tests/numeric_instructions
```

## Phase 2: Component Model Decoding

### Step 2.1: Complete Core Module Section Decoding
- Implement `decode_core_module_section` in `component/decode.rs`
- Reference: [Component Binary Format](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Binary.md)
- Implementation:
  - Decode module count
  - Extract each module according to the binary format
  - Use wrt-format types for compatibility

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- decode_core_module_section
cargo llvm-cov --lib --test-dir=tests/component_decoding
```

### Step 2.2: Complete Core Instance Section Decoding
- Implement `decode_core_instance_section` in `component/decode.rs`
- Reference: [Component Instance Format](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Binary.md)
- Implementation:
  - Decode instance count
  - Parse instance expressions
  - Handle all core instance types

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- decode_core_instance_section
cargo llvm-cov --lib --test-dir=tests/component_decoding
```

### Step 2.3: Complete Import/Export Section Decoding
- Implement `decode_import_section` and `decode_export_section` in `component/decode.rs`
- Reference: [Component Import/Export](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Binary.md)
- Implementation:
  - Decode import counts and types
  - Decode export counts and types
  - Ensure proper decoding of names and references

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- decode_import_export_sections
cargo llvm-cov --lib --test-dir=tests/component_decoding
```

### Step 2.4: Complete Component Type Section Decoding
- Implement `decode_component_type_section` in `component/decode.rs`
- Reference: [Component Type Format](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Binary.md)
- Implementation:
  - Decode component types
  - Handle resource types if enabled
  - Support component function types

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- decode_component_type_section
cargo llvm-cov --lib --test-dir=tests/component_decoding
```

### Step 2.5: Add Canonical Function Decoding
- Implement `decode_canon_section` in `component/decode.rs`
- Reference: [Canonical ABI](https://github.com/WebAssembly/component-model/blob/main/design/mvp/CanonicalABI.md)
- Implementation:
  - Add decoding for lift/lower operations
  - Support resource operations
  - Implement memory handling operations

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- decode_canon_section
cargo llvm-cov --lib --test-dir=tests/component_decoding
```

## Phase 3: Module Structure Analysis

### Step 3.1: Enhance Type Section Analysis
- Implement or improve `analyze_type_section` in `module.rs`
- Reference: [WebAssembly Binary Format - Types](https://webassembly.github.io/spec/core/bikeshed/#binary-typesec)
- Implementation:
  - Extract and validate function types
  - Build appropriate type representations
  - Create mappings for type references

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- analyze_type_section
cargo llvm-cov --lib --test-dir=tests/module_analysis
```

### Step 3.2: Enhance Import and Function Section Analysis
- Implement or improve `analyze_import_section` and `analyze_function_section` in `module.rs`
- Reference: [WebAssembly Binary Format - Imports/Functions](https://webassembly.github.io/spec/core/bikeshed/#binary-importsec)
- Implementation:
  - Extract import entries with names and types
  - Build function tables with appropriate type references
  - Validate compatibility with other sections

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- analyze_import_function_sections
cargo llvm-cov --lib --test-dir=tests/module_analysis
```

### Step 3.3: Enhance Table and Memory Section Analysis
- Implement or improve `analyze_table_section` and `analyze_memory_section` in `module.rs`
- Reference: [WebAssembly Binary Format - Tables/Memories](https://webassembly.github.io/spec/core/bikeshed/#binary-tablesec)
- Implementation:
  - Extract and validate table types and limits
  - Extract and validate memory types and limits
  - Support shared memory if applicable

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- analyze_table_memory_sections
cargo llvm-cov --lib --test-dir=tests/module_analysis
```

### Step 3.4: Enhance Remaining Module Section Analysis
- Implement or improve analysis of global, export, start, element, code, and data sections in `module.rs`
- Reference: [WebAssembly Binary Format - Remaining Sections](https://webassembly.github.io/spec/core/bikeshed/#binary-globalsec)
- Implementation:
  - Extract and validate all section data
  - Create structured representations for runtime consumption
  - Ensure proper instruction analysis in code section

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- analyze_remaining_sections
cargo llvm-cov --lib --test-dir=tests/module_analysis
```

## Phase 4: Component Analysis and Validation

### Step 4.1: Implement Module Extraction
- Complete `extract_module_from_section` in `component/analysis.rs`
- Reference: [Component Model Binary Format](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Binary.md)
- Implementation:
  - Extract modules from component binary
  - Parse module count and sizes
  - Return proper module binaries

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- extract_module_from_section
cargo llvm-cov --lib --test-dir=tests/component_analysis
```

### Step 4.2: Enhance Component Analysis
- Improve `analyze_component` and `analyze_component_extended` in `component/analysis.rs`
- Reference: [Component Model Explainer](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Explainer.md)
- Implementation:
  - Fill in module_info, export_info, and import_info
  - Extract module names and versions
  - Analyze interface types

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- analyze_component
cargo llvm-cov --lib --test-dir=tests/component_analysis
```

### Step 4.3: Complete Component Visitor Implementation
- Implement the TODO in `register_component_for_visit` in `component_validation.rs`
- Reference: [Component Model Validation](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Explainer.md)
- Implementation:
  - Locate component in instance/component hierarchy
  - Add proper validation context
  - Implement visitor pattern for components

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- register_component_for_visit
cargo llvm-cov --lib --test-dir=tests/component_validation
```

### Step 4.4: Enhance Resource Type Validation
- Complete resource type validation in `component_validation.rs`
- Reference: [Component Model Resources](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Explainer.md)
- Implementation:
  - Validate resource operations
  - Check borrow/own semantics
  - Ensure proper resource lifetime tracking

**Verification:**
```bash
cargo build --no-default-features
cargo build
cargo clippy -- -D warnings
cargo test -- validate_resources
cargo llvm-cov --lib --test-dir=tests/component_validation
```

## Phase 5: Std/No_std Compatibility Fixes

### Step 5.1: Fix Conditional Compilation
- Review and fix conditional compilation directives across all modules
- Implementation:
  - Ensure proper #[cfg(...)] attributes
  - Fix inconsistent feature flag usage
  - Address compilation errors specific to no_std environments

**Verification:**
```bash
cargo build --no-default-features
cargo build --no-default-features --features="no_std"
cargo clippy --no-default-features -- -D warnings
```

### Step 5.2: Standardize Prelude Imports
- Create a consistent prelude module for std/no_std compatibility
- Implementation:
  - Ensure all modules import from the same prelude
  - Fix conflicting imports
  - Provide appropriate alternatives for std types in no_std

**Verification:**
```bash
cargo build --no-default-features
cargo build --no-default-features --features="no_std"
cargo clippy --no-default-features -- -D warnings
```

### Step 5.3: Fix Dependency Issues
- Address dependencies that may not be compatible with no_std
- Implementation:
  - Review all dependencies in Cargo.toml
  - Make dependencies with std requirements optional
  - Ensure appropriate feature flags are set

**Verification:**
```bash
cargo build --no-default-features
cargo build --no-default-features --features="no_std"
cargo test --no-default-features --features="no_std"
```

### Step 5.4: Fix Error Handling
- Ensure consistent error types and messages across std/no_std environments
- Implementation:
  - Use wrt-error consistently
  - Avoid std-dependent error handling
  - Provide appropriate no_std alternatives

**Verification:**
```bash
cargo build --no-default-features
cargo build --no-default-features --features="no_std"
cargo test --no-default-features --features="no_std"
```

### Step 5.5: Comprehensive Testing
- Add tests for both std and no_std environments
- Implementation:
  - Create test modules with appropriate cfg attributes
  - Test all functionalities in both environments
  - Ensure consistent behavior

**Verification:**
```bash
cargo test
cargo test --no-default-features --features="no_std"
cargo llvm-cov --no-default-features --features="no_std"
```

## Phase 6: Documentation and Integration

### Step 6.1: Document Module Interfaces
- Improve documentation across all modules
- Implementation:
  - Add docstrings for all public functions
  - Document feature-specific behavior
  - Add examples for key functionalities

**Verification:**
```bash
cargo doc --no-deps
```

### Step 6.2: Clear API Boundaries
- Ensure clear API boundaries between wrt-decoder and other crates
- Implementation:
  - Review and refine public exports
  - Ensure appropriate visibility for internal functions
  - Document integration points with other crates

**Verification:**
```bash
cargo doc --document-private-items
```

### Step 6.3: Add Integration Tests
- Create integration tests with wrt-format and wrt-runtime
- Implementation:
  - Test decoding and passing modules to runtime
  - Verify proper interaction with format crate
  - Test end-to-end workflow

**Verification:**
```bash
cargo test --test '*'
```

## Final Verification

After completing all phases, run a full verification to ensure everything works together properly:

```bash
# Build for both targets
cargo build --no-default-features
cargo build

# Run clippy with strict settings
cargo clippy -- -D warnings

# Run all tests
cargo test

# Test no_std environment
cargo test --no-default-features --features="no_std"

# Generate full code coverage report
cargo llvm-cov --html --output-dir=coverage-report

# Verify the produced binary works with an example component
cargo run -- --decode example.wasm
```

## Implementation Notes

1. **Decoding Focus**: This plan focuses on decoding, analyzing, and validating WebAssembly modules, which aligns with the core responsibilities of wrt-decoder.

2. **Binary Format Correctness**: Follow the binary format specifications exactly to ensure compatibility with other tools.

3. **Error Handling**: Use appropriate error types from wrt-error and add descriptive error messages.

4. **Documentation**: Add doc comments for all implemented functions, including examples where appropriate.

5. **Test Coverage**: Aim for at least 80% code coverage for new implementations.

6. **Performance Considerations**: While correctness is the primary goal, avoid unnecessary allocations or inefficient algorithms.

7. **Compatibility**: Ensure compatibility with both std and no_std environments throughout the implementation. 