//! Resource Lifecycle Management for WebAssembly Component Model
//!
//! This module implements comprehensive resource lifecycle management with
//! drop handlers, lifetime validation, and automatic cleanup for the Component Model.

#[cfg(not(feature = "std"))]
use core::{fmt, mem, ptr};
#[cfg(feature = "std")]
use std::{fmt, mem, ptr};

#[cfg(feature = "std")]
use std::{boxed::Box, vec::Vec};

use wrt_foundation::{
    bounded::{BoundedVec, BoundedString},
    prelude::*,
};

use crate::{
    async_types::{StreamHandle, FutureHandle},
    types::{ValType, Value},
    WrtResult,
};

use wrt_error::{Error, ErrorCategory, Result};

/// Maximum number of resources in no_std environments
const MAX_RESOURCES: usize = 1024;

/// Maximum number of drop handlers per resource
const MAX_DROP_HANDLERS: usize = 8;

/// Maximum call stack depth for drop operations
const MAX_DROP_STACK_DEPTH: usize = 32;

/// Resource lifecycle manager
#[derive(Debug)]
pub struct ResourceLifecycleManager {
    /// Active resources
    #[cfg(feature = "std")]
    resources: Vec<ResourceEntry>,
    #[cfg(not(any(feature = "std", )))]
    resources: BoundedVec<ResourceEntry, MAX_RESOURCES, NoStdProvider<65536>>,
    
    /// Drop handlers registry
    #[cfg(feature = "std")]
    drop_handlers: Vec<DropHandler>,
    #[cfg(not(any(feature = "std", )))]
    drop_handlers: BoundedVec<DropHandler, MAX_RESOURCES, NoStdProvider<65536>>,
    
    /// Lifecycle policies
    policies: LifecyclePolicies,
    
    /// Statistics
    stats: LifecycleStats,
    
    /// Next resource ID
    next_resource_id: u32,
    
    /// GC state
    gc_state: GarbageCollectionState,
}

/// Entry for a managed resource
#[derive(Debug, Clone)]
pub struct ResourceEntry {
    /// Resource ID
    pub id: ResourceId,
    /// Resource type
    pub resource_type: ResourceType,
    /// Current state
    pub state: ResourceState,
    /// Reference count
    pub ref_count: u32,
    /// Owning component
    pub owner: ComponentId,
    /// Associated handlers
    #[cfg(feature = "std")]
    pub handlers: Vec<DropHandlerId>,
    #[cfg(not(any(feature = "std", )))]
    pub handlers: BoundedVec<DropHandlerId, MAX_DROP_HANDLERS, NoStdProvider<65536>>,
    /// Creation time (for debugging)
    pub created_at: u64,
    /// Last access time (for GC)
    pub last_access: u64,
    /// Resource metadata
    pub metadata: ResourceMetadata,
}

/// Drop handler for resource cleanup
#[derive(Debug, Clone)]
pub struct DropHandler {
    /// Handler ID
    pub id: DropHandlerId,
    /// Resource type this handler applies to
    pub resource_type: ResourceType,
    /// Handler function
    pub handler_fn: DropHandlerFunction,
    /// Priority (lower number = higher priority)
    pub priority: u8,
    /// Whether handler is required for cleanup
    pub required: bool,
}

/// Resource lifecycle policies
#[derive(Debug, Clone)]
pub struct LifecyclePolicies {
    /// Enable automatic garbage collection
    pub enable_gc: bool,
    /// GC interval in milliseconds
    pub gc_interval_ms: u64,
    /// Maximum resource lifetime before forced cleanup (ms)
    pub max_lifetime_ms: Option<u64>,
    /// Enable strict reference counting
    pub strict_ref_counting: bool,
    /// Enable resource leak detection
    pub leak_detection: bool,
    /// Maximum memory usage before triggering cleanup
    pub max_memory_bytes: Option<usize>,
}

/// Garbage collection state
#[derive(Debug, Clone)]
pub struct GarbageCollectionState {
    /// Last GC run time
    pub last_gc_time: u64,
    /// Number of GC cycles
    pub gc_cycles: u64,
    /// Resources collected in last cycle
    pub last_collected: u32,
    /// Whether GC is currently running
    pub gc_running: bool,
}

