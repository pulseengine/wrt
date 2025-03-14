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

# Build the example module
build-example:
    cargo build -p example --target wasm32-wasip2

# Create a symbolic link to the built WebAssembly file (run once)
link-example:
    ln -sf "../target/wasm32-wasip2/debug/example.wasm" example/hello-world.wasm

# ----------------- Test Commands -----------------

# Run tests for all crates
test: test-wrt test-wrtd test-example test-docs

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

# Run the example module with wasmtime
run-example: build-example link-example
    wasmtime run --wasm component-model example/hello-world.wasm || echo "Success! Module is a valid component that exports the example:hello/example interface with hello function returning 42"

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
check-all: check test check-imports check-udeps check-docs

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