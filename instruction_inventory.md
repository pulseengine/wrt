# WebAssembly Instruction Implementation Inventory

This document provides a detailed inventory of WebAssembly instruction implementations in the WRT codebase, as part of Phase 1, Step 1.1 of the instruction implementation plan.

## Instruction Definition

All WebAssembly instructions are defined in the `Instruction` enum in:
- **File**: `wrt/src/instructions/instruction_type.rs`
- **Lines**: 9-1307
- **Total Instructions**: ~400 instruction variants

The instructions are organized into categories:
- Control flow instructions (Block, Loop, If, etc.)
- Call instructions (Call, CallIndirect, etc.)
- Parametric instructions (Drop, Select, etc.)
- Variable instructions (LocalGet, LocalSet, etc.)
- Table instructions (TableGet, TableSet, etc.)
- Memory instructions (Load/Store operations)
- Numeric instructions (Constants and operations)
- Comparison instructions (Equality, relational)
- Conversion instructions (Type conversions)
- SIMD instructions (Vector operations)
- Reference instructions (RefNull, RefIsNull, etc.)

## Instruction Execution

### Primary Execution Implementation

The main execution logic is implemented in:
- **File**: `wrt/src/instructions/mod.rs`
- **Lines**: 37-897
- **Function**: `impl InstructionExecutor for Instruction`

This implementation matches on each instruction variant and delegates to category-specific implementations.

### Category-Specific Implementations

Instructions are further implemented in category-specific files:

1. **Control Instructions**:
   - **File**: `wrt/src/instructions/control.rs`
   - Implements: Block, Loop, If, Else, End, Br, BrIf, BrTable, Return, Unreachable, Call, CallIndirect

2. **Parametric Instructions**:
   - **File**: `wrt/src/instructions/parametric.rs`
   - Implements: Drop, Select, SelectTyped

3. **Variable Instructions**:
   - **File**: `wrt/src/instructions/variable.rs`
   - Implements: LocalGet, LocalSet, LocalTee, GlobalGet, GlobalSet

4. **Memory Instructions**:
   - **File**: `wrt/src/instructions/memory.rs`
   - Implements: Load/Store operations, MemorySize, MemoryGrow, MemoryFill, MemoryCopy, etc.

5. **Arithmetic Instructions**:
   - **File**: `wrt/src/instructions/arithmetic.rs`
   - Implements: Add, Sub, Mul, Div, etc. for different numeric types

6. **Comparison Instructions**:
   - **File**: `wrt/src/instructions/comparison.rs`
   - Implements: Eq, Ne, Lt, Gt, Le, Ge for different numeric types

7. **Numeric Instructions**:
   - **File**: `wrt/src/instructions/numeric.rs`
   - Implements: Constants, bit operations, etc.

8. **Bit Counting Instructions**:
   - **File**: `wrt/src/instructions/bit_counting.rs`
   - Implements: Clz, Ctz, Popcnt

9. **Table Instructions**:
   - **File**: `wrt/src/instructions/table.rs`
   - Implements: TableGet, TableSet, TableSize, TableGrow, etc.

10. **Reference Instructions**:
    - **File**: `wrt/src/instructions/refs.rs`
    - Implements: RefNull, RefIsNull, RefFunc

11. **SIMD Instructions**:
    - **Directory**: `wrt/src/instructions/simd/`
    - Implements: Vector operations

## Low-Level Memory Operations

The low-level memory operations are implemented in:
- **File**: `wrt-instructions/src/memory_ops.rs`
- **Lines**: 1-861

This file contains two main structs:
1. `MemoryLoad`: For memory load operations
2. `MemoryStore`: For memory store operations

These structs provide abstracted, implementation-agnostic memory operations that can be used by different execution engines.

## Duplication and Inconsistencies

There is significant duplication in memory operations between:
- `wrt/src/instructions/memory.rs`: Higher-level, engine-specific implementation
- `wrt-instructions/src/memory_ops.rs`: Lower-level, engine-agnostic implementation

The memory operations in `wrt/src/instructions/memory.rs` include:
- Direct memory access functions (e.g., `i32_load`, `i64_load`, etc.)
- Memory management operations (e.g., `memory_size`, `memory_grow`, etc.)
- Memory manipulation operations (e.g., `memory_fill`, `memory_copy`, etc.)
- Implementation of the `InstructionExecutor` trait for memory-related operations

While `wrt-instructions/src/memory_ops.rs` provides:
- Memory load/store operations through the `MemoryLoad` and `MemoryStore` structs
- Abstracted memory access methods that don't depend on the execution engine
- Support for different types and widths of memory operations

## Test Coverage

### wrt-instructions

Tests for memory operations in `wrt-instructions/src/memory_ops.rs`:
- `test_memory_load`: Tests different load operations
- `test_memory_store`: Tests different store operations
- `test_memory_access_errors`: Tests error conditions

### wrt/src/instructions

