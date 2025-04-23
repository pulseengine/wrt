# Migration Plan: Justfile to Bazel+xtasks

This document outlines the plan for migrating our build system from Just to a combination of Bazel and xtasks.

## Overall Strategy

We'll follow a gradual approach:

1. Both systems (Justfile and Bazel+xtasks) will coexist during the transition
2. We'll migrate commands one by one, starting with the simpler ones
3. The Justfile will remain as a fallback until all commands are migrated

## Why Bazel?

Bazel offers:
- Highly incremental and reproducible builds
- Fast and parallel execution
- Advanced dependency management
- Strong caching
- Great multi-language support
- Better scalability for larger projects

## Why xtasks?

xtasks in Rust provide:
- Cargo-integrated tooling without external dependencies
- First-class Rust experience for build scripts and tooling
- Works well alongside Bazel for Rust-specific operations
- Ability to build helpers for bazel management

## Migration Phases

### Phase 1: Setup and Infrastructure (1-2 weeks)

- [x] Create basic Bazel configuration (WORKSPACE)
- [x] Setup xtask integration with Bazel
- [x] Create migration tooling
- [ ] Configure CI to use both systems

### Phase 2: Core Build Rules (2-3 weeks)

- [x] Migrate basic Rust build commands
- [x] Implement test rules
- [ ] Setup WASM compile targets
- [ ] Configure component builds

### Phase 3: Testing and Documentation (2-3 weeks)

- [x] Migrate test execution
- [x] Implement documentation builds
- [ ] Migrate coverage analysis

### Phase 4: Developer Workflow (1-2 weeks)

- [x] Create developer shortcuts
- [x] Migrate formatting and linting
- [ ] Update contribution guides

### Phase 5: Specialized Tools (2-3 weeks)

- [ ] Migrate WebAssembly tools
- [ ] Implement Zephyr SDK support
- [ ] Migrate remaining specialized tools

### Phase 6: Cleanup (1 week)

- [ ] Remove redundant commands from Justfile
- [ ] Deprecate Just dependency
- [ ] Update all documentation

## Using Bazel

To use the migrated Bazel targets, you can use these commands:

```bash
# Build all targets
bazel build //...

# Run tests
bazel test //...

# Format code
bazel run //:fmt

# Run linting checks
bazel run //:check

# Build documentation
bazel run //:docs

# Run everything in sequence
bazel run //:all
```

## Using the Migration Tool

You can use our custom tool to help with migration:

```bash
# Generate a Bazel BUILD file for a cargo package
cargo xtask bazel generate crates/my-package

# Migrate a just command to Bazel
cargo xtask bazel migrate my-command

# Run a specific Bazel build
cargo xtask bazel build //my-package

# Run Bazel tests
cargo xtask bazel test //my-package:my-package_test
```

## Command Mapping

Below is a mapping of Just commands to their Bazel equivalents:

| Just Command | Bazel Equivalent | Status |
|--------------|------------------------|--------|
| `just build` | `bazel build //...` | ✅ Migrated |
| `just build-wrt` | `bazel run //:build-wrt` | ✅ Migrated |
| `just test` | `bazel test //...` | ✅ Migrated |
| `just fmt` | `bazel run //:fmt` | ✅ Migrated |
| `just check` | `bazel run //:check` | ✅ Migrated |
| `just docs` | `bazel run //:docs` | ✅ Migrated |
| `just check-all` | `bazel run //:all` | ✅ Migrated |
| `just build-example` | `bazel run //:build-example` | Pending |
| `just build-adapter` | `bazel run //:build-adapter` | Pending |
| `just test-wast` | `bazel run //:test-wast` | Pending |
| `just setup-zephyr-sdk` | `bazel run //:setup-zephyr-sdk` | Pending |

## Status Tracking

We'll continue to track migration progress in this document. Please update it as you migrate commands.

## Questions and Help

If you have questions about the migration, please reach out to the build system team. 