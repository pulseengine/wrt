//! Component Model resource type implementation
//!
//! This module provides resource type handling for the WebAssembly Component Model,
//! including resource lifetime management, memory optimization, and interception support.

use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock, Weak};
use wrt_error::{kinds, Error, Result};
use wrt_format::component::{ResourceOperation, ValType};

use crate::values::ComponentValue;

/// Maximum number of resources that can be stored in a resource table
const MAX_RESOURCES: usize = 1024;

/// Resource instance representation
pub struct Resource {
    /// Resource type index
    pub type_idx: u32,
    /// Resource data (implementation-specific)
    pub data: Arc<dyn Any + Send + Sync>,
    /// Debug name for the resource (optional)
    pub name: Option<String>,
    /// Creation timestamp
    pub created_at: std::time::Instant,
    /// Last access timestamp
    pub last_accessed: std::time::Instant,
    /// Access count
    pub access_count: u64,
}

impl Resource {
    /// Create a new resource
    pub fn new(type_idx: u32, data: Arc<dyn Any + Send + Sync>) -> Self {
        let now = std::time::Instant::now();
        Self {
            type_idx,
            data,
            name: None,
            created_at: now,
            last_accessed: now,
            access_count: 0,
        }
    }

    /// Create a new resource with a debug name
    pub fn new_with_name(type_idx: u32, data: Arc<dyn Any + Send + Sync>, name: &str) -> Self {
        let mut resource = Self::new(type_idx, data);
        resource.name = Some(name.to_string());
        resource
    }

    /// Record access to this resource
    pub fn record_access(&mut self) {
        self.last_accessed = std::time::Instant::now();
        self.access_count += 1;
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

/// Resource entry in the resource table
struct ResourceEntry {
    /// The resource instance
    resource: Arc<Mutex<Resource>>,
    /// Weak references to borrowed resources
    borrows: Vec<Weak<Mutex<Resource>>>,
    /// Memory strategy for this resource
    memory_strategy: MemoryStrategy,
    /// Verification level
    verification_level: VerificationLevel,
}

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

/// Resource table for tracking resource instances
pub struct ResourceTable {
    /// Map of resource handles to resource entries
    resources: HashMap<u32, ResourceEntry>,
    /// Next available resource handle
    next_handle: u32,
    /// Maximum allowed resources
    max_resources: usize,
    /// Default memory strategy
    default_memory_strategy: MemoryStrategy,
    /// Default verification level
    default_verification_level: VerificationLevel,
    /// Buffer pool for bounded copy operations
    buffer_pool: BufferPool,
    /// Interceptors for resource operations
    interceptors: Vec<Arc<dyn ResourceInterceptor>>,
}

impl fmt::Debug for ResourceTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceTable")
            .field("resource_count", &self.resources.len())
            .field("next_handle", &self.next_handle)
            .field("max_resources", &self.max_resources)
            .field("default_memory_strategy", &self.default_memory_strategy)
            .field(
                "default_verification_level",
                &self.default_verification_level,
            )
            .field("interceptor_count", &self.interceptors.len())
            .finish()
    }
}

