# WebAssembly Component Runtime Implementation Plan

## 1. Executive Summary

This implementation plan outlines the steps to create a functional wrtd daemon that can execute WebAssembly component files using the wrt runtime and its helper crates. The plan focuses on creating a minimal viable implementation with interceptor integration, ensuring all components work together and compile successfully.

A key focus of this implementation is to clean up redundant code in the wrt crate by directly using helper crates instead of wrapping them. This approach will reduce complexity, improve error handling, and create a cleaner, more maintainable codebase.

When encountering issues, we will address them at the lowest level of the dependency chain first. This means fixing problems in the helper crates with the fewest dependencies before moving to crates that depend on them, ensuring a solid foundation for the entire system.

## 2. Current Status Assessment

### Completed Work
- Component Model binary format implementation (Phase 1-3) ✅
- Runtime support for values and resources ✅ 
- Canonical ABI implementation ✅
- Memory optimization framework ✅
- Interceptor extensions ✅
- Binary format parsing optimization ✅

### In-Progress Work
- Functional safety with bounded collections for memory operations 🔄
- SafeMemory implementation with integration challenges 🔄

### Current Issues
- Redundant code in wrt that duplicates functionality in helper crates ⚠️
- Excessive wrapping of helper crate APIs creating error handling complexity ⚠️
- Legacy code patterns that complicate integration of new features ⚠️
- Inconsistent error propagation across component boundaries ⚠️

### Key Capabilities
- WebAssembly Component Model format parsing and validation
- Resource type support with memory optimization
- Interceptor framework for runtime monitoring and modification
- Stackless execution engine with fuel metering

## 3. Implementation Plan

### Phase 0: Code Cleanup and Refactoring (1-2 days)

1. **Audit and Remove Redundant Code**
   - Identify code in wrt that duplicates functionality in helper crates
   - Remove redundant implementations and directly use helper crate APIs
   - Document each removed component and its replacement

2. **Refactor Error Handling Approach**
   - Standardize error types across component boundaries
   - Implement proper error conversion traits
   - Create clear error propagation paths
   - Remove redundant error handling code

3. **Simplify Dependency Structure**
   - Review all internal dependencies for circular references
   - Flatten dependency hierarchy where possible
   - Create clear API boundaries between components
   - Document the intended dependency structure

### Phase 1: Core Integration (1-2 days)

1. **Update wrtd CLI to Support Component Files**
   - Enhance command-line arguments to specify component options
   - Add component-specific configuration parameters
   - Update help text to reflect component model support
   - **Use helper crates directly** rather than through wrt wrappers

2. **Create Basic Component Runner**
   - Implement component instantiation logic using wrt-component directly
   - Add support for component imports/exports through direct crate usage
   - Create interface for component function execution
   - Implement proper error handling for component operations
   - **Avoid adding new code to wrt** when it can be done in helper crates

3. **Integrate Core Runtime Features**
   - Ensure wrt-component and wrt-runtime integration with direct connections
   - Update function calling convention for components
   - Implement proper memory handling for component operations
   - Connect execution statistics for components
   - **Prioritize clean interfaces** over backward compatibility

### Phase 2: Interceptor Integration (1-2 days)

1. **Implement Basic Interceptors**
   - Create logging interceptor for component operations using wrt-intercept directly
   - Implement statistics interceptor for metrics
   - Add resource usage interceptor for monitoring
   - **Avoid wrapping interceptors** in additional abstraction layers

2. **Add Interceptor Configuration**
   - Create configuration system for interceptor selection
   - Implement interceptor chaining mechanism using wrt-intercept primitives directly
   - Add interceptor parameter customization
   - Create default interceptor profiles
   - **Design for extensibility** rather than backward compatibility

3. **Connect Interceptors to Runtime**
   - Wire interceptors into component instantiation path with minimal indirection
   - Add interception points for component function calls
   - Connect resource operations to interceptors
   - Implement memory operation interception
   - **Use helper crate APIs directly** for all interceptor operations

### Phase 3: Memory Strategy Integration (1 day)

1. **Implement Memory Strategy Selection**
   - Add CLI option for memory strategy selection
   - Create configuration for trusted/untrusted components
   - Implement default selection based on component source
   - Add runtime configuration options
   - **Use wrt-component memory strategies directly** without additional wrappers

