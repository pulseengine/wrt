============
Build System
============

This section documents the WRT build system, including the migration from Justfile to Bazel and xtasks.

.. contents:: Table of Contents
   :local:
   :depth: 2

Overview
--------

WRT uses a hybrid build system combining:

1. **Cargo**: Primary Rust build tool for compilation and testing
2. **xtasks**: Rust-based task runner for complex build operations
3. **Bazel** (planned): For advanced build orchestration and caching
4. **Justfile** (legacy): Being phased out in favor of xtasks

Current Build System (xtasks)
-----------------------------

The xtasks system provides a Rust-based alternative to shell scripts and Makefiles.

Available Commands
~~~~~~~~~~~~~~~~~~

Run ``cargo xtask`` to see available commands::

    cargo xtask --help

Common tasks include:

- ``cargo xtask check-imports``: Verify import organization
- ``cargo xtask check-panics``: Audit panic usage in safety-critical code
- ``cargo xtask coverage``: Generate comprehensive test coverage
- ``cargo xtask docs``: Build and publish documentation
- ``cargo xtask qualification``: Run qualification checks

Task Categories
~~~~~~~~~~~~~~~

**CI Tasks**:

- ``ci-static-analysis``: Run static analysis tools
- ``ci-integrity-checks``: Verify codebase integrity
- ``ci-advanced-tests``: Run extended test suite

**Development Tasks**:

- ``fmt-check``: Check code formatting
- ``test-runner``: Run tests with custom configuration
- ``wasm-ops``: WebAssembly-specific operations

**Documentation Tasks**:

- ``docs``: Build documentation locally
- ``publish-docs-dagger``: Publish documentation via Dagger
- ``generate-coverage-summary``: Create coverage reports

Legacy Build System (Justfile)
-------------------------------

The Justfile is being phased out but still contains some useful commands:

Common Commands
~~~~~~~~~~~~~~~

::

    # Build all crates
    just build

    # Run tests
    just test

    # Format code
    just fmt

    # Run CI checks
    just ci-main

Migration Status
~~~~~~~~~~~~~~~~

Most Justfile commands have been migrated to xtasks:

- ✅ ``just build`` → ``cargo build``
- ✅ ``just test`` → ``cargo test`` or ``cargo xtask test-runner``
- ✅ ``just fmt`` → ``cargo fmt`` or ``cargo xtask fmt-check``
- ✅ ``just ci-*`` → ``cargo xtask ci-*``

Planned Bazel Integration
-------------------------

Benefits
~~~~~~~~

1. **Incremental Builds**: Fine-grained caching for faster rebuilds
2. **Reproducible Builds**: Hermetic build environment
3. **Remote Caching**: Share build artifacts across team
4. **Multi-language Support**: Build C/C++ dependencies alongside Rust

Migration Plan
~~~~~~~~~~~~~~

**Phase 1: Analysis**

- Identify build dependencies and relationships
- Map Cargo workspace to Bazel targets
- Design BUILD file structure

**Phase 2: Implementation**

- Create WORKSPACE file with Rust rules
- Generate BUILD files for each crate
- Implement custom build rules for WRT-specific needs

**Phase 3: Integration**

- Parallel builds with Cargo and Bazel
- Migrate CI/CD to use Bazel
- Performance comparison and optimization

**Phase 4: Deprecation**

- Remove Justfile completely
- Update all documentation
- Train team on Bazel usage

Build Configuration
-------------------

Workspace Structure
~~~~~~~~~~~~~~~~~~~

::

    wrt2/
    ├── Cargo.toml          # Workspace configuration
    ├── rust-toolchain.toml # Rust version specification
    ├── .cargo/
    │   └── config.toml     # Cargo configuration
    ├── xtask/
    │   └── src/            # Build tasks implementation
    └── crates/
        ├── wrt/            # Main runtime
        ├── wrt-*/          # Component crates
        └── ...

Feature Flags
~~~~~~~~~~~~~