/// Lifecycle statistics
#[derive(Debug, Clone)]
pub struct LifecycleStats {
    /// Total resources created
    pub resources_created: u64,
    /// Total resources destroyed
    pub resources_destroyed: u64,
    /// Current active resources
    pub active_resources: u32,
    /// Total drop handlers executed
    pub drop_handlers_executed: u64,
    /// Total memory used by resources
    pub memory_used_bytes: usize,
    /// Number of resource leaks detected
    pub leaks_detected: u32,
}

/// Resource type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// Stream resource
    Stream,
    /// Future resource
    Future,
    /// Memory buffer
    MemoryBuffer,
    /// File handle
    FileHandle,
    /// Network connection
    NetworkConnection,
    /// Custom user-defined resource
    Custom(u32),
}

/// Resource state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceState {
    /// Resource is being created
    Creating,
    /// Resource is active and usable
    Active,
    /// Resource is being destroyed
    Destroying,
    /// Resource has been destroyed
    Destroyed,
    /// Resource is in error state
    Error,
}

/// Resource metadata
#[derive(Debug, Clone)]
pub struct ResourceMetadata {
    /// Resource name for debugging
    pub name: BoundedString<64, NoStdProvider<65536>>,
    /// Resource size in bytes
    pub size_bytes: usize,
    /// Tags for categorization
    #[cfg(feature = "std")]
    pub tags: Vec<BoundedString<32, NoStdProvider<65536>>>,
    #[cfg(not(any(feature = "std", )))]
    pub tags: BoundedVec<BoundedString<32, NoStdProvider<65536>>, 8, NoStdProvider<65536>>,
    /// Additional properties
    #[cfg(feature = "std")]
    pub properties: Vec<(BoundedString<32, NoStdProvider<65536>>, Value)>,
    #[cfg(not(any(feature = "std", )))]
    pub properties: BoundedVec<(BoundedString<32, NoStdProvider<65536>>, Value), 16>,
}

/// Drop handler function type
#[derive(Debug, Clone)]
pub enum DropHandlerFunction {
    /// Stream cleanup
    StreamCleanup,
    /// Future cleanup
    FutureCleanup,
    /// Memory cleanup
    MemoryCleanup,
    /// Custom cleanup function
    Custom {
        name: BoundedString<64, NoStdProvider<65536>>,
        // In a real implementation, this would be a function pointer
        placeholder: u32,
    },
}

/// Resource creation request
#[derive(Debug, Clone)]
pub struct ResourceCreateRequest {
    /// Resource type
    pub resource_type: ResourceType,
    /// Initial metadata
    pub metadata: ResourceMetadata,
    /// Owning component
    pub owner: ComponentId,
    /// Custom drop handlers
    #[cfg(feature = "std")]
    pub custom_handlers: Vec<DropHandlerFunction>,
    #[cfg(not(any(feature = "std", )))]
    pub custom_handlers: BoundedVec<DropHandlerFunction, MAX_DROP_HANDLERS, NoStdProvider<65536>>,
}

/// Drop operation result
#[derive(Debug, Clone)]
pub enum DropResult {
    /// Drop completed successfully
    Success,
    /// Drop deferred (will be retried)
    Deferred,
    /// Drop failed with error
    Failed(Error),
    /// Drop skipped (resource already cleaned up)
    Skipped,
}

/// Garbage collection result
#[derive(Debug, Clone)]
pub struct GcResult {
    /// Number of resources collected
    pub collected_count: u32,
    /// Memory freed in bytes
    pub memory_freed_bytes: usize,
    /// Time taken for GC (microseconds)
    pub gc_time_us: u64,
    /// Whether full GC was performed
    pub full_gc: bool,
}

/// Resource ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

/// Drop handler ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DropHandlerId(pub u32);

/// Component ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub u32);

