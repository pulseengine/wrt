# WRT Decoder Reorganization Plan

## Background

The WRT (WebAssembly Runtime) project consists of several crates that handle different aspects of WebAssembly:

- `wrt-types`: Core types shared across all crates
- `wrt-format`: Binary format specifications
- `wrt-decoder`: Parsing and decoding WebAssembly binaries
- `wrt-error`: Error handling shared across all crates

The current structure has issues with duplication, inconsistent organization, and unclear separation of responsibilities.

## Current Issues

1. **Duplication of Resource Operations**:
   - Custom implementations of `ResourceOperation` in both `wrt-decoder` and `wrt-format`
   - Incomplete implementations causing type errors

2. **Inconsistent Module Organization**:
   - `wrt-decoder/src/component/` directory for Component Model handling
   - Several component-related files at the top level (`component_name_section.rs`, `component_val_type.rs`, `component_validation.rs`)

3. **Unclear Separation of Core WebAssembly vs Component Model**:
   - `wrt-decoder/src/wasm.rs` is just a stub that re-exports component functionality
   - No clear organization for core WebAssembly functionality

## Implementation Plan

### Phase 1: Consolidate Component Files

1. Move component-related files into the component directory:
   
   ```
   component_name_section.rs → component/name_section.rs
   component_val_type.rs → component/val_type.rs
   component_validation.rs → component/validation.rs
   ```

2. Update all imports and exports to reflect new file locations

3. Update `component/mod.rs` to properly re-export the moved modules

### Phase 2: Establish Core WebAssembly Module

1. Create a proper `core/` directory structure:
   
   ```
   wrt-decoder/src/core/
   ├── mod.rs
   ├── decode.rs
   ├── encode.rs
   ├── parse.rs
   ├── validation.rs
   ├── name_section.rs
   └── sections.rs
   ```

2. Move core-specific code from top level into the core module

3. Update `wasm.rs` to properly handle core WebAssembly decoding

### Phase 3: Deduplicate Resource Operations

1. Move all resource operations to a single location in `wrt-types/src/resource.rs`:
   
   ```rust
   /// Resource New operation data
   #[derive(Debug, Clone)]
   pub struct ResourceNew {
       /// Type index for resource type
       pub type_idx: u32,
   }

   /// Resource Drop operation data
   #[derive(Debug, Clone)]
   pub struct ResourceDrop {
       /// Type index for resource type
       pub type_idx: u32,
   }

   /// Resource Rep operation data
   #[derive(Debug, Clone)]
   pub struct ResourceRep {
       /// Type index for resource type
       pub type_idx: u32,
   }

   /// Resource operation in a canonical function
   #[derive(Debug, Clone)]
   pub enum ResourceOperation {
       /// New resource operation
       New(ResourceNew),
       /// Drop a resource
       Drop(ResourceDrop),
       /// Resource representation operation
       Rep(ResourceRep),
   }
   ```

2. Import these from `wrt-types` in both `wrt-format` and `wrt-decoder`

3. Remove duplicate implementations from `wrt-decoder/src/component/parse.rs`

### Phase 4: Update Main Library Exports

1. Update `wrt-decoder/src/lib.rs` to properly re-export the reorganized modules:
   
   ```rust
   // Core WebAssembly functionality
   pub mod core;
   // Component Model functionality
   pub mod component;
   // Common imports
   pub mod prelude;

   // Re-exports
   pub use wrt_error::{codes, kinds};
   pub use wrt_types::{Error, ErrorCategory};

   // Type alias
   pub type Result<T> = core::result::Result<T, Error>;
   ```

### Phase 5: Ensure Consistent Feature Flags

1. Update `wrt-decoder/Cargo.toml` to ensure consistent feature flags:
   
   ```toml
   [features]
   default = [
       "std",
       "component-model-core",
       "component-model-values"
   ]
   std = ["wrt-error/std", "wrt-format/std", "wrt-types/std"]
   alloc = ["wrt-error/alloc", "wrt-format/alloc", "wrt-types/alloc"]
   no_std = ["wrt-error/no_std", "wrt-format/no_std", "wrt-types/no_std"]
   component-model-core = ["wrt-format/component-model-core"]
   component-model-values = ["wrt-format/component-model-values"]
   component-model-resources = ["wrt-format/component-model-resources"]
   component-model-fixed-lists = ["wrt-format/component-model-fixed-lists"]
   component-model-namespaces = ["wrt-format/component-model-namespaces"]
   ```

## Validation Criteria

After each phase and at completion, the following criteria must be met:

1. **Build Verification**:
   - Standard build succeeds: `cargo build --features std`
   - No-std build succeeds: `cargo build --no-default-features --features no_std,alloc`

2. **Test Verification**:
   - All tests pass: `cargo test --features std`
   - Core functionality tests pass: `cargo test --test core`
   - Component Model tests pass: `cargo test --test component`

3. **Lint Verification**:
   - No clippy warnings: `cargo clippy -- -D warnings`
   - No build warnings

4. **Documentation Verification**:
   - All public items have documentation: `cargo doc --no-deps`
   - Documentation builds without warnings

## Implementation Sequence

Each phase should be implemented in order and verified before moving to the next phase. For each phase:

1. Make the necessary changes
2. Build with std and no_std features
3. Run tests to ensure functionality is preserved
4. Run clippy to check for warnings
5. Build documentation to verify completeness
6. Mark the phase as complete

## Success Metrics

The reorganization will be considered successful if:

1. All duplication is eliminated
2. Code organization clearly separates core WebAssembly from Component Model
3. All builds (std and no_std) succeed without errors or warnings
4. All tests pass
5. No clippy warnings are present
6. Documentation is complete and builds without warnings 