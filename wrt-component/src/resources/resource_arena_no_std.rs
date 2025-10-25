// WRT - wrt-component
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use wrt_error::kinds::PoisonedLockError;
use wrt_foundation::{
    collections::StaticVec as BoundedVec,
    budget_aware_provider::CrateId,
    safe_managed_alloc,
};

use super::{
    ResourceId,
    ResourceTable,
};
use crate::prelude::*;

/// Maximum number of resources that can be managed by a ResourceArena
pub const MAX_ARENA_RESOURCES: usize = 64;

/// A resource arena for managing resource lifecycles as a group
///
/// ResourceArena provides efficient group management of resources, allowing
/// Binary std/no_std choice
/// particularly useful for component instances and other scenarios where
/// resources have a shared lifetime.
#[derive(Clone)]
pub struct ResourceArena<'a> {
    /// Handles to resources managed by this arena - using BoundedVec for no_std
    resources: BoundedVec<u32, MAX_ARENA_RESOURCES>,
    /// The resource table used for actual resource management
    table:     &'a Mutex<ResourceTable>,
    /// Name of this arena, for debugging
    name:      Option<&'a str>,
}

impl<'a> ResourceArena<'a> {
    /// Create a new resource arena with the given resource table
    pub fn new(table: &'a Mutex<ResourceTable>) -> Result<Self> {
        Ok(Self {
            resources: {
                let provider = safe_managed_alloc!(65536, CrateId::Component)?;
                BoundedVec::new().unwrap()
            },
            table,
            name: None,
        })
    }

    /// Create a new resource arena with the given name
    pub fn new_with_name(table: &'a Mutex<ResourceTable>, name: &'a str) -> Result<Self> {
        Ok(Self {
            resources: {
                let provider = safe_managed_alloc!(65536, CrateId::Component)?;
                BoundedVec::new().unwrap()
            },
            table,
            name: Some(name),
        })
    }

    /// Create a resource in this arena
    ///
    /// The resource will be automatically cleaned up when the arena is dropped
    /// or when release_all() is called.
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn create_resource(
        &mut self,
        type_idx: u32,
        data: Box<dyn Any + Send + Sync>,
    ) -> Result<u32> {
        let mut table =
            self.table.lock();

        let handle = table.create_resource(type_idx, data)?;
        // Add to arena's resources, checking capacity
        if self.resources.len() >= MAX_ARENA_RESOURCES {
            return Err(Error::runtime_execution_error("Error occurred"));
        }
        self.resources
            .push(handle)
            .map_err(|_| Error::resource_error("Error occurred"))?;

        Ok(handle)
    }

    /// Create a named resource in this arena
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn create_named_resource(
        &mut self,
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

        // Add to arena's managed resources
        if self.resources.len() >= MAX_ARENA_RESOURCES {
            // Clean up the resource we just created since we can't track it
            let _ = table.drop_resource(handle);
            return Err(Error::runtime_execution_error("Error occurred"));
        }
        self.resources.push(handle).map_err(|_| {
            // Clean up the resource we just created since we can't track it
            let _ = table.drop_resource(handle);
            Error::resource_error("Error occurred")
        })?;

        Ok(handle)
    }

    /// Get access to a resource
    pub fn get_resource(&self, handle: u32) -> Result<ResourceId> {
        let table =
            self.table.lock();

        table.get_resource(handle)
    }

    /// Check if a resource exists
    pub fn has_resource(&self, id: ResourceId) -> Result<bool> {
        // First check if it's in our arena
        let contains = self.resources.contains(&id.0);
        if !contains {
            return Ok(false);
        }

        // Then check if it exists in the table
        let table =
            self.table.lock();

        match table.get_resource(id.0) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Remove a resource from this arena's management
    ///
    /// This does not drop the resource, just removes it from the arena.
    /// Returns true if the resource was found and removed.
    pub fn remove_resource(&mut self, handle: u32) -> bool {
        if let Some(pos) = self.resources.iter().position(|&h| h == handle) {
            self.resources.remove(pos);
            true
        } else {
            false
        }
    }

    /// Drop a specific resource and remove it from this arena
    pub fn drop_resource(&mut self, handle: u32) -> Result<()> {
        // First remove it from our tracking
        if !self.remove_resource(handle) {
            return Err(Error::resource_error("Error occurred"));
        }

        // Then drop it from the table
        let mut table =
            self.table.lock();

        table.drop_resource(handle)
    }

    /// Release all resources managed by this arena
    pub fn release_all(&mut self) -> Result<()> {
        if self.resources.is_empty() {
            return Ok(());
        }

        let mut table =
            self.table.lock();

        let mut error = None;

        // Try to drop all resources, continuing even if some fail
        for handle in self.resources.iter() {
            if let Err(e) = table.drop_resource(*handle) {
                // Store the first error but continue trying to drop others
                if error.is_none() {
                    error = Some(e);
                }
            }
        }

        // Clear our resources list regardless of errors
        self.resources.clear();

        // Return the first error if any occurred
        if let Some(e) = error {
            Err(e)
        } else {
            Ok(())
        }
    }

    /// Get the number of resources in this arena
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    /// Get the name of this arena
    pub fn name(&self) -> Option<&str> {
        self.name
    }

    /// Get all resources managed by this arena
    pub fn resources(&self) -> &[u32] {
        self.resources.as_slice()
    }
}

impl<'a> Drop for ResourceArena<'a> {
    fn drop(&mut self) {
        // Try to release all resources, ignoring errors
        let _ = self.release_all();
    }
}

impl<'a> Debug for ResourceArena<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceArena")
            .field("name", &self.name)
            .field("resource_count", &self.resources.len())
            .field("resources", &self.resources.as_slice())
            .finish()
    }

}
