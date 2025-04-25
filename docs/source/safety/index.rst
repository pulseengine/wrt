WebAssembly Runtime Safety Manual
===================================

.. image:: ../_static/icons/safety_features.svg
   :width: 64px
   :align: center
   :alt: Safety Features Icon

This document is the Safety Manual (SM) of the qualification material developed for safety-critical applications and systems. It provides the use constraints associated with the WebAssembly Runtime (WRT) qualification scope, in accordance with established safety standards.

.. contents:: Table of Contents
   :depth: 3
   :local:
   :backlinks: none

.. toctree::
   :maxdepth: 2
   :caption: Safety Documentation Components:

   constraints
   verification_strategies
   safety_guidelines
   performance_tuning
   ../safety_mechanisms
   ../safety_implementations
   ../safety_requirements
   ../safety_test_cases

1. Qualification Scope
-------------------------

The WebAssembly Runtime has been developed with safety considerations appropriate for:

* Embedded systems with safety requirements
* Isolated execution environments for untrusted code
* Systems requiring deterministic resource usage
* Applications with memory safety requirements

1.1 Target Applications
~~~~~~~~~~~~~~~~~~~~~~~

The WebAssembly Runtime is designed for, but not limited to, the following applications:

* Embedded systems with mixed-criticality software
* IoT devices requiring isolation between components
* Edge computing platforms executing untrusted code
* Systems requiring predictable resource usage
* Applications requiring memory isolation between components

1.2 Safety Certification Approach
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The safety certification approach for the WebAssembly Runtime includes:

* Static verification of critical components
* Dynamic verification through test suites
* Code review by safety experts
* Formal verification of core algorithms where applicable
* Hazard and risk analysis

2. User Interactions
--------------------

2.1 Support Requests and Bug Reports
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Users must report any observed failures, unexpected behaviors, or safety-related concerns through the official issue tracking system. Each report should include:

* Description of the observed behavior
* Expected behavior
* Steps to reproduce
* Environment details (hardware, OS, compiler version)
* Impact assessment on safety

2.2 Obtaining Documentation
~~~~~~~~~~~~~~~~~~~~~~~~~~~

The complete documentation, including this Safety Manual, is available through:

* Source repository documentation directory
* Generated HTML/PDF documentation
* API documentation generated from source code comments

2.3 Consulting Known Problems
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Users should regularly check the known problems section in the official repository to stay informed about identified issues and available workarounds.

3. Installation Procedures
--------------------------

3.1 Installing Prerequisites
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The following prerequisites must be correctly installed before using the WebAssembly Runtime:

* Rust toolchain (minimum version 1.86.0)
* Required build dependencies as specified in the README
* For development: just command runner and python for documentation

3.2 Installing the Runtime
~~~~~~~~~~~~~~~~~~~~~~~~~~

Follow the installation instructions in the README.md file:

.. code-block:: bash

   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install just command runner (for development)
   cargo install just

   # Setup project dependencies
   just setup

3.3 Installation Validation
~~~~~~~~~~~~~~~~~~~~~~~~~~~

After installation, execute the validation tests to verify the installation:

.. code-block:: bash

   # Run validation tests
   just test-validation

A successful test run confirms the installation is valid.

4. Usage
--------

4.1 Cleaning the Build Space
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Before building for safety-critical applications, ensure a clean build environment:

.. code-block:: bash

   # Clean build artifacts
   just clean

   # Build from clean state
   just build

4.2 Warnings and Errors
~~~~~~~~~~~~~~~~~~~~~~

All compiler warnings must be treated as errors and addressed before deployment in safety-critical applications. Use:

.. code-block:: bash

   # Build with warnings treated as errors
   RUSTFLAGS="-D warnings" just build

4.3 Building WebAssembly Modules
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

When building WebAssembly modules for use with this runtime, follow these safety guidelines:

* Use memory-safe languages where possible
* Enable all compiler safety checks
* Validate WebAssembly modules before execution
* Set appropriate resource limits

4.4 Creating Host Functions
~~~~~~~~~~~~~~~~~~~~~~~~~~

When implementing host functions:

* Validate all inputs from WebAssembly modules
* Handle all error cases explicitly
* Implement resource limitation and monitoring
* Use the SafeMemoryAdapter for memory access

5. Safety-Critical Features
---------------------------

5.1 Memory Safety
~~~~~~~~~~~~~~~~

The WebAssembly Runtime implements several memory safety features to prevent out-of-bounds memory access that could corrupt system memory. All memory accesses must be validated against defined boundaries. Use SafeSlice for all memory operations to ensure bounds checking.

5.2 Resource Limitations
~~~~~~~~~~~~~~~~~~~~~~~

Always define explicit resource limits for:

* Memory usage (pages)
* Stack depth
* Call depth
* Execution time/instruction count