impl ResourceTable {
    /// Create a new resource table with default settings
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            next_handle: 1, // Start at 1 as 0 is reserved
            max_resources: MAX_RESOURCES,
            default_memory_strategy: MemoryStrategy::BoundedCopy,
            default_verification_level: VerificationLevel::Critical,
            buffer_pool: BufferPool::new(4096), // 4KB chunks
            interceptors: Vec::new(),
        }
    }

    /// Create a new resource table with custom settings
    pub fn new_with_config(
        max_resources: usize,
        memory_strategy: MemoryStrategy,
        verification_level: VerificationLevel,
    ) -> Self {
        Self {
            resources: HashMap::new(),
            next_handle: 1,
            max_resources,
            default_memory_strategy: memory_strategy,
            default_verification_level: verification_level,
            buffer_pool: BufferPool::new(4096),
            interceptors: Vec::new(),
        }
    }

    /// Add a resource interceptor
    pub fn add_interceptor(&mut self, interceptor: Arc<dyn ResourceInterceptor>) {
        self.interceptors.push(interceptor);
    }

    /// Create a new resource
    pub fn create_resource(
        &mut self,
        type_idx: u32,
        data: Arc<dyn Any + Send + Sync>,
    ) -> Result<u32> {
        // Check if we've reached the maximum number of resources
        if self.resources.len() >= self.max_resources {
            return Err(Error::resource_limit_exceeded(format!(
                "Maximum number of resources ({}) reached",
                self.max_resources
            )));
        }

        // Create the resource
        let resource = Resource::new(type_idx, data);

        // Notify interceptors about resource creation
        for interceptor in &self.interceptors {
            interceptor.on_resource_create(type_idx, &resource)?;
        }

        // Assign a handle
        let handle = self.next_handle;
        self.next_handle += 1;

        // Store the resource
        self.resources.insert(
            handle,
            ResourceEntry {
                resource: Arc::new(Mutex::new(resource)),
                borrows: Vec::new(),
                memory_strategy: self.default_memory_strategy,
                verification_level: self.default_verification_level,
            },
        );

        Ok(handle)
    }

    /// Create a borrowed reference to a resource
    pub fn borrow_resource(&mut self, handle: u32) -> Result<u32> {
        // Check if the resource exists
        let resource_opt = self
            .resources
            .get(&handle)
            .map(|entry| entry.resource.clone());

        let resource = match resource_opt {
            Some(r) => r,
            None => {
                return Err(Error::resource_access_error(format!(
                    "Resource handle {} not found",
                    handle
                )));
            }
        };

        // Notify interceptors about resource borrowing
        for interceptor in &self.interceptors {
            interceptor.on_resource_borrow(handle)?;
        }

        // Create a new handle for the borrowed resource
        let borrow_handle = self.next_handle;
        self.next_handle += 1;

        // Store the weak reference in the original resource
        let weak_ref = Arc::downgrade(&resource);
        if let Some(entry) = self.resources.get_mut(&handle) {
            entry.borrows.push(weak_ref);
        }

        // Store the borrowed resource
        self.resources.insert(
            borrow_handle,
            ResourceEntry {
                resource,
                borrows: Vec::new(),
                memory_strategy: self.default_memory_strategy,
                verification_level: self.default_verification_level,
            },
        );

        Ok(borrow_handle)
    }

    /// Drop a resource
    pub fn drop_resource(&mut self, handle: u32) -> Result<()> {
        // Check if the resource exists
        if !self.resources.contains_key(&handle) {
            return Err(Error::resource_access_error(format!(
                "Resource handle {} not found",
                handle
            )));
        }

        // Notify interceptors about resource dropping
        for interceptor in &self.interceptors {
            interceptor.on_resource_drop(handle)?;
        }

        // Remove the resource
        self.resources.remove(&handle);

        Ok(())
    }

    /// Get a resource by handle
    pub fn get_resource(&self, handle: u32) -> Result<Arc<Mutex<Resource>>> {
        // Check if the resource exists
        let entry = self.resources.get(&handle).ok_or_else(|| {
            Error::resource_access_error(format!("Resource handle {} not found", handle))
        })?;

        // Record access
        if let Ok(mut resource) = entry.resource.lock() {
            resource.record_access();
        }

        // Notify interceptors about resource access
        for interceptor in &self.interceptors {
            interceptor.on_resource_access(handle)?;
        }

        Ok(entry.resource.clone())
    }

    /// Apply a resource operation
    pub fn apply_operation(
        &mut self,
        handle: u32,
        operation: ResourceOperation,
    ) -> Result<ComponentValue> {
        // Check if the resource exists
        if !self.resources.contains_key(&handle) {
            return Err(Error::resource_access_error(format!(
                "Resource handle {} not found",
                handle
            )));
        }

        // Get the resource entry
        let entry = self.resources.get(&handle).unwrap();

        // Get memory strategy
        let memory_strategy = self
            .get_strategy_from_interceptors(handle)
            .unwrap_or(entry.memory_strategy);

        // Get verification level
        let verification_level = entry.verification_level;

        // Notify interceptors about resource operation
        let mut intercepted_result = None;
        for interceptor in &self.interceptors {
            interceptor.on_resource_operation(handle, &operation)?;

            // Check if any interceptor provides a direct result
            if let Ok(Some(_result_data)) =
                interceptor.intercept_resource_operation(handle, &operation)
            {
                // In a real implementation, this would decode the result data into a ComponentValue
                // For now, return a placeholder value
                intercepted_result = Some(ComponentValue::Bool(true));
                break;
            }
        }

        // If an interceptor provided a direct result, use that
        if let Some(result) = intercepted_result {
            return Ok(result);
        }

        // Record the access
        if let Some(entry) = self.resources.get_mut(&handle) {
            if let Ok(mut resource) = entry.resource.lock() {
                resource.record_access();
            }
        }

        // Apply the operation based on the resource
        match operation {
            ResourceOperation::Rep(rep) => {
                // Implement representation operation
                self.get_resource_value(handle, operation)
            }
            ResourceOperation::Drop(drop) => {
                // Implement drop operation
                self.drop_resource(handle)?;
                Ok(ComponentValue::Bool(true))
            }
            _ => {
                // Unsupported operation
                Err(Error::new(kinds::UnsupportedOperation(format!(
                    "Unsupported resource operation {:?}",
                    operation
                ))))
            }
        }
    }

    /// Set memory strategy for a resource
    pub fn set_memory_strategy(&mut self, handle: u32, strategy: MemoryStrategy) -> Result<()> {
        // Check if the resource exists
        let entry = self.resources.get_mut(&handle).ok_or_else(|| {
            Error::resource_access_error(format!("Resource handle {} not found", handle))
        })?;

        entry.memory_strategy = strategy;
        Ok(())
    }

    /// Set verification level for a resource
    pub fn set_verification_level(&mut self, handle: u32, level: VerificationLevel) -> Result<()> {
        // Check if the resource exists
        let entry = self.resources.get_mut(&handle).ok_or_else(|| {
            Error::resource_access_error(format!("Resource handle {} not found", handle))
        })?;

        entry.verification_level = level;
        Ok(())
    }

    /// Get the number of resources in the table
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    /// Clean up unused resources
    pub fn cleanup_unused_resources(&mut self) -> usize {
        let handles_to_remove: Vec<u32> = self
            .resources
            .iter()
            .filter(|(_, entry)| Arc::strong_count(&entry.resource) <= 1)
            .map(|(handle, _)| *handle)
            .collect();

        for handle in &handles_to_remove {
            self.resources.remove(handle);
        }

        handles_to_remove.len()
    }

    /// Get a buffer from the pool
    pub fn get_buffer(&mut self, size: usize) -> Vec<u8> {
        self.buffer_pool.allocate(size)
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&mut self, buffer: Vec<u8>) {
        self.buffer_pool.return_buffer(buffer);
    }

    /// Get memory strategy from interceptors
    pub fn get_strategy_from_interceptors(&self, handle: u32) -> Option<MemoryStrategy> {
        for interceptor in &self.interceptors {
            if let Some(strategy_val) = interceptor.get_memory_strategy(handle) {
                if let Some(strategy) = MemoryStrategy::from_u8(strategy_val) {
                    return Some(strategy);
                }
            }
        }
        None
    }
}

