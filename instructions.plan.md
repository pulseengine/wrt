# WebAssembly Instruction Implementation Plan

This document outlines the detailed plan for implementing Option 3: Clear Separation of Concerns for WebAssembly instructions in the WRT project. This approach maintains both the `wrt-instructions` crate and the `wrt/src/instructions` module but with clearly defined responsibilities and reduced duplication.

## Goals

- Create a clear separation of concerns between `wrt-instructions` and `wrt/src/instructions`
- Eliminate duplication in instruction implementations
- Prepare the codebase for ahead-of-time (AOT) compilation
- Ensure compatibility with both `std` and `no_std` environments
- Fix all clippy warnings, build errors, and build warnings
- Maintain test coverage and code quality

## Validation Criteria

Each step in this plan includes validation criteria. All crates modified during a step should pass these checks before proceeding to the next step:

1. **Build Validation**:
   ```bash
   cargo build --all-targets --features std
   cargo build --all-targets --no-default-features --features no_std
   ```

2. **Clippy Validation**:
   ```bash
   cargo clippy --all-targets --features std -- -D warnings
   cargo clippy --all-targets --no-default-features --features no_std -- -D warnings
   ```

3. **Test Validation**:
   ```bash
   cargo test --all-targets --features std
   ```

4. **Code Coverage Validation**:
   ```bash
   cargo llvm-cov --all-targets --features std
   ```

The final validation for the `wrt` crate is only required at the end of the implementation process.

## Implementation Steps

### Phase 1: Analysis and Planning (Preparatory Work)

#### Step 1.1: Create Detailed Inventory of Instruction Implementations ✅ COMPLETED

**Specific Tasks:**
1. Create a spreadsheet or document that lists all WebAssembly instructions
2. For each instruction, document:
   - Where it is defined (crate, file, line number)
   - Where it is implemented (crate, file, line number)
   - Dependencies and interactions with other components
   - Test coverage status
3. Identify duplications and inconsistencies

**Validation:**
- Document is complete and accurate
- All instructions are accounted for

#### Step 1.2: Define Clear Boundaries Between Crates ✅ COMPLETED

**Specific Tasks:**
1. Document the specific responsibilities of `wrt-instructions`:
   - Define it as the home for pure, stateless instruction implementations
   - Specify its dependencies (only on `wrt-types` and `wrt-error`)
   - Define the interfaces it should expose
2. Document the specific responsibilities of `wrt/src/instructions`:
   - Define it as the integration layer connecting instructions to the runtime
   - Specify how it should interact with `wrt-instructions`
3. Create diagrams of the desired architecture

**Validation:**
- Documentation clearly defines the responsibilities of each crate
- Interface definitions are complete and coherent

### Phase 2: Restructure wrt-instructions

#### Step 2.1: Expand wrt-instructions Structure  ✅ COMPLETED

**Specific Tasks:**
1. Create new module structure in `wrt-instructions/src/`:
   ```
   src/
     lib.rs                 # Main crate entry point
     memory_ops.rs          # Existing memory operations
     arithmetic_ops.rs      # Pure arithmetic operations
     control_ops.rs         # Pure control flow operations
     comparison_ops.rs      # Pure comparison operations
     conversion_ops.rs      # Pure conversion operations
     variable_ops.rs        # Pure variable access operations
     table_ops.rs           # Pure table operations
     instruction_traits.rs  # Traits for instruction implementation
     execution.rs           # Pure execution context
   ```
2. Update `Cargo.toml` to ensure both `std` and `no_std` are properly supported
3. Ensure proper exports in `lib.rs`

**Validation:**
- Run build validation
- Run clippy validation
- No functionality change required yet

#### Step 2.2: Implement Pure Instruction Traits  ✅ COMPLETED

**Specific Tasks:**
1. Define traits in `instruction_traits.rs` for instruction execution:
   ```rust
   pub trait PureInstruction<T, E> {
       fn execute(&self, context: &mut T) -> Result<(), E>;
   }
   
   pub trait PureMemoryInstruction<T, E> {
       fn execute_memory(&self, memory: &mut T) -> Result<(), E>;
   }
   ```
