//! Test no_std compatibility for wrt-component
//!
//! This file validates that the wrt-component crate works correctly in no_std
//! environments.

// For testing in a no_std environment
#![cfg_attr(not(feature = "std"), no_std)]

// External crate imports
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Tests that run in all environments (std, no_std+alloc, pure no_std)
#[cfg(test)]
mod common_tests {
    // Use the appropriate imports based on environment
    // Import from wrt-foundation that is available in all environments
    // Import from wrt-component's no_alloc module (available in all environments)
    use wrt_component::no_alloc::{
        validate_component_no_alloc, validate_component_with_level, ComponentSectionId,
        MinimalComponent, ValidationLevel, COMPONENT_MAGIC,
    };
    use wrt_foundation::verification::VerificationLevel;

    // Constants for testing
    // Minimal valid WebAssembly Component - just magic number and version
    const MINIMAL_COMPONENT: [u8; 8] = [0x00, 0x61, 0x73, 0x6D, 0x0A, 0x00, 0x01, 0x00];

    #[test]
    fn test_section_id_conversion() {
        // Test conversion from u8 to ComponentSectionId
        assert_eq!(ComponentSectionId::from(0), ComponentSectionId::Custom);
        assert_eq!(ComponentSectionId::from(1), ComponentSectionId::ComponentType);
        assert_eq!(ComponentSectionId::from(2), ComponentSectionId::CoreModule);
        assert_eq!(ComponentSectionId::from(3), ComponentSectionId::Instance);
        assert_eq!(ComponentSectionId::from(4), ComponentSectionId::Component);
        assert_eq!(ComponentSectionId::from(255), ComponentSectionId::Unknown);
    }

    #[test]
    fn test_validation_levels() {
        // Test basic validation of a minimal component
        let basic_result =
            validate_component_with_level(&MINIMAL_COMPONENT, ValidationLevel::Basic);
        assert!(basic_result.is_ok());

        // Test standard validation of a minimal component
        let standard_result =
            validate_component_with_level(&MINIMAL_COMPONENT, ValidationLevel::Standard);
        assert!(standard_result.is_ok());

        // Test full validation of a minimal component
        let full_result = validate_component_with_level(&MINIMAL_COMPONENT, ValidationLevel::Full);
        assert!(full_result.is_ok());
    }

    #[test]
    fn test_minimal_component() {
        // Create a minimal component with standard verification level
        let component = MinimalComponent::new(&MINIMAL_COMPONENT, VerificationLevel::Standard);
        assert!(component.is_ok());

        // Check properties of the minimal component
        let component = component.unwrap();
        assert_eq!(component.size(), 8);
        assert_eq!(component.export_count(), 0);
        assert_eq!(component.import_count(), 0);
        assert_eq!(component.module_count(), 0);
        assert!(!component.has_start());
    }

    #[test]
    fn test_component_validation() {
        // Test validation of a minimal component
        let result = validate_component_no_alloc(&MINIMAL_COMPONENT);
        assert!(result.is_ok());

        // Invalid component with incorrect magic number
        let invalid_component = [0x01, 0x61, 0x73, 0x6D, 0x0A, 0x00, 0x01, 0x00];
        let result = validate_component_no_alloc(&invalid_component);
        assert!(result.is_err());

        // Component that's too small
        let too_small = [0x00, 0x61];
        let result = validate_component_no_alloc(&too_small);
        assert!(result.is_err());
    }
}

// Tests for features requiring alloc (runs in std or no_std+alloc)
#[cfg(test)]
#[cfg(any(feature = "std", feature = "alloc"))]
mod alloc_tests {
    // Import necessary types for no_std environment
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
    #[cfg(feature = "std")]
    use std::{boxed::Box, string::String, vec, vec::Vec};

    // Import from wrt-component
    use wrt_component::{
        export::Export,
        export_map::{ExportMap, SafeExportMap},
        import::Import,
        import_map::{ImportMap, SafeImportMap},
        resources::{
            buffer_pool::BufferPool, resource_strategy::ResourceStrategy, ResourceManager,
            ResourceOperation,
        },
    };
    // Import from wrt-foundation
    use wrt_foundation::{
        component_value::{ComponentValue, ValType},
        resource::{ResourceOperation as FormatResourceOperation, ResourceType},
        safe_memory::{SafeSlice, SafeStack},
        values::Value,
        verification::VerificationLevel,
    };

    #[test]
    fn test_import_map() {
        // Create an import map
        let mut import_map = ImportMap::new();

        // Add imports
        let import1 = Import::new("module1".to_string(), "func1".to_string());
        let import2 = Import::new("module2".to_string(), "func2".to_string());

        import_map.insert("import1".to_string(), import1);
        import_map.insert("import2".to_string(), import2);

        // Verify imports
        assert_eq!(import_map.len(), 2);
        assert!(import_map.contains_key("import1"));
        assert!(import_map.contains_key("import2"));

        // Get import
        let retrieved = import_map.get("import1").unwrap();
        assert_eq!(retrieved.module(), "module1");
        assert_eq!(retrieved.name(), "func1");
    }

