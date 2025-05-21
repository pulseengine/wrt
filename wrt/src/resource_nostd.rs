//! WebAssembly Component Model resource types for no_std/no_alloc environments
//!
//! This module contains no_std and no_alloc compatible implementations for
//! the WebAssembly Component Model resource types, including resource tables
//! and lifetime management.

use core::{fmt, marker::PhantomData};

use wrt_error::{codes, Error, Result};
use wrt_types::{
    bounded::{BoundedStack, BoundedString, BoundedVec, WasmName},
    resource::{ResourceId, ResourceItem, ResourceType},
    safe_memory::{NoStdProvider, SafeMemoryHandler},
    verification::VerificationLevel,
    MemoryProvider,
};

// Constants for bounded collection limits
const MAX_RESOURCE_COUNT: usize = 256;
const MAX_RESOURCE_NAME_LENGTH: usize = 64;
const MAX_RESOURCE_TYPE_COUNT: usize = 32;

/// A no_std compatible resource instance
#[derive(Debug)]
pub struct BoundedResource<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Resource identifier
    pub id: ResourceId,
    /// Resource type
    pub resource_type: ResourceType<P>,
    /// Optional name for the resource
    pub name: Option<WasmName<MAX_RESOURCE_NAME_LENGTH, P>>,
    /// Whether this resource has been dropped
    pub is_dropped: bool,
}

/// A no_std/no_alloc compatible resource table implementation
#[derive(Debug)]
pub struct BoundedResourceTable<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Resources stored in this table
    resources: BoundedVec<BoundedResource<P>, MAX_RESOURCE_COUNT, P>,
    /// Resource types in this table
    resource_types: BoundedVec<ResourceType<P>, MAX_RESOURCE_TYPE_COUNT, P>,
    /// Memory provider
    provider: P,
    /// Verification level
    verification_level: VerificationLevel,
}

impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> BoundedResourceTable<P> {
    /// Creates a new resource table
    pub fn new(provider: P, verification_level: VerificationLevel) -> Result<Self> {
        Ok(Self {
            resources: BoundedVec::with_verification_level(provider.clone(), verification_level)?,
            resource_types: BoundedVec::with_verification_level(
                provider.clone(),
                verification_level,
            )?,
            provider,
            verification_level,
        })
    }

    /// Registers a new resource type
    pub fn register_resource_type(&mut self, resource_type: ResourceType<P>) -> Result<u32> {
        // Check if the type is already registered
        for (idx, rt) in self.resource_types.iter().enumerate() {
            if *rt == resource_type {
                return Ok(idx as u32);
            }
        }

        // Register new type
        let type_idx = self.resource_types.len();
        self.resource_types.push(resource_type)?;
        Ok(type_idx as u32)
    }

    /// Creates a new resource instance
    pub fn create_resource(&mut self, type_idx: u32, name_str: Option<&str>) -> Result<ResourceId> {
        // Validate type index
        if type_idx as usize >= self.resource_types.len() {
            return Err(Error::new(
                wrt_error::ErrorCategory::Resource,
                codes::INVALID_RESOURCE_TYPE,
                "Invalid resource type index",
            ));
        }

        // Get the resource type
        let resource_type = self.resource_types[type_idx as usize].clone();

        // Create name if provided
        let name = if let Some(n) = name_str {
            Some(WasmName::from_str(n, self.provider.clone())?)
        } else {
            None
        };

        // Create resource
        let resource_id = ResourceId(self.resources.len() as u64);
        let resource = BoundedResource { id: resource_id, resource_type, name, is_dropped: false };

        // Add to table
        self.resources.push(resource)?;

        Ok(resource_id)
    }

    /// Gets a resource by ID
    pub fn get_resource(&self, id: ResourceId) -> Result<&BoundedResource<P>> {
        let idx = id.0 as usize;
        if idx >= self.resources.len() {
            return Err(Error::new(
                wrt_error::ErrorCategory::Resource,
                codes::RESOURCE_NOT_FOUND,
                "Resource not found",
            ));
        }

        // Check if the resource is dropped
        let resource = &self.resources[idx];
        if resource.is_dropped {
            return Err(Error::new(
                wrt_error::ErrorCategory::Resource,
                codes::RESOURCE_DROPPED,
                "Resource has been dropped",
            ));
        }

        Ok(resource)
    }

    /// Drops a resource by ID
    pub fn drop_resource(&mut self, id: ResourceId) -> Result<()> {
        let idx = id.0 as usize;
        if idx >= self.resources.len() {
            return Err(Error::new(
                wrt_error::ErrorCategory::Resource,
                codes::RESOURCE_NOT_FOUND,
                "Resource not found",
            ));
        }

        // Mark as dropped
        if self.resources[idx].is_dropped {
            return Err(Error::new(
                wrt_error::ErrorCategory::Resource,
                codes::RESOURCE_ALREADY_DROPPED,
                "Resource already dropped",
            ));
        }

        self.resources[idx].is_dropped = true;
        Ok(())
    }

    /// Gets the number of resources in the table
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    /// Gets the number of active (not dropped) resources
    pub fn active_resource_count(&self) -> usize {
        self.resources.iter().filter(|r| !r.is_dropped).count()
    }

    /// Gets the number of resource types
    pub fn resource_type_count(&self) -> usize {
        self.resource_types.len()
    }

    /// Gets the verification level
    pub fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }
}

/// Creates a default resource table using NoStdProvider
pub fn create_default_resource_table() -> Result<BoundedResourceTable<NoStdProvider>> {
    let provider = NoStdProvider::default();
    BoundedResourceTable::new(provider, VerificationLevel::Standard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_resource_table() {
        let provider = NoStdProvider::default();
        let mut table = BoundedResourceTable::new(provider.clone(), VerificationLevel::Standard)
            .expect("Failed to create resource table");

        // Create a resource type
        let record_fields = BoundedVec::new(provider.clone()).unwrap();
        let resource_type = ResourceType::Record(record_fields);

        // Register the type
        let type_idx =
            table.register_resource_type(resource_type).expect("Failed to register resource type");

        // Create a resource
        let resource_id = table
            .create_resource(type_idx, Some("test_resource"))
            .expect("Failed to create resource");

        // Get the resource
        let resource = table.get_resource(resource_id).expect("Failed to get resource");

        assert_eq!(resource.id, resource_id);
        assert!(!resource.is_dropped);
        assert_eq!(resource.name.as_ref().unwrap().as_str().unwrap(), "test_resource");

        // Drop the resource
        table.drop_resource(resource_id).expect("Failed to drop resource");

        // Try to get it after dropping (should fail)
        let result = table.get_resource(resource_id);
        assert!(result.is_err());

        // Check counts
        assert_eq!(table.resource_count(), 1);
        assert_eq!(table.active_resource_count(), 0);
        assert_eq!(table.resource_type_count(), 1);
    }
}