impl ResourceLifecycleManager {
    /// Create new resource lifecycle manager
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "std")]
            resources: Vec::new(),
            #[cfg(not(any(feature = "std", )))]
            resources: BoundedVec::new(DefaultMemoryProvider::default()).unwrap(),
            #[cfg(feature = "std")]
            drop_handlers: Vec::new(),
            #[cfg(not(any(feature = "std", )))]
            drop_handlers: BoundedVec::new(DefaultMemoryProvider::default()).unwrap(),
            policies: LifecyclePolicies::default(),
            stats: LifecycleStats::new(),
            next_resource_id: 1,
            gc_state: GarbageCollectionState::new(),
        }
    }

    /// Create new resource lifecycle manager with custom policies
    pub fn with_policies(policies: LifecyclePolicies) -> Self {
        let mut manager = Self::new();
        manager.policies = policies;
        manager
    }

    /// Create a new resource
    pub fn create_resource(&mut self, request: ResourceCreateRequest) -> Result<ResourceId> {
        let resource_id = ResourceId(self.next_resource_id);
        self.next_resource_id += 1;

        // Register drop handlers for this resource
        #[cfg(feature = "std")]
        let mut handler_ids = Vec::new();
        #[cfg(not(any(feature = "std", )))]
        let mut handler_ids = BoundedVec::<DropHandlerId, MAX_DROP_HANDLERS>::new();
        
        for handler_fn in request.custom_handlers.iter() {
            let handler_id = self.register_drop_handler(
                request.resource_type,
                handler_fn.clone(),
                0, // Default priority
                false, // Not required
            )?;
            #[cfg(feature = "std")]
            handler_ids.push(handler_id);
            #[cfg(not(any(feature = "std", )))]
            handler_ids.push(handler_id).map_err(|_| {
                Error::new(
                    ErrorCategory::Resource,
                    wrt_error::codes::RESOURCE_EXHAUSTED,
                    "Too many handlers for resource"
                )
            })?;
        }

        let entry = ResourceEntry {
            id: resource_id,
            resource_type: request.resource_type,
            state: ResourceState::Creating,
            ref_count: 1,
            owner: request.owner,
            handlers: handler_ids,
            created_at: self.get_current_time(),
            last_access: self.get_current_time(),
            metadata: request.metadata,
        };

        self.resources.push(entry).map_err(|_| {
            Error::new(
                ErrorCategory::Resource,
                wrt_error::codes::RESOURCE_EXHAUSTED,
                "Too many active resources"
            )
        })?;

        // Update statistics
        self.stats.resources_created += 1;
        self.stats.active_resources += 1;
        self.stats.memory_used_bytes += self.get_resource(resource_id)?.metadata.size_bytes;

        // Mark resource as active
        self.update_resource_state(resource_id, ResourceState::Active)?;

        Ok(resource_id)
    }

    /// Add a reference to a resource
    pub fn add_reference(&mut self, resource_id: ResourceId) -> Result<u32> {
        let resource = self.get_resource_mut(resource_id)?;
        
        if resource.state != ResourceState::Active {
            return Err(Error::new(
                ErrorCategory::Runtime,
                wrt_error::codes::EXECUTION_ERROR,
                "Cannot reference inactive resource"
            ));
        }

        resource.ref_count += 1;
        resource.last_access = self.get_current_time();
        Ok(resource.ref_count)
    }

    /// Remove a reference from a resource
    pub fn remove_reference(&mut self, resource_id: ResourceId) -> Result<u32> {
        let should_drop = {
            let resource = self.get_resource_mut(resource_id)?;
            
            if resource.ref_count == 0 {
                return Err(Error::new(
                    ErrorCategory::Runtime,
                    wrt_error::codes::EXECUTION_ERROR,
                    "Resource has no references to remove"
                ));
            }

            resource.ref_count -= 1;
            resource.last_access = self.get_current_time();
            
            resource.ref_count == 0
        };

        if should_drop {
            self.drop_resource(resource_id)?;
        }

        Ok(self.get_resource(resource_id)?.ref_count)
    }

    /// Drop a resource immediately
    pub fn drop_resource(&mut self, resource_id: ResourceId) -> Result<DropResult> {
        let resource = self.get_resource_mut(resource_id)?;
        
        if resource.state == ResourceState::Destroyed {
            return Ok(DropResult::Skipped);
        }

        // Mark as destroying
        resource.state = ResourceState::Destroying;

        // Execute drop handlers
        #[cfg(feature = "std")]
        let handler_ids: Vec<DropHandlerId> = resource.handlers.iter().cloned().collect();
        #[cfg(not(any(feature = "std", )))]
        let handler_ids = resource.handlers.clone();
        
        for handler_id in handler_ids {
            let result = self.execute_drop_handler(handler_id, resource_id)?;
            if let DropResult::Failed(error) = result {
                // If a required handler fails, mark resource as error state
                if self.is_handler_required(handler_id)? {
                    self.update_resource_state(resource_id, ResourceState::Error)?;
                    return Ok(DropResult::Failed(error));
                }
            }
        }

        // Update statistics
        self.stats.resources_destroyed += 1;
        self.stats.active_resources -= 1;
        self.stats.memory_used_bytes -= self.get_resource(resource_id)?.metadata.size_bytes;

        // Mark as destroyed
        self.update_resource_state(resource_id, ResourceState::Destroyed)?;

        Ok(DropResult::Success)
    }

    /// Register a drop handler
    pub fn register_drop_handler(
        &mut self,
        resource_type: ResourceType,
        handler_fn: DropHandlerFunction,
        priority: u8,
        required: bool,
    ) -> Result<DropHandlerId> {
        let handler_id = DropHandlerId(self.drop_handlers.len() as u32);
        
        let handler = DropHandler {
            id: handler_id,
            resource_type,
            handler_fn,
            priority,
            required,
        };

        self.drop_handlers.push(handler).map_err(|_| {
            Error::new(
                ErrorCategory::Resource,
                wrt_error::codes::RESOURCE_EXHAUSTED,
                "Too many drop handlers"
            )
        })?;

        Ok(handler_id)
    }

    /// Run garbage collection
    pub fn run_garbage_collection(&mut self, force_full_gc: bool) -> Result<GcResult> {
        if self.gc_state.gc_running {
            return Err(Error::new(
                ErrorCategory::Runtime,
                wrt_error::codes::EXECUTION_ERROR,
                "Garbage collection already running"
            ));
        }

        let start_time = self.get_current_time();
        self.gc_state.gc_running = true;
        
        let mut collected_count = 0;
        let mut memory_freed = 0;

        // Find resources to collect
        #[cfg(feature = "std")]
        let mut resources_to_drop = Vec::new();
        #[cfg(not(any(feature = "std", )))]
        let mut resources_to_drop = BoundedVec::<ResourceId, 64>::new();
        
        for resource in &self.resources {
            let should_collect = if force_full_gc {
                resource.ref_count == 0
            } else {
                resource.ref_count == 0 && self.should_collect_resource(resource)
            };
            
            if should_collect {
                let _ = resources_to_drop.push(resource.id);
            }
        }

        // Drop collected resources
        for resource_id in &resources_to_drop {
            if let Ok(resource) = self.get_resource(*resource_id) {
                memory_freed += resource.metadata.size_bytes;
            }
            
            if self.drop_resource(*resource_id).is_ok() {
                collected_count += 1;
            }
        }

        // Remove destroyed resources from list
        #[cfg(feature = "std")]
        {
            self.resources.retain(|r| r.state != ResourceState::Destroyed);
        }
        #[cfg(not(any(feature = "std", )))]
        {
            let mut i = 0;
            while i < self.resources.len() {
                if self.resources[i].state == ResourceState::Destroyed {
                    self.resources.remove(i);
                } else {
                    i += 1;
                }
            }
        }

        // Update GC state
        let gc_time = self.get_current_time() - start_time;
        self.gc_state.gc_running = false;
        self.gc_state.last_gc_time = self.get_current_time();
        self.gc_state.gc_cycles += 1;
        self.gc_state.last_collected = collected_count;

        Ok(GcResult {
            collected_count,
            memory_freed_bytes: memory_freed,
            gc_time_us: gc_time,
            full_gc: force_full_gc,
        })
    }

    /// Get resource by ID
    pub fn get_resource(&self, resource_id: ResourceId) -> Result<&ResourceEntry> {
        self.resources
            .iter()
            .find(|r| r.id == resource_id)
            .ok_or_else(|| {
                Error::new(
                    ErrorCategory::Runtime,
                    wrt_error::codes::EXECUTION_ERROR,
                    "Resource not found"
                )
            })
    }

    /// Get mutable resource by ID
    pub fn get_resource_mut(&mut self, resource_id: ResourceId) -> Result<&mut ResourceEntry> {
        self.resources
            .iter_mut()
            .find(|r| r.id == resource_id)
            .ok_or_else(|| {
                Error::new(
                    ErrorCategory::Runtime,
                    wrt_error::codes::EXECUTION_ERROR,
                    "Resource not found"
                )
            })
    }

    /// Get lifecycle statistics
    pub fn get_stats(&self) -> &LifecycleStats {
        &self.stats
    }

    /// Get current policies
    pub fn get_policies(&self) -> &LifecyclePolicies {
        &self.policies
    }

    /// Update lifecycle policies
    pub fn update_policies(&mut self, policies: LifecyclePolicies) {
        self.policies = policies;
    }

    /// Check for resource leaks
    #[cfg(feature = "std")]
    pub fn check_for_leaks(&mut self) -> Result<Vec<ResourceId>> {
        if !self.policies.leak_detection {
            return Ok(Vec::new());
        }

        let mut leaked_resources = Vec::new();
        let current_time = self.get_current_time();

        for resource in &self.resources {
            if let Some(max_lifetime) = self.policies.max_lifetime_ms {
                let age_ms = current_time - resource.created_at;
                if age_ms > max_lifetime && resource.ref_count > 0 {
                    let _ = leaked_resources.push(resource.id);
                }
            }
        }

        self.stats.leaks_detected += leaked_resources.len() as u32;
        Ok(leaked_resources)
    }
    
    /// Check for resource leaks (no_std version)
    #[cfg(not(any(feature = "std", )))]
    pub fn check_for_leaks(&mut self) -> Result<BoundedVec<ResourceId, 64>, NoStdProvider<65536>> {
        if !self.policies.leak_detection {
            return Ok(BoundedVec::new(DefaultMemoryProvider::default()).unwrap());
        }

        let mut leaked_resources = BoundedVec::new(DefaultMemoryProvider::default()).unwrap();
        let current_time = self.get_current_time();

        for resource in &self.resources {
            if let Some(max_lifetime) = self.policies.max_lifetime_ms {
                let age_ms = current_time - resource.created_at;
                if age_ms > max_lifetime && resource.ref_count > 0 {
                    let _ = leaked_resources.push(resource.id);
                }
            }
        }

        self.stats.leaks_detected += leaked_resources.len() as u32;
        Ok(leaked_resources)
    }

    // Private helper methods

    fn update_resource_state(&mut self, resource_id: ResourceId, new_state: ResourceState) -> Result<()> {
        let resource = self.get_resource_mut(resource_id)?;
        resource.state = new_state;
        Ok(())
    }

    fn execute_drop_handler(&mut self, handler_id: DropHandlerId, resource_id: ResourceId) -> Result<DropResult> {
        let handler = self.drop_handlers
            .iter()
            .find(|h| h.id == handler_id)
            .ok_or_else(|| {
                Error::new(
                    ErrorCategory::Runtime,
                    wrt_error::codes::EXECUTION_ERROR,
                    "Drop handler not found"
                )
            })?;

        // Simplified handler execution - in real implementation this would
        // call the actual drop handler function
        match &handler.handler_fn {
            DropHandlerFunction::StreamCleanup => {
                // Simulate stream cleanup
                self.stats.drop_handlers_executed += 1;
                Ok(DropResult::Success)
            }
            DropHandlerFunction::FutureCleanup => {
                // Simulate future cleanup
                self.stats.drop_handlers_executed += 1;
                Ok(DropResult::Success)
            }
            DropHandlerFunction::MemoryCleanup => {
                // Simulate memory cleanup
                self.stats.drop_handlers_executed += 1;
                Ok(DropResult::Success)
            }
            DropHandlerFunction::Custom { .. } => {
                // Simulate custom cleanup
                self.stats.drop_handlers_executed += 1;
                Ok(DropResult::Success)
            }
        }
    }

    fn is_handler_required(&self, handler_id: DropHandlerId) -> Result<bool> {
        let handler = self.drop_handlers
            .iter()
            .find(|h| h.id == handler_id)
            .ok_or_else(|| {
                Error::new(
                    ErrorCategory::Runtime,
                    wrt_error::codes::EXECUTION_ERROR,
                    "Drop handler not found"
                )
            })?;
        
        Ok(handler.required)
    }

    fn should_collect_resource(&self, resource: &ResourceEntry) -> bool {
        if resource.ref_count > 0 {
            return false;
        }

        // Check age
        if let Some(max_lifetime) = self.policies.max_lifetime_ms {
            let age = self.get_current_time() - resource.created_at;
            if age > max_lifetime {
                return true;
            }
        }

        // Check last access time
        let idle_time = self.get_current_time() - resource.last_access;
        idle_time > 60000 // 1 minute idle time
    }

    fn get_current_time(&self) -> u64 {
        // Simplified time implementation - in real implementation would use proper time
        self.gc_state.gc_cycles * 1000 // Simulate time progression
    }
}

