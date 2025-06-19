# WRT Documentation Index

This document provides a comprehensive index of all documentation in the WRT project.

## Core Documentation

### Project Overview
- [README.md](README.md) - Main project overview and quick start
- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute to the project
- [CLAUDE.md](CLAUDE.md) - Guidelines for Claude Code assistance

### Main Documentation Site
- [docs/](docs/) - Comprehensive documentation built with Sphinx
  - Architecture guides
  - API documentation
  - Safety and qualification documentation
  - Development guides

## Crate-Specific Documentation

### Core Crates

#### wrt (Main Library)
- [wrt/README.md](wrt/README.md) - Main library overview
- [wrt/tests/README.md](wrt/tests/README.md) - Test suite documentation
- [wrt/tests/PROPOSAL_TESTING.md](wrt/tests/PROPOSAL_TESTING.md) - WebAssembly proposal testing

#### wrt-runtime (Execution Engine)
- [wrt-runtime/README.md](wrt-runtime/README.md) - Runtime architecture and features

#### wrt-component (Component Model)
- [wrt-component/README.md](wrt-component/README.md) - Component Model implementation
- [wrt-component/COMPONENT_STATUS.md](wrt-component/COMPONENT_STATUS.md) - Implementation status and features
- [wrt-component/README_ASYNC_FEATURES.md](wrt-component/README_ASYNC_FEATURES.md) - Async features guide

#### wrt-foundation (Core Types)
- [wrt-foundation/README.md](wrt-foundation/README.md) - Foundation types and safe memory

### Specialized Crates

#### Decoder and Format
- [wrt-decoder/README.md](wrt-decoder/README.md) - Binary parsing and decoding
- [wrt-format/README.md](wrt-format/README.md) - Format specifications

#### Platform and System
- [wrt-platform/README.md](wrt-platform/README.md) - Platform abstraction layer
- [wrt-platform/README-SAFETY.md](wrt-platform/README-SAFETY.md) - Safety features documentation
- [wrt-sync/README.md](wrt-sync/README.md) - Synchronization primitives

#### Instructions and Execution
- [wrt-instructions/README.md](wrt-instructions/README.md) - Instruction implementations
- [wrt-intercept/README.md](wrt-intercept/README.md) - Function interception

#### Host Integration
- [wrt-host/README.md](wrt-host/README.md) - Host interface
- [wrt-logging/README.md](wrt-logging/README.md) - Logging infrastructure

#### Error Handling and Math
- [wrt-error/README.md](wrt-error/README.md) - Error handling system
- [wrt-math/README.md](wrt-math/README.md) - Mathematical operations

#### Utilities and Tools
- [wrt-helper/README.md](wrt-helper/README.md) - Helper utilities
- [wrtd/README.md](wrtd/README.md) - WRT daemon/CLI tool
- [cargo-wrt/README.md](cargo-wrt/README.md) - Unified build tool (cargo-wrt)
- [wrt-build-core/README.md](wrt-build-core/README.md) - Core build system library

### Testing and Quality Assurance

#### Test Infrastructure
- [wrt-tests/README.md](wrt-tests/README.md) - Integration test suite
- [wrt-tests/fixtures/README.md](wrt-tests/fixtures/README.md) - Test fixtures
- [wrt-test-registry/README.md](wrt-test-registry/README.md) - Test registry system
- [wrt-verification-tool/README.md](wrt-verification-tool/README.md) - Verification tools

#### Debugging and Verification
- [wrt-debug/README.md](wrt-debug/README.md) - Debug support
- [wrt-debug/DEBUG_ARCHITECTURE.md](wrt-debug/DEBUG_ARCHITECTURE.md) - Debug architecture
- [wrt-debug/DEBUG_FEATURES.md](wrt-debug/DEBUG_FEATURES.md) - Debug features

#### Fuzzing and Property Testing
- [wrt-component/fuzz/README.md](wrt-component/fuzz/README.md) - Component fuzzing
- [wrt-foundation/wrt-tests/fuzz/README.md](wrt-foundation/wrt-tests/fuzz/README.md) - Foundation fuzzing

### Examples and Templates

#### Example Code
- [example/README.md](example/README.md) - Example implementations
- [wrt-component/examples/README.md](wrt-component/examples/README.md) - Component examples
- [wrt-platform/examples/README.md](wrt-platform/examples/README.md) - Platform examples

#### Templates
- [templates/README.md](templates/README.md) - Project templates

### External Dependencies
- [external/testsuite/README.md](external/testsuite/README.md) - WebAssembly test suite
- [external/testsuite/Contributing.md](external/testsuite/Contributing.md) - Test suite contributing

## Documentation Standards

### Markdown Files
- Each crate should have a clear README.md explaining its purpose and usage
- Use consistent formatting and structure across READMEs
- Include examples where appropriate
- Link to the main documentation site for comprehensive guides

### Main Documentation Site (docs/)
- Use reStructuredText (.rst) format for Sphinx documentation
- Comprehensive architecture and API documentation
- Safety and qualification documentation for critical systems
- Development and contribution guides

## Navigation Tips

### For Users
1. Start with the main [README.md](README.md)
2. Check crate-specific READMEs for detailed usage
3. Visit [docs/](docs/) for comprehensive guides

### For Contributors
1. Read [CONTRIBUTING.md](CONTRIBUTING.md)
2. Check [CLAUDE.md](CLAUDE.md) for AI assistance guidelines
3. Review architecture documentation in [docs/](docs/)

### For Quality Assurance
1. Check [wrt-component/COMPONENT_STATUS.md](wrt-component/COMPONENT_STATUS.md) for implementation status
2. Review testing documentation in test-related crates
3. Examine safety documentation in [wrt-platform/README-SAFETY.md](wrt-platform/README-SAFETY.md)

## Maintenance

This index should be updated when:
- New crates are added
- New major documentation files are created
- Documentation structure changes significantly

Last updated: 2025-01-06