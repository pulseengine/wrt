# WebAssembly Runtime Tests

This directory contains various tests for the WebAssembly runtime implementation.

## Test Structure

- `lib.rs`: Main test file that aggregates all test modules
- `simple_spec_tests.rs`: Simple tests for basic WebAssembly functionality
- `simd_tests.rs`: Tests for SIMD instructions implementation
- `wasm_testsuite.rs`: Tests that use the official WebAssembly test suite

## Running WebAssembly Testsuite Tests

To run the tests that use the official WebAssembly test suite:

1. Clone the official WebAssembly testsuite repository:
   ```bash
   git clone https://github.com/WebAssembly/testsuite.git
   ```

2. Set the `WASM_TESTSUITE` environment variable to point to the path of the cloned repository:
   ```bash
   export WASM_TESTSUITE=/path/to/testsuite
   ```

3. Run the tests:
   ```bash
   cargo test
   ```

If the `WASM_TESTSUITE` environment variable is not set, the testsuite tests will be skipped gracefully.

## WebAssembly Testsuite Structure

The official WebAssembly testsuite contains test modules organized by feature:

- `simd/`: Tests for SIMD instructions
- `threads/`: Tests for threading features
- `reference-types/`: Tests for reference types
- And many more...

Our test runner will look for specific files in these directories and run them. If a file is not found, the test will be skipped. 