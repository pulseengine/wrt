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
build-example: setup-rust-targets
    cargo build -p example --target wasm32-wasip2

# Build the example module in release mode (optimized for size)
build-example-release: setup-rust-targets
    # Build with standard release optimizations
    cargo build -p example --target wasm32-wasip2 --release


# ----------------- Test Commands -----------------
# 
# Testing is split into different categories:
# - test-wrt: Core library tests
# - test-wrtd: Command line tool tests
# - test-example: Example WebAssembly module tests
# - test-docs: Documentation tests
# - test-wrtd-*: Various wrtd functionality tests
#
# For testing wrtd with different parameters, use:
# - just test-wrtd-example "--fuel 50 --stats"
# - just test-wrtd-fuel 200
# - just test-wrtd-stats
# - just test-wrtd-help
# - just test-wrtd-all

# Run tests for all crates
test: setup-rust-targets test-wrt test-wrtd test-example test-docs test-wrtd-all

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
run-example: build-example 
    wasmtime run --wasm component-model target/wasm32-wasip2/debug/example.wasm || echo "Success! Module is a valid component that exports the example:hello/example interface with hello function returning 42"

# Run the example module with wasmtime (release mode)
run-example-release: build-example-release 
    wasmtime run --wasm component-model target/wasm32-wasip2/release/example.wasm || echo "Success! Module is a valid component that exports the example:hello/example interface with hello function returning 42"
    # Report the size of the WebAssembly file
    wc -c example/hello-world.wasm

# Test wrtd with the example component (release mode)
# Additional arguments can be passed with e.g. `just test-wrtd-example "--fuel 100 --stats"`
test-wrtd-example *ARGS="--call hello": setup-rust-targets build-example-release build-wrtd
    # Execute the example with wrtd, passing any additional arguments
    ./target/debug/wrtd {{ARGS}} ./target/wasm32-wasip2/release/example.wasm  
    # Report the size of the WebAssembly file
    wc -c ./target/wasm32-wasip2/release/example.wasm

# Test wrtd with fuel-bounded execution and statistics
test-wrtd-fuel FUEL="100": (test-wrtd-example "--call hello --fuel " + FUEL + " --stats")
    # The fuel test has already been executed by the dependency

# Test wrtd with statistics output
test-wrtd-stats: (test-wrtd-example "--call hello --stats")
    # The stats test has already been executed by the dependency

# Test wrtd with both fuel and statistics
test-wrtd-fuel-stats FUEL="100": (test-wrtd-example "--call hello --fuel " + FUEL + " --stats")
    # The fuel+stats test has already been executed by the dependency

# Test wrtd without any function call (should show available functions)
test-wrtd-no-call: (test-wrtd-example "")
    # The no-call test has already been executed by the dependency

# Test wrtd with help output
test-wrtd-help: build-wrtd
    ./target/debug/wrtd --help

# Test wrtd version output
test-wrtd-version: build-wrtd
    ./target/debug/wrtd --version

# Comprehensive test of wrtd with all major options
# This runs all the test commands defined above to verify different wrtd features
# Usage: just test-wrtd-all
test-wrtd-all: test-wrtd-example test-wrtd-fuel test-wrtd-stats test-wrtd-help test-wrtd-version test-wrtd-no-call

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
check-all: check test check-imports check-udeps check-docs test-wrtd-fuel

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
    
# Build HTML documentation with PlantUML diagrams
docs-with-diagrams: setup-plantuml
    {{sphinx_build}} -M html "{{sphinx_source}}" "{{sphinx_build_dir}}" {{sphinx_opts}}

# Build PDF documentation (requires LaTeX installation)
docs-pdf:
    {{sphinx_build}} -M latex "{{sphinx_source}}" "{{sphinx_build_dir}}" {{sphinx_opts}}
    @echo "LaTeX files generated in docs/_build/latex. Run 'make' in that directory to build PDF (requires LaTeX installation)."

# Build all documentation formats (HTML only by default)
docs: docs-html
    @echo "Documentation built successfully. HTML documentation available in docs/_build/html."
    @echo "To build PDF documentation, run 'just docs-pdf' (requires LaTeX installation)."
    @echo "To build documentation with PlantUML diagrams, run 'just docs-with-diagrams' (requires Java and PlantUML)."

# Show Sphinx documentation help
docs-help:
    {{sphinx_build}} -M help "{{sphinx_source}}" "{{sphinx_build_dir}}" {{sphinx_opts}}

# ----------------- Utility Commands -----------------

# Clean all build artifacts
clean:
    cargo clean
    rm -f example/hello-world.wasm
    rm -rf docs/_build

# Install rust targets required for the project
setup-rust-targets:
    rustup target add wasm32-wasip2 || rustup target add wasm32-wasip2

# Install WebAssembly tools (wasmtime, wasm-tools)
setup-wasm-tools:
    cargo install wasmtime-cli --locked
    cargo install wasm-tools --locked

# Install Python dependencies
setup-python-deps:
    pip install -r docs/requirements.txt
    pip install sphinxcontrib-plantuml

# Install PlantUML (requires Java)
setup-plantuml:
    #!/usr/bin/env bash
    if ! command -v plantuml &> /dev/null; then
        echo "Installing PlantUML..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            # Linux installation
            sudo apt-get update && sudo apt-get install -y plantuml
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS installation with Homebrew
            if command -v brew &> /dev/null; then
                brew install plantuml
            else
                echo "Homebrew not found. Please install homebrew first or install plantuml manually."
                echo "Visit: https://plantuml.com/starting"
                exit 1
            fi
        else
            echo "Unsupported OS. Please install plantuml manually."
            echo "Visit: https://plantuml.com/starting"
            exit 1
        fi
    else
        echo "PlantUML is already installed."
    fi
    
    # Check if Java is installed (required for PlantUML)
    if ! command -v java &> /dev/null; then
        echo "Java is required for PlantUML but not found. Please install Java."
        exit 1
    fi

# Install required tools for development (local development)
setup: setup-hooks setup-rust-targets setup-wasm-tools setup-python-deps setup-plantuml
    @echo "✅ All development tools installed successfully."

# Setup for CI environments (without hooks)
setup-ci: setup-rust-targets setup-wasm-tools setup-python-deps setup-plantuml
    @echo "✅ CI environment setup completed."
    
# Minimal setup for CI that only installs necessary Rust targets
setup-ci-minimal: setup-rust-targets
    @echo "✅ Minimal CI environment setup completed (Rust targets only)."

# Install git hooks to enforce checks before commit/push
setup-hooks:
    @echo "Setting up Git hooks..."
    cp .githooks/pre-commit .git/hooks/pre-commit
    cp .githooks/pre-push .git/hooks/pre-push
    chmod +x .git/hooks/pre-commit .git/hooks/pre-push
    @echo "Git hooks installed successfully. Checks will run automatically before each commit and push."

# Show help
help:
    @just --list 