5.3 Bounded Collections
~~~~~~~~~~~~~~~~~~~~~~

When using bounded collections, always provide explicit capacity limits and handle capacity errors appropriately.

.. code-block:: rust

   // Good practice: Explicit capacity
   let stack = BoundedStack::<u32>::with_capacity(256);
   
   // Handle capacity errors
   if let Err(e) = stack.push(value) {
       if let BoundedError::CapacityExceeded { .. } = e {
           // Handle capacity overflow appropriately
           log::warn!("Stack capacity exceeded: {}", e);
           // Take recovery action
       }
   }

5.4 Verification Levels
~~~~~~~~~~~~~~~~~~~~~~

The runtime supports different verification levels for balancing safety and performance. Select the appropriate verification level based on safety criticality:

* ``VerificationLevel::Full`` - For safety-critical operations
* ``VerificationLevel::Standard`` - For normal operations
* ``VerificationLevel::Sampling`` - For performance-critical paths
* ``VerificationLevel::None`` - Only when safety is guaranteed by other means

6. WebAssembly-Specific Considerations
--------------------------------------

6.1 Module Validation
~~~~~~~~~~~~~~~~~~~~

All WebAssembly modules must be fully validated before execution:

.. code-block:: rust

   // Always validate modules before instantiation
   let validation_config = ValidationConfig::default();
   let validation_result = validate_module(&wasm_bytes, &validation_config)?;
   
   // Only proceed if validation was successful
   let module = Module::from_validated(validation_result);

6.2 Handling Imports
~~~~~~~~~~~~~~~~~~~

When defining imports for WebAssembly modules:

* Validate all parameters from WebAssembly
* Handle all error cases explicitly
* Apply appropriate resource limits
* Use memory safety mechanisms for memory access

6.3 Linear Memory Safety
~~~~~~~~~~~~~~~~~~~~~~~

When interacting with WebAssembly linear memory:

* Use SafeMemoryAdapter for all memory operations
* Verify offsets and lengths before memory operations
* Check for potential integer overflows in offset calculations
* Validate pointers received from WebAssembly modules

7. Handling Unsafety
-------------------

The WebAssembly Runtime uses Rust's unsafe code in certain critical sections for performance or when interfacing with external systems. The following safety measures are implemented for unsafe code:

* All unsafe blocks must be justified with clear comments explaining why unsafe is needed
* Document all invariants that must be maintained
* Each unsafe block should be reviewed by at least two developers
* Explicit test cases should verify safety properties

7.1 Unsafe Code Inventory
~~~~~~~~~~~~~~~~~~~~~~~~

An inventory of all unsafe code blocks is maintained and periodically reviewed. Each unsafe block includes:

* Location in source code
* Justification for using unsafe
* Invariants that must be maintained
* Associated test cases
* Last review date and reviewer

7.2 Handling Rust Panics
~~~~~~~~~~~~~~~~~~~~~~~

Applications using the WebAssembly Runtime must implement appropriate panic handling:

* Use panic hooks to log panic information
* In embedded environments, define custom panic handlers
* For safety-critical systems, consider restarting components on panic

8. Degraded Environment
-----------------------

8.1 Error Recovery
~~~~~~~~~~~~~~~~~

In case of detected errors, implement appropriate error recovery strategies:

* Log detailed error information
* Reset to known-good state when possible
* Implement graceful degradation modes
* Consider redundancy for critical operations

8.2 Resource Exhaustion Handling
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

When resources are exhausted, implement strategies to handle resource exhaustion:

* Prioritize critical operations
* Release non-essential resources
* Provide clear error messages indicating resource limits
* Consider implementing resource usage quotas

Refer to the constraint documentation for detailed safety constraints that must be followed when using the WebAssembly Runtime.

Appendix A: Terms, Definitions, and Abbreviations
-------------------------------------------------

A.1 Definition of Terms
~~~~~~~~~~~~~~~~~~~~~~

.. glossary::

   WRT
      WebAssembly Runtime - the subject of this safety manual

   WebAssembly
      A binary instruction format for a stack-based virtual machine

   Linear Memory
      The main memory model used by WebAssembly modules

   Safe Memory
      Memory access with bounds checking and other safety features

   Bounded Collection
      Data structures with explicit capacity limits

   Verification Level
      The degree of runtime safety checking performed

   Component Model
      An extension to WebAssembly enabling language-agnostic interfaces

A.2 Abbreviated Terms
~~~~~~~~~~~~~~~~~~~~

* **WRT**: WebAssembly Runtime
* **Wasm**: WebAssembly
* **VM**: Virtual Machine
* **MCU**: Microcontroller Unit
* **SLOC**: Source Lines of Code

.. note::

   For detailed information about panic handling and documentation, see the :doc:`../development/panic_documentation` guide. 