/// Buffer pool for reusing memory buffers
#[derive(Debug)]
pub struct BufferPool {
    /// Map of buffer sizes to pools of buffers
    pools: HashMap<usize, Vec<Vec<u8>>>,
    /// Maximum buffer size to keep in the pool
    max_buffer_size: usize,
    /// Maximum number of buffers per size
    max_buffers_per_size: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(max_buffer_size: usize) -> Self {
        Self {
            pools: HashMap::new(),
            max_buffer_size,
            max_buffers_per_size: 8, // Reasonable default
        }
    }

    /// Get a buffer of at least the specified size
    pub fn allocate(&mut self, min_size: usize) -> Vec<u8> {
        // Find the smallest buffer size that fits
        let size_key = self
            .pools
            .keys()
            .filter(|&size| *size >= min_size)
            .min()
            .cloned();

        if let Some(size) = size_key {
            if let Some(buffers) = self.pools.get_mut(&size) {
                if let Some(buffer) = buffers.pop() {
                    return buffer;
                }
            }
        }

        // No suitable buffer found, create a new one
        // Round up to nearest power of 2 for better reuse
        let size = min_size.next_power_of_two();
        let mut buffer = Vec::with_capacity(size);
        buffer.resize(min_size, 0);
        buffer
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&mut self, mut buffer: Vec<u8>) {
        let capacity = buffer.capacity();

        // Only keep buffers below max size
        if capacity > self.max_buffer_size {
            return;
        }

        // Clear the buffer
        buffer.clear();

        // Add to the pool
        let buffers = self.pools.entry(capacity).or_insert_with(Vec::new);
        if buffers.len() < self.max_buffers_per_size {
            buffers.push(buffer);
        }
    }

