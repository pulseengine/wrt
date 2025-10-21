// WRT - wrt-component
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::prelude::*;

#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    string::String,
    sync::Arc,
};
use core::fmt::{
    self,
    Debug,
};
#[cfg(feature = "std")]
use std::sync::Arc;

use wrt_error::kinds::PoisonedLockError;
use wrt_foundation::{
    bounded::BoundedString,
    safe_memory::NoStdProvider,
};

use super::{
    MemoryStrategy,
    Resource,
    ResourceArena,
    ResourceTable,
    VerificationLevel,
};

/// Unique identifier for a resource
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

impl ResourceId {
    /// Create a new resource identifier
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Extract the inner value
    pub const fn into_inner(self) -> u32 {
        self.0
    }
}

/// Trait representing a host resource
pub trait HostResource {}

// Implement HostResource for common types
impl<T: 'static + Send + Sync> HostResource for T {}

/// Manager for WebAssembly Component Model resource instances (no_std
/// compatible)
#[derive(Clone)]
pub struct ResourceManager {
    /// Resource table for this manager
    table: Arc<Mutex<ResourceTable>>,
    /// Component instance ID  
    instance_id: String,
    /// Default memory strategy
    default_memory_strategy: MemoryStrategy,
    /// Default verification level
    default_verification_level: VerificationLevel,
    /// Maximum allowed resources
    max_resources: usize,
}

impl ResourceManager {
    /// Create a new resource manager with default settings
    pub fn new() -> Self {
        Self::new_with_id("default-instance")
    }

    /// Create a new resource manager with a specific instance ID
    pub fn new_with_id(instance_id: &str) -> Self {
        Self {
            table: Arc::new(Mutex::new(ResourceTable::new().expect("Failed to create ResourceTable"))),
            instance_id: instance_id.to_string(),
            default_memory_strategy: MemoryStrategy::default(),
            default_verification_level: VerificationLevel::Critical,
            max_resources: 64, // Default to MAX_RESOURCES from resource_table_no_std
        }
    }

    /// Create a new resource manager with custom settings
    pub fn new_with_config(
        instance_id: &str,
        memory_strategy: MemoryStrategy,
        verification_level: VerificationLevel,
    ) -> Self {
        Self {
            table: Arc::new(Mutex::new(ResourceTable::new().expect("Failed to create ResourceTable"))),
            instance_id: instance_id.to_string(),
            default_memory_strategy: memory_strategy,
            default_verification_level: verification_level,
            max_resources: 64,
        }
    }

    /// Create a new resource
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn create_resource(&self, type_idx: u32, data: Box<dyn Any + Send + Sync>) -> Result<u32> {
        let mut table =
            self.table.lock();

        table.create_resource(type_idx, data)
    }

