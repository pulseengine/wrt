# WebAssembly Component Model Built-ins Implementation Plan

This document outlines the comprehensive implementation plan for enhancing the WebAssembly Component Model implementation with a flexible built-in interception system across all affected crates.

## Goals

- Create a flexible interception system for all Component Model built-ins
- Support all built-in types defined in the Component Model specification
- Provide a builder pattern for configuration
- Ensure compatibility with both `std` and `no_std` environments
- Maintain strict code quality, test coverage, and documentation standards

## Implementation Overview

### Phase 1: Core Types and Interfaces
### Phase 2: Default Implementations
### Phase 3: Validation and Integration
### Phase 4: Runtime Adaptation
### Phase 5: Testing and Documentation

## Detailed Implementation Plan

### Phase 1: Core Types and Interfaces (Week 1)

#### 1.1 Define Built-in Types in `wrt-types`

**Tasks:**
- Create `builtin.rs` module with `BuiltinType` enumeration for all built-ins
- Implement utility methods for string conversion, feature detection, etc.
- Update `lib.rs` to re-export the new types

**Validation:**
- [x] All code builds with both `std` and `no_std` features
- [x] Clippy runs with no warnings
- [x] Unit tests cover all utility methods
- [x] Documentation is complete with examples

#### 1.2 Create Interception Interface in `wrt-intercept`

**Tasks:**
- Create `builtins.rs` module with `BuiltinInterceptor` trait
- Define `InterceptContext` struct for sharing context between interceptors
- Add serialization helpers for built-in arguments and results
- Update `lib.rs` to re-export the new types

**Validation:**
- [x] All code builds with both `std` and `no_std` features
- [x] Clippy runs with no warnings
- [x] Unit tests for serialization helpers
- [x] Documentation is complete with examples

#### 1.3 Add Core Infrastructure to `wrt-component`

**Tasks:**
- Create `builtins` directory with `mod.rs`
- Define base traits and structures for built-in handling
- Create module structure for different built-in categories
- Update `lib.rs` to include the new modules

**Validation:**
- [x] All code builds with both `std` and `no_std` features
- [x] Clippy runs with no warnings
- [x] Basic unit tests for module structure
- [x] Documentation is complete with examples

### Phase 2: Default Implementations (Week 2)

#### 2.1 Implement Resource Built-ins in `wrt-component`

**Tasks:**
- Create `builtins/resource.rs` with default implementations
- Implement all resource-related built-ins (create, drop, rep, get)
- Add configuration options for memory strategies
- Integrate with existing resource management code

**Validation:**
- [x] All code builds with both `std` and `no_std` features
- [x] Clippy runs with no warnings
- [x] Unit tests for all resource built-ins
- [x] Integration tests with resource management
- [x] Documentation is complete with examples
- [x] Code coverage meets minimum threshold (>80%)

#### 2.2 Implement Async Built-ins in `wrt-component`

**Tasks:**
- Create `builtins/async.rs` with default implementations
- Implement all async-related built-ins (new, get, poll, wait)
- Add configuration for async execution models
- Feature-gate with `async-builtins` feature

**Validation:**
- [x] All code builds with both `std` and `no_std` features
- [x] Clippy runs with no warnings
- [x] Unit tests for all async built-ins
- [x] Integration tests with async execution
- [x] Documentation is complete with examples
- [x] Code coverage meets minimum threshold (>80%)

#### 2.3 Implement Error Context Built-ins in `wrt-component`

**Tasks:**
- Create `builtins/error.rs` with default implementations
- Implement all error-related built-ins (new, trace)
- Feature-gate with `error-context` feature

**Validation:**
- [x] All code builds with both `std` and `no_std` features
- [x] Clippy runs with no warnings
- [x] Unit tests for all error built-ins
- [x] Integration tests with error handling
- [x] Documentation is complete with examples
- [x] Code coverage meets minimum threshold (>80%)

#### 2.4 Implement Threading Built-ins in `wrt-component`

**Tasks:**
- Create `builtins/threading.rs` with default implementations
- Implement all threading-related built-ins (spawn, join, sync)
- Feature-gate with `threading-builtins` feature

**Validation:**
- [x] All code builds with both `std` and `no_std` features
- [x] Clippy runs with no warnings
- [x] Unit tests for all threading built-ins
- [x] Integration tests with threading
- [x] Documentation is complete with examples
- [x] Code coverage meets minimum threshold (>80%)

