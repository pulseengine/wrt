==================
Technical Internals
==================

Deep technical documentation for WRT internals and implementation details.

.. toctree::
   :maxdepth: 2

   crate_structure
   no_std_development
   panic_handling
   memory_management
   performance_considerations

Architecture Overview
=====================

WRT is built as a modular Rust workspace with clear separation of concerns:

* **Core Runtime**: WebAssembly execution engine
* **Component Model**: WASI and Component Model support
* **Platform Layer**: OS-specific optimizations
* **Foundation**: Common utilities and safe abstractions
* **Format**: Binary format parsing and validation

Crate Organization
==================

Workspace Structure
-------------------

The WRT workspace follows a hierarchical organization:

.. code-block::

   wrt2/
   ├── wrt/                    # Main runtime library
   ├── wrt-component/          # Component Model implementation  
   ├── wrt-runtime/            # Core execution engine
   ├── wrt-decoder/            # WASM binary parsing
   ├── wrt-format/             # Format specifications
   ├── wrt-foundation/         # Common utilities
   ├── wrt-platform/           # Platform abstractions
   ├── wrt-instructions/       # Instruction implementations
   ├── wrt-math/               # Mathematical operations
   ├── wrt-error/              # Error handling
   ├── wrt-logging/            # Logging infrastructure
   └── wrt-sync/               # Synchronization primitives

Dependency Graph
----------------

The crates maintain a clear dependency hierarchy to avoid cycles:

1. **Foundation Layer**: wrt-foundation, wrt-error, wrt-sync
2. **Platform Layer**: wrt-platform, wrt-logging
3. **Core Layer**: wrt-math, wrt-instructions, wrt-format
4. **Runtime Layer**: wrt-decoder, wrt-runtime
5. **High-level Layer**: wrt-component, wrt

no_std Support
==============

WRT supports three compilation modes:

1. **std**: Full standard library (default)
2. **alloc**: Heap allocation without std
3. **no_std**: Pure no_std for embedded systems

Feature flag structure:

.. code-block:: toml

   [features]
   default = ["std"]
   std = ["alloc"]
   alloc = []

Safety Architecture
===================

Memory Safety
-------------

* **No unsafe code**: All crates forbid unsafe code
* **Bounded collections**: Custom collections for no_std
* **Stack overflow protection**: Configurable stack limits
* **Integer overflow checks**: Enabled in all builds

Error Handling
--------------

* **No panics in runtime**: All errors are Result types
* **Documented panic conditions**: Limited to development builds
* **Graceful degradation**: Runtime continues after recoverable errors

Performance Considerations
==========================

Optimization Strategies
-----------------------

1. **Zero-cost abstractions**: Trait-based designs that compile to efficient code
2. **Minimal allocations**: Pre-allocated buffers and object pools
3. **Branch prediction**: Hint hot paths for better CPU performance
4. **SIMD utilization**: Platform-specific optimizations where available

Memory Layout
-------------

* **Compact structures**: Minimize padding and alignment waste
* **Pool allocation**: Reuse objects to reduce allocation pressure
* **Stack preference**: Prefer stack allocation over heap when possible

Development Guidelines
======================

Code Organization
-----------------

1. **Single responsibility**: Each crate has a focused purpose
2. **Clear interfaces**: Public APIs are minimal and well-documented
3. **Internal consistency**: Similar patterns across crates
4. **Testability**: Design for easy unit testing

Performance Requirements
------------------------

1. **Constant-time operations**: Avoid O(n) operations in hot paths
2. **Bounded resource usage**: All operations have resource limits
3. **Predictable performance**: Consistent timing characteristics
4. **Low latency**: Minimize worst-case execution time

Contributing to Internals
=========================

Before making internal changes:

1. **Understand the architecture**: Read existing code and documentation
2. **Discuss design changes**: Use GitHub issues for architectural discussions
3. **Maintain compatibility**: Preserve public API stability
4. **Add comprehensive tests**: Include unit and integration tests
5. **Document behavior**: Update documentation for any changes

Common Patterns
===============

Error Propagation
-----------------

.. code-block:: rust

   use wrt_error::{WrtError, WrtResult};

   fn operation() -> WrtResult<Value> {
       let input = validate_input()?;
       let result = process(input)?;
       Ok(result)
   }

Resource Management
-------------------

.. code-block:: rust

   use wrt_foundation::BoundedVec;

   fn with_bounded_storage<T>(capacity: usize) -> BoundedVec<T> {
       BoundedVec::with_capacity(capacity)
           .expect("capacity within bounds")
   }

Platform Abstraction
---------------------

.. code-block:: rust

   #[cfg(feature = "std")]
   use std::collections::HashMap;
   
   #[cfg(not(feature = "std"))]
   use wrt_foundation::NoStdHashMap as HashMap;

Next Steps
==========

* Review :doc:`crate_structure` for detailed module organization
* See :doc:`no_std_development` for embedded development
* Check :doc:`performance_considerations` for optimization guidelines