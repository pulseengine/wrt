//! CFI Testing Framework - Isolated Component Testing
//! 
//! This crate provides comprehensive testing for CFI components
//! independent of the main WRT build issues.

pub mod cfi_core_tests;
pub mod cfi_hardware_tests;
pub mod cfi_metadata_tests;
pub mod cfi_runtime_tests;
pub mod cfi_integration_tests;
pub mod cfi_security_tests;
pub mod cfi_mocks;

// Re-export core CFI types for testing
pub use cfi_core_tests::*;
pub use cfi_mocks::*;