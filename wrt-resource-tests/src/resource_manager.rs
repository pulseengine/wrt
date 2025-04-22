use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use wrt_error::{Error, Result};

/// Unique identifier for a resource
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u32);

/// Trait representing a host resource
pub trait HostResource {}

// Implement HostResource for common types
impl<T: 'static + Send + Sync> HostResource for T {}

/// Manager for resource instances
#[derive(Debug)]
pub struct ResourceManager {
    /// Resources stored by ID
    resources: HashMap<ResourceId, Arc<Mutex<Box<dyn Any + Send + Sync>>>>,
    /// Next available resource ID
    next_id: u32,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            next_id: 1, // Start at 1, 0 is reserved
        }
    }
    
    /// Generate a new resource ID
    pub fn generate_id(&mut self) -> ResourceId {
        let id = ResourceId(self.next_id);
        self.next_id += 1;
        id
    }
    
    /// Add a host resource to the manager
    pub fn add_host_resource<T: 'static + Send + Sync>(&mut self, resource: T) -> ResourceId {
        let id = self.generate_id();
        self.resources.insert(id, Arc::new(Mutex::new(Box::new(resource))));
        id
    }
    
    /// Get a host resource by ID and type
    pub fn get_host_resource<T: 'static + Send + Sync>(&self, id: ResourceId) -> Result<T> 
    where 
        T: Clone,
    {
        match self.resources.get(&id) {
            Some(resource) => {
                let resource_guard = resource.lock().unwrap();
                
                // Check if the type matches
                if let Some(typed_resource) = resource_guard.downcast_ref::<T>() {
                    // Return a clone of the resource
                    Ok(typed_resource.clone())
                } else {
                    Err(Error::new(format!("Resource type mismatch for ID: {:?}", id)))
                }
            },
            None => Err(Error::new(format!("Resource not found with ID: {:?}", id))),
        }
    }
    
    /// Check if a resource exists
    pub fn has_resource(&self, id: ResourceId) -> bool {
        self.resources.contains_key(&id)
    }
    
    /// Get the type ID of a resource
    pub fn get_resource_type(&self, id: ResourceId) -> Option<String> {
        self.resources.get(&id).map(|resource| {
            let resource_guard = resource.lock().unwrap();
            format!("{:?}", (**resource_guard).type_id())
        })
    }
    
    /// Delete a resource
    pub fn delete_resource(&mut self, id: ResourceId) {
        self.resources.remove(&id);
    }
    
    /// Clear all resources
    pub fn clear(&mut self) {
        self.resources.clear();
    }
    
    /// Get the number of managed resources
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc as StdArc;
    
    #[test]
    fn test_resource_id_generation() {
        let mut manager = ResourceManager::new();
        
        let id1 = manager.generate_id();
        let id2 = manager.generate_id();
        
        assert_ne!(id1, id2);
        assert_eq!(id2.0, id1.0 + 1);
    }
    
    #[test]
    fn test_add_and_get_resource() {
        let mut manager = ResourceManager::new();
        
        // Add a string resource
        let id = manager.add_host_resource(String::from("test"));
        
        // Check we can retrieve it
        let resource = manager.get_host_resource::<String>(id).unwrap();
        assert_eq!(resource, "test");
        
        // Check type mismatch
        let result = manager.get_host_resource::<i32>(id);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_resource_lifecycle() {
        let mut manager = ResourceManager::new();
        
        // Add a resource
        let id = manager.add_host_resource(42);
        assert!(manager.has_resource(id));
        
        // Delete it
        manager.delete_resource(id);
        assert!(!manager.has_resource(id));
    }
    
    /// Custom test resource that tracks whether it's been dropped
    #[derive(Clone)]
    struct DropTracker {
        id: usize,
        dropped: StdArc<AtomicUsize>,
    }

    impl DropTracker {
        fn new(id: usize, dropped: StdArc<AtomicUsize>) -> Self {
            Self { id, dropped }
        }
    }

    impl Drop for DropTracker {
        fn drop(&mut self) {
            self.dropped.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_resource_cleanup() {
        let dropped = StdArc::new(AtomicUsize::new(0));
        
        {
            let mut resource_manager = ResourceManager::new();
            
            // Add resources with drop trackers
            let id1 = resource_manager.add_host_resource(DropTracker::new(1, dropped.clone()));
            let id2 = resource_manager.add_host_resource(DropTracker::new(2, dropped.clone()));
            let id3 = resource_manager.add_host_resource(DropTracker::new(3, dropped.clone()));
            
            // Delete one resource explicitly
            resource_manager.delete_resource(id2);
            assert_eq!(dropped.load(Ordering::SeqCst), 1);
            
            // Let the resource manager go out of scope
        }
        
        // Verify all resources were dropped
        // Three drop trackers total, and we already deleted one
        assert_eq!(dropped.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_get_resource_type() {
        let mut manager = ResourceManager::new();
        
        // Add different types of resources
        let id_string = manager.add_host_resource(String::from("test"));
        let id_int = manager.add_host_resource(42);
        let id_bool = manager.add_host_resource(true);
        
        // Verify type IDs are returned correctly
        assert!(manager.get_resource_type(id_string).unwrap().contains("String"));
        assert!(manager.get_resource_type(id_int).unwrap().contains("i32"));
        assert!(manager.get_resource_type(id_bool).unwrap().contains("bool"));
        
        // Verify non-existent resource returns None
        assert_eq!(manager.get_resource_type(ResourceId(9999)), None);
    }
} 