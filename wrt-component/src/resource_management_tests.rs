//! Comprehensive tests for Resource Management System
//!
//! This module provides extensive test coverage for the WebAssembly Component Model
//! resource management functionality, including edge cases, error conditions,
//! and cross-environment compatibility.

#[cfg(test)]
mod tests {
    use super::super::component_instantiation::InstanceId;
    use super::super::resource_management::*;
    use wrt_error::ErrorCategory;

    // ====== RESOURCE HANDLE TESTS ======

    #[test]
    fn test_resource_handle_creation() {
        let handle = ResourceHandle::new(42);
        assert_eq!(handle.value(), 42);
        assert!(handle.is_valid());

        let invalid = INVALID_HANDLE;
        assert!(!invalid.is_valid());
        assert_eq!(invalid.value(), u32::MAX);
    }

    #[test]
    fn test_resource_handle_comparison() {
        let handle1 = ResourceHandle::new(100);
        let handle2 = ResourceHandle::new(100);
        let handle3 = ResourceHandle::new(200);

        assert_eq!(handle1, handle2);
        assert_ne!(handle1, handle3);
        assert_ne!(handle2, handle3);
    }

    #[test]
    fn test_resource_type_id_creation() {
        let type_id = ResourceTypeId::new(123);
        assert_eq!(type_id.value(), 123);

        let type_id2 = ResourceTypeId::new(456);
        assert_eq!(type_id2.value(), 456);
        assert_ne!(type_id, type_id2);
    }

    // ====== RESOURCE DATA TESTS ======

    #[test]
    fn test_resource_data_types() {
        // Test empty data
        let empty = ResourceData::Empty;
        assert!(matches!(empty, ResourceData::Empty));

        // Test bytes data
        let bytes = create_resource_data_bytes(vec![1, 2, 3, 4]);
        assert!(matches!(bytes, ResourceData::Bytes(_)));
        if let ResourceData::Bytes(data) = bytes {
            assert_eq!(data, vec![1, 2, 3, 4]);
        }

        // Test external handle
        let external = create_resource_data_external(12_345);
        assert!(matches!(external, ResourceData::ExternalHandle(12_345)));

        // Test custom data
        let custom = create_resource_data_custom("FileHandle".to_string(), vec![5, 6, 7, 8]);
        assert!(matches!(custom, ResourceData::Custom { .. }));
        if let ResourceData::Custom { type_id, data } = custom {
            assert_eq!(type_id, "FileHandle");
            assert_eq!(data, vec![5, 6, 7, 8]);
        }
    }

    #[test]
    fn test_resource_data_cloning() {
        let original = create_resource_data_bytes(vec![1, 2, 3]);
        let cloned = original.clone();

        assert!(matches!(cloned, ResourceData::Bytes(_)));
        if let (ResourceData::Bytes(orig_data), ResourceData::Bytes(clone_data)) =
            (&original, &cloned)
        {
            assert_eq!(orig_data, clone_data);
        }
    }

    // ====== RESOURCE TYPE TESTS ======

    #[test]
    fn test_resource_type_creation() {
        let mut manager = ResourceManager::new();

        let type_id = manager
            .register_resource_type(
                "file".to_string(),
                "File handle resource".to_string(),
                true, // borrowable
                true, // needs_finalization
            )
            .unwrap();

        assert!(type_id.value() > 0);
        assert_eq!(manager.get_stats().types_registered, 1);

        let resource_type = manager.get_resource_type(type_id).unwrap();
        assert_eq!(resource_type.name, "file");
        assert_eq!(resource_type.description, "File handle resource");
        assert!(resource_type.borrowable);
        assert!(resource_type.needs_finalization);
        assert_eq!(resource_type.max_instances, None);
    }

    #[test]
    fn test_resource_type_metadata() {
        let metadata = ResourceTypeMetadata {
            size_hint: Some(1024),
            alignment: 8,
            custom_fields: {
                #[cfg(feature = "std")]
                {
                    let mut fields = std::collections::HashMap::new();
                    fields.insert("compression".to_string(), "gzip".to_string());
                    fields.insert("version".to_string(), "1.0".to_string());
                    fields
                }
                #[cfg(not(feature = "std"))]
                {
                    use wrt_foundation::NoStdHashMap;
                    let mut fields = NoStdHashMap::new();
                    fields.insert("compression".to_string(), "gzip".to_string());
                    fields.insert("version".to_string(), "1.0".to_string());
                    fields
                }
            },
        };

        assert_eq!(metadata.size_hint, Some(1024));
        assert_eq!(metadata.alignment, 8);
        assert_eq!(metadata.custom_fields.get("compression"), Some(&"gzip".to_string()));
        assert_eq!(metadata.custom_fields.get("version"), Some(&"1.0".to_string()));
    }

