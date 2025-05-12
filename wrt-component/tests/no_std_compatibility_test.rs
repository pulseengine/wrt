//! Test no_std compatibility for wrt-component
//!
//! This file validates that the wrt-component crate works correctly in no_std
//! environments.

// For testing in a no_std environment
#![cfg_attr(not(feature = "std"), no_std)]

// External crate imports
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(test)]
mod tests {
    // Import necessary types for no_std environment
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
    #[cfg(feature = "std")]
    use std::{boxed::Box, string::String, vec, vec::Vec};

    // Import from wrt-component
    use wrt_component::{
        component::Component,
        export::Export,
        export_map::{ExportMap, SafeExportMap},
        import::Import,
        import_map::{ImportMap, SafeImportMap},
        resources::{
            buffer_pool::BufferPool, resource_strategy::ResourceStrategy, ResourceManager,
            ResourceOperation,
        },
    };
    // Import from wrt-types
    use wrt_types::{
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
