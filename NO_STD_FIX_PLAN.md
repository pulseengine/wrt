# No-std Compatibility Issues and Fix Plan

This document outlines the issues discovered while investigating no_std compatibility in the WebAssembly Runtime (WRT) codebase and provides a plan for fixing them. The focus is on four crates that need the most work: wrt-decoder, wrt-runtime, wrt-component, and wrt-logging.

## Summary of Issues

After investigating the codebase, we identified several common problems affecting no_std compatibility:

1. **Inappropriate Feature Configuration**: Many crates have incorrect feature flag configurations, preventing proper compilation in different environments.

2. **Import/Dependency Issues**: Issues with imports and dependencies not properly gated with feature flags or not adapted for no_std environments.

3. **Improper Use of std Types**: Direct usage of std types without proper feature gating or alternatives for no_std environments.

4. **Structural Design Issues**: Some components are fundamentally designed with assumptions about heap allocations or std features.

5. **Missing or Incorrect Re-exports**: Prelude modules often have incorrect re-exports, leading to compilation failures.

6. **Unsafe Code and Unwrap Usage**: Some code uses unsafe patterns or unwrap/expect that should be replaced with Result handling.

## Issues by Crate

### 1. wrt-foundation (Foundational Crate)

This crate underpins all the others and needs to be fixed first.

#### Core Issues:
- **Prelude Module**: Syntax errors with cfg attributes in re-export blocks
- **Component Constants**: Private constants needed in other modules not exposed
- **Type Organization**: Types not correctly organized for different feature sets
- **Import Management**: Inconsistent import patterns across modules
- **Error Handling**: Various modules are missing proper Error imports
- **Bounded Collections**: Inconsistent use of bounded collections in no_std environments

#### Files Needing Changes:
- `src/prelude.rs` - Fix re-export syntax and organize imports
- `src/component_value_store.rs` - Make needed constants public
- `src/atomic_memory.rs` - Fix WrtMutex import
- `src/safe_memory.rs` - Add proper Error, ErrorCategory, and fmt imports
- `src/component_builder.rs` - Gate Vec usage properly for no_std
- `src/bounded.rs` - Make MAX_WASM_NAME_LENGTH available in all configurations

### 2. wrt-decoder

#### Core Issues:
- **Feature Configuration**: Depends on wrt-foundation fixes
- **Pure no_std Support**: Needs better support for environments without allocation
- **Decoder Implementation**: Some decoders assume allocation is available

#### Files Needing Changes:
- `src/lib.rs` - Ensure proper feature gating
- `src/decoder_no_alloc.rs` - Expand functionality for pure no_std
- Various parser implementations - Add no_alloc alternatives

### 3. wrt-runtime

#### Core Issues:
- **Allocator Dependencies**: Core runtime depends on heap allocation
- **Memory Management**: Needs alternatives for memory operations in no_std
- **Error Handling**: Uses unwrap/expect in critical paths

#### Files Needing Changes:
- `src/component_impl.rs` - Fix HashMap usage for different environments
- `src/memory.rs` - Provide alternatives for memory allocation
- `src/stackless/engine.rs` - Address potential unwrap/panic issues

### 4. wrt-component

#### Core Issues:
- **Component Model Support**: Current implementation requires heap allocation
- **Resource Management**: Resource handling relies on dynamic allocation
- **No-alloc Functionality**: Limited support for pure no_std environments

#### Files Needing Changes:
- `src/lib.rs` - Handle feature gating and re-exports
- `src/no_alloc.rs` - Complete implementation for pure no_std environments
- `src/resources/*.rs` - Provide no_alloc alternatives for resource management

### 5. wrt-logging

#### Core Issues:
- **Feature Flags**: Incorrect feature configuration
- **Global State**: Uses global state that's not no_std compatible
- **Log Handlers**: Current handlers assume std capabilities

#### Files Needing Changes:
- `src/handler.rs` - Implement no_std compatible handlers
- `src/level.rs` - Ensure no dependencies on std
- `src/operation.rs` - Fix feature gating

## Fix Implementation Plan

### Phase 1: Foundation Fixes (wrt-foundation)

1. Fix `bounded.rs` constant availability
2. Fix `prelude.rs` re-export issues
3. Fix import issues in `atomic_memory.rs` and `safe_memory.rs`
4. Make constants public in `component_value_store.rs`
5. Fix feature gating in `component_builder.rs`

### Phase 2: Crate-specific Fixes

#### wrt-decoder
1. Expand `decoder_no_alloc.rs` functionality
2. Ensure proper feature gating in all parsers
3. Add no_std compatibility tests

#### wrt-runtime
1. Implement alternative collections for no_std in `component_impl.rs`
2. Create bounded alternatives for memory operations
3. Fix unwrap/expect usage in critical paths

#### wrt-component
1. Complete `no_alloc.rs` implementation
2. Create no_std compatible resource management
3. Update tests to cover pure no_std environment

#### wrt-logging
1. Implement no_std compatible handlers
2. Ensure level and operation modules have no std dependencies
3. Add no_std compatibility tests

### Phase 3: Integration and Testing

1. Run the verification script on individual crates
2. Fix any remaining issues that arise during testing
3. Ensure all crates pass the verification in all three environments:
   - std (default)
   - no_std with alloc
   - pure no_std (without alloc)

### Phase 4: Documentation and Examples

1. Update documentation to clarify what functionality is available in each environment
2. Add examples showing proper usage in different environments
3. Document any limitations or degraded functionality in more constrained environments

## Implementation Guidance

When implementing these fixes, follow these guidelines:

1. **Progressive Feature Degradation**: Ensure core functionality works in all environments, with enhanced capabilities in less constrained ones.

2. **Bounded Alternatives**: For allocation-dependent structures, provide bounded alternatives for memory-constrained environments.

3. **Clear Feature Gates**: Use consistent feature flags across the codebase to control which functionality is available in each environment.

4. **Error Handling**: Replace unwrap/expect with proper Result handling, especially in core functionality.

5. **Test Coverage**: Ensure tests exist for all three environments to prevent regressions.

6. **Documentation**: Clearly document what works in each environment to guide users.

## Conclusion

Fixing no_std compatibility in the WRT codebase requires addressing fundamental issues in wrt-foundation before moving on to the specific crates. The most critical issues are related to feature gating, import management, and providing alternatives for allocation-dependent functionality. By following this plan, we can achieve proper no_std support according to the project's support matrix.