### Phase 3: Host Builder and Configuration (Week 3)

#### 3.1 Create Host Builder in `wrt-host`

**Tasks:**
- Create `builder.rs` with `HostBuilder` implementation
- Implement all builder methods for configuration
- Add validation logic for required built-ins
- Update `lib.rs` to include builder pattern

**Validation:**
- [x] All code builds with both `std` and `no_std` features
- [x] Clippy runs with no warnings
- [x] Unit tests for builder patterns
- [x] Documentation is complete with examples
- [x] Code coverage meets minimum threshold (>80%)

#### 3.2 Modify Host Implementation in `wrt-host`

**Tasks:**
- Update `host.rs` to support built-in interception
- Implement built-in execution methods
- Add fallback mechanisms for critical built-ins
- Ensure proper resource management

**Validation:**
- [x] All code builds with both `std` and `no_std` features
- [x] Clippy runs with no warnings
- [x] Unit tests for all host methods
- [x] Integration tests with different configurations
- [x] Documentation is complete with examples
- [x] Code coverage meets minimum threshold (>80%)

#### 3.3 Add Component Validation in `wrt-component`

##### 3.3.1 Resolve Build Issues

**Tasks:**
- Fix import resolution in `component.rs` (e.g., add missing `binary` module)
- Resolve type compatibility between `wrt_types::ExternType` and `wrt_format::component::ExternType`
- Implement proper conversions between similar types across crates
- Fix incorrect or missing implementations for `ComponentType`
- Add missing function implementations (e.g., `extern_type_to_func_type`)
- Ensure no_std compatibility with proper type imports

**Validation:**
- [ ] All code builds with both `std` and `no_std` features
- [ ] Clippy runs with no warnings
- [ ] Code passes basic tests after fixes

##### 3.3.2 Implement Built-in Scanner

**Tasks:**
- Create `scan_builtins` function in `component.rs` to detect built-in usage in components
- Implement detection logic for resource built-ins (create, drop, rep, get)
- Add feature-gated detection for async built-ins when enabled
- Add feature-gated detection for error context built-ins when enabled
- Add feature-gated detection for threading built-ins when enabled
- Return comprehensive report of detected built-ins

**Validation:**
- [ ] Scanner works with WebAssembly component binaries
- [ ] Correctly identifies all types of built-ins
- [ ] Handles feature-gated built-ins appropriately
- [ ] Good test coverage for different component configurations

##### 3.3.3 Add Requirement Detection API

**Tasks:**
- Create `BuiltinRequirements` struct to represent component requirements
- Implement methods to query if specific built-ins are required
- Add helper function to check if requirements can be satisfied
- Create API to map detected built-ins to required host capabilities
- Add serialization/deserialization support for requirements

**Validation:**
- [ ] API is consistent with existing built-in types
- [ ] Requirements detection is accurate across test cases
- [ ] API handles feature-gated built-ins correctly
- [ ] Well-documented with examples

##### 3.3.4 Update Instantiation Flow

**Tasks:**
- Modify component instantiation to validate built-in requirements
- Add validation during component loading to detect built-in usage
- Implement appropriate error handling for missing built-ins
- Add conditional validation based on available features
- Ensure proper integration with `HostBuilder` requirements

**Validation:**
- [ ] Instantiation fails gracefully for unavailable built-ins
- [ ] Requirements are properly communicated to host environment
- [ ] Feature-gated built-ins are only required when enabled
- [ ] Testing covers all validation paths

**Validation (Overall for 3.3):**
- [ ] All code builds with both `std` and `no_std` features
- [ ] Clippy runs with no warnings
- [ ] Unit tests for validation logic
- [ ] Integration tests with valid/invalid components
- [ ] Documentation is complete with examples
- [ ] Code coverage meets minimum threshold (>80%)

### Phase 4: Runtime Integration (Week 4)

#### 4.1 Update `wrt-runtime` for Built-in Support

**Tasks:**
- Modify `component_impl.rs` to use built-in interceptors
- Update execution flow to check for built-ins
- Integrate with host environment
- Add performance optimizations for common built-ins

**Validation:**
- [ ] All code builds with both `std` and `no_std` features
- [ ] Clippy runs with no warnings
- [ ] Unit tests for runtime changes
- [ ] Performance benchmarks for built-in operations
- [ ] Documentation is complete with examples
- [ ] Code coverage meets minimum threshold (>80%)

