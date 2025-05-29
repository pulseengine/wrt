=======================
Development Environment
=======================

This guide covers setting up a complete development environment for contributing to WRT.

.. toctree::
   :maxdepth: 2

   environment_setup
   toolchain_requirements
   ide_setup
   debugging_setup

Prerequisites
=============

System Requirements
-------------------

* **Rust 1.86.0 or newer**
* **Git 2.20 or newer**
* **Docker or Podman** (for Dagger CI)
* **4GB RAM minimum** (8GB recommended)

Platform Support
-----------------

* **Linux**: Primary development platform
* **macOS**: Full support with native tools
* **Windows**: WSL2 recommended

Essential Tools
===============

Core Toolchain
--------------

.. code-block:: bash

   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env

   # Install just (task runner)
   cargo install just

   # Install Dagger (CI tool)
   curl -fsSL https://dl.dagger.io/dagger/install.sh | sh

Development Tools
-----------------

.. code-block:: bash

   # WASM tooling
   cargo install cargo-component
   cargo install wasm-tools

   # Code quality
   cargo install cargo-llvm-cov
   cargo install cargo-deny

   # Documentation
   pip install -r docs/requirements.txt

Quick Setup
===========

.. code-block:: bash

   # Clone repository
   git clone https://github.com/pulseengine/wrt.git
   cd wrt

   # Verify build
   just build

   # Run tests
   just ci-test

   # Format code
   just fmt

Next Steps
==========

* See :doc:`ide_setup` for editor configuration
* See :doc:`../contributing/index` for contribution guidelines
* See :doc:`../build_system/index` for build system details