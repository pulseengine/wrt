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
    pub fn get_host_resource<T: 'static + Send + Sync>(&self, id: ResourceId) -> Result<Arc<Mutex<T>>> {
        match self.resources.get(&id) {
            Some(resource) => {
                let resource_guard = resource.lock().unwrap();
                
                // Check if the type matches
                if let Some(typed_resource) = resource_guard.downcast_ref::<T>() {
                    // Create a reference to the resource
                    let typed_resource_arc = Arc::new(Mutex::new(typed_resource.clone()));
                    Ok(typed_resource_arc)
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
    pub fn get_resource_type(&self, id: ResourceId) -> Option<TypeId> {
        self.resources.get(&id).map(|resource| {
            let resource_guard = resource.lock().unwrap();
            resource_guard.type_id()
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

impl fmt::Debug for ResourceManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceManager")
            .field("resource_count", &self.resource_count())
            .field("next_id", &self.next_id)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
        assert_eq!(*resource.lock().unwrap(), "test");
        
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
} 