#### 4.2 Enhance `wrt-component` Execution

**Tasks:**
- Update `execution.rs` to handle built-in interception
- Modify function resolution to detect built-ins
- Add special handling for async built-ins
- Ensure proper error propagation

**Validation:**
- [ ] All code builds with both `std` and `no_std` features
- [ ] Clippy runs with no warnings
- [ ] Unit tests for execution flow
- [ ] Integration tests with built-in calls
- [ ] Documentation is complete with examples
- [ ] Code coverage meets minimum threshold (>80%)

#### 4.3 Modify `wrtd` for Built-in Configuration

**Tasks:**
- Update `main.rs` to support built-in configuration options
- Add command-line flags for built-in settings
- Implement configuration loading from files
- Provide sensible defaults

**Validation:**
- [ ] All code builds successfully
- [ ] Clippy runs with no warnings
- [ ] Integration tests with command-line options
- [ ] Documentation is complete with examples

### Phase 5: Core API Integration (Week 5)

#### 5.1 Update `wrt` Engine API

**Tasks:**
- Modify `engine.rs` to support built-in configuration
- Add builder pattern for engine creation
- Update component loading to check built-ins
- Add documentation and examples

**Validation:**
- [ ] All code builds with both `std` and `no_std` features
- [ ] Clippy runs with no warnings
- [ ] Unit tests for engine API
- [ ] Integration tests with built-in configuration
- [ ] Documentation is complete with examples
- [ ] Code coverage meets minimum threshold (>80%)

#### 5.2 Add `wrt` Public API for Interceptors

**Tasks:**
- Create adapter types for custom interceptors
- Add convenience methods for common configurations
- Update documentation and examples
- Ensure proper type re-exports

**Validation:**
- [ ] All code builds with both `std` and `no_std` features
- [ ] Clippy runs with no warnings
- [ ] Unit tests for public API
- [ ] Integration tests with custom interceptors
- [ ] Documentation is complete with examples
- [ ] Code coverage meets minimum threshold (>80%)

#### 5.3 Final Integration Testing

**Tasks:**
- Create comprehensive integration tests
- Test across all crates
- Benchmark performance impact
- Fix any issues discovered

**Validation:**
- [ ] All integration tests pass
- [ ] Performance meets or exceeds baseline
- [ ] Code coverage meets minimum threshold (>85% across crates)
- [ ] Documentation is complete for all public APIs

## Feature Matrix

| Built-in Type    | Feature Flag       | std Support | no_std Support | Priority |
|------------------|-------------------|------------|---------------|----------|
| Resource built-ins | (always enabled)  | Yes        | Yes           | High     |
| Async built-ins  | `async-builtins`  | Yes        | No            | Medium   |
| Error Context    | `error-context`   | Yes        | Yes           | Medium   |
| Threading        | `threading-builtins` | Yes      | No            | Low      |

## Validation Checklist

For each crate at each phase:

### Build Validation
- [ ] `cargo build` passes with default features
- [ ] `cargo build --no-default-features` passes
- [ ] `cargo build --no-default-features --features="alloc"` passes
- [ ] `cargo build --all-features` passes

### Code Quality
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] No compiler warnings

### Testing
- [ ] `cargo test` passes with default features
- [ ] `cargo test --all-features` passes

### Coverage
- [ ] `cargo llvm-cov` shows minimum 80% coverage
- [ ] `cargo llvm-cov --all-features` shows minimum 80% coverage

### Documentation
- [ ] `cargo doc --no-deps` builds without warnings
- [ ] All public APIs are documented with examples

## Final Validation (wrt only)

- [ ] `cargo build` succeeds with all features
- [ ] `cargo test` succeeds with all features
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo llvm-cov` shows minimum 85% coverage
- [ ] All example code builds and runs successfully
- [ ] Documentation is comprehensive and accurate
- [ ] API is consistent and ergonomic

## Timeline

- Week 1: Phase 1 (Core Types and Interfaces)
- Week 2: Phase 2 (Default Implementations)
- Week 3: Phase 3 (Host Builder and Configuration)
- Week 4: Phase 4 (Runtime Integration)
- Week 5: Phase 5 (Core API Integration)
- Week 6: Final testing, documentation, and polish 