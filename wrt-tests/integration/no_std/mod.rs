//! No-std Integration Tests
//!
//! This module consolidates all no_std compatibility tests across the WRT project.

use wrt_test_registry::prelude::*;

mod no_std_compatibility_tests;
mod bounded_collections_tests;
mod memory_safety_tests;
mod alloc_compatibility_tests;

/// Run all no_std integration tests
pub fn run_tests() -> TestResult {
    let mut runner = TestRunner::new("No-std Integration");
    
    runner.add_test_suite("No-std Compatibility", no_std_compatibility_tests::run_tests)?;
    runner.add_test_suite("Bounded Collections", bounded_collections_tests::run_tests)?;
    runner.add_test_suite("Memory Safety", memory_safety_tests::run_tests)?;
    
    #[cfg(feature = "alloc")]
    runner.add_test_suite("Alloc Compatibility", alloc_compatibility_tests::run_tests)?;
    
    runner.run_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn no_std_integration() {
        let result = run_tests();
        assert!(result.is_success(), "No-std integration tests failed: {:?}", result);
    }
}