    /// Get statistics about the buffer pool
    pub fn stats(&self) -> BufferPoolStats {
        let mut total_buffers = 0;
        let mut total_capacity = 0;

        for (size, buffers) in &self.pools {
            total_buffers += buffers.len();
            total_capacity += size * buffers.len();
        }

        BufferPoolStats {
            total_buffers,
            total_capacity,
            size_count: self.pools.len(),
        }
    }

    /// Clear all buffers from the pool
    pub fn reset(&mut self) {
        self.pools.clear();
    }
}

/// Statistics about the buffer pool
pub struct BufferPoolStats {
    /// Total number of buffers in the pool
    pub total_buffers: usize,
    /// Total capacity of all buffers in bytes
    pub total_capacity: usize,
    /// Number of different buffer sizes
    pub size_count: usize,
}

/// Trait for intercepting resource operations
pub trait ResourceInterceptor: Send + Sync {
    /// Called when a resource is created
    fn on_resource_create(&self, type_idx: u32, resource: &Resource) -> Result<()>;

    /// Called when a resource is dropped
    fn on_resource_drop(&self, handle: u32) -> Result<()>;

    /// Called when a resource is borrowed
    fn on_resource_borrow(&self, handle: u32) -> Result<()>;

    /// Called when a resource is accessed
    fn on_resource_access(&self, handle: u32) -> Result<()>;

    /// Called when an operation is performed on a resource
    fn on_resource_operation(&self, handle: u32, operation: &ResourceOperation) -> Result<()>;

    /// Get memory strategy for a resource
    fn get_memory_strategy(&self, handle: u32) -> Option<u8> {
        None
    }

    /// Intercept a resource operation
    fn intercept_resource_operation(
        &self,
        handle: u32,
        operation: &ResourceOperation,
    ) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct TestData {
        value: i32,
    }

    struct TestInterceptor {
        operations: std::sync::Mutex<Vec<String>>,
    }

