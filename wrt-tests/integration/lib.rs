//! WRT Integration Tests
//!
//! Main integration test library that coordinates all test suites.

use wrt_test_registry::prelude::*;

// Include all test modules
pub mod component_model;
pub mod runtime;
pub mod platform;
pub mod no_std;
pub mod security;
pub mod parser;

/// Run all integration tests
pub fn run_all_integration_tests() -> TestResult {
    let mut runner = TestRunner::new("WRT Integration Tests");
    
    // Add all test suites
    runner.add_test_suite("Component Model", component_model::run_tests)?;
    runner.add_test_suite("Runtime", runtime::run_tests)?;
    runner.add_test_suite("Platform", platform::run_tests)?;
    runner.add_test_suite("No-std", no_std::run_tests)?;
    runner.add_test_suite("Security", security::run_tests)?;
    runner.add_test_suite("Parser", parser::run_tests)?;
    
    runner.run_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn all_integration_tests() {
        let result = run_all_integration_tests();
        assert!(result.is_success(), "Integration tests failed: {:?}", result);
    }
    
    #[test]
    fn component_model_suite() {
        let result = component_model::run_tests();
        assert!(result.is_success(), "Component model tests failed: {:?}", result);
    }
    
    #[test]
    fn runtime_suite() {
        let result = runtime::run_tests();
        assert!(result.is_success(), "Runtime tests failed: {:?}", result);
    }
    
    #[test]
    fn platform_suite() {
        let result = platform::run_tests();
        assert!(result.is_success(), "Platform tests failed: {:?}", result);
    }
    
    #[test]
    fn no_std_suite() {
        let result = no_std::run_tests();
        assert!(result.is_success(), "No-std tests failed: {:?}", result);
    }
    
    #[test]
    fn security_suite() {
        let result = security::run_tests();
        assert!(result.is_success(), "Security tests failed: {:?}", result);
    }
    
    #[test]
    fn parser_suite() {
        let result = parser::run_tests();
        assert!(result.is_success(), "Parser tests failed: {:?}", result);
    }
}