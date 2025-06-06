// WRT - wrt-component
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use wrt_format::component::ResourceOperation as FormatResourceOperation;
use wrt_foundation::{
    bounded::{BoundedCollection, BoundedVec},
    component_value::ComponentValue,
};

use super::{
    bounded_buffer_pool::{BoundedBufferPool, MAX_BUFFERS_PER_CLASS},
    resource_interceptor::ResourceInterceptor,
    resource_operation_no_std::{from_format_resource_operation, to_format_resource_operation},
};
use crate::prelude::*;

/// Maximum resources in the table
pub const MAX_RESOURCES: usize = 64;

/// Maximum interceptors per resource table
pub const MAX_INTERCEPTORS: usize = 8;

/// Verification level for resource operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationLevel {
    /// No verification, fastest performance
    None,
    /// Verify only critical operations
    Critical,
    /// Verify all operations
    Full,
}

/// Trait for buffer pools that can be used by ResourceTable in no_std
pub trait BufferPoolTrait {
    /// Allocate a buffer of at least the specified size
    fn allocate(&mut self, size: usize) -> Result<BoundedVec<u8, MAX_BUFFERS_PER_CLASS>>;

    /// Return a buffer to the pool
    fn return_buffer(&mut self, buffer: BoundedVec<u8, MAX_BUFFERS_PER_CLASS>) -> Result<()>;

    /// Reset the buffer pool
    fn reset(&mut self);
}

impl BufferPoolTrait for BoundedBufferPool {
    fn allocate(&mut self, size: usize) -> Result<BoundedVec<u8, MAX_BUFFERS_PER_CLASS>> {
        self.allocate(size)
    }

    fn return_buffer(&mut self, buffer: BoundedVec<u8, MAX_BUFFERS_PER_CLASS>) -> Result<()> {
        self.return_buffer(buffer)
    }

    fn reset(&mut self) {
        self.reset()
    }
}

/// Memory strategy for resource operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryStrategy {
    /// Zero-copy strategy for trusted components
    ZeroCopy,
    /// Bounded-copy strategy with buffer pooling
    BoundedCopy,
    /// Full isolation with validation
    Isolated,
    /// Copy strategy - creates a copy of memory for safety
    Copy,
    /// Reference strategy - provides a direct reference to memory
    Reference,
    /// Full isolation with complete memory validation
    FullIsolation,
}

impl Default for MemoryStrategy {
    fn default() -> Self {
        MemoryStrategy::BoundedCopy
    }
}

impl MemoryStrategy {
    /// Convert from u8 to MemoryStrategy
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(MemoryStrategy::ZeroCopy),
            1 => Some(MemoryStrategy::BoundedCopy),
            2 => Some(MemoryStrategy::Isolated),
            3 => Some(MemoryStrategy::Copy),
            4 => Some(MemoryStrategy::Reference),
            5 => Some(MemoryStrategy::FullIsolation),
            _ => None,
        }
    }

    /// Convert from MemoryStrategy to u8
    pub fn to_u8(self) -> u8 {
        match self {
            MemoryStrategy::ZeroCopy => 0,
            MemoryStrategy::BoundedCopy => 1,
            MemoryStrategy::Isolated => 2,
            MemoryStrategy::Copy => 3,
            MemoryStrategy::Reference => 4,
            MemoryStrategy::FullIsolation => 5,
        }
    }
}

/// Resource instance representation
pub struct Resource {
    /// Resource type index
    pub type_idx: u32,
    /// Resource data (implementation-specific)
    pub data: Box<dyn Any + Send + Sync>,
    /// Debug name for the resource (optional)
    pub name: Option<String>,
    /// Creation timestamp
    pub created_at: Instant,
    /// Last access timestamp
    pub last_accessed: Instant,
    /// Access count
    pub access_count: u64,
}

impl Resource {
    /// Create a new resource
    pub fn new(type_idx: u32, data: Box<dyn Any + Send + Sync>) -> Self {
        let now = Instant::now();
        Self { type_idx, data, name: None, created_at: now, last_accessed: now, access_count: 0 }
    }