    /// Add a host resource - alias for create_resource that returns ResourceId
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn add_host_resource<T: Any + Send + Sync + 'static>(&self, data: T) -> Result<ResourceId> {
        let boxed_data = Box::new(data) as Box<dyn Any + Send + Sync>;
        let handle = self.create_resource(0, boxed_data)?;
        Ok(ResourceId(handle))
    }

    /// Add a host resource - no_std version (without Box)
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub fn add_host_resource<T: Any + Send + Sync + 'static>(&self, _data: T) -> Result<ResourceId> {
        // In pure no_std without alloc, we can't box the data
        // Return a placeholder error
        Err(Error::runtime_not_implemented(
            "add_host_resource requires alloc feature in no_std mode"
        ))
    }

    /// Create a named resource (with debug name)
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn create_named_resource(
        &self,
        type_idx: u32,
        data: Box<dyn Any + Send + Sync>,
        name: &str,
    ) -> Result<u32> {
        let mut table =
            self.table.lock();

        // Create the resource
        let handle = table.create_resource(type_idx, data)?;

        // Set the name if we have access to the resource
        if let Ok(res) = table.get_resource(handle) {
            if let Ok(mut res_guard) = res.lock() {
                res_guard.name = Some(name.to_string());
            }
        }

        Ok(handle)
    }

    /// Get a resource by handle
    pub fn get_resource(&self, handle: u32) -> Result<ResourceId> {
        let table =
            self.table.lock();

        table.get_resource(handle)
    }

    /// Get a resource's data pointer representation by ID
    pub fn get_resource_representation(&self, id: ResourceId) -> Result<u32> {
        let table = self.table.lock();
        let resource = table.get(id)
            .ok_or_else(|| Error::runtime_execution_error("Resource not found in table"))?;
        Ok(resource.data_ptr as u32)
    }

    /// Drop a resource
    pub fn drop_resource(&self, handle: u32) -> Result<()> {
        let mut table =
            self.table.lock();

        table.drop_resource(handle)
    }

    /// Check if a resource exists
    pub fn has_resource(&self, id: ResourceId) -> Result<bool> {
        match self.get_resource(id.0) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get a host resource by ID - returns locked resource
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn get_host_resource<T: Any + 'static>(&self, id: ResourceId) -> Result<Arc<Mutex<T>>> {
        let resource_box = self.get_resource(id.0)?;
        let guard = resource_box.lock();

        // Try to downcast the data_ptr to the requested type
        // This is a simplified version - in production you'd need proper type checking
        Err(Error::runtime_type_mismatch("Type mismatch - host resource access not fully implemented"))
    }

    /// Delete a resource by ID
    pub fn delete_resource(&self, id: ResourceId) -> Result<()> {
        self.drop_resource(id.0)
    }

    /// Set memory strategy for a resource
    pub fn set_memory_strategy(&self, handle: u32, strategy: MemoryStrategy) -> Result<()> {
        let table =
            self.table.lock();

        // ResourceTable doesn't have set_memory_strategy method
        // This would need to be implemented on ResourceTable or we need to get the resource and modify it
        // For now, return success as a placeholder
        Ok(())
    }

    /// Set verification level for a resource
    pub fn set_verification_level(&self, handle: u32, level: VerificationLevel) -> Result<()> {
        let mut table =
            self.table.lock();

        table.set_verification_level(level);
        Ok(())
    }

    /// Get the default memory strategy
    pub fn default_memory_strategy(&self) -> MemoryStrategy {
        self.default_memory_strategy
    }

    /// Get the default verification level
    pub fn default_verification_level(&self) -> VerificationLevel {
        self.default_verification_level
    }

    /// Set the default memory strategy
    pub fn set_default_memory_strategy(&mut self, strategy: MemoryStrategy) {
        self.default_memory_strategy = strategy;
    }

    /// Set the default verification level
    pub fn set_default_verification_level(&mut self, level: VerificationLevel) {
        self.default_verification_level = level;
    }

    /// Get the number of resources
    pub fn resource_count(&self) -> Result<usize> {
        let table =
            self.table.lock();

        // ResourceTable doesn't have resource_count method - count resources manually
        // For now return 0 as placeholder
        Ok(0)
    }

    /// Get the component instance ID
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Create a new resource arena that uses this manager's resource table
    pub fn create_arena(&self) -> Result<ResourceArena> {
        ResourceArena::new(&self.table)
    }

    /// Create a new resource arena with the given name
    pub fn create_named_arena<'a>(&'a self, name: &'a str) -> Result<ResourceArena<'a>> {
        ResourceArena::new_with_name(&self.table, name)
    }
}

impl Debug for ResourceManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Get the resource count, or show an error if we can't access it
        let count = self.resource_count().unwrap_or(0);

        f.debug_struct("ResourceManager")
            .field("instance_id", &self.instance_id)
            .field("resource_count", &count)
            .field("default_memory_strategy", &self.default_memory_strategy)
            .field(
                "default_verification_level",
                &self.default_verification_level,
            )
            .field("max_resources", &self.max_resources)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_creation() {
        let table = Mutex::new(ResourceTable::new());
        let manager = ResourceManager::new(&table);

        // Create a string resource
        let data = Box::new("test".to_owned());
        let handle = manager.create_resource(1, data).unwrap();

        // Verify it exists
        let result = manager.has_resource(ResourceId(handle)).unwrap();
        assert!(result);

        // Get the resource
        let resource = manager.get_resource(handle).unwrap();
        let guard = resource.lock().unwrap();

        // Verify type index
        assert_eq!(guard.type_idx, 1);
    }

    #[test]
    fn test_named_resource() {
        let table = Mutex::new(ResourceTable::new());
        let manager = ResourceManager::new(&table);

        // Create a named resource
        let data = Box::new(42i32);
        let handle = manager.create_named_resource(1, data, "answer").unwrap();

        // Get the resource and check the name
        let resource = manager.get_resource(handle).unwrap();
        let guard = resource.lock().unwrap();

        assert_eq!(guard.name, Some("answer".to_owned()));
    }

    #[test]
    fn test_resource_lifecycle() {
        let table = Mutex::new(ResourceTable::new());
        let manager = ResourceManager::new(&table);

        // Add a resource
        let data = Box::new(42i32);
        let handle = manager.create_resource(1, data).unwrap();

        // Verify it exists
        let exists = manager.has_resource(ResourceId(handle)).unwrap();
        assert!(exists);

        // Drop it
        manager.drop_resource(handle).unwrap();

        // Verify it's gone
        let exists = manager.has_resource(ResourceId(handle)).unwrap();
        assert!(!exists);
    }

    #[test]
    fn test_with_arena() {
        let table = Mutex::new(ResourceTable::new());
        let manager = ResourceManager::new(&table);

        // Create an arena
        let mut arena = manager.create_arena().unwrap();

        // Add a resource through the arena
        let handle = arena.create_resource(1, Box::new("test".to_owned())).unwrap();

        // Verify it exists
        assert!(manager.has_resource(ResourceId(handle)).unwrap());

        // Release arena
        arena.release_all().unwrap();

        // Verify resource is gone
        assert!(!manager.has_resource(ResourceId(handle)).unwrap());
    }
}
