# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### 🚀 Features

- *(xtask)* Add automatic panic registry update tool
- *(resources)* Implement resource management system - Add canonical format for binary resources - Add verification functionality - Delete deprecated resource-tests module - Update panic registry documentation
- *(component)* Implement WebAssembly Component Model built-ins
- *(host)* Implement host builder and built-in hosting system
- *(test)* Add test program for control instructions
- Add instructions adapter for pure instruction execution
- Enhance component implementation with resource support
- Add decoder examples and improve component validation
- Enhance host implementation with resource handling
- Extend type system with resource type support
- *(resource)* Implement WebAssembly Component Model resource management
- *(memory)* Implement safe memory abstractions with integrity verification
- *(component)* Implement component registry and module management
- *(resources)* Implement resource management system
- *(api)* Add prelude modules for consistent public API
- *(types)* Implement type conversion system for components
- *(error)* Enhance error handling and validation
- *(component)* Implement RuntimeInstance with function execution support
- Configure editor settings and git attributes for consistent line endings and file encoding
- *(wrt-error)* Introduce comprehensive error code system
- *(wrt-decoder)* Add custom section utils and runtime module adapter
- *(wrt-runtime)* Implement new runtime module, instance, and stackless execution core
- *(wrt-types)* Implement WebAssembly numeric math operations
- *(wrt-sync)* Add WrtOnce for one-time initialization
- *(wrt-test-registry)* Add prelude and criterion benchmarks
- *(xtask)* Add Dagger CI pipelines for tests, checks, and documentation
- *(types)* Add component_value_store and restructure math_ops
- *(.ai)* Implement new nodes and update AI flows
- *(resources)* Add ResourceArena and SizeClassBufferPool
- *(resources)* Add no_std and no_alloc support for resource management
- *(platform)* Add platform-specific memory and synchronization optimizations
- *(types)* Add improved bounded collections and memory types with better no_std support
- *(wrt)* Add no_std resource implementation to core runtime
- *(helper)* Add wrt-helper crate for common utilities
- *(no_std)* Strengthen and document no_std compatibility across workspace
- *(platform)* Add advanced platform abstraction and Tock/Zephyr support
- *(runtime)* Integrate CFI engine and debug support
- *(docs)* Enhance architecture documentation and fix build issues

### 🐛 Bug Fixes

- Unused variable warning in validation code
- *(wrt-sync)* Align RwLock error type with parking_lot update (ParkingLockError -> PoisonError)
- *(CI)* Failure in github CI build
- *(resource)* Correct no_std implementation of resource strategy
- Improve no_std compatibility across crates
- Add missing safe-memory feature to wrt-format and wrt-runtime
- Add safe-memory feature to wrt-types
- Apply formatter and linter fixes
- Remove panic=abort from test and bench profiles
- Update publish workflow to use modern Rust toolchain action
- *(logging)* Correct log level error handling and remove invalid alloc dependency
- *(intercept)* Simplify error messages and update value formatting
- *(ci)* Ensure Dagger CLI is available in PATH for all workflow steps - Use  to persist Dagger binary directory in the workflow - Split Dagger version check into a separate step to verify installation - Fixes 'dagger: command not found' error in GitHub Actions
- *(ci)* Ensure Dagger CLI is in PATH for all steps in publish workflow
- *(xtask)* Correct string escaping in coverage_ci.rs

### 💼 Other

- *(deps)* Bump anyhow from 1.0.97 to 1.0.98
- *(deps)* Bump wast from 227.0.1 to 229.0.0
- *(deps)* Bump wat from 1.227.1 to 1.229.0
- Add build scripts and documentation templates
- Update dependencies
- *(deps)* Bump codecov/codecov-action from 4 to 5
- Remove obsolete references and improve documentation
- *(deps)* Bump ctor from 0.2.9 to 0.4.2
- *(deps)* Bump toml from 0.7.8 to 0.8.22
- *(deps)* Bump colored from 2.2.0 to 3.0.0
- *(deps)* Bump hashbrown from 0.14.5 to 0.15.3
- *(deps)* Bump proptest-derive from 0.4.0 to 0.5.1
- *(deps)* Bump kani-verifier from 0.61.0 to 0.62.0
- *(deps)* Bump wast from 229.0.0 to 230.0.0

