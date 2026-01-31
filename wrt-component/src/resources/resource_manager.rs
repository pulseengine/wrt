// Resource management for WebAssembly Component Model
//
// This module provides resource management functionality for the Component
// Model, including resource creation, access control, and lifetime management.

use std::sync::Mutex;

use wrt_error::kinds::PoisonedLockError;
use wrt_foundation::{
    ResourceOperation as FormatResourceOperation, component_value::ComponentValue,
};
use wrt_sync::WrtMutex;

use super::{
    MemoryStrategy, Resource, ResourceArena, ResourceInterceptor, ResourceTable, VerificationLevel,
};
use crate::prelude::*;

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

/// Manager for WebAssembly Component Model resource instances
///
/// This struct manages resources for a component instance, providing
/// creation, access, and lifecycle management capabilities.
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
    /// Whether to use optimized memory management
    use_optimized_memory: bool,
}

impl ResourceManager {
    /// Create a new resource manager with default settings
    pub fn new() -> Result<Self> {
        Self::new_with_id("default-instance")
    }

    /// Create a new resource manager with optimized memory management
    pub fn new_optimized() -> Result<Self> {
        Self::new_with_id_and_optimized_memory("default-instance")
    }

    /// Create a new resource manager with a specific instance ID
    pub fn new_with_id(instance_id: &str) -> Result<Self> {
        Ok(Self {
            table: Arc::new(Mutex::new(ResourceTable::new()?)),
            instance_id: instance_id.to_string(),
            default_memory_strategy: MemoryStrategy::default(),
            default_verification_level: VerificationLevel::Critical,
            max_resources: 1024,
            use_optimized_memory: false,
        })
    }

    /// Create a new resource manager with a specific instance ID and optimized
    /// memory
    pub fn new_with_id_and_optimized_memory(instance_id: &str) -> Result<Self> {
        Ok(Self {
            table: Arc::new(Mutex::new(ResourceTable::new_with_optimized_memory()?)),
            instance_id: instance_id.to_string(),
            default_memory_strategy: MemoryStrategy::default(),
            default_verification_level: VerificationLevel::Critical,
            max_resources: 1024,
            use_optimized_memory: true,
        })
    }

    /// Create a new resource manager with custom settings
    pub fn new_with_config(
        instance_id: &str,
        max_resources: usize,
        memory_strategy: MemoryStrategy,
        verification_level: VerificationLevel,
    ) -> Self {
        Self {
            table: Arc::new(Mutex::new(ResourceTable::new_with_config(
                max_resources,
                memory_strategy,
                verification_level,
            ))),
            instance_id: instance_id.to_string(),
            default_memory_strategy: memory_strategy,
            default_verification_level: verification_level,
            max_resources,
            use_optimized_memory: false,
        }
    }

    /// Create a new resource manager with custom settings and optimized memory
    pub fn new_with_config_and_optimized_memory(
        instance_id: &str,
        max_resources: usize,
        memory_strategy: MemoryStrategy,
        verification_level: VerificationLevel,
    ) -> Self {
        Self {
            table: Arc::new(Mutex::new(
                ResourceTable::new_with_config_and_optimized_memory(
                    max_resources,
                    memory_strategy,
                    verification_level,
                ),
            )),
            instance_id: instance_id.to_string(),
            default_memory_strategy: memory_strategy,
            default_verification_level: verification_level,
            max_resources,
            use_optimized_memory: true,
        }
    }

    /// Add a resource interceptor
    pub fn add_interceptor(&self, interceptor: Arc<dyn ResourceInterceptor>) -> Result<()> {
        let mut table = self
            .table
            .lock()
            .map_err(|_| Error::runtime_poisoned_lock("Failed to acquire resource table lock"))?;
        table.add_interceptor(interceptor)?;
        Ok(())
    }

    /// Create a new resource
    pub fn create_resource(&self, type_idx: u32, data: Arc<dyn Any + Send + Sync>) -> Result<u32> {
        let mut table = self
            .table
            .lock()
            .map_err(|_| Error::runtime_poisoned_lock("Failed to acquire resource table lock"))?;
        table.create_resource(type_idx, data)
    }