2. **Update BufferPool Integration**
   - Configure buffer pool size via configuration
   - Add buffer pool metrics collection
   - Implement buffer recycling for component operations
   - Ensure bounded memory usage
   - **Keep implementation in helper crates** rather than in wrt

### Phase 4: Sample Component Support (1 day)

1. **Create Sample Component**
   - Develop simple "hello world" component
   - Add component with resource usage
   - Create component with imports/exports
   - Implement component that exercises all features

2. **Update Documentation**
   - Document CLI options for component execution
   - Create example workflows for component usage
   - Add troubleshooting guide for common issues
   - Update success criteria documentation
   - **Document direct helper crate usage patterns** to encourage clean design

## 4. Success Criteria

The implementation will be considered successful when:

1. **Basic Component Execution Works**
   - Can load and instantiate a WebAssembly component file
   - Can execute exported component functions
   - Results are correctly returned and displayed
   - Errors are properly handled and reported

2. **Interceptor Integration Functions**
   - At least one interceptor can be configured and active
   - Interceptors correctly monitor component operations
   - Interceptor chain properly modifies execution as needed
   - Configuration options work correctly

3. **Memory Strategies Are Applied**
   - Different memory strategies can be selected
   - Bounded memory usage is properly enforced
   - Resource usage is correctly tracked and limited
   - Memory operations are optimized based on context

4. **Everything Compiles Successfully**
   - All crates build without errors
   - Integration between crates works correctly
   - No linking or dependency issues
   - Clean compilation with no warnings

5. **Code Quality Improvements**
   - Reduced code duplication between wrt and helper crates
   - Clearly defined API boundaries between components
   - Improved error handling across module boundaries
   - Decreased complexity in the dependency graph
   - Reduction in overall codebase size

## 5. Implementation Details

### Component Loading Process

```rust
// Pseudocode for component loading process - uses helper crates directly
fn load_component(engine: &mut StacklessEngine, bytes: &[u8], options: ComponentOptions) -> Result<ComponentInstance> {
    // Use wrt-component directly to parse component binary
    let component = wrt_component::Component::parse(bytes)?;
    
    // Use wrt-component memory strategies directly
    let memory_strategy = select_memory_strategy(&options);
    
    // Use wrt-intercept directly for interceptor configuration
    let interceptors = configure_interceptors(&options);
    
    // Use wrt-component instantiation directly
    let instance = component.instantiate(engine, memory_strategy, interceptors)?;
    
    // Return the component instance
    Ok(instance)
}
```

### Interceptor Configuration

```rust
// Pseudocode for interceptor configuration - uses wrt-intercept directly
fn configure_interceptors(options: &ComponentOptions) -> Vec<Box<dyn wrt_intercept::Interceptor>> {
    let mut interceptors = Vec::new();
    
    // Use wrt-intercept interceptors directly
    if options.enable_logging {
        interceptors.push(Box::new(wrt_intercept::LoggingInterceptor::new(options.log_level)));
    }
    
    if options.enable_stats {
        interceptors.push(Box::new(wrt_intercept::StatisticsInterceptor::new()));
    }
    
    if options.enable_resource_monitoring {
        interceptors.push(Box::new(wrt_intercept::ResourceMonitorInterceptor::new(options.resource_limits)));
    }
    
    interceptors
}
```

### Memory Strategy Selection

```rust
// Pseudocode for memory strategy selection - uses wrt-component strategies directly
fn select_memory_strategy(options: &ComponentOptions) -> wrt_component::MemoryStrategy {
    match options.memory_strategy.as_deref() {
        Some("zero-copy") => wrt_component::MemoryStrategy::ZeroCopy,
        Some("bounded-copy") => wrt_component::MemoryStrategy::BoundedCopy { 
            buffer_size: options.buffer_size.unwrap_or(1024 * 1024) 
        },
        Some("full-isolation") => wrt_component::MemoryStrategy::FullIsolation,
        None => {
            // Default strategy based on trust level
            match options.trust_level {
                TrustLevel::Trusted => wrt_component::MemoryStrategy::ZeroCopy,
                TrustLevel::Standard => wrt_component::MemoryStrategy::BoundedCopy { buffer_size: 1024 * 1024 },
                TrustLevel::Untrusted => wrt_component::MemoryStrategy::FullIsolation,
            }
        },
        Some(unknown) => {
            warn!("Unknown memory strategy: {}, using BoundedCopy", unknown);
            wrt_component::MemoryStrategy::BoundedCopy { buffer_size: 1024 * 1024 }
        }
    }
}
```

