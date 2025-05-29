//! Runtime Integration Tests
//!
//! This module contains integration tests for the WRT runtime system.

use wrt_test_registry::prelude::*;

mod control_instructions_tests;
mod memory_management_tests;
mod execution_engine_tests;
mod cfi_security_tests;

/// Run all runtime integration tests
pub fn run_tests() -> TestResult {
    let mut runner = TestRunner::new("Runtime Integration");
    
    runner.add_test_suite("Control Instructions", control_instructions_tests::run_tests)?;
    runner.add_test_suite("Memory Management", memory_management_tests::run_tests)?;
    runner.add_test_suite("Execution Engine", execution_engine_tests::run_tests)?;
    runner.add_test_suite("CFI Security", cfi_security_tests::run_tests)?;
    
    runner.run_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn runtime_integration() {
        let result = run_tests();
        assert!(result.is_success(), "Runtime integration tests failed: {:?}", result);
    }
}