    impl TestInterceptor {
        fn new() -> Self {
            Self {
                operations: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn get_operations(&self) -> Vec<String> {
            self.operations.lock().unwrap().clone()
        }
    }

    impl ResourceInterceptor for TestInterceptor {
        fn on_resource_create(&self, type_idx: u32, _resource: &Resource) -> Result<()> {
            self.operations
                .lock()
                .unwrap()
                .push(format!("create: {}", type_idx));
            Ok(())
        }

        fn on_resource_drop(&self, handle: u32) -> Result<()> {
            self.operations
                .lock()
                .unwrap()
                .push(format!("drop: {}", handle));
            Ok(())
        }

        fn on_resource_borrow(&self, handle: u32) -> Result<()> {
            self.operations
                .lock()
                .unwrap()
                .push(format!("borrow: {}", handle));
            Ok(())
        }

        fn on_resource_access(&self, handle: u32) -> Result<()> {
            self.operations
                .lock()
                .unwrap()
                .push(format!("access: {}", handle));
            Ok(())
        }

        fn on_resource_operation(&self, handle: u32, operation: &ResourceOperation) -> Result<()> {
            self.operations
                .lock()
                .unwrap()
                .push(format!("operation: {} - {:?}", handle, operation));
            Ok(())
        }

        fn get_memory_strategy(&self, handle: u32) -> Option<u8> {
            if handle % 2 == 0 {
                Some(1) // BoundedCopy for even handles
            } else {
                None
            }
        }

        fn intercept_resource_operation(
            &self,
            handle: u32,
            operation: &ResourceOperation,
        ) -> Result<Option<Vec<u8>>> {
            self.operations
                .lock()
                .unwrap()
                .push(format!("intercept_operation: {} - {:?}", handle, operation));

            // For testing, we intercept only for handle 42
            if handle == 42 {
                Ok(Some(vec![1, 2, 3]))
            } else {
                Ok(None)
            }
        }
    }

    #[test]
    fn test_resource_creation() {
        let mut table = ResourceTable::new();
        let data = Arc::new(TestData { value: 42 });

        let handle = table.create_resource(1, data).unwrap();
        assert_eq!(handle, 1);
        assert_eq!(table.resource_count(), 1);

        let resource = table.get_resource(handle).unwrap();
        let resource = resource.lock().unwrap();
        assert_eq!(resource.type_idx, 1);

        let data = resource.data.downcast_ref::<TestData>().unwrap();
        assert_eq!(data.value, 42);
    }

    #[test]
    fn test_resource_borrowing() {
        let mut table = ResourceTable::new();
        let data = Arc::new(TestData { value: 42 });

        let handle = table.create_resource(1, data).unwrap();
        let borrow_handle = table.borrow_resource(handle).unwrap();

        assert_ne!(handle, borrow_handle);
        assert_eq!(table.resource_count(), 2);

        let resource1 = table.get_resource(handle).unwrap();
        let resource2 = table.get_resource(borrow_handle).unwrap();

        let data1 = resource1
            .lock()
            .unwrap()
            .data
            .downcast_ref::<TestData>()
            .unwrap();
        let data2 = resource2
            .lock()
            .unwrap()
            .data
            .downcast_ref::<TestData>()
            .unwrap();

        assert_eq!(data1.value, 42);
        assert_eq!(data2.value, 42);
    }

    #[test]
    fn test_resource_dropping() {
        let mut table = ResourceTable::new();
        let data = Arc::new(TestData { value: 42 });

        let handle = table.create_resource(1, data).unwrap();
        assert_eq!(table.resource_count(), 1);

        table.drop_resource(handle).unwrap();
        assert_eq!(table.resource_count(), 0);

        assert!(table.get_resource(handle).is_err());
    }

    #[test]
    fn test_buffer_pool() {
        let mut pool = BufferPool::new(4096);

        // Get a buffer
        let buffer1 = pool.allocate(100);
        assert!(buffer1.capacity() >= 100);

        // Return the buffer
        pool.return_buffer(buffer1);

        // Get another buffer of the same size
        let buffer2 = pool.allocate(100);
        assert!(buffer2.capacity() >= 100);

        // Pool should be empty now
        let stats = pool.stats();
        assert_eq!(stats.total_buffers, 0);

        // Return the second buffer
        pool.return_buffer(buffer2);

        // Pool should have one buffer now
        let stats = pool.stats();
        assert_eq!(stats.total_buffers, 1);
    }

    #[test]
    fn test_memory_strategy() {
        let mut table = ResourceTable::new();
        let data = Arc::new(TestData { value: 42 });

        let handle = table.create_resource(1, data).unwrap();

        // Default strategy is BoundedCopy
        table
            .set_memory_strategy(handle, MemoryStrategy::ZeroCopy)
            .unwrap();

        // Invalid handle should fail
        assert!(table
            .set_memory_strategy(999, MemoryStrategy::ZeroCopy)
            .is_err());
    }

    #[test]
    fn test_resource_count_limit() {
        let mut table =
            ResourceTable::new_with_config(2, MemoryStrategy::BoundedCopy, VerificationLevel::None);

        let data1 = Arc::new(TestData { value: 1 });
        let data2 = Arc::new(TestData { value: 2 });
        let data3 = Arc::new(TestData { value: 3 });

        let handle1 = table.create_resource(1, data1).unwrap();
        let handle2 = table.create_resource(1, data2).unwrap();

        // Third resource should fail due to limit
        assert!(table.create_resource(1, data3).is_err());

        // After dropping one, we should be able to create another
        table.drop_resource(handle1).unwrap();
        let handle3 = table.create_resource(1, data3).unwrap();

        assert_eq!(table.resource_count(), 2);
        assert_ne!(handle1, handle3);
    }

    #[test]
    fn test_resource_interceptor() {
        let mut table = ResourceTable::new();
        let interceptor = Arc::new(TestInterceptor::new());

        table.add_interceptor(interceptor.clone());

        let data = Arc::new(TestData { value: 42 });
        let handle = table.create_resource(1, data).unwrap();

        // Access the resource
        let _resource = table.get_resource(handle).unwrap();

        // Apply an operation
        table
            .apply_operation(handle, ResourceOperation::Rep)
            .unwrap();

        // Check interceptor operations
        let operations = interceptor.get_operations();
        assert!(operations.contains(&format!("create:1")));
        assert!(operations.contains(&format!("access:{}", handle)));
        assert!(operations.contains(&format!("op:{}:rep", handle)));
    }

    #[test]
    fn test_resource_interception() {
        let interceptor = Arc::new(TestInterceptor::new());

        let mut table = ResourceTable::new();
        table.add_interceptor(interceptor.clone());

        // Create a resource
        let data = Arc::new(TestData { value: 42 });
        let handle = table.create_resource(1, data).unwrap();

        // Create a special resource with handle 42 (manually assign)
        let data = Arc::new(TestData { value: 99 });
        table.resources.insert(
            42,
            ResourceEntry {
                resource: Arc::new(Mutex::new(Resource::new(1, data))),
                borrows: Vec::new(),
                memory_strategy: MemoryStrategy::BoundedCopy,
                verification_level: VerificationLevel::Critical,
            },
        );

        // Test regular operation
        let result = table
            .apply_operation(handle, ResourceOperation::Rep)
            .unwrap();
        assert!(matches!(result, ComponentValue::U32(_)));

        // Test intercepted operation
        let result = table.apply_operation(42, ResourceOperation::Rep).unwrap();
        assert!(matches!(result, ComponentValue::Bool(true)));

        // Check that operations were recorded
        let ops = interceptor.get_operations();
        assert!(ops.contains(&format!("create: 1")));
        assert!(ops.contains(&format!("operation: {} - Rep", handle)));
        assert!(ops.contains(&format!("operation: 42 - Rep")));
        assert!(ops.contains(&format!("intercept_operation: 42 - Rep")));
    }

    #[test]
    fn test_memory_strategy_selection() {
        let interceptor = Arc::new(TestInterceptor::new());

        let mut table = ResourceTable::new();
        table.add_interceptor(interceptor.clone());

        // Create even and odd handle resources
        let even_handle = 2;
        let odd_handle = 3;

        table.resources.insert(
            even_handle,
            ResourceEntry {
                resource: Arc::new(Mutex::new(Resource::new(
                    1,
                    Arc::new(TestData { value: 2 }),
                ))),
                borrows: Vec::new(),
                memory_strategy: MemoryStrategy::ZeroCopy,
                verification_level: VerificationLevel::Critical,
            },
        );

        table.resources.insert(
            odd_handle,
            ResourceEntry {
                resource: Arc::new(Mutex::new(Resource::new(
                    1,
                    Arc::new(TestData { value: 3 }),
                ))),
                borrows: Vec::new(),
                memory_strategy: MemoryStrategy::ZeroCopy,
                verification_level: VerificationLevel::Critical,
            },
        );

        // Test strategy selection from interceptor
        let even_strategy = table.get_strategy_from_interceptors(even_handle);
        assert_eq!(even_strategy, Some(MemoryStrategy::BoundedCopy));

        // Test default strategy when interceptor returns None
        let odd_strategy = table.get_strategy_from_interceptors(odd_handle);
        assert_eq!(odd_strategy, None);
    }
}