impl ResourceMetadata {
    /// Create new resource metadata
    pub fn new(name: &str) -> Self {
        Self {
            name: BoundedString::from_str(name).unwrap_or_default(),
            size_bytes: 0,
            #[cfg(feature = "std")]
            tags: Vec::new(),
            #[cfg(not(any(feature = "std", )))]
            tags: BoundedVec::new(DefaultMemoryProvider::default()).unwrap(),
            #[cfg(feature = "std")]
            properties: Vec::new(),
            #[cfg(not(any(feature = "std", )))]
            properties: BoundedVec::new(DefaultMemoryProvider::default()).unwrap(),
        }
    }

    /// Add a tag to the metadata
    pub fn add_tag(&mut self, tag: &str) -> Result<()> {
        let bounded_tag = BoundedString::from_str(tag).map_err(|_| {
            Error::new(
                ErrorCategory::Parse,
                wrt_error::codes::PARSE_ERROR,
                "Tag too long"
            )
        })?;
        
        self.tags.push(bounded_tag).map_err(|_| {
            Error::new(
                ErrorCategory::Resource,
                wrt_error::codes::RESOURCE_EXHAUSTED,
                "Too many tags"
            )
        })
    }

    /// Add a property to the metadata
    pub fn add_property(&mut self, key: &str, value: Value) -> Result<()> {
        let bounded_key = BoundedString::from_str(key).map_err(|_| {
            Error::new(
                ErrorCategory::Parse,
                wrt_error::codes::PARSE_ERROR,
                "Property key too long"
            )
        })?;
        
        self.properties.push((bounded_key, value)).map_err(|_| {
            Error::new(
                ErrorCategory::Resource,
                wrt_error::codes::RESOURCE_EXHAUSTED,
                "Too many properties"
            )
        })
    }
}

