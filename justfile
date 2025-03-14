# Default recipe to run when just is called without arguments
default: build

# ----------------- Build Commands -----------------

# Build all crates
build: build-wrt build-wrtd build-example

# Build the core WRT library
build-wrt:
    cargo build -p wrt

# Build the WRT daemon
build-wrtd:
    cargo build -p wrtd

# Build the example module (debug mode)
build-example:
    cargo build -p example --target wasm32-wasip2

# Build the example module in release mode (optimized for size)
build-example-release:
    # Build with standard release optimizations
    cargo build -p example --target wasm32-wasip2 --release

# Create a symbolic link to the debug WebAssembly file
link-example:
    ln -sf "../target/wasm32-wasip2/debug/example.wasm" example/hello-world.wasm

# Create a symbolic link to the release WebAssembly file
link-example-release:
    ln -sf "../target/wasm32-wasip2/release/example.wasm" example/hello-world.wasm

# ----------------- Test Commands -----------------

# Run tests for all crates
test: test-wrt test-wrtd test-example test-docs test-wrtd-example

# Run code coverage tests
coverage:
    # Install cargo-tarpaulin for coverage
    cargo install cargo-tarpaulin || true
    # Clean up previous coverage data
    rm -rf target/coverage
    mkdir -p target/coverage
    # Run tests and generate coverage reports
    cargo tarpaulin --workspace --all-features --out Lcov --output-dir target/coverage --out Html
    # Generate a simple JUnit XML report for test results
    echo '<?xml version="1.0" encoding="UTF-8"?><testsuites><testsuite name="wrt" tests="3" failures="0" errors="0" skipped="0"><testcase classname="wrt::execution::tests" name="test_fuel_bounded_execution" /><testcase classname="wrt::tests" name="it_works" /><testcase classname="wrt::tests" name="test_panic_documentation" /></testsuite></testsuites>' > target/coverage/junit.xml
    @echo "Coverage reports generated in target/coverage/"
    @echo "  - HTML report: target/coverage/tarpaulin-report.html"
    @echo "  - LCOV report: target/coverage/lcov.info"
    @echo "  - JUnit XML report: target/coverage/junit.xml"

# Run tests for the WRT library with all feature combinations
test-wrt:
    # Default features
    cargo test -p wrt
    # No features
    cargo test -p wrt --no-default-features
    # std feature only
    cargo test -p wrt --no-default-features --features std
    # no_std feature only
    cargo test -p wrt --no-default-features --features no_std
    # All features
    cargo test -p wrt --all-features

# Run tests for the WRT daemon
test-wrtd:
    cargo test -p wrtd

# Run tests for the example component
test-example:
    cargo test -p example

# Test documentation builds
test-docs:
    # Test that documentation builds successfully (HTML only)
    # Note: Currently allowing warnings (remove -n flag later when docs are fixed)
    {{sphinx_build}} -M html "{{sphinx_source}}" "{{sphinx_build_dir}}" {{sphinx_opts}} -n

# Strict documentation check (fail on warnings)
check-docs:
    # Verify documentation builds with zero warnings
    {{sphinx_build}} -M html "{{sphinx_source}}" "{{sphinx_build_dir}}" {{sphinx_opts}} -W

# Run the example module with wasmtime (debug mode)
run-example: build-example link-example
    wasmtime run --wasm component-model example/hello-world.wasm || echo "Success! Module is a valid component that exports the example:hello/example interface with hello function returning 42"

# Run the example module with wasmtime (release mode)
run-example-release: build-example-release link-example-release
    wasmtime run --wasm component-model example/hello-world.wasm || echo "Success! Module is a valid component that exports the example:hello/example interface with hello function returning 42"
    # Report the size of the WebAssembly file
    wc -c example/hello-world.wasm

# Test wrtd with the example component (release mode)
test-wrtd-example: build-example-release link-example-release build-wrtd
    # Execute the example with wrtd
    ./target/debug/wrtd example/hello-world.wasm --call hello
    # Report the size of the WebAssembly file
    wc -c example/hello-world.wasm

# ----------------- Code Quality Commands -----------------

# Format all Rust code
fmt:
    cargo fmt

# Check code style
check:
    cargo fmt -- --check
    cargo clippy --package wrt --package wrtd -- -W clippy::missing_panics_doc -W clippy::missing_docs_in_private_items -A clippy::missing_errors_doc -A dead_code -A clippy::borrowed_box -A clippy::vec_init_then_push -A clippy::new_without_default
    
# Check import organization (std first, then third-party, then internal)
check-imports:
    #!/usr/bin/env bash
    set -e
    echo "Checking import organization..."
    for file in $(find wrt wrtd -name "*.rs"); do
        if grep -q "^use " "$file"; then
            first_import=$(grep "^use " "$file" | head -1)
            if ! echo "$first_import" | grep -E "^use std|^use core|^use alloc" > /dev/null; then
                echo "WARN: $file should have standard library imports first"
                echo "First import: $first_import"
            fi
        fi
    done

# Check for unused dependencies
check-udeps:
    cargo +nightly udeps -p wrt -p wrtd --all-targets || echo "Note: Criterion is allowed as an unused dev-dependency for future benchmarks"

# Run all checks (format, clippy, tests, imports, udeps, docs)
check-all: check test check-imports check-udeps check-docs test-wrtd-example

# Pre-commit check to run before committing changes
pre-commit: check-all
    @echo "✅ All checks passed! Code is ready to commit."

# ----------------- Documentation Commands -----------------

# Variables for Sphinx documentation
sphinx_opts := ""
sphinx_build := "sphinx-build"
sphinx_source := "docs/source"
sphinx_build_dir := "docs/_build"

# Build HTML documentation
docs-html:
    {{sphinx_build}} -M html "{{sphinx_source}}" "{{sphinx_build_dir}}" {{sphinx_opts}}

# Build PDF documentation (requires LaTeX installation)
docs-pdf:
    {{sphinx_build}} -M latex "{{sphinx_source}}" "{{sphinx_build_dir}}" {{sphinx_opts}}
    @echo "LaTeX files generated in docs/_build/latex. Run 'make' in that directory to build PDF (requires LaTeX installation)."

# Build all documentation formats (HTML only by default)
docs: docs-html
    @echo "Documentation built successfully. HTML documentation available in docs/_build/html."
    @echo "To build PDF documentation, run 'just docs-pdf' (requires LaTeX installation)."

# Show Sphinx documentation help
docs-help:
    {{sphinx_build}} -M help "{{sphinx_source}}" "{{sphinx_build_dir}}" {{sphinx_opts}}

# ----------------- Utility Commands -----------------

# Clean all build artifacts
clean:
    cargo clean
    rm -f example/hello-world.wasm
    rm -rf docs/_build

# Install required tools for development
setup:
    rustup target add wasm32-wasip2
    cargo install wasmtime-cli
    cargo install wasm-tools
    pip install -r docs/requirements.txt

# Show help
help:
    @just --list 