    #[test]
    fn test_safe_import_map() {
        // Create a safe import map
        let mut import_map = SafeImportMap::new();

        // Add imports
        let import1 = Import::new("module1".to_string(), "func1".to_string());
        let import2 = Import::new("module2".to_string(), "func2".to_string());

        import_map.insert("import1".to_string(), import1);
        import_map.insert("import2".to_string(), import2);

        // Verify imports
        assert_eq!(import_map.len(), 2);
        assert!(import_map.contains_key("import1"));
        assert!(import_map.contains_key("import2"));

        // Get import
        let retrieved = import_map.get("import1").unwrap();
        assert_eq!(retrieved.module(), "module1");
        assert_eq!(retrieved.name(), "func1");
    }

    #[test]
    fn test_export_map() {
        // Create an export map
        let mut export_map = ExportMap::new();

        // Add exports
        let export1 = Export::new("func1".to_string());
        let export2 = Export::new("func2".to_string());

        export_map.insert("export1".to_string(), export1);
        export_map.insert("export2".to_string(), export2);

        // Verify exports
        assert_eq!(export_map.len(), 2);
        assert!(export_map.contains_key("export1"));
        assert!(export_map.contains_key("export2"));

        // Get export
        let retrieved = export_map.get("export1").unwrap();
        assert_eq!(retrieved.name(), "func1");
    }

    #[test]
    fn test_safe_export_map() {
        // Create a safe export map
        let mut export_map = SafeExportMap::new();

        // Add exports
        let export1 = Export::new("func1".to_string());
        let export2 = Export::new("func2".to_string());

        export_map.insert("export1".to_string(), export1);
        export_map.insert("export2".to_string(), export2);

        // Verify exports
        assert_eq!(export_map.len(), 2);
        assert!(export_map.contains_key("export1"));
        assert!(export_map.contains_key("export2"));

        // Get export
        let retrieved = export_map.get("export1").unwrap();
        assert_eq!(retrieved.name(), "func1");
    }

    #[test]
    fn test_resource_operations() {
        // Test resource operations
        let format_resource_op = FormatResourceOperation::New(ResourceType::new(0));

        // Convert to runtime resource operation
        let runtime_resource_op = ResourceOperation::from_format_operation(&format_resource_op);

        match runtime_resource_op {
            ResourceOperation::New(resource_type) => {
                assert_eq!(resource_type.get_id(), 0);
            }
            _ => panic!("Expected New operation"),
        }

        // Test other operations
        let drop_op = FormatResourceOperation::Drop(ResourceType::new(0));
        let runtime_drop_op = ResourceOperation::from_format_operation(&drop_op);

        match runtime_drop_op {
            ResourceOperation::Drop(resource_type) => {
                assert_eq!(resource_type.get_id(), 0);
            }
            _ => panic!("Expected Drop operation"),
        }
    }

    #[test]
    fn test_buffer_pool() {
        // Create a buffer pool with verification level
        let mut pool = BufferPool::with_verification_level(VerificationLevel::Standard);

        // Allocate a buffer
        let buffer = pool.allocate(10).unwrap();

        // Verify buffer properties
        assert_eq!(buffer.len(), 10);

        // Write to the buffer
        let mut slice = SafeSlice::new(buffer);
        for i in 0..10 {
            slice.write_u8(i as usize, i as u8).unwrap();
        }

        // Read from the buffer
        for i in 0..10 {
            assert_eq!(slice.read_u8(i as usize).unwrap(), i as u8);
        }
    }
}

// Tests specific to std environment
#[cfg(test)]
#[cfg(feature = "std")]
mod std_tests {
    use std::{boxed::Box, string::String};

    #[cfg(feature = "component-model-all")]
    use wrt_component::component::Component;

    // Add std-specific tests here if needed
    #[test]
    fn test_std_feature_flag() {
        // This test only runs in std mode to verify the feature flag is working
        assert!(true);
    }
}

// Tests specific to no_std with alloc environment
#[cfg(test)]
#[cfg(all(not(feature = "std"), feature = "alloc"))]
mod no_std_alloc_tests {
    use alloc::{boxed::Box, string::String};

    #[cfg(feature = "component-model-all")]
    use wrt_component::component_no_std::Component;

    // Add no_std+alloc specific tests here if needed
    #[test]
    fn test_no_std_alloc_feature_flag() {
        // This test only runs in no_std+alloc mode to verify the feature flag is
        // working
        assert!(true);
    }
}

// Tests specific to pure no_std (no alloc) environment
#[cfg(test)]
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
mod pure_no_std_tests {
    use wrt_component::no_alloc::{
        validate_component_with_level, ComponentHeader, ComponentSectionId, ComponentSectionInfo,
        MinimalComponent, ValidationLevel,
    };
    use wrt_foundation::verification::VerificationLevel;

    // Add pure no_std specific tests here
    #[test]
    fn test_pure_no_std_feature_flag() {
        // This test only runs in pure no_std mode to verify the feature flag is working
        assert!(true);
    }

    #[test]
    fn test_component_header_defaults() {
        // Create a default ComponentHeader
        let header = ComponentHeader::default();

        // Check properties
        assert_eq!(header.size, 0);
        assert_eq!(header.module_count, 0);
        assert_eq!(header.export_count, 0);
        assert_eq!(header.import_count, 0);
        assert!(!header.has_start);

        // All sections should be None
        for section in &header.sections {
            assert!(section.is_none());
        }
    }
}