    /// Create a new resource with a debug name
    pub fn new_with_name(type_idx: u32, data: Box<dyn Any + Send + Sync>, name: &str) -> Self {
        let mut resource = Self::new(type_idx, data);
        resource.name = Some(name.to_string());
        resource
    }

    /// Record access to this resource
    pub fn record_access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }
}

/// Resource entry in the resource table
struct ResourceEntry {
    /// The resource instance
    resource: Box<Mutex<Resource>>,
    /// Memory strategy for this resource
    memory_strategy: MemoryStrategy,
    /// Verification level
    verification_level: VerificationLevel,
}

/// Resource table for tracking resource instances
///
/// This is a no_std compatible version of ResourceTable that
/// uses fixed-size BoundedVec instead of HashMap for resource storage.
pub struct ResourceTable {
    /// Resource handles and entries
    resource_handles: BoundedVec<u32, MAX_RESOURCES>,
    /// Resource entries
    resource_entries: BoundedVec<ResourceEntry, MAX_RESOURCES>,
    /// Next available resource handle
    next_handle: u32,
    /// Default memory strategy
    pub default_memory_strategy: MemoryStrategy,
    /// Default verification level
    pub default_verification_level: VerificationLevel,
    /// Buffer pool for bounded copy operations
    buffer_pool: Box<Mutex<dyn BufferPoolTrait>>,
    /// Interceptors for resource operations
    interceptors: BoundedVec<Box<dyn ResourceInterceptor>, MAX_INTERCEPTORS>,
}

impl ResourceTable {
    /// Create a new resource table with default settings
    pub fn new() -> Self {
        Self {
            resource_handles: BoundedVec::new(),
            resource_entries: BoundedVec::new(),
            next_handle: 1, // Start at 1 as 0 is reserved
            default_memory_strategy: MemoryStrategy::default(),
            default_verification_level: VerificationLevel::Critical,
            buffer_pool: Box::new(Mutex::new(BoundedBufferPool::new())),
            interceptors: BoundedVec::new(),
        }
    }

    /// Add a resource interceptor
    pub fn add_interceptor(&mut self, interceptor: Box<dyn ResourceInterceptor>) -> Result<()> {
        if self.interceptors.len() >= MAX_INTERCEPTORS {
            return Err(Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_ERROR,
                ComponentValue::String("Component operation result".into()) reached", MAX_INTERCEPTORS),
            ));
        }

