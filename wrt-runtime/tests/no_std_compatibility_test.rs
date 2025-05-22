//! Test no_std compatibility for wrt-runtime
//!
//! This file validates that the wrt-runtime crate works correctly in no_std
//! environments.

// For testing in a no_std environment
#![cfg_attr(not(feature = "std"), no_std)]

// External crate imports
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Tests that run in all environments (std, no_std+alloc, pure no_std)
#[cfg(test)]
mod common_tests {
    // Import from wrt-foundation that is available in all environments
    use wrt_foundation::verification::VerificationLevel;

    // Constants for testing
    // Minimal valid WebAssembly Component - just magic number and version
    const MINIMAL_COMPONENT: [u8; 8] = [0x00, 0x61, 0x73, 0x6D, 0x0A, 0x00, 0x01, 0x00];

    #[test]
    fn test_module_page_size() {
        // Test that the PAGE_SIZE constant is correct
        assert_eq!(wrt_runtime::PAGE_SIZE, 65536);
    }
}

// Tests for pure no_std environments
#[cfg(test)]
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
mod pure_no_std_tests {
    // Import necessary types
    use wrt_foundation::verification::VerificationLevel;
    use wrt_runtime::MinimalComponent;

    // Constants for testing
    const MINIMAL_COMPONENT: [u8; 8] = [0x00, 0x61, 0x73, 0x6D, 0x0A, 0x00, 0x01, 0x00];

    #[test]
    fn test_minimal_component() {
        // Create a minimal component
        let component = MinimalComponent::new(VerificationLevel::Basic);

        // Check verification level
        assert_eq!(component.verification_level(), VerificationLevel::Basic);

        // Validate minimal component
        let result = MinimalComponent::validate(&MINIMAL_COMPONENT);
        assert!(result.is_ok());

        // Validate invalid component
        let invalid_component = [0x01, 0x61, 0x73, 0x6D, 0x0A, 0x00, 0x01, 0x00]; // Invalid magic
        let result = MinimalComponent::validate(&invalid_component);
        assert!(result.is_err());
    }
}

// Tests that require alloc (std or no_std+alloc)
#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod alloc_tests {
    // Import necessary types for environments with allocation
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::{boxed::Box, vec::Vec};
    #[cfg(feature = "std")]
    use std::{boxed::Box, vec::Vec};

    use wrt_foundation::verification::VerificationLevel;
    use wrt_runtime::{Module, ModuleBuilder};

    #[test]
    fn test_module_builder() {
        // Create an empty module builder
        let builder = ModuleBuilder::new();

        // Verify builder created successfully
        assert!(builder.is_ok());
    }
}