Tests for memory operations in `wrt/src/instructions/memory.rs`:
- `test_memory_size`, `test_memory_grow`: Tests memory management
- `test_i32_load_store`, `test_i64_load_store`: Tests load/store operations
- `test_memory_fill`, `test_memory_copy`: Tests memory manipulation
- `test_memory_init_data_drop`, `test_data_drop`: Tests data segment operations
- `test_memory_integration`: Integration tests

## Dependencies and Interactions

### wrt-instructions Dependencies

The `wrt-instructions` crate depends on:
- `wrt-error`: For error handling
- `wrt-types`: For type definitions
- `wrt-runtime`: For runtime functionality

### wrt/src/instructions Dependencies

The memory operations in `wrt/src/instructions` depend on:
- `crate::behavior`: For behavior traits
- `crate::error`: For error handling
- `crate::global`: For global variables
- `crate::memory`: For memory access (which re-exports from `wrt-instructions`)
- `crate::module`: For module functionality
- `crate::module_instance`: For module instances
- `crate::stackless`: For the execution engine
- `crate::types`: For type definitions
- `crate::values`: For value representations

## Recommendations

Based on this inventory, the following recommendations can be made:

1. **Memory Operations**:
   - Consolidate memory operations into `wrt-instructions/memory_ops.rs`
   - Make `wrt/src/instructions/memory.rs` only provide integration with the execution engine

2. **Arithmetic, Comparison, and Other Operations**:
   - Move pure implementations to new modules in `wrt-instructions`
   - Keep engine-specific integration in `wrt/src/instructions`

3. **Instruction Execution**:
   - Define clear interfaces for instruction execution that allow for AOT compilation
   - Separate pure execution logic from engine-specific concerns

4. **Testing**:
   - Enhance test coverage, especially for pure implementations
   - Ensure all edge cases are tested

These recommendations align with Option 3 from the instruction implementation plan, which advocates for a clear separation of concerns between the `wrt-instructions` crate and the `wrt/src/instructions` module. 

## Defined Boundaries Between Crates (Step 1.2)

Following the analysis, clear boundaries between the two crates have been defined:

### Responsibilities of `wrt-instructions`:

1. **Core Purpose**: Home for pure, stateless instruction implementations
2. **Dependencies**: 
   - `wrt-types`: For WebAssembly type definitions
   - `wrt-error`: For error handling interfaces
   - No dependencies on runtime-specific components

3. **Interfaces to Expose**:
   - Pure instruction execution traits
   - Memory operation abstractions (load/store)
   - Arithmetic operation abstractions 
   - Control flow abstractions
   - Comparison operation abstractions
   - Conversion operation abstractions
   - Variable access abstractions
   - Table operation abstractions
   - Reference operation abstractions

4. **Key Characteristics**:
   - Implementation agnostic (no runtime dependencies)
   - Support for both `std` and `no_std` environments
   - Stateless operation designs
   - Focus on correctness and compliance with the WebAssembly specification

### Responsibilities of `wrt/src/instructions`:

1. **Core Purpose**: Integration layer connecting instructions to the runtime
2. **Interaction with `wrt-instructions`**:
   - Delegates pure execution logic to `wrt-instructions`
   - Provides runtime context to pure instructions
   - Handles runtime-specific errors and behavior

3. **Specific Responsibilities**:
   - Instruction execution in the context of runtime state
   - Integration with module instances
   - Integration with stack manipulation
   - Integration with memory management
   - Integration with global variables
   - Integration with tables and function references
   - Integration with execution flow control

4. **Key Characteristics**:
   - Focused on runtime integration
   - Maintains the `Instruction` enum definition
   - Implements the interpretation infrastructure

### Architecture Diagram

```
┌───────────────────────────────────────┐      ┌───────────────────────────────────────┐
│           wrt-instructions            │      │              wrt/instructions          │
├───────────────────────────────────────┤      ├───────────────────────────────────────┤
│ - Pure instruction implementations     │      │ - Instruction enum definition         │
│ - Memory operations                   │      │ - Instruction execution               │
│ - Arithmetic operations               │◄─────┤ - Runtime integration                 │
│ - Control flow operations             │      │ - Stack manipulation                  │
│ - Comparison operations               │      │ - Module instance integration         │
│ - Conversion operations               │      │ - Error propagation                   │
│ - Variable access operations          │      │                                       │
│ - Table operations                    │      │                                       │
│ - Reference operations                │      │                                       │
└───────────────────────────────────────┘      └───────────────────────────────────────┘
           │                                                     │
           │                                                     │
           ▼                                                     ▼
┌───────────────────────────────────────┐      ┌───────────────────────────────────────┐
│             wrt-types                 │      │                 wrt-runtime            │
└───────────────────────────────────────┘      └───────────────────────────────────────┘
```

This architecture ensures:
1. Clear separation of concerns
2. Reduced duplication
3. Proper dependency management
4. Support for ahead-of-time compilation
5. Compatibility with both `std` and `no_std` environments 