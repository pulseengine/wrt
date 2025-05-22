===============================
No_std Verification Process
===============================

This document outlines the verification process for ensuring WRT crates work correctly in no_std environments.

Configuration Matrix
-------------------

WRT supports three distinct configurations:

+----------------------+------------------------+----------------------------------------+
| Configuration        | Features               | Description                            |
+======================+========================+========================================+
| Standard             | ``std``                | Full standard library support          |
+----------------------+------------------------+----------------------------------------+
| No_std with alloc    | ``no_std``, ``alloc`` | Heap allocation without std            |
+----------------------+------------------------+----------------------------------------+
| No_std without alloc | ``no_std``            | Bare-metal compatibility (restrictive) |
+----------------------+------------------------+----------------------------------------+

Verification Process
-------------------

The verification process tests each crate in all applicable configurations. Not all crates support all configurations,
but each crate should clearly document its supported environments.

Automated Verification
~~~~~~~~~~~~~~~~~~~~~

Use the verification script to test all crates:

.. code-block:: bash

   ./scripts/verify_no_std.sh

For detailed output:

.. code-block:: bash

   ./scripts/verify_no_std.sh --verbose

To continue testing even if one crate fails:

.. code-block:: bash

   ./scripts/verify_no_std.sh --continue-on-error

Manual Verification
~~~~~~~~~~~~~~~~~~

For manual verification, use the following commands for each crate:

Standard Configuration:

.. code-block:: bash

   cargo build -p <crate-name> --features std
   cargo test -p <crate-name> --features std

No_std with Alloc:

.. code-block:: bash

   cargo build -p <crate-name> --no-default-features --features "no_std,alloc"
   cargo test -p <crate-name> --no-default-features --features "no_std,alloc"

No_std without Alloc:

.. code-block:: bash

   cargo build -p <crate-name> --no-default-features --features "no_std"
   cargo test -p <crate-name> --no-default-features --features "no_std"

Integration Testing
~~~~~~~~~~~~~~~~~~

After verifying individual crates, run integration tests across the workspace:

.. code-block:: bash

   # Standard
   cargo test --workspace --features std
   
   # No_std with alloc
   cargo test --workspace --no-default-features --features "no_std,alloc"
   
   # No_std without alloc
   cargo test --workspace --no-default-features --features "no_std"

Specific Test Patterns
~~~~~~~~~~~~~~~~~~~~~

Each crate has specific test files targeting no_std compatibility. Run these with:

.. code-block:: bash

   cargo test -p <crate-name> --no-default-features --features "no_std" -- no_std_compatibility_test

Verification Points
------------------

For each configuration, verify:

1. **Memory Management**
   
   * Bounded collections work correctly
   * Safe memory abstractions function properly
   * Memory allocation respects configuration limitations

2. **Error Handling**
   
   * Error propagation works without panicking
   * Error contexts are properly handled

3. **Resource Management**
   
   * Resource creation/destruction works as expected
   * Resource tables function in all environments

4. **Component Model**
   
   * Component instantiation works in supported environments
   * Component values are properly handled

5. **Type Conversions**
   
   * Conversions between formats work correctly
   * No unexpected allocations occur in no_alloc mode

Crate Support Matrix
-------------------

+------------------+----------+-------------------+-------------------+
| Crate            | std      | no_std with alloc | no_std no alloc   |
+==================+==========+===================+===================+
| wrt-math         | ✓        | ✓                 | ✓                 |
+------------------+----------+-------------------+-------------------+
| wrt-sync         | ✓        | ✓                 | ✓                 |
+------------------+----------+-------------------+-------------------+
| wrt-error        | ✓        | ✓                 | ✓                 |
+------------------+----------+-------------------+-------------------+
| wrt-foundation        | ✓        | ✓                 | ✓                 |
+------------------+----------+-------------------+-------------------+
| wrt-format       | ✓        | ✓                 | ✓                 |
+------------------+----------+-------------------+-------------------+
| wrt-decoder      | ✓        | ✓                 | Partial           |
+------------------+----------+-------------------+-------------------+
| wrt-instructions | ✓        | ✓                 | ✓                 |
+------------------+----------+-------------------+-------------------+
| wrt-runtime      | ✓        | ✓                 | Partial           |
+------------------+----------+-------------------+-------------------+
| wrt-host         | ✓        | ✓                 | No                |
+------------------+----------+-------------------+-------------------+
| wrt-intercept    | ✓        | ✓                 | No                |
+------------------+----------+-------------------+-------------------+
| wrt-component    | ✓        | ✓                 | Partial           |
+------------------+----------+-------------------+-------------------+
| wrt-platform     | ✓        | ✓                 | ✓                 |
+------------------+----------+-------------------+-------------------+
| wrt-logging      | ✓        | ✓                 | Partial           |
+------------------+----------+-------------------+-------------------+
| wrt (core)       | ✓        | ✓                 | ✓                 |
+------------------+----------+-------------------+-------------------+

Common Issues
------------

1. **Hidden std Dependencies**:
   
   * Use of global allocators
   * Inadvertent use of std collections
   * String formatting outside alloc features

2. **Testing Issues**:
   
   * Test harnesses may require std
   * Tests may need to adapt based on configuration

3. **Resource Management**:
   
   * Careful handling of resources required in no_alloc mode
   * Bounded collections used instead of growable ones

CI Integration
-------------

The CI pipeline automatically runs all no_std verification steps. The process is integrated into the quality gate and
must pass before merging changes.

Debug Recommendations
--------------------

When debugging no_std issues:

1. First verify with a specific test pattern
2. Use conditional compilation to isolate problematic code sections
3. Check for any required dependencies when building without std
4. Examine error messages carefully for hidden std dependencies