    /// Add a host resource to the manager (legacy API)
    pub fn add_host_resource<T: 'static + Send + Sync>(&self, resource: T) -> Result<ResourceId> {
        let id = self.create_resource(0, Arc::new(resource))?;
        Ok(ResourceId(id))
    }

    /// Create a named resource (with debug name)
    pub fn create_named_resource(
        &self,
        type_idx: u32,
        data: Arc<dyn Any + Send + Sync>,
        name: &str,
    ) -> Result<u32> {
        let mut table = self
            .table
            .lock()
            .map_err(|_| Error::runtime_poisoned_lock("Failed to acquire resource table lock"))?;

        // Create the resource
        let handle = table.create_resource(type_idx, data)?;

        // Set the name if we have access to the resource
        if let Ok(res) = table.get_resource(handle) {
            let mut res_guard = res.lock();
            res_guard.name = Some(name.to_string());
        }

        Ok(handle)
    }

    /// Borrow a resource
    pub fn borrow_resource(&self, handle: u32) -> Result<u32> {
        let mut table = self
            .table
            .lock()
            .map_err(|_| Error::runtime_poisoned_lock("Failed to acquire resource table lock"))?;
        table.borrow_resource(handle)
    }

    /// Get a host resource by ID and type (legacy API)
    pub fn get_host_resource<T: 'static + Send + Sync + Clone>(
        &self,
        id: ResourceId,
    ) -> Result<Arc<Mutex<T>>> {
        // Get the resource from the table
        let resource = self.get_resource(id.0)?;

        // Attempt to downcast to the requested type
        let resource_guard = resource.lock();

        // Check if we can access the data as the requested type
        if let Some(typed_data) = resource_guard.data.downcast_ref::<T>() {
            // Create a cloned Arc<Mutex<T>> to return
            let cloned_data = Arc::new(Mutex::new((*typed_data).clone()));
            Ok(cloned_data)
        } else {
            Err(Error::component_not_found("Resource type mismatch"))
        }
    }

    /// Drop a resource
    pub fn drop_resource(&self, handle: u32) -> Result<()> {
        let mut table = self
            .table
            .lock()
            .map_err(|_| Error::runtime_poisoned_lock("Failed to acquire resource table lock"))?;
        table.drop_resource(handle)
    }

    /// Delete a resource (legacy API)
    pub fn delete_resource(&self, id: ResourceId) -> Result<()> {
        self.drop_resource(id.0)
    }

    /// Get a resource by handle
    pub fn get_resource(&self, handle: u32) -> Result<Arc<WrtMutex<Resource>>> {
        let table = self
            .table
            .lock()
            .map_err(|_| Error::runtime_poisoned_lock("Failed to acquire resource table lock"))?;
        table.get_resource(handle)
    }

    /// Check if a resource exists (legacy API)
    pub fn has_resource(&self, id: ResourceId) -> Result<bool> {
        match self.get_resource(id.0) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Apply an operation to a resource
    pub fn apply_operation(
        &self,
        handle: u32,
        operation: FormatResourceOperation,
    ) -> Result<ComponentValue<ComponentProvider>> {
        let mut table = self
            .table
            .lock()
            .map_err(|_| Error::runtime_poisoned_lock("Failed to acquire resource table lock"))?;
        table.apply_operation(handle, operation)
    }

    /// Set memory strategy for a resource
    pub fn set_memory_strategy(&self, handle: u32, strategy: MemoryStrategy) -> Result<()> {
        let mut table = self
            .table
            .lock()
            .map_err(|_| Error::runtime_poisoned_lock("Failed to acquire resource table lock"))?;
        table.set_memory_strategy(handle, strategy)
    }

    /// Set verification level for a resource
    pub fn set_verification_level(&self, handle: u32, level: VerificationLevel) -> Result<()> {
        let mut table = self
            .table
            .lock()
            .map_err(|_| Error::runtime_poisoned_lock("Failed to acquire resource table lock"))?;
        table.set_verification_level(handle, level)
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
        let table = self
            .table
            .lock()
            .map_err(|_| Error::runtime_poisoned_lock("Failed to acquire resource table lock"))?;
        Ok(table.resource_count())
    }

    /// Clean up unused resources
    pub fn cleanup_unused_resources(&self) -> Result<usize> {
        let mut table = self
            .table
            .lock()
            .map_err(|_| Error::runtime_poisoned_lock("Failed to acquire resource table lock"))?;
        Ok(table.cleanup_unused_resources())
    }

    /// Clear all resources (legacy API)
    pub fn clear(&self) -> Result<()> {
        let mut table = self
            .table
            .lock()
            .map_err(|_| Error::runtime_poisoned_lock("Failed to acquire resource table lock"))?;
        let _ = table.cleanup_unused_resources();
        Ok(())
    }

    /// Get the component instance ID
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Get a reference to the resource table
    pub fn get_resource_table(&self) -> Arc<Mutex<ResourceTable>> {
        Arc::clone(&self.table)
    }

    /// Create a new resource arena that uses this manager's resource table
    pub fn create_arena(&self) -> ResourceArena {
        ResourceArena::new(Arc::clone(&self.table))
    }

    /// Create a new resource arena with the given name
    pub fn create_named_arena(&self, name: &str) -> ResourceArena {
        ResourceArena::new_with_name(Arc::clone(&self.table), name)
    }

    /// Check if this manager is using optimized memory
    pub fn uses_optimized_memory(&self) -> bool {
        self.use_optimized_memory
    }
}

impl fmt::Debug for ResourceManager {
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
            .field("optimized_memory", &self.use_optimized_memory)
            .finish()
    }
}