    #[test]
    fn test_resource_type_registration_limits() {
        let mut manager = ResourceManager::new();
        let mut registered_types = Vec::new();

        // Register up to the maximum
        for i in 0..MAX_RESOURCE_TYPES {
            let result = manager.register_resource_type(
                ComponentValue::String("Component operation result".into()),
                ComponentValue::String("Component operation result".into()),
                true,
                false,
            );
            assert!(result.is_ok());
            registered_types.push(result.unwrap());
        }

        // Try to register one more - should fail
        let result = manager.register_resource_type(
            "overflow_type".to_string(),
            "This should fail".to_string(),
            true,
            false,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Resource);
    }

    // ====== RESOURCE TABLE TESTS ======

    #[test]
    fn test_resource_table_creation() {
        let table = ResourceTable::new(1);
        assert_eq!(table.instance_id, 1);
        assert_eq!(table.get_stats().active_resources, 0);
        assert_eq!(table.get_stats().resources_created, 0);
    }

    #[test]
    fn test_resource_table_resource_creation() {
        let mut table = ResourceTable::new(1);

        let type_id = ResourceTypeId::new(1);
        let data = create_resource_data_bytes(vec![1, 2, 3, 4]);
        let ownership = ResourceOwnership::Owned;

        let handle = table.create_resource(type_id, data, ownership).unwrap();
        assert!(handle.is_valid());
        assert_eq!(table.get_stats().active_resources, 1);
        assert_eq!(table.get_stats().resources_created, 1);

        let resource = table.get_resource(handle).unwrap();
        assert_eq!(resource.handle, handle);
        assert_eq!(resource.resource_type, type_id);
        assert_eq!(resource.state, ResourceState::Active);
        assert_eq!(resource.ownership, ResourceOwnership::Owned);
        assert_eq!(resource.ref_count, 1);
    }

