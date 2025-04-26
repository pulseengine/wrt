# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### 🚀 Features

- *(xtask)* Add automatic panic registry update tool
- *(resources)* Implement resource management system - Add canonical format for binary resources - Add verification functionality - Delete deprecated resource-tests module - Update panic registry documentation

### 💼 Other

- *(deps)* Bump anyhow from 1.0.97 to 1.0.98
- *(deps)* Bump wast from 227.0.1 to 229.0.0
- *(deps)* Bump wat from 1.227.1 to 1.229.0

### 🚜 Refactor

- *(wrt-error)* Improve error handling system
- *(wrt-types)* Update type system implementation

### 📚 Documentation

- Remove outdated panic documentation
- Restructure documentation with new API and development sections

### ⚙️ Miscellaneous Tasks

- Update GitHub workflow and dependencies

## [0.2.0] - 2025-04-24

### 🚀 Features

- Set default feature to std in Cargo.toml
- Update configuration, execution improvements, and testing additions
- Add benchmark tests for engine performance
- Implement Drop instruction and fix ModuleInstance in tests
- Implement TableFill, TableCopy, TableInit, and ElemDrop instructions
- Implement numeric instructions (I32, I64, F32, F64 arithmetic operations)
- Add simplified example for testing I32 operations
- Add Comparison instruction category for tracking comparison operations
- *(simd)* Add SIMD instruction implementations and test infrastructure
- *(test)* Add execution tests for all WebAssembly proposal features
- *(simd)* Implement relaxed SIMD operations and tests
- *(testing)* Implement WRT engine integration for WAST test validation
- Implement i64 comparison operations
- Add basic WAT test infrastructure
- *(exec)* Implement i32 operations and fix global instance handling
- *(exec)* Implement i64 comparison instructions and add test case
- *(instructions)* Implement i64 comparison operations
- *(instructions)* Implement i64 comparison operations in stackless engine, fix unused variables in SIMD module
- Complete i64 comparison implementation with dedicated tests
- Enhance component model with proper imports, exports, and linking
- Implement i64 comparisons and floating-point operations
- Implement proper SIMD instruction handling
- Implement proper SIMD instruction helpers
- *(xtask)* Add lint, test, build, coverage, and symbols commands
- *(xtask)* Enhance coverage and symbols tasks
- Add modular crates for decoder, error handling, format, and sync
- Implement SIMD instruction functions and improve v128 handling
- Update WIT interfaces and add component model examples
- *(decoder)* Implement SIMD binary format updates
- *(wrt-format)* Add GlobalType struct and update usages
- *(wrt-format)* Add no_std support to core format
- *(wrt-format)* Add initial WebAssembly Component Model types
- *(wrt-decoder)* Add WebAssembly Component Model decoding and validation
- *(resources)* Implement new resource management system
- *(component)* Implement component model architecture improvements
- *(modules)* Add new specialized component modules
- *(component)* Add component examples and tests
- Implement new features and update modules
- *(docs)* Add version switcher for documentation

### 🐛 Bug Fixes

- Resolve compilation errors and warnings
- Improve stackless execution error handling for underflows
- Re-enable no_std tests in justfile
- Update imports and formatting for no_std compatibility
- Implement CallIndirect and execute_call methods
- Correct memory region classification for addresses
- *(no_std)* Add accessor methods for Engine instances and vec macro support for no_std
- Repair memory_search_test to use public API
- *(simd)* Fix V128Store instruction to match WebAssembly stack semantics
- *(simd)* Support alternative opcode 0xBA for i32x4.dot_i16x8_s instruction
- Comment out broken test macros to fix compilation errors
- Update test assertions to match actual implementation values
- *(tests)* Adjust test assertions to match actual engine behavior
- *(stackless)* Implement stackless loop execution with proper branching
- Corrected Component Model implementation with proper types
- *(engine)* Resolve borrowing issues in resume implementation
- Implement WASM runtime execution and module parsing
- Resolve clippy errors and improve documentation
- Resolve duplicate handle_log function and consolidate match arms in matches_type
- Implement proper V128 result handling for SIMD address tests
- *(simd)* Add feature flags to log calls and fix unused variable warnings
- Correct rotr operation implementation
- *(engine)* Correct enter_else signature in StacklessFrame
- *(engine)* Refine table fill and stack frame logic
- *(engine)* Use parameter count for frame/block arity calculation
- *(wasm)* Improve WebAssembly memory section specification compliance
- *(decoder)* Update error type in encode function - Replace non-existent EncodeError with RuntimeError to fix compilation issues in wrt-decoder
- *(decoder)* Implement CoreInstanceExport variant in encode_alias_section
- Add missing parameter annotations for unused variables

