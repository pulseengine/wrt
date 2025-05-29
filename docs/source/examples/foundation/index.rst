==========================
Foundation Library Examples
==========================

.. image:: ../../_static/icons/safe_memory.svg
   :width: 64px
   :align: right
   :alt: Foundation Icon

.. epigraph::

   "The foundation is the most important part of any building."
   
   -- Unknown Architect (but definitely not a software one)

The ``wrt-foundation`` crate is where the magic happens. It provides the core building blocks that make WRT safe, fast, and suitable for embedded systems. No heap allocations, no panics, just rock-solid primitives you can trust.

What's in the Foundation? 🏗️
-----------------------------

Think of ``wrt-foundation`` as your Swiss Army knife for safety-critical WebAssembly:

- **Bounded Collections**: Fixed-size collections that never allocate
- **Safe Memory**: Bounds-checked slices and memory adapters
- **Atomic Operations**: Lock-free primitives for concurrent code
- **No-std Utilities**: HashMap and friends without the standard library
- **Resource Management**: Handle tables and lifecycle tracking

.. contents:: Foundation Examples
   :local:
   :depth: 2

.. toctree::
   :maxdepth: 1
   :caption: Examples in this section:

   bounded_collections
   safe_memory
   atomic_memory
   sync_primitives
   no_std_hashmap
   component_values
   resources
   async_examples

Why These Matter 🎯
-------------------

**For Embedded Systems:**
   When you're running on a microcontroller with 64KB of RAM, you can't afford heap allocations or unbounded growth. Every byte counts.

**For Safety-Critical Code:**
   In automotive or aerospace applications, a panic is not an option. These primitives are designed to fail gracefully and predictably.

**For Performance:**
   Zero-cost abstractions aren't just a Rust thing - they're a WRT thing too. These primitives compile down to efficient machine code.

Quick Comparison 📊
-------------------

.. list-table:: Foundation vs Standard Library
   :header-rows: 1
   :widths: 30 35 35

   * - Feature
     - Standard Library
     - WRT Foundation
   * - Memory Allocation
     - Dynamic (heap)
     - Static (stack/compile-time)
   * - Panic Behavior
     - Can panic on OOM
     - Returns Result/Option
   * - Thread Safety
     - Varies by type
     - Explicit, always safe
   * - no_std Support
     - ❌ Not available
     - ✅ First-class support
   * - Size Overhead
     - Includes allocator
     - Zero overhead

Getting Started 🚀
------------------

Add ``wrt-foundation`` to your ``Cargo.toml``:

.. code-block:: toml

   [dependencies]
   wrt-foundation = "0.1"

Then import the prelude:

.. code-block:: rust

   use wrt_foundation::prelude::*;

That's it! You're ready to build bulletproof WebAssembly modules.

.. admonition:: Design Philosophy
   :class: note

   Every type in ``wrt-foundation`` follows these principles:
   
   1. **No Hidden Allocations**: What you see is what you get
   2. **Explicit Error Handling**: No surprises, no panics
   3. **Const-Friendly**: Use them in const contexts where possible
   4. **Zero-Cost**: Abstractions compile away completely
   5. **Verified Safe**: Formally verified or extensively tested

Pick Your Adventure 🗺️
-----------------------

Where do you want to start?

- **New to bounded collections?** → :doc:`bounded_collections`
- **Need safe memory access?** → :doc:`safe_memory`
- **Building concurrent code?** → :doc:`atomic_memory`
- **Working without std?** → :doc:`no_std_hashmap`
- **Handling component data?** → :doc:`component_values`
- **Managing resources?** → :doc:`resources`
- **Writing async code?** → :doc:`async_examples`

Remember: These aren't just examples - they're patterns you'll use in every WRT application!