### 🚜 Refactor

- *(wrt-error)* Improve error handling system
- *(wrt-types)* Update type system implementation
- *(instructions)* Implement pure instruction traits and operations
- Extract pure control flow operations to wrt-instructions
- Update format module for improved resource handling
- Update error handling for resource operations
- Update table and variable operations
- *(decoder)* Reorganize decoder structure with core and WASM implementations
- Remove obsolete files and reorganize codebase
- *(runtime)* Improve memory and stack implementations
- Fix import paths and update code structure
- Remove Bazel build system and old instruction/type definitions
- *(xtask)* Integrate Dagger, clap, and update task implementations
- Align crates with new types, error handling, and runtime design
- *(ci)* Overhaul GitHub Actions workflow to use Daggerized xtasks
- *(wrt-types)* Enhance bounded collections, memory provider, and component values
- *(xtask)* Update task scripts, remove bazel_ops and add generate_source_needs
- *(core)* Apply widespread updates and fixes across WRT modules
- *(wrt-types)* Major overhaul of core types, memory handling, and traits
- Move math ops to new crate and add platform crate
- *(resources)* Split mod.rs into separate resource-specific files
- *(examples)* Remove component_graph_view example and its dependencies
- *(resources)* Enhance resource management implementation with no_std support
- *(decoder)* Improve component parsing and handling of custom sections
- *(error)* Enhance error handling system with better context support
- *(math)* Improve math operations with enhanced floating-point support
- *(sync)* Improve synchronization primitives and host API with better no_std support
- *(instructions)* Optimize instruction execution with improved type handling
- *(runtime)* Improve memory management in core runtime
- Finalize codebase with consistent imports and type references
- Major workspace restructuring and no_std compatibility improvements
- Clean up error handling and panic messages
- *(core)* Update core modules and documentation - Update core library and host modules for improved resource and error handling - Refactor decoder and instruction modules for better maintainability - Update documentation for architecture and development sections - Improve test coverage and fix minor issues in helper and sync crates

### 📚 Documentation

- Remove outdated panic documentation
- Restructure documentation with new API and development sections
- Add planning documents for builtins, decoder, and instructions
- Update planning documents and implement core runtime changes
- Add project planning and agent prompts
- Update agent prompt with implementation sequence and success metrics
- Update README.md
- *(conf)* Update sphinx configuration
- Update documentation and reorganize architecture section
- Update documentation structure and styling
- Add improvements summary document
- Add custom fonts and no_std collections documentation
- Update NO_STD_FIXES.md with additional changes
- *(examples)* Add new documentation, guides, and debug tools - Add new architecture and development documentation, including CFI and QNX platform - Add new example and debug modules for improved test coverage - Add migration guides, build system docs, and workspace improvements - Add new README files and CFI control ops implementation
- Reworked architecture documentation and added getting started

### 🧪 Testing

- *(decoder)* Add tests for call_indirect and control instructions
- *(instructions)* Add arithmetic operations test
- Add comprehensive test suite with no_std compatibility tests
- Remove unused memory search test
- Add platform optimizations tests and improve test infrastructure

### ⚙️ Miscellaneous Tasks

- Update GitHub workflow and dependencies
- Update dependencies and integration for resource implementation
- Update dependencies and configuration files
- Update CI/CD workflows
- Add project configuration and developer tooling docs
- Update Rust toolchain from 1.78.0 to 1.86.0
- Update CI workflow and Justfile to use Dagger xtasks
- Add .ai directory for development tooling
- Update .gitignore
- Update dependencies and lockfile
- Update root Cargo.toml and add/remove misc files
- Remove obsolete BUILD files, docker files, and githooks
- Update build configuration and project metadata
- Update CI, docs, and build scripts for new features
- *(cleanup)* Remove obsolete and migrated files - Delete legacy migration, QNX, and improvement plan documents - Remove outdated icons and static resources from documentation - Clean up obsolete markdown and plan files from project root - Ensure workspace is free of deprecated and unused files
- *(foundation)* Clean up code formatting and remove unused mutability

<!-- generated by git-cliff -->