### 💼 Other

- Update dependencies and simplify benchmarks
- *(deps)* Bump actions/setup-java from 3 to 4
- *(deps)* Bump actions/checkout from 3 to 4
- *(deps)* Bump bincode from 1.3.3 to 2.0.1
- *(deps)* Bump clap from 4.5.32 to 4.5.37

### 🚜 Refactor

- Change module visibility to public
- Adjust Mutex implementation for no_std environments
- Add debug_println macro for conditional logging
- Extend debug_println macro usage throughout codebase
- Organize imports in execution.rs
- Reorganize instruction code into modular structure
- Reorganize and improve test structure with new spec tests
- *(tests)* Rename simd_wast_tests.rs to wast_tests.rs
- *(exec)* Migrate i32 instruction implementations to instruction modules
- *(instructions)* Implement InstructionExecutor for SIMD operations
- *(simd)* Change function visibility from pub to pub(crate)
- Clean up unused code and fix variable warnings
- *(xtask)* Introduce xtask crate for build and utility tasks
- *(tests)* Update WAST tests for StacklessEngine and float comparison
- *(simd)* Reorganize SIMD instruction implementation
- *(engine)* Update stackless engine and related instructions
- *(tests)* Update various tests and remove old WAST runner
- *(examples)* Update example crate and remove old examples
- *(instr)* Adapt table_fill instruction to Table type changes
- *(engine)* Improve label arity and branching logic in StacklessFrame
- *(instr)* Simplify value assignment in table_fill
- *(runtime)* Cleanup and adjust stackless frame implementation
- *(runtime)* Overhaul memory implementation and behavior
- *(runtime)* Update stackless engine and behaviors
- *(runtime)* Update instruction implementations and add SIMD stubs
- *(runtime)* Update execution logic and core library file
- *(wit)* Update logging adapter WIT files
- Improve instruction handling and add new instruction types
- Migrate to stackless execution model and improve error handling
- Improve no_std compatibility for GlobalType implementation
- *(wrt-decoder)* Update instructions module to use prelude
- *(decoder)* Restructure component decoder implementation
- *(core)* Update core modules for resource implementation
- *(wrt-component)* Update component logic and refactor code structures
- *(wrt-error)* Reorganize test structure for better maintainability
- Update and refactor source code in wrt-component and wrt directories
- *(build)* Rename fs commands for consistency
- *(docs)* Remove Python scripts replaced by Rust implementation
- *(types)* Implement AsRef/AsMut traits for BoundedVec
- *(format)* Update binary format implementation
- *(intercept)* Update intercept strategies
- *(runtime)* Update table implementation
- *(qualification)* Update qualification and test utilities

### 📚 Documentation

- Fix RST heading underlining and remove PlantUML theme
- Update documentation for new features and architecture enhancements
- Add missing documentation for instruction variants and stackless methods
- Update NEXT_STEPS.md to reflect Component Model implementation progress
- Improve documentation formatting and reduce duplicated match arms
- Update README and architecture documentation
- Remove Component Model Tools requirement
- *(wrt-format)* Update documentation for no_std support
- Add Component Model improvement suggestions
- *(safety)* Add qualification framework and safety documentation
- *(architecture)* Add architecture diagrams and qualification module
- *(safety)* Update safety requirements and implementation details
- Update documentation and guidelines
- Update documentation requirements and templates
- Update changelog and safety documentation

### 🎨 Styling

- Fix formatting issues

### 🧪 Testing

- Add memory search test
- Comment out test_memory_bounds in tests
- Add comprehensive test cases for components
- Add tests for error creation and display
- Enhance test coverage for global, instruction, and memory modules
- Add and update stack operation tests in execution module
- Add tests for log level parsing and defaults
- Add tests for module creation and import handling
- Add WebAssembly Text format tests for memory operations
- Add WebAssembly spec test downloader framework
- *(simd)* Extend WAST tests to cover all SIMD files
- *(simd)* Add specific test for i32x4.dot_i16x8_s instruction
- *(wasm)* Add conditional proposal tests for std and no_std
- Temporarily disable WAST tests with placeholders
- Fix component binary parsing tests and update progress
- *(wrt)* Update test frame implementations
- *(wrt-format)* Update tests for no_std compatibility
- *(wrt-decoder)* Add tests for Component Model features
- *(safety)* Add fuzz testing and resource verification tests

