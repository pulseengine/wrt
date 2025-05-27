//! Resource handle management for WebAssembly Component Model
//! 
//! This module implements resource handle tables using bounded collections,
//! providing predictable memory usage for embedded/no_std environments.
//! 
//! Based on the Component Model MVP design:
//! - Owned handles (own<T>) represent unique ownership
//! - Borrowed handles (borrow<T>) represent temporary access
//! - Handles are 32-bit integers indexing into type-specific tables

use wrt_foundation::{
    bounded::BoundedVec,
    traits::BoundedCapacity,
    MemoryProvider,
};
use wrt_error::{Error, ErrorCategory, codes};

/// Maximum number of resources per type
/// Component Model suggests this as a reasonable limit
pub const MAX_RESOURCES_PER_TYPE: usize = 1024;

/// Resource handle (32-bit index)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceHandle(pub u32);

/// Resource ownership type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceOwnership {
    /// Owned resource - must be explicitly dropped
    Owned,
    /// Borrowed resource - temporary access
    Borrowed,
}

impl wrt_foundation::traits::Checksummable for ResourceOwnership {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        match self {
            ResourceOwnership::Owned => 0u8.update_checksum(checksum),
            ResourceOwnership::Borrowed => 1u8.update_checksum(checksum),
        }
    }
}

impl wrt_foundation::traits::ToBytes for ResourceOwnership {
    fn serialized_size(&self) -> usize {
        1 // single byte for discriminant
    }

    fn to_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        _provider: &PStream,
    ) -> wrt_foundation::Result<()> {
        match self {
            ResourceOwnership::Owned => writer.write_u8(0),
            ResourceOwnership::Borrowed => writer.write_u8(1),
        }
    }
}

impl wrt_foundation::traits::FromBytes for ResourceOwnership {
    fn from_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &PStream,
    ) -> wrt_foundation::Result<Self> {
        match reader.read_u8()? {
            0 => Ok(ResourceOwnership::Owned),
            1 => Ok(ResourceOwnership::Borrowed),
            _ => Err(Error::new(ErrorCategory::InvalidInput, codes::INVALID_STATE, "Invalid ResourceOwnership discriminant")),
        }
    }
}

/// Resource entry in the handle table
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEntry<T> 
where 
    T: Clone + PartialEq + Eq,
{
    /// The actual resource
    pub resource: T,
    /// Ownership type
    pub ownership: ResourceOwnership,
    /// Reference count for borrowed handles
    pub ref_count: u32,
}

impl<T> wrt_foundation::traits::Checksummable for ResourceEntry<T>
where
    T: Clone + PartialEq + Eq + wrt_foundation::traits::Checksummable,
{
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        self.resource.update_checksum(checksum);
        self.ownership.update_checksum(checksum);
        self.ref_count.update_checksum(checksum);
    }
}

impl<T> wrt_foundation::traits::ToBytes for ResourceEntry<T>
where
    T: Clone + PartialEq + Eq + wrt_foundation::traits::ToBytes,
{
    fn serialized_size(&self) -> usize {
        self.resource.serialized_size() + self.ownership.serialized_size() + self.ref_count.serialized_size()
    }

    fn to_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        provider: &PStream,
    ) -> wrt_foundation::Result<()> {
        self.resource.to_bytes_with_provider(writer, provider)?;
        self.ownership.to_bytes_with_provider(writer, provider)?;
        self.ref_count.to_bytes_with_provider(writer, provider)?;
        Ok(())
    }
}

impl<T> wrt_foundation::traits::FromBytes for ResourceEntry<T>
where
    T: Clone + PartialEq + Eq + wrt_foundation::traits::FromBytes,
{
    fn from_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        provider: &PStream,
    ) -> wrt_foundation::Result<Self> {
        let resource = T::from_bytes_with_provider(reader, provider)?;
        let ownership = ResourceOwnership::from_bytes_with_provider(reader, provider)?;
        let ref_count = u32::from_bytes_with_provider(reader, provider)?;
        Ok(Self {
            resource,
            ownership,
            ref_count,
        })
    }
}

/// Resource handle table for a specific resource type
pub struct ResourceTable<T, P: MemoryProvider + Default + Clone + PartialEq + Eq> 
where
    T: Clone + PartialEq + Eq + wrt_foundation::traits::Checksummable + wrt_foundation::traits::ToBytes + wrt_foundation::traits::FromBytes,
{
    /// Table entries indexed by handle
    entries: BoundedVec<Option<ResourceEntry<T>>, MAX_RESOURCES_PER_TYPE, P>,
    /// Next available handle
    next_handle: u32,
}