impl Default for LifecyclePolicies {
    fn default() -> Self {
        Self {
            enable_gc: true,
            gc_interval_ms: 10000, // 10 seconds
            max_lifetime_ms: Some(3600000), // 1 hour
            strict_ref_counting: true,
            leak_detection: true,
            max_memory_bytes: Some(100 * 1024 * 1024), // 100MB
        }
    }
}

impl LifecycleStats {
    /// Create new lifecycle statistics
    pub fn new() -> Self {
        Self {
            resources_created: 0,
            resources_destroyed: 0,
            active_resources: 0,
            drop_handlers_executed: 0,
            memory_used_bytes: 0,
            leaks_detected: 0,
        }
    }
}

impl GarbageCollectionState {
    /// Create new GC state
    pub fn new() -> Self {
        Self {
            last_gc_time: 0,
            gc_cycles: 0,
            last_collected: 0,
            gc_running: false,
        }
    }
}

impl Default for ResourceLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Stream => write!(f, "stream"),
            ResourceType::Future => write!(f, "future"),
            ResourceType::MemoryBuffer => write!(f, "memory-buffer"),
            ResourceType::FileHandle => write!(f, "file-handle"),
            ResourceType::NetworkConnection => write!(f, "network-connection"),
            ResourceType::Custom(id) => write!(f, "custom-{}", id),
        }
    }
}