        self.interceptors.push(interceptor).map_err(|_| {
            Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_ERROR,
                ComponentValue::String("Component operation result".into()),
            )
        })
    }

    /// Create a new resource
    pub fn create_resource(
        &mut self,
        type_idx: u32,
        data: Box<dyn Any + Send + Sync>,
    ) -> Result<u32> {
        // Check if we've reached the maximum number of resources
        if self.resource_entries.len() >= MAX_RESOURCES {
            return Err(Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_ERROR,
                ComponentValue::String("Component operation result".into()) reached", MAX_RESOURCES),
            ));
        }

        // Create the resource
        let resource = Resource::new(type_idx, data);

        // Notify interceptors about resource creation
        for interceptor in self.interceptors.iter_mut() {
            interceptor.on_resource_create(type_idx, &resource)?;
        }

        // Assign a handle
        let handle = self.next_handle;
        self.next_handle += 1;

        // Create the entry
        let entry = ResourceEntry {
            resource: Box::new(Mutex::new(resource)),
            memory_strategy: self
                .get_strategy_from_interceptors(handle)
                .unwrap_or(self.default_memory_strategy),
            verification_level: self.default_verification_level,
        };

        // Add to our collections
        self.resource_handles.push(handle).map_err(|_| {
            Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_ERROR,
                ComponentValue::String("Component operation result".into()),
            )
        })?;

        self.resource_entries.push(entry).map_err(|_| {
            // Remove the handle we just added
            let last_idx = self.resource_handles.len() - 1;
            self.resource_handles.remove(last_idx);

            Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_ERROR,
                ComponentValue::String("Component operation result".into()),
            )
        })?;

        Ok(handle)
    }

    /// Drop a resource
    pub fn drop_resource(&mut self, handle: u32) -> Result<()> {
        // Find the resource index
        let idx = self.find_resource_index(handle).ok_or_else(|| {
            Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_ERROR,
                ComponentValue::String("Component operation result".into()),
            )
        })?;

        // Notify interceptors about resource dropping
        for interceptor in self.interceptors.iter_mut() {
            interceptor.on_resource_drop(handle)?;
        }

        // Remove the entry
        self.resource_handles.remove(idx);
        self.resource_entries.remove(idx);

        Ok(())
    }

    /// Get a resource by handle
    pub fn get_resource(&self, handle: u32) -> Result<Box<Mutex<Resource>>> {
        // Find the resource index
        let idx = self.find_resource_index(handle).ok_or_else(|| {
            Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_ERROR,
                ComponentValue::String("Component operation result".into()),
            )
        })?;

        // Get the entry
        let entry = &self.resource_entries[idx];

        // Record access
        if let Ok(mut resource) = entry.resource.lock() {
            resource.record_access();
        }

        // Notify interceptors about resource access
        for interceptor in self.interceptors.iter() {
            interceptor.on_resource_access(handle)?;
        }

        // Create a copy of the resource mutex
        let resource_copy = Box::new(Mutex::new(Resource {
            type_idx: entry.resource.lock().unwrap().type_idx,
            data: entry.resource.lock().unwrap().data.clone(),
            name: entry.resource.lock().unwrap().name.clone(),
            created_at: entry.resource.lock().unwrap().created_at,
            last_accessed: entry.resource.lock().unwrap().last_accessed,
            access_count: entry.resource.lock().unwrap().access_count,
        }));

        Ok(resource_copy)
    }

    /// Apply an operation to a resource
    pub fn apply_operation(
        &mut self,
        handle: u32,
        operation: FormatResourceOperation,
    ) -> Result<ComponentValue> {
        // Find the resource index
        let idx = self.find_resource_index(handle).ok_or_else(|| {
            Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_ERROR,
                ComponentValue::String("Component operation result".into()),
            )
        })?;

        // Get the operation kind for interception
        let local_op = from_format_resource_operation(&operation);

        // Check interceptors first
        for interceptor in self.interceptors.iter_mut() {
            // Pass the format operation to interceptors
            interceptor.on_resource_operation(handle, &operation)?;

            // Check if the interceptor will override the operation
            if let Some(result) = interceptor.intercept_resource_operation(handle, &operation)? {
                // If the interceptor provides a result, use it
                return Ok(ComponentValue::U32(handle));
            }
        }

        // Apply the operation based on the resource
        match operation {
            FormatResourceOperation::Rep(_) => {
                // Representation operation - convert resource to its representation
                Ok(ComponentValue::U32(handle))
            }
            FormatResourceOperation::Drop(_) => {
                // Drop operation - remove the resource from the table
                // Since we're already in apply_operation, we call drop_resource separately
                self.drop_resource(handle)?;
                Ok(ComponentValue::Void)
            }
            FormatResourceOperation::Destroy(_) => {
                // Destroy operation - similar to drop but may perform cleanup
                self.drop_resource(handle)?;
                Ok(ComponentValue::Void)
            }
            FormatResourceOperation::New(_) => {
                // New operation - creates a resource from its representation
                Ok(ComponentValue::U32(handle))
            }
            FormatResourceOperation::Transfer => {
                // Transfer operation - transfers ownership
                Ok(ComponentValue::U32(handle))
            }
            FormatResourceOperation::Borrow => {
                // Borrow operation - temporarily borrows the resource
                Ok(ComponentValue::U32(handle))
            }
            _ => Err(Error::new(
                ErrorCategory::Operation,
                codes::UNSUPPORTED_OPERATION,
                ComponentValue::String("Component operation result".into()),
            )),
        }
    }

    /// Set memory strategy for a resource
    pub fn set_memory_strategy(&mut self, handle: u32, strategy: MemoryStrategy) -> Result<()> {
        // Find the resource index
        let idx = self.find_resource_index(handle).ok_or_else(|| {
            Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_ERROR,
                ComponentValue::String("Component operation result".into()),
            )
        })?;

        // Update the strategy
        self.resource_entries[idx].memory_strategy = strategy;

        Ok(())
    }

    /// Set verification level for a resource
    pub fn set_verification_level(&mut self, handle: u32, level: VerificationLevel) -> Result<()> {
        // Find the resource index
        let idx = self.find_resource_index(handle).ok_or_else(|| {
            Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_ERROR,
                ComponentValue::String("Component operation result".into()),
            )
        })?;

        // Update the level
        self.resource_entries[idx].verification_level = level;

        Ok(())
    }

    /// Get the number of resources in the table
    pub fn resource_count(&self) -> usize {
        self.resource_entries.len()
    }

    /// Get a buffer from the pool
    pub fn get_buffer(&mut self, size: usize) -> Result<BoundedVec<u8, MAX_BUFFERS_PER_CLASS>> {
        self.buffer_pool.lock().unwrap().allocate(size)
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&mut self, buffer: BoundedVec<u8, MAX_BUFFERS_PER_CLASS>) -> Result<()> {
        self.buffer_pool.lock().unwrap().return_buffer(buffer)
    }

    /// Reset the buffer pool
    pub fn reset_buffer_pool(&mut self) {
        self.buffer_pool.lock().unwrap().reset()
    }

    /// Get memory strategy from interceptors
    pub fn get_strategy_from_interceptors(&self, handle: u32) -> Option<MemoryStrategy> {
        for interceptor in self.interceptors.iter() {
            if let Some(strategy_val) = interceptor.get_memory_strategy(handle) {
                if let Some(strategy) = MemoryStrategy::from_u8(strategy_val) {
                    return Some(strategy);
                }
            }
        }
        None
    }

    /// Find the index of a resource by handle
    fn find_resource_index(&self, handle: u32) -> Option<usize> {
        self.resource_handles.iter().position(|&h| h == handle)
    }
}