2. Define the execution context interfaces needed for pure instructions
3. Implement minimal versions of these contexts for testing

**Validation:**
- Run build validation
- Run clippy validation
- Write unit tests for trait implementations
- Run test validation
- Run code coverage validation

#### Step 2.3: Migrate Memory Operations  ✅ COMPLETED

**Specific Tasks:**
1. Review `wrt/src/instructions/memory.rs` and `wrt-instructions/memory_ops.rs`
2. Consolidate implementations into `wrt-instructions/memory_ops.rs`
3. Ensure the implementations are pure and don't depend on specific runtime details
4. Create interfaces for memory operations that can be easily compiled ahead-of-time
5. Ensure support for both `std` and `no_std` environments

**Validation:**
- Run build validation
- Run clippy validation
- Run test validation
- Run code coverage validation

#### Step 2.4: Implement Pure Arithmetic Operations ✅ COMPLETED

**Specific Tasks:**
1. Create pure implementations of all arithmetic operations in `arithmetic_ops.rs`
2. Ensure implementations support both `std` and `no_std`
3. Add proper error handling
4. Add unit tests for all arithmetic operations

**Validation:**
- Run build validation
- Run clippy validation
- Run test validation
- Run code coverage validation

#### Step 2.5: Implement Remaining Pure Operations ✅ COMPLETED

**Specific Tasks:**
1. Implement control flow operations in `control_ops.rs`
2. Implement comparison operations in `comparison_ops.rs`
3. Implement conversion operations in `conversion_ops.rs`
4. Implement variable access operations in `variable_ops.rs`
5. Implement table operations in `table_ops.rs`
6. Ensure all implementations are pure and stateless
7. Add unit tests for all operations

**Validation:**
- Run build validation
- Run clippy validation
- Run test validation
- Run code coverage validation

### Phase 3: Restructure wrt/src/instructions

#### Step 3.1: Update wrt Dependencies ✅ COMPLETED

**Specific Tasks:**
1. Update `wrt/Cargo.toml` to properly depend on the expanded `wrt-instructions` crate
2. Ensure feature flags for `std` and `no_std` are properly propagated

**Validation:**
- Run build validation for `wrt-instructions` and affected crates
- Run clippy validation for `wrt-instructions` and affected crates

#### Step 3.2: Refactor Instruction Type Definition ✅ COMPLETED

**Specific Tasks:**
1. Review `wrt/src/instructions/instruction_type.rs`
2. Ensure the `Instruction` enum definition aligns with the pure implementations in `wrt-instructions`
3. Add documentation to clarify the relationship between the enum and implementations
4. Refactor if needed to support ahead-of-time compilation

**Validation:**
- Run build validation for `wrt-instructions` and affected crates
- Run clippy validation for `wrt-instructions` and affected crates
- Run test validation for `wrt-instructions` and affected crates
- Run code coverage validation for `wrt-instructions` and affected crates

#### Step 3.3: Update Instruction Execution Logic

**Specific Tasks:**
1. Refactor `wrt/src/instructions/mod.rs` to use the pure implementations from `wrt-instructions`
2. Update the `InstructionExecutor` trait implementation to delegate to `wrt-instructions`
3. Create adapter code to bridge between `wrt` execution context and `wrt-instructions` execution context
4. Ensure all instructions use the pure implementations

**Validation:**
- Run build validation for `wrt-instructions` and affected crates
- Run clippy validation for `wrt-instructions` and affected crates
- Run test validation for `wrt-instructions` and affected crates
- Run code coverage validation for `wrt-instructions` and affected crates

#### Step 3.4: Update Module-Specific Files

**Specific Tasks:**
1. Update all module-specific files in `wrt/src/instructions/` to use `wrt-instructions`:
   - `arithmetic.rs`
   - `comparison.rs`
   - `control.rs`
   - `memory.rs`
   - `numeric.rs`
   - `parametric.rs`
   - `refs.rs`
   - `table.rs`
   - `variable.rs`
   - etc.
2. Refactor these files to focus on integration with the runtime engine
3. Remove duplicated implementations