Standard feature configuration across crates::

    [features]
    default = ["std"]
    std = ["alloc"]
    alloc = []
    safety = []
    
    # Platform features
    platform-linux = ["wrt-platform/linux"]
    platform-macos = ["wrt-platform/macos"]
    platform-qnx = ["wrt-platform/qnx"]
    platform-bare = ["wrt-platform/bare"]
    
    # Hardening features
    arm-hardening = ["wrt-platform/arm-hardening"]
    cfi = ["wrt-platform/cfi"]

Dependencies Management
~~~~~~~~~~~~~~~~~~~~~~~

Workspace dependencies are centralized in the root ``Cargo.toml``::

    [workspace.dependencies]
    thiserror = { version = "2.0", default-features = false }
    cfg-if = "1.0"
    bitflags = "2.4"

Crates reference workspace dependencies::

    [dependencies]
    thiserror = { workspace = true }
    cfg-if = { workspace = true }

Build Optimization
------------------

Release Profiles
~~~~~~~~~~~~~~~~

Optimized profiles for different use cases::

    [profile.release]
    opt-level = 3
    lto = true
    codegen-units = 1
    strip = true

    [profile.release-debug]
    inherits = "release"
    debug = true
    strip = false

    [profile.bench]
    inherits = "release"
    debug = true

Platform-Specific Builds
~~~~~~~~~~~~~~~~~~~~~~~~

Target-specific configuration::

    # ARM embedded
    cargo build --target thumbv7em-none-eabi --no-default-features

    # WebAssembly
    cargo build --target wasm32-unknown-unknown --no-default-features

    # QNX
    cargo build --target aarch64-unknown-nto-qnx7.1.0 --features platform-qnx

Continuous Integration
----------------------

GitHub Actions
~~~~~~~~~~~~~~

The CI pipeline includes:

1. **Format Check**: Ensure code follows style guidelines
2. **Clippy**: Static analysis for common mistakes
3. **Test Matrix**: Test across feature combinations
4. **Coverage**: Generate and upload coverage reports
5. **Documentation**: Build and validate docs

CI Configuration
~~~~~~~~~~~~~~~~

Key CI jobs::

    - name: Check
      run: cargo xtask ci-static-analysis
    
    - name: Test
      run: cargo xtask ci-advanced-tests
    
    - name: Coverage
      run: cargo xtask coverage

Development Workflow
--------------------

Local Development
~~~~~~~~~~~~~~~~~

1. **Setup**::

       # Clone repository
       git clone https://github.com/pulseengine/wrt.git
       cd wrt2
       
       # Install Rust toolchain
       rustup update

2. **Build**::

       # Build all crates
       cargo build
       
       # Build specific crate
       cargo build -p wrt-runtime

3. **Test**::

       # Run all tests
       cargo test
       
       # Run specific test
       cargo test -p wrt-runtime test_name

4. **Documentation**::

       # Build docs locally
       cargo xtask docs
       
       # Open in browser
       open target/doc/wrt/index.html

Pre-commit Checks
~~~~~~~~~~~~~~~~~

Run before committing::

    # Format code
    cargo fmt

    # Run clippy
    cargo clippy --all-targets --all-features

    # Check imports
    cargo xtask check-imports

    # Run tests
    cargo test

Troubleshooting
---------------

Common Issues
~~~~~~~~~~~~~

**Build Failures**:

- Check ``rust-toolchain.toml`` for required Rust version
- Ensure all dependencies are available
- Try ``cargo clean`` and rebuild

**Feature Conflicts**:

- Some features are mutually exclusive
- Check feature documentation in Cargo.toml
- Use ``--no-default-features`` when testing specific configurations

**Platform-Specific Issues**:

- Ensure target is installed: ``rustup target add <target>``
- Check platform-specific dependencies
- Verify cross-compilation tools are available

Future Improvements
-------------------

1. **Complete Bazel migration** for improved build performance
2. **Distributed builds** using remote execution
3. **Build metrics** and performance tracking
4. **Automated dependency updates** with security scanning
5. **Custom lint rules** for WRT-specific patterns