impl Debug for ResourceTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceTable")
            .field("resource_count", &self.resource_entries.len())
            .field("next_handle", &self.next_handle)
            .field("default_memory_strategy", &self.default_memory_strategy)
            .field("default_verification_level", &self.default_verification_level)
            .field("interceptor_count", &self.interceptors.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestInterceptor {
        executed_operations: BoundedVec<String, 32>,
    }

    impl TestInterceptor {
        fn new() -> Self {
            Self { executed_operations: BoundedVec::new() }
        }
    }

    impl ResourceInterceptor for TestInterceptor {
        fn on_resource_create(&mut self, type_idx: u32, _resource: &Resource) -> Result<()> {
            self.executed_operations.push(ComponentValue::String("Component operation result".into())).unwrap();
            Ok(())
        }

        fn on_resource_drop(&mut self, handle: u32) -> Result<()> {
            self.executed_operations.push(ComponentValue::String("Component operation result".into())).unwrap();
            Ok(())
        }

        fn on_resource_access(&mut self, handle: u32) -> Result<()> {
            self.executed_operations.push(ComponentValue::String("Component operation result".into())).unwrap();
            Ok(())
        }

        fn on_resource_operation(
            &mut self,
            handle: u32,
            _operation: &FormatResourceOperation,
        ) -> Result<()> {
            self.executed_operations.push(ComponentValue::String("Component operation result".into())).unwrap();
            Ok(())
        }

        fn intercept_resource_operation(
            &mut self,
            handle: u32,
            _operation: &FormatResourceOperation,
        ) -> Result<Option<BoundedVec<u8, MAX_BUFFERS_PER_CLASS>>> {
            self.executed_operations.push(ComponentValue::String("Component operation result".into())).unwrap();

            // Special case for testing
            if handle == 42 {
                let mut vec = BoundedVec::new();
                vec.push(1).unwrap();
                vec.push(2).unwrap();
                vec.push(3).unwrap();
                Ok(Some(vec))
            } else {
                Ok(None)
            }
        }

        fn get_memory_strategy(&self, handle: u32) -> Option<u8> {
            if handle % 2 == 0 {
                Some(1) // BoundedCopy for even handles
            } else {
                None
            }
        }
    }

    #[test]
    fn test_resource_creation() {
        let mut table = ResourceTable::new();
        let data = Box::new(42i32);

        let handle = table.create_resource(1, data).unwrap();
        assert_eq!(handle, 1);
        assert_eq!(table.resource_count(), 1);

        let resource = table.get_resource(handle).unwrap();
        let resource = resource.lock().unwrap();
        assert_eq!(resource.type_idx, 1);

        let data = resource.data.downcast_ref::<i32>().unwrap();
        assert_eq!(*data, 42);
    }

    #[test]
    fn test_resource_dropping() {
        let mut table = ResourceTable::new();
        let data = Box::new(42i32);

        let handle = table.create_resource(1, data).unwrap();
        assert_eq!(table.resource_count(), 1);

        table.drop_resource(handle).unwrap();
        assert_eq!(table.resource_count(), 0);

        assert!(table.get_resource(handle).is_err());
    }

    #[test]
    fn test_memory_strategy() {
        let mut table = ResourceTable::new();
        let data = Box::new(42i32);

        let handle = table.create_resource(1, data).unwrap();

        // Default strategy is BoundedCopy
        table.set_memory_strategy(handle, MemoryStrategy::ZeroCopy).unwrap();

        // Invalid handle should fail
        assert!(table.set_memory_strategy(999, MemoryStrategy::ZeroCopy).is_err());
    }

    #[test]
    fn test_resource_count_limit() {
        let mut table = ResourceTable::new();

        // Create MAX_RESOURCES resources
        for i in 0..MAX_RESOURCES {
            let data = Box::new(i);
            let _ = table.create_resource(1, data).unwrap();
        }

        // Try to create one more - should fail
        let data = Box::new(100);
        assert!(table.create_resource(1, data).is_err());
    }

    #[test]
    fn test_apply_operation() {
        let mut table = ResourceTable::new();
        let data = Box::new(42i32);

        let handle = table.create_resource(1, data).unwrap();

        // Test Rep operation
        let result = table
            .apply_operation(
                handle,
                FormatResourceOperation::Rep(wrt_foundation::resource::ResourceRep { type_idx: 1 }),
            )
            .unwrap();

        if let ComponentValue::U32(h) = result {
            assert_eq!(h, handle);
        } else {
            panic!("Expected U32 result");
        }

        // Test Borrow operation
        let result = table.apply_operation(handle, FormatResourceOperation::Borrow).unwrap();
        if let ComponentValue::U32(h) = result {
            assert_eq!(h, handle);
        } else {
            panic!("Expected U32 result");
        }

        // Test Drop operation
        let result = table
            .apply_operation(
                handle,
                FormatResourceOperation::Drop(wrt_foundation::resource::ResourceDrop {
                    type_idx: 1,
                }),
            )
            .unwrap();

        assert!(matches!(result, ComponentValue::Void));

        // Resource should be dropped now
        assert_eq!(table.resource_count(), 0);
    }

    #[test]
    fn test_resource_interceptor() {
        let mut table = ResourceTable::new();
        let interceptor = Box::new(TestInterceptor::new());

        table.add_interceptor(interceptor).unwrap();

        let data = Box::new(42i32);
        let handle = table.create_resource(1, data).unwrap();

        // Access the resource
        let _resource = table.get_resource(handle).unwrap();

        // Apply an operation
        table
            .apply_operation(
                handle,
                FormatResourceOperation::Rep(wrt_foundation::resource::ResourceRep { type_idx: 1 }),
            )
            .unwrap();

        // Resource should exist and interceptor should have been called
        assert!(table.find_resource_index(handle).is_some());
    }

    #[test]
    fn test_interceptor_strategy() {
        let mut table = ResourceTable::new();
        let interceptor = Box::new(TestInterceptor::new());

        table.add_interceptor(interceptor).unwrap();

        // Create resources with even and odd handles
        let handle1 = 1; // Odd
        let handle2 = 2; // Even

        // Check strategy selection
        let strategy1 = table.get_strategy_from_interceptors(handle1);
        let strategy2 = table.get_strategy_from_interceptors(handle2);

        assert_eq!(strategy1, None);
        assert_eq!(strategy2, Some(MemoryStrategy::BoundedCopy));
    }
}