    #[test]
    fn test_resource_table_resource_limits() {
        let mut table = ResourceTable::new(1);
        let type_id = ResourceTypeId::new(1);
        let mut handles = Vec::new();

        // Create resources up to the limit
        for i in 0..MAX_RESOURCES_PER_INSTANCE {
            let data = create_resource_data_bytes(vec![i as u8]);
            let result = table.create_resource(type_id, data, ResourceOwnership::Owned);
            assert!(result.is_ok());
            handles.push(result.unwrap());
        }

        // Try to create one more - should fail
        let data = create_resource_data_bytes(vec![255]);
        let result = table.create_resource(type_id, data, ResourceOwnership::Owned);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Resource);
    }

    #[test]
    fn test_resource_table_borrowing() {
        let mut table = ResourceTable::new(1);
        let type_id = ResourceTypeId::new(1);
        let data = create_resource_data_bytes(vec![1, 2, 3]);

        let handle = table.create_resource(type_id, data, ResourceOwnership::Owned).unwrap();

        // Borrow the resource
        let result = table.borrow_resource(handle, 2);
        assert!(result.is_ok());
        assert_eq!(table.get_stats().borrowed_resources, 1);

        // Check resource state
        let resource = table.get_resource(handle).unwrap();
        match &resource.state {
            ResourceState::Borrowed { borrower, .. } => {
                assert_eq!(*borrower, 2);
            }
            _ => panic!("Expected borrowed state"),
        }
        assert_eq!(resource.ref_count, 2);

        // Return the resource
        let result = table.return_resource(handle);
        assert!(result.is_ok());
        assert_eq!(table.get_stats().borrowed_resources, 0);

        let resource = table.get_resource(handle).unwrap();
        assert_eq!(resource.state, ResourceState::Active);
        assert_eq!(resource.ref_count, 1);
    }

    #[test]
    fn test_resource_table_drop_resource() {
        let mut table = ResourceTable::new(1);
        let type_id = ResourceTypeId::new(1);
        let data = create_resource_data_bytes(vec![1, 2, 3]);

        let handle = table.create_resource(type_id, data, ResourceOwnership::Owned).unwrap();
        assert_eq!(table.get_stats().active_resources, 1);

        let result = table.drop_resource(handle);
        assert!(result.is_ok());
        assert_eq!(table.get_stats().active_resources, 0);
        assert_eq!(table.get_stats().resources_dropped, 1);

        // Resource should no longer exist
        assert!(table.get_resource(handle).is_none());
    }

    #[test]
    fn test_resource_table_cleanup_expired() {
        let mut table = ResourceTable::new(1);
        let type_id = ResourceTypeId::new(1);

        // Create several resources
        for i in 0..5 {
            let data = create_resource_data_bytes(vec![i]);
            table.create_resource(type_id, data, ResourceOwnership::Owned).unwrap();
        }
        assert_eq!(table.get_stats().active_resources, 5);

        // Cleanup expired resources (in a real implementation, this would check timestamps)
        let cleaned = table.cleanup_expired(1000).unwrap();
        // In this test, no resources are actually expired, so cleaned should be 0
        assert_eq!(cleaned, 0);
        assert_eq!(table.get_stats().active_resources, 5);
    }

    #[test]
    fn test_resource_table_clear_all() {
        let mut table = ResourceTable::new(1);
        let type_id = ResourceTypeId::new(1);

        // Create several resources
        for i in 0..3 {
            let data = create_resource_data_bytes(vec![i]);
            table.create_resource(type_id, data, ResourceOwnership::Owned).unwrap();
        }
        assert_eq!(table.get_stats().active_resources, 3);

        table.clear_all();
        assert_eq!(table.get_stats().active_resources, 0);
        assert_eq!(table.get_stats().resources_dropped, 3);
    }

    #[test]
    fn test_resource_table_get_resources_by_type() {
        let mut table = ResourceTable::new(1);
        let type_id1 = ResourceTypeId::new(1);
        let type_id2 = ResourceTypeId::new(2);

        // Create resources of different types
        let handle1 = table
            .create_resource(
                type_id1,
                create_resource_data_bytes(vec![1]),
                ResourceOwnership::Owned,
            )
            .unwrap();
        let handle2 = table
            .create_resource(
                type_id2,
                create_resource_data_bytes(vec![2]),
                ResourceOwnership::Owned,
            )
            .unwrap();
        let handle3 = table
            .create_resource(
                type_id1,
                create_resource_data_bytes(vec![3]),
                ResourceOwnership::Owned,
            )
            .unwrap();

        let type1_resources = table.get_resources_by_type(type_id1);
        assert_eq!(type1_resources.len(), 2);
        assert!(type1_resources.contains(&handle1));
        assert!(type1_resources.contains(&handle3));

        let type2_resources = table.get_resources_by_type(type_id2);
        assert_eq!(type2_resources.len(), 1);
        assert!(type2_resources.contains(&handle2));
    }

    // ====== RESOURCE MANAGER TESTS ======

    #[test]
    fn test_resource_manager_creation() {
        let manager = ResourceManager::new();
        assert_eq!(manager.get_stats().types_registered, 0);
        assert_eq!(manager.get_stats().instances_managed, 0);
        assert_eq!(manager.get_stats().global_resources, 0);

        let custom_config = ResourceManagerConfig {
            auto_gc: false,
            gc_interval: 500,
            allow_borrowing: false,
            max_borrow_duration: 10_000_000,
            allow_cross_instance_sharing: false,
            validation_level: ResourceValidationLevel::Basic,
        };

        let custom_manager = ResourceManager::with_config(custom_config.clone());
        assert!(!custom_manager.config.auto_gc);
        assert_eq!(custom_manager.config.gc_interval, 500);
        assert!(!custom_manager.config.allow_borrowing);
    }

    #[test]
    fn test_resource_manager_instance_table_management() {
        let mut manager = ResourceManager::new();

        // Create instance table
        let result = manager.create_instance_table(1);
        assert!(result.is_ok());
        assert_eq!(manager.get_stats().instances_managed, 1);

        // Get table
        let table = manager.get_instance_table(1);
        assert!(table.is_some());
        assert_eq!(table.unwrap().instance_id, 1);

        // Try to create duplicate - should fail
        let result = manager.create_instance_table(1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Validation);

        // Remove table
        let result = manager.remove_instance_table(1);
        assert!(result.is_ok());
        assert_eq!(manager.get_stats().instances_managed, 0);

        // Table should no longer exist
        assert!(manager.get_instance_table(1).is_none());
    }

    #[test]
    fn test_resource_manager_resource_creation() {
        let mut manager = ResourceManager::new();

        // Register resource type
        let file_type = manager
            .register_resource_type("file".to_string(), "File handle".to_string(), true, false)
            .unwrap();

        // Create instance table
        manager.create_instance_table(1).unwrap();

        // Create resource
        let data = create_resource_data_bytes(vec![1, 2, 3, 4]);
        let handle = manager.create_resource(1, file_type, data).unwrap();

        assert!(handle.is_valid());
        assert_eq!(manager.get_stats().global_resources, 1);

        // Verify resource exists in instance table
        let table = manager.get_instance_table(1).unwrap();
        let resource = table.get_resource(handle);
        assert!(resource.is_some());
        assert_eq!(resource.unwrap().resource_type, file_type);
    }

    #[test]
    fn test_resource_manager_cross_instance_transfer() {
        let mut manager = ResourceManager::new();

        // Register resource type
        let file_type = manager
            .register_resource_type("file".to_string(), "File handle".to_string(), true, false)
            .unwrap();

        // Create instance tables
        manager.create_instance_table(1).unwrap(); // source
        manager.create_instance_table(2).unwrap(); // target

        // Create resource in source instance
        let data = create_resource_data_bytes(vec![1, 2, 3, 4]);
        let source_handle = manager.create_resource(1, file_type, data).unwrap();

        // Transfer ownership
        let target_handle = manager.transfer_ownership(source_handle, 1, 2).unwrap();
        assert!(target_handle.is_valid());
        assert_ne!(source_handle, target_handle);
        assert_eq!(manager.get_stats().cross_instance_transfers, 1);

        // Verify resource moved
        let source_table = manager.get_instance_table(1).unwrap();
        assert!(source_table.get_resource(source_handle).is_none());

        let target_table = manager.get_instance_table(2).unwrap();
        assert!(target_table.get_resource(target_handle).is_some());
    }

    #[test]
    fn test_resource_manager_cross_instance_borrowing() {
        let mut manager = ResourceManager::new();

        // Register borrowable resource type
        let file_type = manager
            .register_resource_type(
                "file".to_string(),
                "File handle".to_string(),
                true, // borrowable
                false,
            )
            .unwrap();

        // Create instance tables
        manager.create_instance_table(1).unwrap(); // owner
        manager.create_instance_table(2).unwrap(); // borrower

        // Create resource in owner instance
        let data = create_resource_data_bytes(vec![1, 2, 3, 4]);
        let owner_handle = manager.create_resource(1, file_type, data).unwrap();

        // Borrow resource
        let borrowed_handle = manager.borrow_resource(owner_handle, 1, 2).unwrap();
        assert!(borrowed_handle.is_valid());

        // Verify borrowed resource exists in borrower table
        let borrower_table = manager.get_instance_table(2).unwrap();
        let borrowed_resource = borrower_table.get_resource(borrowed_handle);
        assert!(borrowed_resource.is_some());

        // Verify ownership information
        match &borrowed_resource.unwrap().ownership {
            ResourceOwnership::Borrowed { owner, owner_handle } => {
                assert_eq!(*owner, 1);
                assert_eq!(*owner_handle, owner_handle);
            }
            _ => panic!("Expected borrowed ownership"),
        }

        // Return borrowed resource
        let result = manager.return_borrowed_resource(borrowed_handle, 2);
        assert!(result.is_ok());

        // Borrowed resource should be gone from borrower table
        let borrower_table = manager.get_instance_table(2).unwrap();
        assert!(borrower_table.get_resource(borrowed_handle).is_none());
    }

    #[test]
    fn test_resource_manager_non_borrowable_type() {
        let mut manager = ResourceManager::new();

        // Register non-borrowable resource type
        let secret_type = manager
            .register_resource_type(
                "secret".to_string(),
                "Secret data".to_string(),
                false, // not borrowable
                false,
            )
            .unwrap();

        // Create instance tables
        manager.create_instance_table(1).unwrap();
        manager.create_instance_table(2).unwrap();

        // Create resource
        let data = create_resource_data_bytes(vec![1, 2, 3, 4]);
        let handle = manager.create_resource(1, secret_type, data).unwrap();

        // Try to borrow - should fail
        let result = manager.borrow_resource(handle, 1, 2);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Runtime);
    }

    #[test]
    fn test_resource_manager_disabled_features() {
        let config = ResourceManagerConfig {
            auto_gc: false,
            gc_interval: 1000,
            allow_borrowing: false,
            max_borrow_duration: 30_000_000,
            allow_cross_instance_sharing: false,
            validation_level: ResourceValidationLevel::Full,
        };

        let mut manager = ResourceManager::with_config(config);

        // Register resource type
        let file_type = manager
            .register_resource_type("file".to_string(), "File handle".to_string(), true, false)
            .unwrap();

        // Create instance tables
        manager.create_instance_table(1).unwrap();
        manager.create_instance_table(2).unwrap();

        // Create resource
        let data = create_resource_data_bytes(vec![1, 2, 3, 4]);
        let handle = manager.create_resource(1, file_type, data).unwrap();

        // Try to borrow - should fail (borrowing disabled)
        let result = manager.borrow_resource(handle, 1, 2);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Runtime);

        // Try to transfer - should fail (cross-instance sharing disabled)
        let result = manager.transfer_ownership(handle, 1, 2);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Runtime);
    }

    #[test]
    fn test_resource_manager_garbage_collection() {
        let mut manager = ResourceManager::new();

        // Register resource type
        let file_type = manager
            .register_resource_type("file".to_string(), "File handle".to_string(), true, false)
            .unwrap();

        // Create instance table
        manager.create_instance_table(1).unwrap();

        // Create several resources
        for i in 0..5 {
            let data = create_resource_data_bytes(vec![i]);
            manager.create_resource(1, file_type, data).unwrap();
        }

        // Run garbage collection
        let cleaned = manager.garbage_collect().unwrap();
        // In this simple implementation, no resources should be cleaned
        assert_eq!(cleaned, 0);
        assert_eq!(manager.get_stats().garbage_collections, 1);
    }

    #[test]
    fn test_resource_manager_validation() {
        let mut manager = ResourceManager::new();

        // Register resource type
        let file_type = manager
            .register_resource_type("file".to_string(), "File handle".to_string(), true, false)
            .unwrap();

        // Create instance table
        manager.create_instance_table(1).unwrap();

        // Create resource
        let data = create_resource_data_bytes(vec![1, 2, 3, 4]);
        manager.create_resource(1, file_type, data).unwrap();

        // Validate all resources
        let result = manager.validate_all_resources();
        assert!(result.is_ok());
    }

    // ====== ERROR HANDLING TESTS ======

    #[test]
    fn test_resource_error_display() {
        let handle = ResourceHandle::new(42);
        let type_id = ResourceTypeId::new(1);
        let state = ResourceState::Active;

        let error1 = ResourceError::HandleNotFound(handle);
        assert_eq!(ComponentValue::String("Component operation result".into()), "Resource handle 42 not found");

        let error2 = ResourceError::TypeNotFound(type_id);
        assert_eq!(ComponentValue::String("Component operation result".into()), "Resource type 1 not found");

        let error3 = ResourceError::InvalidState(handle, state);
        assert_eq!(ComponentValue::String("Component operation result".into()), "Resource 42 in invalid state: Active");

        let error4 = ResourceError::AccessDenied(handle);
        assert_eq!(ComponentValue::String("Component operation result".into()), "Access denied to resource 42");

        let error5 = ResourceError::LimitExceeded("Too many resources".to_string());
        assert_eq!(ComponentValue::String("Component operation result".into()), "Resource limit exceeded: Too many resources");

        let error6 = ResourceError::TypeMismatch("Expected file, got socket".to_string());
        assert_eq!(ComponentValue::String("Component operation result".into()), "Resource type mismatch: Expected file, got socket");

        let error7 =
            ResourceError::OwnershipViolation("Cannot transfer owned resource".to_string());
        assert_eq!(ComponentValue::String("Component operation result".into()), "Ownership violation: Cannot transfer owned resource");

        let error8 = ResourceError::AlreadyExists(handle);
        assert_eq!(ComponentValue::String("Component operation result".into()), "Resource 42 already exists");
    }

    #[test]
    fn test_resource_states() {
        let state1 = ResourceState::Active;
        assert_eq!(state1, ResourceState::Active);

        let state2 = ResourceState::Borrowed { borrower: 2, borrowed_at: 12345 };
        if let ResourceState::Borrowed { borrower, borrowed_at } = state2 {
            assert_eq!(borrower, 2);
            assert_eq!(borrowed_at, 12345);
        }

        let state3 = ResourceState::Finalizing;
        assert_eq!(state3, ResourceState::Finalizing);

        let state4 = ResourceState::Dropped;
        assert_eq!(state4, ResourceState::Dropped);
    }

    #[test]
    fn test_resource_ownership() {
        let owned = ResourceOwnership::Owned;
        assert_eq!(owned, ResourceOwnership::Owned);

        let borrowed =
            ResourceOwnership::Borrowed { owner: 1, owner_handle: ResourceHandle::new(42) };
        if let ResourceOwnership::Borrowed { owner, owner_handle } = borrowed {
            assert_eq!(owner, 1);
            assert_eq!(owner_handle.value(), 42);
        }
    }

    // ====== CROSS-ENVIRONMENT COMPATIBILITY TESTS ======

    #[cfg(feature = "std")]
    #[test]
    fn test_std_environment_compatibility() {
        let mut manager = ResourceManager::new();

        // Should work in std environment
        let file_type = manager
            .register_resource_type(
                "std_file".to_string(),
                "File for std test".to_string(),
                true,
                false,
            )
            .unwrap();

        manager.create_instance_table(1).unwrap();
        let data = create_resource_data_bytes(vec![1, 2, 3, 4]);
        let handle = manager.create_resource(1, file_type, data).unwrap();

        assert!(handle.is_valid());
        assert_eq!(manager.get_stats().global_resources, 1);
    }

    #[cfg(all(not(feature = "std")))]
    #[test]
    fn test_alloc_environment_compatibility() {
        let mut manager = ResourceManager::new();

        // Binary std/no_std choice
        let file_type = manager
            .register_resource_type(
                "alloc_file".to_string(),
                "File for alloc test".to_string(),
                true,
                false,
            )
            .unwrap();

        manager.create_instance_table(1).unwrap();
        let data = create_resource_data_bytes(vec![1, 2, 3, 4]);
        let handle = manager.create_resource(1, file_type, data).unwrap();

        assert!(handle.is_valid());
        assert_eq!(manager.get_stats().global_resources, 1);
    }

    #[cfg(not(any(feature = "std", )))]
    #[test]
    fn test_no_std_environment_compatibility() {
        // In pure no_std, we can at least create configurations and validate types
        let config = ResourceManagerConfig::default();
        assert!(config.auto_gc);
        assert_eq!(config.gc_interval, 1000);
        assert!(config.allow_borrowing);
        assert_eq!(config.validation_level, ResourceValidationLevel::Full);

        let handle = ResourceHandle::new(42);
        assert!(handle.is_valid());
        assert_eq!(handle.value(), 42);

        let type_id = ResourceTypeId::new(1);
        assert_eq!(type_id.value(), 1);
    }

    // ====== INTEGRATION WITH COMPONENT INSTANTIATION ======

    #[test]
    fn test_component_instance_resource_integration() {
        use super::super::component_instantiation::*;

        let config = InstanceConfig::default();
        let mut instance = ComponentInstance::new(
            1,
            "resource_test_component".to_string(),
            config,
            vec![],
            vec![],
        )
        .unwrap();

        // Initialize the instance
        instance.initialize().unwrap();

        // Get the resource manager
        assert!(instance.get_resource_manager().is_some());

        // Register a resource type through the instance's resource manager
        let resource_manager = instance.get_resource_manager_mut().unwrap();
        let file_type = resource_manager
            .register_resource_type("file".to_string(), "File handle".to_string(), true, false)
            .unwrap();

        // Create a resource in the instance
        let data = create_resource_data_bytes(vec![1, 2, 3, 4]);
        let handle = instance.create_resource(file_type, data).unwrap();
        assert!(handle.is_valid());

        // Verify the resource exists
        let resource_manager = instance.get_resource_manager().unwrap();
        let table = resource_manager.get_instance_table(instance.id).unwrap();
        assert!(table.get_resource(handle).is_some());

        // Drop the resource
        let result = instance.drop_resource(handle);
        assert!(result.is_ok());

        // Resource should no longer exist
        let table = resource_manager.get_instance_table(instance.id).unwrap();
        assert!(table.get_resource(handle).is_none());
    }

    #[test]
    fn test_component_instance_resource_cleanup_on_termination() {
        use super::super::component_instantiation::*;

        let config = InstanceConfig::default();
        let mut instance =
            ComponentInstance::new(1, "cleanup_test_component".to_string(), config, vec![], vec![])
                .unwrap();

        instance.initialize().unwrap();

        // Register resource type and create resources
        let resource_manager = instance.get_resource_manager_mut().unwrap();
        let file_type = resource_manager
            .register_resource_type("file".to_string(), "File handle".to_string(), true, false)
            .unwrap();

        // Create multiple resources
        for i in 0..3 {
            let data = create_resource_data_bytes(vec![i]);
            let _handle = instance.create_resource(file_type, data).unwrap();
        }

        // Verify resources exist
        let resource_manager = instance.get_resource_manager().unwrap();
        let table = resource_manager.get_instance_table(instance.id).unwrap();
        assert_eq!(table.get_stats().active_resources, 3);

        // Terminate the instance
        instance.terminate();
        assert_eq!(instance.state, InstanceState::Terminated);

        // Instance table should be cleaned up
        let resource_manager = instance.get_resource_manager().unwrap();
        assert!(resource_manager.get_instance_table(instance.id).is_none());
    }

    #[test]
    fn test_multiple_component_instances_with_resources() {
        use super::super::component_instantiation::*;
        use super::super::component_linker::*;

        // Create multiple instances with their own resource managers
        let mut instances = Vec::new();

        for i in 1..=3 {
            let config = InstanceConfig::default();
            let mut instance =
                ComponentInstance::new(i, ComponentValue::String("Component operation result".into()), config, vec![], vec![])
                    .unwrap();

            instance.initialize().unwrap();

            // Register resource type and create resources
            let resource_manager = instance.get_resource_manager_mut().unwrap();
            let file_type = resource_manager
                .register_resource_type(
                    ComponentValue::String("Component operation result".into()),
                    ComponentValue::String("Component operation result".into()),
                    true,
                    false,
                )
                .unwrap();

            // Create some resources
            for j in 0..i {
                let data = create_resource_data_bytes(vec![j as u8]);
                let _handle = instance.create_resource(file_type, data).unwrap();
            }

            instances.push(instance);
        }

        // Verify each instance has the correct number of resources
        for (index, instance) in instances.iter().enumerate() {
            let expected_resources = index + 1;
            let resource_manager = instance.get_resource_manager().unwrap();
            let table = resource_manager.get_instance_table(instance.id).unwrap();
            assert_eq!(table.get_stats().active_resources, expected_resources as u32);
        }
    }

    #[test]
    fn test_resource_transfer_between_component_instances() {
        use super::super::component_instantiation::*;

        // Create two component instances
        let config = InstanceConfig::default();
        let mut instance1 = ComponentInstance::new(
            1,
            "source_instance".to_string(),
            config.clone(),
            vec![],
            vec![],
        )
        .unwrap();

        let mut instance2 =
            ComponentInstance::new(2, "target_instance".to_string(), config, vec![], vec![])
                .unwrap();

        instance1.initialize().unwrap();
        instance2.initialize().unwrap();

        // Register the same resource type in both instances
        let resource_manager1 = instance1.get_resource_manager_mut().unwrap();
        let file_type = resource_manager1
            .register_resource_type("file".to_string(), "File handle".to_string(), true, false)
            .unwrap();

        let resource_manager2 = instance2.get_resource_manager_mut().unwrap();
        let _file_type2 = resource_manager2
            .register_resource_type("file".to_string(), "File handle".to_string(), true, false)
            .unwrap();

        // Create a resource in instance1
        let data = create_resource_data_bytes(vec![1, 2, 3, 4]);
        let source_handle = instance1.create_resource(file_type, data).unwrap();

        // Verify resource exists in instance1
        let resource_manager1 = instance1.get_resource_manager().unwrap();
        let table1 = resource_manager1.get_instance_table(1).unwrap();
        assert!(table1.get_resource(source_handle).is_some());

        // Note: In a real implementation, resource transfer between instances
        // would require coordination through a global resource manager.
        // For now, we just verify that each instance manages its own resources independently.

        // Verify instance2 doesn't have the resource
        let resource_manager2 = instance2.get_resource_manager().unwrap();
        let table2 = resource_manager2.get_instance_table(2).unwrap();
        assert!(table2.get_resource(source_handle).is_none());
    }

    // ====== EDGE CASES AND STRESS TESTS ======

    #[test]
    fn test_resource_handle_edge_cases() {
        let handle_zero = ResourceHandle::new(0);
        assert!(handle_zero.is_valid()); // 0 is valid, only u32::MAX is invalid

        let handle_max_minus_one = ResourceHandle::new(u32::MAX - 1);
        assert!(handle_max_minus_one.is_valid());

        let handle_invalid = ResourceHandle::new(u32::MAX);
        assert!(!handle_invalid.is_valid());
        assert_eq!(handle_invalid, INVALID_HANDLE);
    }

    #[test]
    fn test_resource_type_creation_helper() {
        let (name, description, borrowable, needs_finalization) =
            create_resource_type("test_type".to_string(), "Test type description".to_string());

        assert_eq!(name, "test_type");
        assert_eq!(description, "Test type description");
        assert!(borrowable); // Default is true
        assert!(!needs_finalization); // Default is false
    }

    #[test]
    fn test_large_resource_data() {
        // Test with large data
        let large_data = vec![42u8; 1024 * 1024]; // 1MB
        let resource_data = create_resource_data_bytes(large_data.clone());

        if let ResourceData::Bytes(data) = resource_data {
            assert_eq!(data.len(), 1024 * 1024);
            assert_eq!(data[0], 42);
            assert_eq!(data[data.len() - 1], 42);
        }
    }

    #[test]
    fn test_multiple_resource_types() {
        let mut manager = ResourceManager::new();

        // Register many different resource types
        let mut type_ids = Vec::new();

        for i in 0..10 {
            let type_id = manager
                .register_resource_type(
                    ComponentValue::String("Component operation result".into()),
                    ComponentValue::String("Component operation result".into()),
                    i % 2 == 0, // Alternate borrowable
                    i % 3 == 0, // Every third needs finalization
                )
                .unwrap();
            type_ids.push(type_id);
        }

        assert_eq!(manager.get_stats().types_registered, 10);

        // Verify all types were registered correctly
        for (i, type_id) in type_ids.iter().enumerate() {
            let resource_type = manager.get_resource_type(*type_id).unwrap();
            assert_eq!(resource_type.name, ComponentValue::String("Component operation result".into()));
            assert_eq!(resource_type.borrowable, i % 2 == 0);
            assert_eq!(resource_type.needs_finalization, i % 3 == 0);
        }
    }

    #[test]
    fn test_resource_statistics_tracking() {
        let mut manager = ResourceManager::new();

        // Track statistics through various operations
        assert_eq!(manager.get_stats().types_registered, 0);
        assert_eq!(manager.get_stats().instances_managed, 0);
        assert_eq!(manager.get_stats().global_resources, 0);
        assert_eq!(manager.get_stats().cross_instance_transfers, 0);
        assert_eq!(manager.get_stats().garbage_collections, 0);

        // Register type
        let file_type = manager
            .register_resource_type("file".to_string(), "File handle".to_string(), true, false)
            .unwrap();
        assert_eq!(manager.get_stats().types_registered, 1);

        // Create instances
        manager.create_instance_table(1).unwrap();
        manager.create_instance_table(2).unwrap();
        assert_eq!(manager.get_stats().instances_managed, 2);

        // Create resources
        for i in 0..3 {
            let data = create_resource_data_bytes(vec![i]);
            manager.create_resource(1, file_type, data).unwrap();
        }
        assert_eq!(manager.get_stats().global_resources, 3);

        // Transfer resource
        let data = create_resource_data_bytes(vec![99]);
        let handle = manager.create_resource(1, file_type, data).unwrap();
        manager.transfer_ownership(handle, 1, 2).unwrap();
        assert_eq!(manager.get_stats().cross_instance_transfers, 1);

        // Run garbage collection
        manager.garbage_collect().unwrap();
        assert_eq!(manager.get_stats().garbage_collections, 1);
    }

    #[test]
    fn test_resource_table_statistics() {
        let mut table = ResourceTable::new(1);
        let type_id = ResourceTypeId::new(1);

        let stats = table.get_stats();
        assert_eq!(stats.resources_created, 0);
        assert_eq!(stats.resources_dropped, 0);
        assert_eq!(stats.active_resources, 0);
        assert_eq!(stats.borrowed_resources, 0);
        assert_eq!(stats.peak_resources, 0);
        assert_eq!(stats.finalizations, 0);

        // Create resources and track peak
        let mut handles = Vec::new();
        for i in 0..5 {
            let data = create_resource_data_bytes(vec![i]);
            let handle = table.create_resource(type_id, data, ResourceOwnership::Owned).unwrap();
            handles.push(handle);
        }

        let stats = table.get_stats();
        assert_eq!(stats.resources_created, 5);
        assert_eq!(stats.active_resources, 5);
        assert_eq!(stats.peak_resources, 5);

        // Borrow a resource
        table.borrow_resource(handles[0], 2).unwrap();
        let stats = table.get_stats();
        assert_eq!(stats.borrowed_resources, 1);

        // Drop some resources
        table.drop_resource(handles[1]).unwrap();
        table.drop_resource(handles[2]).unwrap();

        let stats = table.get_stats();
        assert_eq!(stats.resources_dropped, 2);
        assert_eq!(stats.active_resources, 3);
        assert_eq!(stats.peak_resources, 5); // Peak doesn't decrease
    }
}