### ⚙️ Miscellaneous Tasks

- Fix PlanUML integration
- Update dependencies and configuration files
- Update wrtd dependencies to specific versions
- Improve CI workflow with Rust setup and dependency management
- Add cliff.toml for changelog configuration and management
- Improve logging in runtime daemon
- Fix an ci error during publishing
- Changelog generation is missing
- Update project configuration files
- Update dependencies, build scripts, and CI workflow
- Update documentation and build script
- Update dependencies and remove NEXT_STEPS.md
- Update wrt crate dependencies
- *(cleanup)* Remove legacy files and update xtask
- *(config)* Update configuration and workflows
- Remove deprecated scripts and wrt-common modules
- Commit remaining untracked files
- *(config)* Update configuration files and Bazel build scripts
- Commit all untracked files
- Update dependencies and MSRV to 1.86.0
- Update MSRV to 1.86.0 in project files
- Update Cargo.lock with new dependencies
- Upgrade all crates to version 0.2.0

## [0.1.0] - 2025-03-15

### 🚀 Features

- Initialize WebAssembly Runtime project
- Extend wrtd to execute WebAssembly component functions
- Add integration between wrtd and example component
- Optimize example WebAssembly size with release builds
- *(wrtd)* Add tests for fuel-bounded execution
- *(example)* Implement looping with log output
- *(execution)* Add execution statistics
- *(module)* Add WebAssembly binary loading
- *(wrtd)* Add statistics and fuel options to CLI
- *(module)* Implement basic WebAssembly binary parser
- *(logging)* Implement WebAssembly component logging bridge
- *(example)* Enhance WIT interface with logging support
- *(wrt)* Implement Component Model binary format detection
- *(component)* Implement WebAssembly Component Model execution
- *(module)* Implement import, function, and code section parsing
- *(wrtd)* Enhance module inspection and function execution
- *(example)* Update component to use WASI logging interface
- *(execution)* Add WebAssembly string reading support for WASI logging
- *(wrtd)* Improve component model handling and CLI output formatting
- *(module)* Add comprehensive wasm opcode support for component model
- *(memory)* Implement memory growth and proper memory instances
- *(wasm)* Improve WebAssembly Component Model string handling
- *(wasm)* Add stackless WebAssembly execution engine

### 🐛 Bug Fixes

- Cleanup the Readme
- *(tests)* Fix fuel test and suppress unexpected_cfgs warnings
- Resolve clippy warnings throughout codebase
- Prefix unused variable with underscore
- Simplify WebAssembly component loop to prevent infinite execution
- *(execution)* Update execution engine to support imported functions
- *(wrtd)* Export Function and Export types, fix test failures
- Update justfile to use correct component model function syntax

### 💼 Other

- *(deps)* Bump wit-component from 0.12.0 to 0.14.7
- *(deps)* Bump actions/setup-python from 4 to 5
- *(deps)* Bump actions/cache from 3 to 4
- *(deps)* Bump actions/checkout from 3 to 4
- Enhance justfile with improved setup and test commands

### 🚜 Refactor

- Improve build and execution scripts
- *(wrtd)* Improve code structure and remove mock component
- *(module)* Implement minimal component model support without example code
- Clean up imports and remove unnecessary code

### 📚 Documentation

- Update requirements status
- *(conventions)* Add conventional commit format guidelines
- Add software architecture documentation
- Adjust architecture documentation for compatibility
- Integrate PlantUML diagrams into architecture documentation
- Add logging flow diagram and improve cross-platform compatibility

### 🎨 Styling

- Fix code formatting issues in multiple files
- Apply code formatting fixes

### 🧪 Testing

- Fix fuel test and component detection

### ⚙️ Miscellaneous Tasks

- Fix test workflow and remove redundant checks
- Improve code quality, test coverage and workflow reliability
- Add strict documentation check with zero warnings
- Add Dependabot config and fix document warnings
- *(hooks)* Add git hooks for enforcing pre-commit checks
- Improve workflow with rust target setup
- Enhance CI with multi-platform matrix builds
- Improve cross-platform compatibility for unused dependency check
- Fix PlantUML integration in CI documentation build

<!-- generated by git-cliff -->