impl<T, P: MemoryProvider + Default + Clone + PartialEq + Eq> ResourceTable<T, P> 
where
    T: Clone + PartialEq + Eq + wrt_foundation::traits::Checksummable + wrt_foundation::traits::ToBytes + wrt_foundation::traits::FromBytes,
{
    /// Create a new resource table
    pub fn new(provider: P) -> Result<Self, Error> {
        let mut entries = BoundedVec::new(provider)?;
        
        // Initialize with None values
        for _ in 0..MAX_RESOURCES_PER_TYPE {
            entries.push(None).map_err(|_| {
                Error::new(
                    ErrorCategory::Memory,
                    codes::MEMORY_ALLOCATION_ERROR,
                    "Failed to initialize resource table"
                )
            })?;
        }
        
        Ok(Self {
            entries,
            next_handle: 1, // 0 is reserved for null handle
        })
    }
    
    /// Allocate a new owned resource
    pub fn new_own(&mut self, resource: T) -> Result<ResourceHandle, Error> {
        let handle = self.allocate_handle()?;
        let entry = ResourceEntry {
            resource,
            ownership: ResourceOwnership::Owned,
            ref_count: 0,
        };
        
        self.entries[handle.0 as usize] = Some(entry);
        Ok(handle)
    }
    
    /// Create a borrowed handle from an owned handle
    pub fn new_borrow(&mut self, owned: ResourceHandle) -> Result<ResourceHandle, Error> {
        let entry = self.entries.get_mut(owned.0 as usize)
            .and_then(|e| e.as_mut())
            .ok_or_else(|| Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_INVALID_HANDLE,
                "Invalid owned handle"
            ))?;
            
        if entry.ownership != ResourceOwnership::Owned {
            return Err(Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_INVALID_HANDLE,
                "Can only borrow from owned resources"
            ));
        }
        
        entry.ref_count += 1;
        Ok(owned) // Borrowed handle is same as owned handle
    }
    
    /// Get a resource by handle
    pub fn get(&self, handle: ResourceHandle) -> Option<&T> {
        self.entries.get(handle.0 as usize)
            .and_then(|e| e.as_ref())
            .map(|entry| &entry.resource)
    }
    
    /// Get a mutable resource by handle (only for owned)
    pub fn get_mut(&mut self, handle: ResourceHandle) -> Option<&mut T> {
        self.entries.get_mut(handle.0 as usize)
            .and_then(|e| e.as_mut())
            .filter(|entry| entry.ownership == ResourceOwnership::Owned && entry.ref_count == 0)
            .map(|entry| &mut entry.resource)
    }
    
    /// Drop a resource handle
    pub fn drop_handle(&mut self, handle: ResourceHandle) -> Result<Option<T>, Error> {
        let entry = self.entries.get_mut(handle.0 as usize)
            .and_then(|e| e.take())
            .ok_or_else(|| Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_INVALID_HANDLE,
                "Invalid resource handle"
            ))?;
            
        match entry.ownership {
            ResourceOwnership::Owned => {
                if entry.ref_count > 0 {
                    // Put it back, still has borrows
                    self.entries[handle.0 as usize] = Some(entry);
                    return Err(Error::new(
                        ErrorCategory::Resource,
                        codes::RESOURCE_ERROR,
                        "Cannot drop owned resource with active borrows"
                    ));
                }
                Ok(Some(entry.resource))
            }
            ResourceOwnership::Borrowed => {
                // Decrement ref count on the owned resource
                if let Some(owned_entry) = self.entries.get_mut(handle.0 as usize).and_then(|e| e.as_mut()) {
                    owned_entry.ref_count = owned_entry.ref_count.saturating_sub(1);
                }
                Ok(None)
            }
        }
    }
    
    /// Allocate a new handle
    fn allocate_handle(&mut self) -> Result<ResourceHandle, Error> {
        // Simple linear search for now
        let start = self.next_handle as usize;
        for i in 0..MAX_RESOURCES_PER_TYPE {
            let index = (start + i) % MAX_RESOURCES_PER_TYPE;
            if index == 0 { continue; } // Skip 0 (null handle)
            
            if self.entries[index].is_none() {
                self.next_handle = (index + 1) as u32;
                return Ok(ResourceHandle(index as u32));
            }
        }
        
        Err(Error::new(
            ErrorCategory::Resource,
            codes::RESOURCE_LIMIT_EXCEEDED,
            "Resource table full"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrt_foundation::traits::DefaultMemoryProvider;
    
    #[test]
    fn test_resource_table_basic() {
        let provider = DefaultMemoryProvider::default();
        let mut table = ResourceTable::<String, _>::new(provider).unwrap();
        
        // Create owned resource
        let owned = table.new_own("Hello".to_string()).unwrap();
        assert_eq!(table.get(owned), Some(&"Hello".to_string()));
        
        // Create borrowed handle
        let borrowed = table.new_borrow(owned).unwrap();
        assert_eq!(table.get(borrowed), Some(&"Hello".to_string()));
        
        // Cannot drop owned while borrowed
        assert!(table.drop_handle(owned).is_err());
        
        // Drop borrowed first
        table.drop_handle(borrowed).unwrap();
        
        // Now can drop owned
        let resource = table.drop_handle(owned).unwrap();
        assert_eq!(resource, Some("Hello".to_string()));
    }
}