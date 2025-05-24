==========================
Architectural Overview
==========================

.. image:: ../../_static/icons/wrt_architecture.svg
   :width: 64px
   :align: right
   :alt: Architecture Icon

**Teaching Point**: This overview shows how Pulseengine (WRT Edition) is decomposed into manageable, testable components that work together to execute WebAssembly safely.

System Context
--------------

.. arch_component:: WRT System
   :id: ARCH_COMP_SYSTEM
   :type: system
   :tags: core

Pulseengine (WRT Edition) is a WebAssembly runtime designed for safety-critical systems. It provides:

- WebAssembly Core specification execution
- Component Model support
- Multi-platform deployment (Linux, macOS, QNX, Zephyr, bare-metal)
- Configurable memory allocation strategies
- Formal safety verification

High-Level Architecture
-----------------------

The system follows a layered architecture with clear separation of concerns.

Component Decomposition
~~~~~~~~~~~~~~~~~~~~~~~

.. arch_decision:: Component-Based Architecture
   :id: ARCH_DEC_COMP_001
   :status: implemented
   :rationale: Modular design enables independent testing, certification, and platform-specific optimization
   :impacts: All architectural components
   :links: ARCH_COMP_SYSTEM

The following diagram shows the high-level decomposition of Pulseengine into its constituent components:

.. uml:: ../../_static/high_level_decomposition_simple.puml

**Teaching Point**: Each component has a single, well-defined responsibility. The layered architecture ensures that:

- Higher layers depend only on lower layers
- Each layer provides a stable interface to the layer above
- Platform-specific code is isolated in the Platform Abstraction Layer

Deployment Architecture
~~~~~~~~~~~~~~~~~~~~~~~

.. arch_decision:: Multi-Platform Deployment Strategy
   :id: ARCH_DEC_DEPLOY_001
   :status: implemented
   :rationale: Different deployment environments require different memory models and security features
   :impacts: Platform layer, memory management
   :links: ARCH_CON_001, ARCH_CON_002

Pulseengine can be deployed across various platforms with platform-specific optimizations:

.. uml:: ../../_static/deployment_architecture_simple.puml

**Key Deployment Configurations**:

1. **Server/Desktop** (Linux, macOS)
   
   - Full ``std`` library support
   - Dynamic memory allocation
   - Multi-threading with OS primitives
   - Hardware security features (ARM PAC/BTI, Intel CET)

2. **Embedded Linux** (Yocto, BuildRoot)
   
   - ``no_std`` with custom allocator
   - Bounded memory usage
   - Optional ARM MTE support
   - Static or dynamic linking

3. **QNX Neutrino** (Automotive, Medical)
   
   - Memory partitioning for isolation
   - Capability-based security
   - Real-time scheduling
   - ASIL-D compliance support

4. **Zephyr RTOS** (IoT, Embedded)
   
   - ``no_std``, no heap allocation
   - Static memory pools
   - Minimal footprint
   - Direct hardware access

5. **Bare Metal** (Safety-critical)
   
   - No OS dependencies
   - Compile-time memory allocation
   - Deterministic execution
   - Minimal runtime overhead

Internal Module Structure
~~~~~~~~~~~~~~~~~~~~~~~~~

.. arch_decision:: Crate Organization Strategy
   :id: ARCH_DEC_CRATE_001
   :status: implemented
   :rationale: Fine-grained crates enable selective feature inclusion and minimize dependencies
   :impacts: Build system, dependency management
   :links: ARCH_CON_003

The following diagram shows the internal structure of key crates and their modules:

.. uml:: ../../_static/crate_module_structure_simple.puml

**Teaching Point**: The modular structure enables:

- Selective feature compilation (e.g., exclude Component Model for embedded)
- Platform-specific implementations behind common interfaces
- Clear dependency boundaries for safety analysis
- Independent testing and verification of each module

Workspace Organization
----------------------

The implementation consists of 24 specialized crates:

.. list-table:: Crate Organization
   :header-rows: 1
   :widths: 20 50 30

   * - Category
     - Crates
     - Purpose
   * - Foundation
     - ``wrt-error``, ``wrt-foundation``, ``wrt-format``, ``wrt-sync``
     - Core types, error handling, memory safety
   * - Decoding
     - ``wrt-decoder``
     - WebAssembly binary parsing
   * - Execution
     - ``wrt-runtime``, ``wrt-instructions``
     - Instruction execution and runtime
   * - Component Model
     - ``wrt-component``
     - Component Model implementation
   * - Platform
     - ``wrt-platform``
     - OS abstraction layer
   * - Integration
     - ``wrt-host``, ``wrt-intercept``, ``wrt-logging``
     - Host integration and monitoring
   * - Applications
     - ``wrt``, ``wrtd``
     - Library facade and CLI daemon

Environment Support Strategy
----------------------------

.. arch_decision:: Multi-Environment Architecture
   :id: ARCH_DEC_ENV_001
   :status: implemented
   :rationale: Different deployment scenarios require different resource trade-offs
   :impacts: All components

**Teaching Point**: The architecture supports four distinct environment configurations, each with specific trade-offs:

1. **Full std Environment**
   
   .. code-block:: rust
   
      // All standard library features available
      use std::collections::{HashMap, Vec};
      use std::sync::{Arc, Mutex};

   - **Use Case**: Server deployments, development
   - **Benefits**: Full functionality, familiar APIs
   - **Trade-offs**: Larger binary size, not suitable for embedded

2. **no_std with alloc**
   
   .. code-block:: rust
   
      #![no_std]
      extern crate alloc;
      use alloc::vec::Vec;
      use alloc::collections::BTreeMap as HashMap;

   - **Use Case**: Embedded systems with heap
   - **Benefits**: Dynamic allocation, smaller binary
   - **Trade-offs**: No file I/O, threading, or OS integration

3. **no_std without alloc**
   
   .. code-block:: rust
   
      #![no_std]
      use wrt_foundation::bounded::{BoundedVec, BoundedString};
      
      // Fixed capacity, no heap allocation
      let mut vec: BoundedVec<u32, 100> = BoundedVec::new();

   - **Use Case**: Safety-critical embedded, bare-metal
   - **Benefits**: Predictable memory usage, no heap fragmentation
   - **Trade-offs**: Fixed capacity limits, manual memory management

4. **Bare-metal**
   
   - **Use Case**: Minimal embedded systems
   - **Benefits**: Minimal overhead, direct hardware access
   - **Trade-offs**: Limited functionality, platform-specific

Key Architectural Principles
----------------------------

.. arch_constraint:: Safety First
   :id: ARCH_CON_001
   :priority: high
   
   All components must be memory-safe and avoid undefined behavior.

.. arch_constraint:: Deterministic Execution
   :id: ARCH_CON_002
   :priority: high
   
   Execution time and resource usage must be predictable.

.. arch_constraint:: Modular Design
   :id: ARCH_CON_003
   :priority: medium
   
   Components must be independently testable and replaceable.

Component Interaction Model
---------------------------

**Teaching Point**: Components interact through well-defined interfaces:

.. code-block:: rust

   // Example: How the decoder interacts with the runtime
   let module = wrt_decoder::decode_module(&wasm_bytes)?;
   let instance = wrt_runtime::instantiate(module, imports)?;
   let result = instance.invoke("function_name", &args)?;

Cross-References
----------------

- **Implementation Examples**: See :doc:`/examples/hello_world` for basic usage
- **Component Details**: See :doc:`components` for detailed component descriptions
- **Layer Architecture**: See :doc:`layers` for layer responsibilities
- **Design Patterns**: See :doc:`patterns` for architectural patterns used