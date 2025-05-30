#![cfg(test)]

//! Comprehensive verification test for no_std with alloc functionality
//!
//! This file provides comprehensive testing of alloc-specific functionality
//! to ensure consistent behavior across std and no_std with alloc environments.
//! It's specifically designed to catch regressions in the alloc feature set.

// For testing in a no_std environment with alloc support
#![cfg_attr(not(feature = "std"), no_std)]

// External crate imports
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(test)]
mod tests {
    // Import necessary types for no_std with alloc environment
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
    
    #[cfg(feature = "std")]
    use std::{boxed::Box, string::String, vec, vec::Vec};
    
    // Import from wrt-error
    use wrt_error::{Error, ErrorCategory, Result};
    
    // Import from wrt-foundation
    use wrt_foundation::{
        bounded::{BoundedVec, BoundedStack},
        component_value::ComponentValue,
        component_value_store::ComponentValueStore,
        component_value_store_builder::ComponentValueStoreBuilder,
        component_builder::ComponentBuilder,
    };
    
    // Import from wrt-decoder
    use wrt_decoder::component::types::ComponentTypeId;
    
    // Import from wrt-component
    use wrt_component::{
        resources::{
            resource_manager::ResourceManager,
            buffer_pool::BufferPool,
        },
    };
    
    #[test]
    fn test_alloc_string_handling() {
        // Create strings in no_std with alloc environment
        let string1 = String::from("Hello");
        let string2 = String::from(" World");
        
        // Test string concatenation
        let result = format!("{}{}", string1, string2);
        assert_eq!(result, "Hello World");
        
        // Test string capacity and manipulation
        let mut growable = String::with_capacity(20);
        growable.push_str("Growing string");
        growable.push_str(" with alloc");
        
        assert_eq!(growable, "Growing string with alloc");
    }
    
    #[test]
    fn test_alloc_vec_operations() {
        // Create a vector in no_std with alloc environment
        let mut vec = Vec::<u32>::with_capacity(10);
        
        // Test vector operations
        for i in 0..10 {
            vec.push(i);
        }
        
        assert_eq!(vec.len(), 10);
        assert_eq!(vec[5], 5);
        
        // Test filtering (requires heap allocation)
        let evens: Vec<u32> = vec.iter().filter(|&&x| x % 2 == 0).cloned().collect();
        assert_eq!(evens, vec![0, 2, 4, 6, 8]);
        
        // Test mapping (requires heap allocation)
        let doubled: Vec<u32> = vec.iter().map(|&x| x * 2).collect();
        assert_eq!(doubled[5], 10);
    }
    
    #[test]
    fn test_boxed_values() {
        // Test Box<T> in no_std with alloc
        let boxed_value = Box::new(42);
        assert_eq!(*boxed_value, 42);
        
        // Test more complex boxed types
        let boxed_vec = Box::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(boxed_vec.len(), 5);
        
        // Test Box<dyn Trait> if needed
        // (Assuming we have a trait that can be used here)
    }
    
    #[test]
    fn test_component_value_store() {
        // Create a component value store builder
        let mut builder = ComponentValueStoreBuilder::new();
        
        // Add some string values (requires alloc)
        let string_id = builder.add_string("hello world");
        
        // Build the store
        let store = builder.build();
        
        // Test retrieving values
        let retrieved = store.get_string(string_id).unwrap();
        assert_eq!(retrieved, "hello world");
    }
    
    #[test]
    fn test_error_with_context() {
        // Create an error with a context string (requires alloc)
        let error = Error::new(
            ErrorCategory::Resource,
            42,
            "This is a test error with context".to_string(),
        );
        
        // Check the error message
        assert_eq!(error.code(), 42);
        assert_eq!(error.category(), ErrorCategory::Resource);
        assert!(error.to_string().contains("test error with context"));
    }
    
    #[test]
    fn test_resource_management() {
        // Create a resource manager (uses alloc internally)
        let mut resource_manager = ResourceManager::new();
        
        // Test resource creation
        let resource_id = resource_manager.create_resource(
            wrt_foundation::resource::ResourceType::new(1)
        ).unwrap();
        
        // Verify resource exists
        assert!(resource_manager.has_resource(&resource_id));
        
        // Test dropping resource
        resource_manager.drop_resource(&resource_id).unwrap();
        assert!(!resource_manager.has_resource(&resource_id));
    }
    
    #[test]
    fn test_bounded_vec_with_complex_type() {
        // Create a bounded vec with a complex type that requires alloc
        let mut vec = BoundedVec::<String, 5>::new();
        
        // Add strings to it
        assert!(vec.push("string1".to_string()).is_ok());
        assert!(vec.push("string2".to_string()).is_ok());
        
        // Verify contents
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(0).unwrap(), "string1");
    }
    
    #[test]
    fn test_component_builder() {
        // Create a component builder (requires alloc)
        let mut builder = ComponentBuilder::new();
        
        // Add a type (requires alloc internally)
        let type_id = ComponentTypeId::Func(0);
        builder.add_type(type_id);
        
        // Verify builder state
        assert!(builder.has_type(&type_id));
    }
    
    #[test]
    fn test_buffer_pool_allocations() {
        // Create a buffer pool
        let mut pool = BufferPool::new();
        
        // Allocate multiple buffers of different sizes
        let buffer1 = pool.allocate(100).unwrap();
        let buffer2 = pool.allocate(200).unwrap();
        let buffer3 = pool.allocate(300).unwrap();
        
        // Verify buffer sizes
        assert_eq!(buffer1.len(), 100);
        assert_eq!(buffer2.len(), 200);
        assert_eq!(buffer3.len(), 300);
    }
}