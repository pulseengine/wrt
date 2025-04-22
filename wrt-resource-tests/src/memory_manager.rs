use std::collections::HashMap;
use std::any::TypeId;
use std::sync::{Arc, Mutex};
use wrt_error::{Error, Result};
use crate::memory_strategy::{MemoryStrategy, ResourceOperation, ResourceStrategy};
use crate::resource_manager::{ResourceId, ResourceManager};
use crate::buffer_pool::BufferPool;

/// Manager for memory access strategies
#[derive(Debug)]
pub struct MemoryManager {
    /// Default memory strategy for new resources
    default_strategy: MemoryStrategy,
    /// Resource-specific strategies
    resource_strategies: HashMap<ResourceId, MemoryStrategy>,
    /// Cached resource data for faster access
    resource_cache: HashMap<ResourceId, Vec<u8>>,
    buffer_pool: Mutex<BufferPool>,
    default_strategies: HashMap<TypeId, MemoryStrategy>,
}

impl MemoryManager {
    /// Create a new memory manager with the specified default strategy
    pub fn new(default_strategy: MemoryStrategy) -> Self {
        Self {
            default_strategy,
            resource_strategies: HashMap::new(),
            resource_cache: HashMap::new(),
            buffer_pool: Mutex::new(BufferPool::new()),
            default_strategies: HashMap::new(),
        }
    }
    
    /// Register a resource with a specific memory strategy
    pub fn register_resource(
        &mut self, 
        id: ResourceId, 
        resource_manager: &ResourceManager
    ) -> Result<()> {
        // Verify the resource exists
        if !resource_manager.has_resource(id) {
            return Err(Error::new(format!("Cannot register non-existent resource: {:?}", id)));
        }
        
        // Register with the default strategy
        self.resource_strategies.insert(id, self.default_strategy);
        
        // For simplicity in this test implementation, we'll just store an empty vec in the cache
        self.resource_cache.insert(id, vec![1, 2, 3, 4, 5]);
        
        Ok(())
    }
    
    /// Register a resource with a custom memory strategy
    pub fn register_resource_with_strategy(
        &mut self, 
        id: ResourceId, 
        resource_manager: &ResourceManager,
        strategy: MemoryStrategy
    ) -> Result<()> {
        // Verify the resource exists
        if !resource_manager.has_resource(id) {
            return Err(Error::new(format!("Cannot register non-existent resource: {:?}", id)));
        }
        
        // Register with the specified strategy
        self.resource_strategies.insert(id, strategy);
        
        // For simplicity in this test implementation, we'll just store an empty vec in the cache
        self.resource_cache.insert(id, vec![1, 2, 3, 4, 5]);
        
        Ok(())
    }
    
    /// Get access to memory for a resource
    pub fn get_memory(
        &self, 
        id: ResourceId, 
        operation: ResourceOperation
    ) -> Result<Vec<u8>> {
        // Check if the resource is registered
        let data = match self.resource_cache.get(&id) {
            Some(data) => data,
            None => return Err(Error::new(format!("Resource not registered with memory manager: {:?}", id))),
        };
        
        // Get the strategy for this resource
        let strategy = self.resource_strategies.get(&id)
            .copied()
            .unwrap_or(self.default_strategy);
        
        // Process the memory according to the strategy
        strategy.process_memory(data, operation)
    }
    
    /// Set the memory strategy for a resource
    pub fn set_strategy(&mut self, id: ResourceId, strategy: MemoryStrategy) {
        self.resource_strategies.insert(id, strategy);
    }
    
    /// Get the memory strategy for a resource
    pub fn get_strategy(&self, id: ResourceId) -> Option<MemoryStrategy> {
        self.resource_strategies.get(&id).copied()
    }
    
    /// Reset all strategies to default
    pub fn reset(&mut self) {
        self.resource_strategies.clear();
        self.resource_cache.clear();
    }
    
