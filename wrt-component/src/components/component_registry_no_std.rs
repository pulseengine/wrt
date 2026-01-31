// WRT - wrt-component
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! No-std component registry for the WebAssembly Component Model
//!
//! This module provides registry functionality for components in a no_std
//! environment.

use wrt_foundation::{
    bounded::MAX_COMPONENT_TYPES, budget_aware_provider::CrateId,
    collections::StaticVec as BoundedVec, safe_managed_alloc,
};

use crate::{components::component_no_std::Component, prelude::*};

/// Maximum number of components allowed in the registry
pub const MAX_COMPONENTS: usize = 32;

/// No-std registry for WebAssembly components
///
/// This registry uses fixed-size bounded collections to manage component
/// Binary std/no_std choice
#[derive(Debug)]
pub struct ComponentRegistry {
    /// Component names
    names: BoundedVec<String, MAX_COMPONENTS>,
    /// Component references - in no_std we use indices instead of references
    components: BoundedVec<usize, MAX_COMPONENTS>,
    /// Actual components
    component_store: BoundedVec<Component, MAX_COMPONENTS>,
}

impl ComponentRegistry {
    /// Create a new empty registry
    pub fn new() -> Result<Self> {
        Ok(Self {
            names: BoundedVec::new().unwrap(),
            components: BoundedVec::new().unwrap(),
            component_store: BoundedVec::new().unwrap(),
        })
    }

    /// Register a component by name
    pub fn register(&mut self, name: &str, component: Component) -> Result<()> {
        // Check if we've reached the maximum number of components
        if self.names.len() >= MAX_COMPONENTS {
            return Err(Error::runtime_execution_error(
                "Maximum number of components exceeded",
            ));
        }

        // Check if component already exists
        if self.get_index(name).is_some() {
            // Remove existing component with the same name
            self.remove(name)?;
        }

        // Add the component to the store and get its index
        let component_idx = self.component_store.len();
        self.component_store.push(component).map_err(|_| {
            Error::new(
                ErrorCategory::Resource,
                codes::CAPACITY_EXCEEDED,
                "Failed to add component to store",
            )
        })?;

        // Register the component
        self.names.push(name.to_string()).map_err(|_| {
            // Roll back the component addition if name addition fails
            self.component_store.remove(component_idx);

            Error::runtime_execution_error("Failed to add component name")
        })?;

        self.components.push(component_idx).map_err(|_| {
            // Roll back the name and component addition if mapping addition fails
            let last_idx = self.names.len() - 1;
            self.names.remove(last_idx);
            self.component_store.remove(component_idx);

            Error::new(
                ErrorCategory::Resource,
                codes::CAPACITY_EXCEEDED,
                "Failed to add component index",
            )
        })?;

        Ok(())
    }

    /// Get a component by name
    pub fn get(&self, name: &str) -> Option<&Component> {
        let idx = self.get_index(name)?;
        let component_idx = self.components[idx];
        Some(&self.component_store[component_idx])
    }

    /// Get a mutable component by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Component> {
        let idx = self.get_index(name)?;
        let component_idx = self.components[idx];
        Some(&mut self.component_store[component_idx])
    }

    /// Remove a component by name
    pub fn remove(&mut self, name: &str) -> Result<Component> {
        let idx = self
            .get_index(name)
            .ok_or_else(|| Error::resource_error("Component not found"))?;

        // Get the component index
        let component_idx = self.components[idx];

        // Remove the entry from the name and component index vectors
        self.names.remove(idx);
        self.components.remove(idx);

        // Remove the component from the store and return it
        // Note: This changes the indices of components after this one,
        // but since we reference by index with a direct lookup table,
        // there's no need to update other indices
        Ok(self.component_store.remove(component_idx))
    }

    /// Check if a component exists by name
    pub fn contains(&self, name: &str) -> bool {
        self.get_index(name).is_some()
    }

    /// Get all component names
    pub fn names(&self) -> Result<BoundedVec<String, MAX_COMPONENTS>> {
        let mut result = BoundedVec::new().unwrap();
        for name in self.names.iter() {
            result.push(name.clone()).map_err(|_| {
                Error::runtime_execution_error("Failed to add component name to result")
            })?;
        }
        Ok(result)
    }

    /// Get the number of registered components
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    /// Helper function to find the index of a component by name
    fn get_index(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|n| n == name)
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        // Use new() which properly handles allocation or panic in development
        Self::new().expect("ComponentRegistry allocation should not fail in default construction")
    }
}