## 6. Proposed CLI Interface

```
USAGE:
    wrtd [OPTIONS] <WASM_FILE>

ARGUMENTS:
    <WASM_FILE>    Path to WebAssembly component file to execute

OPTIONS:
    -c, --call <FUNCTION>             Function to call after instantiation
    -f, --fuel <AMOUNT>               Limit execution to specified amount of fuel
    -s, --stats                       Show execution statistics
    -i, --interceptors <NAMES>        Comma-separated list of interceptors to enable [possible values: logging, stats, resources, firewall]
    -m, --memory-strategy <STRATEGY>  Memory strategy to use [possible values: zero-copy, bounded-copy, full-isolation]
    -t, --trust-level <LEVEL>         Trust level of the component [possible values: trusted, standard, untrusted]
    -h, --help                        Print help information
    -V, --version                     Print version information
```

## 7. Implementation Tasks

### Phase 0: Code Cleanup and Refactoring

- [ ] **Audit and Remove Redundant Code**
  - [ ] Review wrt crate for code duplicating helper crate functionality
  - [ ] Document each identified redundancy
  - [ ] Create replacement patterns using direct helper crate APIs
  - [ ] Remove redundant code

- [ ] **Refactor Error Handling**
  - [ ] Identify error conversion paths
  - [ ] Standardize error types across boundaries
  - [ ] Implement conversion traits
  - [ ] Document error flow

- [ ] **Simplify Dependency Structure**
  - [ ] Map current dependency graph
  - [ ] Identify circular dependencies
  - [ ] Restructure to flatten hierarchy
  - [ ] Document intended structure

### Phase 1: Core Integration

- [ ] **Update wrtd/src/main.rs CLI Arguments**
  - [ ] Add component-specific command line options
  - [ ] Update help text to include component model info
  - [ ] Add component configuration types
  - [ ] Use helper crates directly instead of through wrt

- [ ] **Enhance Component Loading**
  - [ ] Update `load_component` function to use wrt-component directly
  - [ ] Add proper error handling for component operations
  - [ ] Support component imports/exports correctly
  - [ ] Remove unnecessary abstractions

- [ ] **Integrate with Runtime**
  - [ ] Ensure direct connection between wrt-component and stackless engine
  - [ ] Update execution model for components
  - [ ] Add proper resource cleanup
  - [ ] Avoid adding new wrapper code to wrt

### Phase 2: Interceptor Integration

- [ ] **Create Basic Interceptors**
  - [ ] Implement LoggingInterceptor using wrt-intercept directly
  - [ ] Implement StatisticsInterceptor for metrics
  - [ ] Implement ResourceMonitorInterceptor
  - [ ] Avoid additional abstraction layers

- [ ] **Add Configuration System**
  - [ ] Create InterceptorConfig struct
  - [ ] Add chain builder for interceptors
  - [ ] Implement parameter support
  - [ ] Design for direct helper crate usage

- [ ] **Connect to Runtime**
  - [ ] Add interceptor support to component instantiation with minimal indirection
  - [ ] Add interception points for function calls
  - [ ] Implement memory operation interception
  - [ ] Use helper crate APIs directly

### Phase 3: Memory Strategy Integration

- [ ] **Implement Strategy Selection**
  - [ ] Create MemoryStrategyConfig struct
  - [ ] Add CLI options for memory strategy
  - [ ] Implement default strategy selection
  - [ ] Use wrt-component memory strategies directly

- [ ] **Enhance Buffer Pool**
  - [ ] Add configuration options for buffer pool
  - [ ] Implement metrics collection
  - [ ] Ensure bounded memory usage
  - [ ] Keep implementation in helper crates

### Phase 4: Sample Component Support

- [ ] **Create Sample Components**
  - [ ] Simple "hello world" component
  - [ ] Component with resource usage
  - [ ] Component with imports/exports
  - [ ] Component demonstrating direct helper crate usage

- [ ] **Update Documentation**
  - [ ] Add CLI option documentation
  - [ ] Create usage examples
  - [ ] Add troubleshooting section
  - [ ] Document direct helper crate usage patterns

## 8. Timeline