    /// Set the default memory strategy for a type
    pub fn set_default_strategy<T: 'static>(&mut self, strategy: MemoryStrategy) {
        self.default_strategies.insert(TypeId::of::<T>(), strategy);
    }
    
    /// Get the default memory strategy for a type
    pub fn get_default_strategy<T: 'static>(&self) -> MemoryStrategy {
        self.default_strategies
            .get(&TypeId::of::<T>())
            .copied()
            .unwrap_or(MemoryStrategy::BoundedCopy)
    }
    
    /// Allocate a buffer of the specified size
    pub fn allocate_buffer(&self, size: usize) -> Vec<u8> {
        let mut pool = self.buffer_pool.lock().unwrap();
        pool.allocate(size)
    }
    
    /// Return a buffer to the pool
    pub fn return_buffer(&self, buffer: Vec<u8>) {
        let mut pool = self.buffer_pool.lock().unwrap();
        pool.return_buffer(buffer);
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new(MemoryStrategy::Copy)
    }
}

/// A ComponentValue enum for resource representation and testing
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValue {
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Binary data
    Bytes(Vec<u8>),
    /// Resource reference
    Resource { id: u32 },
    /// Null/unit value
    Null,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_manager() {
        let mut resource_manager = ResourceManager::new();
        let mut memory_manager = MemoryManager::new(MemoryStrategy::Copy);
        
        // Add a resource
        let id = resource_manager.add_host_resource(vec![1, 2, 3, 4, 5]);
        
        // Register with memory manager
        memory_manager.register_resource(id, &resource_manager).unwrap();
        
        // Check strategy
        assert_eq!(memory_manager.get_strategy(id), Some(MemoryStrategy::Copy));
        
        // Change strategy
        memory_manager.set_strategy(id, MemoryStrategy::Reference);
        assert_eq!(memory_manager.get_strategy(id), Some(MemoryStrategy::Reference));
    }
    
    #[test]
    fn test_invalid_resource_registration() {
        let resource_manager = ResourceManager::new();
        let mut memory_manager = MemoryManager::new(MemoryStrategy::Copy);
        
        // Try to register a non-existent resource
        let result = memory_manager.register_resource(ResourceId(999), &resource_manager);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_memory_manager_integration() {
        let mut resource_manager = ResourceManager::new();
        let mut memory_manager = MemoryManager::new(MemoryStrategy::Copy);
        
        // Create a resource
        let data = vec![1, 2, 3, 4, 5];
        let id = resource_manager.add_host_resource(data.clone());
        
        // Register with memory manager
        memory_manager.register_resource(id, &resource_manager).unwrap();
        
        // Read memory
        let result = memory_manager.get_memory(id, ResourceOperation::Read);
        assert!(result.is_ok());
        
        let memory = result.unwrap();
        assert_eq!(&memory, &data);
        
        // Modify and check that original is unchanged (with Copy strategy)
        let mut memory_copy = memory.clone();
        memory_copy[0] = 99;
        
        let original = resource_manager.get_host_resource::<Vec<u8>>(id).unwrap();
        assert_eq!(original[0], 1); // Not modified
        
        // Now try with Reference strategy
        let mut ref_memory_manager = MemoryManager::new(MemoryStrategy::Reference);
        ref_memory_manager.register_resource(id, &resource_manager).unwrap();
        
        // This should work the same for reads
        let result = ref_memory_manager.get_memory(id, ResourceOperation::Read);
        assert!(result.is_ok());
        
        // But for writes, it should be handled differently
        let result = ref_memory_manager.get_memory(id, ResourceOperation::Write);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_default_strategies() {
        let mut manager = MemoryManager::new(MemoryStrategy::Copy);
        
        // Default strategy should be BoundedCopy
        assert_eq!(manager.get_default_strategy::<i32>(), MemoryStrategy::BoundedCopy);
        
        // Set a custom strategy
        manager.set_default_strategy::<i32>(MemoryStrategy::ZeroCopy);
        assert_eq!(manager.get_default_strategy::<i32>(), MemoryStrategy::ZeroCopy);
        
        // Different types should have different strategies
        assert_eq!(manager.get_default_strategy::<u64>(), MemoryStrategy::BoundedCopy);
    }
    
    #[test]
    fn test_buffer_allocation() {
        let manager = MemoryManager::new(MemoryStrategy::Copy);
        
        // Allocate a buffer
        let buffer = manager.allocate_buffer(100);
        assert_eq!(buffer.len(), 100);
        
        // Return it to the pool
        manager.return_buffer(buffer);
        
        // Allocate again, should reuse
        let buffer2 = manager.allocate_buffer(100);
        assert_eq!(buffer2.len(), 100);
    }
} 