**Validation:**
- Run build validation for `wrt-instructions` and affected crates
- Run clippy validation for `wrt-instructions` and affected crates
- Run test validation for `wrt-instructions` and affected crates
- Run code coverage validation for `wrt-instructions` and affected crates

### Phase 4: Ahead-of-Time Compilation Preparation

#### Step 4.1: Define AOT Interfaces

**Specific Tasks:**
1. Define interfaces for ahead-of-time compilation in `wrt-instructions/src/aot.rs`
2. Create structures to represent compiled instruction blocks
3. Implement serialization for compiled instruction blocks

**Validation:**
- Run build validation for `wrt-instructions` and affected crates
- Run clippy validation for `wrt-instructions` and affected crates
- Run test validation for `wrt-instructions` and affected crates
- Run code coverage validation for `wrt-instructions` and affected crates

#### Step 4.2: Implement Instruction Analysis

**Specific Tasks:**
1. Implement instruction analysis tools in `wrt-instructions/src/analysis.rs`
2. Add functionality to identify instruction patterns suitable for optimization
3. Add functionality to analyze instruction dependencies

**Validation:**
- Run build validation for `wrt-instructions` and affected crates
- Run clippy validation for `wrt-instructions` and affected crates
- Run test validation for `wrt-instructions` and affected crates
- Run code coverage validation for `wrt-instructions` and affected crates

#### Step 4.3: Create Basic AOT Compilation Framework

**Specific Tasks:**
1. Implement a basic framework for ahead-of-time compilation
2. Create a mechanism to cache compiled instruction blocks
3. Add interfaces for different compilation strategies

**Validation:**
- Run build validation for `wrt-instructions` and affected crates
- Run clippy validation for `wrt-instructions` and affected crates
- Run test validation for `wrt-instructions` and affected crates
- Run code coverage validation for `wrt-instructions` and affected crates

### Phase 5: Integration and Testing

#### Step 5.1: Comprehensive Integration Testing

**Specific Tasks:**
1. Create integration tests that use both `wrt-instructions` and `wrt`
2. Verify all WebAssembly instructions work correctly
3. Test edge cases and error conditions
4. Measure performance impact of the refactoring

**Validation:**
- Run build validation for all crates
- Run clippy validation for all crates
- Run test validation for all crates
- Run code coverage validation for all crates

#### Step 5.2: Documentation Update

**Specific Tasks:**
1. Update README files for `wrt-instructions` and `wrt`
2. Create detailed documentation explaining the instruction architecture
3. Add examples of how to use the pure instruction implementations
4. Document the ahead-of-time compilation interfaces

**Validation:**
- Documentation is clear and accurate
- Examples are runnable and correct

#### Step 5.3: Final Validation and Cleanup

**Specific Tasks:**
1. Perform final validation of all crates
2. Clean up any temporary code or comments
3. Ensure consistent code style across all crates
4. Verify feature flags work correctly

**Validation:**
- Run build validation for all crates
- Run clippy validation for all crates
- Run test validation for all crates
- Run code coverage validation for all crates

## Final Validation for wrt

After all phases are complete, perform a final validation of the `wrt` crate:

```bash
# Build validation
cargo build --all-targets --features std
cargo build --all-targets --no-default-features --features no_std

# Clippy validation
cargo clippy --all-targets --features std -- -D warnings
cargo clippy --all-targets --no-default-features --features no_std -- -D warnings

# Test validation
cargo test --all-targets --features std

# Code coverage validation
cargo llvm-cov --all-targets --features std
```

## Success Criteria

The implementation is considered successful if:

1. Clear separation of concerns between `wrt-instructions` and `wrt/src/instructions`
2. No duplication of instruction implementations
3. Support for both `std` and `no_std` environments
4. No clippy warnings, build errors, or build warnings
5. Maintained or improved test coverage
6. Basic framework for ahead-of-time compilation is in place
7. Final validation for `wrt` passes

## Timeline

Estimated timeline for each phase:

- Phase 1: 1-2 weeks
- Phase 2: 3-4 weeks
- Phase 3: 2-3 weeks
- Phase 4: 2-3 weeks
- Phase 5: 1-2 weeks

Total estimated timeline: 9-14 weeks 