| Task | Timeline | Priority |
|------|----------|----------|
| Code cleanup and audit | Day 1 | Critical |
| Refactor error handling | Day 1 | Critical |
| Simplify dependencies | Day 2 | High |
| Update wrtd CLI | Day 2 | High |
| Create basic component runner | Day 3 | High |
| Integrate runtime features | Day 3 | High |
| Implement basic interceptors | Day 4 | Medium |
| Add interceptor configuration | Day 4 | Medium |
| Connect interceptors to runtime | Day 5 | Medium |
| Implement memory strategy selection | Day 5 | Medium |
| Update buffer pool integration | Day 6 | Low |
| Create sample components | Day 6 | Low |
| Update documentation | Day 6 | Low |

## 9. Dependencies and Risks

### Dependencies
- Completion of core WebAssembly component model features
- Functional safety plan implementation status
- Interceptor framework readiness

### Risks
- Potential integration challenges between crates
- Memory safety concerns with different strategies
- Performance impact of bounded collections
- Legacy code dependencies that resist removal

### Risk Mitigation
- Start with minimal integration first and expand gradually
- Add comprehensive error handling and reporting
- Focus on getting basic functionality working before optimizing
- Use bounded collections with appropriate verification levels
- Begin with a thorough code audit to identify problematic patterns
- Document all legacy code removal to help track impacts

## 10. Next Steps for Implementers

1. Start with code audit to identify redundant code in wrt
2. Refactor error handling to use helper crate error types directly
3. Update wrtd CLI to support component files using helper crates directly
4. Implement basic component loading and instantiation with minimal wrt usage
5. Add simple logging interceptor integration using wrt-intercept directly
6. Test with a basic component file
7. Gradually add more features once basic execution works
8. Ensure all changes compile successfully at each step
9. Document helper crate usage patterns to encourage clean design

## 11. Code Cleanup Guidelines

To ensure we create a clean design and avoid legacy patterns, follow these guidelines:

1. **Direct Usage Over Wrapping**
   - Always prefer direct usage of helper crate APIs over creating wrapper functions
   - If a helper crate already provides functionality, use it directly rather than reimplementing
   - Minimize abstraction layers between the client code and helper crate functionality

2. **Error Handling Best Practices**
   - Use explicit error types from helper crates
   - Implement conversion traits for error types when crossing crate boundaries
   - Avoid generic error types that obscure the actual error cause
   - Propagate detailed error information up the call stack

3. **Clean API Design**
   - Prefer composition over inheritance
   - Design for future extensibility
   - Create clear ownership boundaries between components
   - Document all public interfaces thoroughly

4. **Dependency Management**
   - Keep dependencies between crates as flat as possible
   - Avoid circular dependencies at all costs
   - Minimize dependency chains
   - Make dependencies explicit in the API design

## 12. Error Resolution Strategy

When encountering errors during implementation, follow this structured approach:

1. **Bottom-Up Problem Resolution**
   - Always fix issues in the lowest-level crates first
   - Only proceed to dependent crates after lower-level crates are fully fixed
   - Work through the dependency chain systematically
   - Validate each crate compiles and passes tests before moving up the chain

2. **Crate Prioritization by Dependencies**
   - Start with the crates that have the fewest dependencies:
     1. wrt-error, wrt-sync, wrt-types (foundation crates)
     2. wrt-format, wrt-decoder (format handling crates)
     3. wrt-instructions, wrt-runtime (execution crates)
     4. wrt-component, wrt-intercept (component model crates)
     5. wrt, wrtd (top-level crates)

3. **Complete Crate Resolution**
   - Never leave a crate partially fixed
   - Ensure each crate fully compiles before moving to dependent crates
   - Run comprehensive tests on each crate after fixing issues
   - Document all issues and their resolutions

4. **Error Documentation**
   - For each error encountered:
     - Document which crate owns the issue
     - Record the root cause analysis
     - Document the solution approach
     - Track any dependent issues that might arise

5. **Integration Testing**
   - After fixing issues in a crate, verify that all dependent crates still work
   - Run integration tests at each step
   - Create regression tests for fixed issues
   - Ensure no new issues are introduced

This approach ensures we build on a solid foundation, avoiding the cascade of errors that can occur when trying to fix high-level issues without addressing underlying problems in dependencies.

As you implement each part, update this document to track progress and capture any challenges or design decisions that arise during implementation. 