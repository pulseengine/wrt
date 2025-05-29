============
Installation
============

This page provides detailed installation instructions for WRT across all supported platforms.

.. contents:: On this page
   :local:
   :depth: 2

System Requirements
===================

Hardware Requirements
---------------------

**Minimum:**

* 64 MB RAM (for basic embedded usage)
* 10 MB storage for runtime
* Any 32-bit or 64-bit processor with WebAssembly support

**Recommended for Development:**

* 4 GB RAM
* 1 GB storage for full toolchain
* x86_64 or ARM64 processor

Software Dependencies
=====================

Core Dependencies
-----------------

All platforms require:

1. **Rust Toolchain**: Version 1.86.0 or newer (stable)
2. **just**: Build automation tool
3. **Git**: Source code management

Install Rust
~~~~~~~~~~~~~

.. tabs::

   .. tab:: Linux/macOS

      .. code-block:: bash

         curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
         source ~/.cargo/env

   .. tab:: Windows

      Download and run the installer from `rustup.rs <https://rustup.rs/>`_

   .. tab:: Manual

      Download from the `official Rust website <https://forge.rust-lang.org/infra/channel-layout.html#archives>`_

Install just
~~~~~~~~~~~~

.. code-block:: bash

   cargo install just

Verify installation:

.. code-block:: bash

   just --version
   rustc --version

WebAssembly Targets
~~~~~~~~~~~~~~~~~~~

WRT requires WebAssembly compilation targets:

.. code-block:: bash

   rustup target add wasm32-unknown-unknown
   rustup target add wasm32-wasip1
   rustup target add wasm32-wasip2

Additional Development Tools
----------------------------

For full development workflow:

.. code-block:: bash

   # Component tooling
   cargo install cargo-component

   # WebAssembly tools
   cargo install wasmtime-cli wasm-tools

   # Code coverage (optional)
   cargo install cargo-llvm-cov

Installation Methods
====================

Source Installation
-------------------

**Recommended for most users**

1. Clone the repository:

   .. code-block:: bash

      git clone https://github.com/pulseengine/wrt.git
      cd wrt

2. Build from source:

   .. code-block:: bash

      just build

3. Run tests to verify:

   .. code-block:: bash

      just ci-test

4. (Optional) Install system-wide:

   .. code-block:: bash

      cargo install --path wrtd

Binary Installation
-------------------

**For production deployment**

Pre-built binaries are available for major platforms:

.. code-block:: bash

   # Download and install (example)
   wget https://releases.example.com/wrt/latest/wrt-linux-x86_64.tar.gz
   tar -xzf wrt-linux-x86_64.tar.gz
   sudo cp wrtd /usr/local/bin/

Package Managers
----------------

**Platform-specific packages**

.. tabs::

   .. tab:: Cargo

      .. code-block:: bash

         cargo install wrt-runtime

   .. tab:: Homebrew (macOS)

      .. code-block:: bash

         brew install wrt

   .. tab:: Debian/Ubuntu

      .. code-block:: bash

         sudo apt install wrt

Configuration
=============

Environment Variables
---------------------

Set these for optimal performance:

.. code-block:: bash

   # Runtime configuration
   export WRT_STACK_SIZE=1048576
   export WRT_FUEL_LIMIT=1000000

   # Development options
   export WRT_LOG_LEVEL=info
   export WRT_DEBUG_MODE=1

Build Configuration
-------------------

Create a ``.wrt/config.toml`` file in your project:

.. code-block:: toml

   [runtime]
   stack_size = 1048576
   fuel_limit = 1000000
   
   [security]
   enable_cfi = true
   sandbox_memory = true
   
   [performance]
   optimize_for_size = false
   enable_simd = true

Verification
============

Verify your installation works correctly:

.. code-block:: bash

   # Check WRT installation
   wrtd --version

   # Build and run example
   just test-wrtd-example

   # Run comprehensive tests
   just ci-main

Expected output should show successful compilation and test execution.

Troubleshooting
===============

Common Issues
-------------

**Rust version mismatch:**

.. code-block:: bash

   rustup update stable
   rustup default stable

**Missing WebAssembly targets:**

.. code-block:: bash

   just setup-rust-targets

**Build failures:**

.. code-block:: bash

   cargo clean
   just build

**Permission errors:**

.. code-block:: bash

   # Use cargo install without sudo
   cargo install --path wrtd

Platform-Specific Notes
=======================

For detailed platform-specific instructions, see:

* :doc:`linux` - Linux distributions
* :doc:`macos` - macOS and Apple Silicon
* :doc:`qnx` - QNX Neutrino real-time systems
* :doc:`zephyr` - Zephyr RTOS embedded systems
* :doc:`bare_metal` - Bare-metal and custom hardware

Next Steps
==========

After installation:

1. Try the :doc:`../examples/hello_world` example
2. Read the :doc:`../architecture/index` overview
3. Explore :doc:`../examples/index` for your use case