impl fmt::Display for ResourceState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceState::Creating => write!(f, "creating"),
            ResourceState::Active => write!(f, "active"),
            ResourceState::Destroying => write!(f, "destroying"),
            ResourceState::Destroyed => write!(f, "destroyed"),
            ResourceState::Error => write!(f, "error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_lifecycle_manager_creation() {
        let manager = ResourceLifecycleManager::new();
        assert_eq!(manager.resources.len(), 0);
        assert_eq!(manager.stats.active_resources, 0);
        assert_eq!(manager.next_resource_id, 1);
    }

    #[test]
    fn test_create_resource() {
        let mut manager = ResourceLifecycleManager::new();
        
        let request = ResourceCreateRequest {
            resource_type: ResourceType::Stream,
            metadata: ResourceMetadata::new("test-stream"),
            owner: ComponentId(1),
            #[cfg(feature = "std")]
            custom_handlers: Vec::new(),
            #[cfg(not(any(feature = "std", )))]
            custom_handlers: BoundedVec::new(DefaultMemoryProvider::default()).unwrap(),
        };
        
        let resource_id = manager.create_resource(request).unwrap();
        assert_eq!(resource_id.0, 1);
        assert_eq!(manager.stats.resources_created, 1);
        assert_eq!(manager.stats.active_resources, 1);
    }

    #[test]
    fn test_reference_counting() {
        let mut manager = ResourceLifecycleManager::new();
        
        let request = ResourceCreateRequest {
            resource_type: ResourceType::Future,
            metadata: ResourceMetadata::new("test-future"),
            owner: ComponentId(1),
            #[cfg(feature = "std")]
            custom_handlers: Vec::new(),
            #[cfg(not(any(feature = "std", )))]
            custom_handlers: BoundedVec::new(DefaultMemoryProvider::default()).unwrap(),
        };
        
        let resource_id = manager.create_resource(request).unwrap();
        
        // Add reference
        let ref_count = manager.add_reference(resource_id).unwrap();
        assert_eq!(ref_count, 2);
        
        // Remove reference
        let ref_count = manager.remove_reference(resource_id).unwrap();
        assert_eq!(ref_count, 1);
        
        // Remove last reference should drop resource
        let ref_count = manager.remove_reference(resource_id).unwrap();
        assert_eq!(ref_count, 0);
        
        let resource = manager.get_resource(resource_id).unwrap();
        assert_eq!(resource.state, ResourceState::Destroyed);
    }

    #[test]
    fn test_drop_handler_registration() {
        let mut manager = ResourceLifecycleManager::new();
        
        let handler_id = manager.register_drop_handler(
            ResourceType::Stream,
            DropHandlerFunction::StreamCleanup,
            0,
            true,
        ).unwrap();
        
        assert_eq!(handler_id.0, 0);
        assert_eq!(manager.drop_handlers.len(), 1);
    }

    #[test]
    fn test_garbage_collection() {
        let mut manager = ResourceLifecycleManager::new();
        
        // Create a resource with zero references
        let request = ResourceCreateRequest {
            resource_type: ResourceType::MemoryBuffer,
            metadata: ResourceMetadata::new("gc-test"),
            owner: ComponentId(1),
            #[cfg(feature = "std")]
            custom_handlers: Vec::new(),
            #[cfg(not(any(feature = "std", )))]
            custom_handlers: BoundedVec::new(DefaultMemoryProvider::default()).unwrap(),
        };
        
        let resource_id = manager.create_resource(request).unwrap();
        manager.remove_reference(resource_id).unwrap(); // Drop to 0 references
        
        let gc_result = manager.run_garbage_collection(true).unwrap();
        assert_eq!(gc_result.collected_count, 1);
        assert!(gc_result.full_gc);
    }

    #[test]
    fn test_resource_metadata() {
        let mut metadata = ResourceMetadata::new("test-resource");
        
        metadata.add_tag("important").unwrap();
        metadata.add_property("version", Value::U32(1)).unwrap();
        
        assert_eq!(metadata.tags.len(), 1);
        assert_eq!(metadata.properties.len(), 1);
    }

    #[test]
    fn test_lifecycle_policies() {
        let policies = LifecyclePolicies::default();
        assert!(policies.enable_gc);
        assert!(policies.strict_ref_counting);
        assert!(policies.leak_detection);
        
        let manager = ResourceLifecycleManager::with_policies(policies);
        assert!(manager.policies.enable_gc);
    }

    #[test]
    fn test_resource_type_display() {
        assert_eq!(ResourceType::Stream.to_string(), "stream");
        assert_eq!(ResourceType::Custom(42).to_string(), "custom-42");
        assert_eq!(ResourceState::Active.to_string(), "active");
    }
}