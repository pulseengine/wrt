// WRT - wrt-component
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use super::{MemoryStrategy, VerificationLevel};
use crate::prelude::*;

/// Builder for creating Resource instances
#[derive(Clone, Debug)]
pub struct ResourceBuilder<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Resource type index
    type_idx: u32,
    /// Resource data
    data: T,
    /// Optional debug name
    name: Option<String>,
    /// Memory strategy for this resource
    memory_strategy: MemoryStrategy,
    /// Verification level
    verification_level: VerificationLevel,
}

impl<T> ResourceBuilder<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new ResourceBuilder with required parameters
    pub fn new(type_idx: u32, data: T) -> Self {
        Self {
            type_idx,
            data,
            name: None,
            memory_strategy: MemoryStrategy::default(),
            verification_level: VerificationLevel::Critical,
        }
    }

    /// Set a debug name for the resource
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Set the memory strategy for the resource
    pub fn with_memory_strategy(mut self, strategy: MemoryStrategy) -> Self {
        self.memory_strategy = strategy;
        self
    }

    /// Set the verification level for the resource
    pub fn with_verification_level(mut self, level: VerificationLevel) -> Self {
        self.verification_level = level;
        self
    }

    /// Build the resource
    #[cfg(feature = "std")]
    pub fn build(self) -> (super::Resource, MemoryStrategy, VerificationLevel) {
        // Create the resource
        let resource = if let Some(name) = self.name {
            super::Resource::new_with_name(self.type_idx, Arc::new(self.data), &name)
        } else {
            super::Resource::new(self.type_idx, Arc::new(self.data))
        };

        (resource, self.memory_strategy, self.verification_level)
    }

    /// Build the resource (no_std version)
        pub fn build(self) -> (super::Resource, MemoryStrategy, VerificationLevel) {
        // Create the resource
        let resource = if let Some(name) = self.name {
            super::Resource::new_with_name(self.type_idx, Box::new(self.data), &name)
        } else {
            super::Resource::new(self.type_idx, Box::new(self.data))
        };

        (resource, self.memory_strategy, self.verification_level)
    }
}

/// Builder for creating ResourceTable instances
#[derive(Clone, Debug)]
pub struct ResourceTableBuilder {
    /// Maximum allowed resources
    max_resources: usize,
    /// Default memory strategy
    default_memory_strategy: MemoryStrategy,
    /// Default verification level
    default_verification_level: VerificationLevel,
    /// Whether to use optimized memory
    use_optimized_memory: bool,
}

impl Default for ResourceTableBuilder {
    fn default() -> Self {
        Self {
            max_resources: 64, // Default to MAX_RESOURCES value
            default_memory_strategy: MemoryStrategy::default(),
            default_verification_level: VerificationLevel::Critical,
            use_optimized_memory: false,
        }
    }
}

impl ResourceTableBuilder {
    /// Create a new ResourceTableBuilder with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of resources
    pub fn with_max_resources(mut self, max_resources: usize) -> Self {
        self.max_resources = max_resources;
        self
    }

    /// Set the default memory strategy
    pub fn with_default_memory_strategy(mut self, strategy: MemoryStrategy) -> Self {
        self.default_memory_strategy = strategy;
        self
    }

    /// Set the default verification level
    pub fn with_default_verification_level(mut self, level: VerificationLevel) -> Self {
        self.default_verification_level = level;
        self
    }

    /// Enable optimized memory management
    pub fn with_optimized_memory(mut self) -> Self {
        self.use_optimized_memory = true;
        self
    }

    /// Build the ResourceTable
    #[cfg(feature = "std")]
    pub fn build(self) -> super::ResourceTable {
        if self.use_optimized_memory {
            super::ResourceTable::new_with_config_and_optimized_memory(
                self.max_resources,
                self.default_memory_strategy,
                self.default_verification_level,
            )
        } else {
            super::ResourceTable::new_with_config(
                self.max_resources,
                self.default_memory_strategy,
                self.default_verification_level,
            )
        }
    }

    /// Build the ResourceTable (no_std version)
        pub fn build(self) -> super::ResourceTable {
        // In no_std there's only one implementation
        super::ResourceTable::new()
    }
}

/// Builder for creating ResourceManager instances
#[derive(Clone, Debug)]
pub struct ResourceManagerBuilder<'a> {
    /// Component instance ID
    instance_id: &'a str,
    /// Default memory strategy
    default_memory_strategy: MemoryStrategy,
    /// Default verification level
    default_verification_level: VerificationLevel,
    /// Whether to use optimized memory
    use_optimized_memory: bool,
}

impl<'a> Default for ResourceManagerBuilder<'a> {
    fn default() -> Self {
        Self {
            instance_id: "default-instance",
            default_memory_strategy: MemoryStrategy::default(),
            default_verification_level: VerificationLevel::Critical,
            use_optimized_memory: false,
        }
    }
}

impl<'a> ResourceManagerBuilder<'a> {
    /// Create a new ResourceManagerBuilder with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the instance ID
    pub fn with_instance_id(mut self, id: &'a str) -> Self {
        self.instance_id = id;
        self
    }

    /// Set the default memory strategy
    pub fn with_default_memory_strategy(mut self, strategy: MemoryStrategy) -> Self {
        self.default_memory_strategy = strategy;
        self
    }

    /// Set the default verification level
    pub fn with_default_verification_level(mut self, level: VerificationLevel) -> Self {
        self.default_verification_level = level;
        self
    }

    /// Enable optimized memory management
    pub fn with_optimized_memory(mut self) -> Self {
        self.use_optimized_memory = true;
        self
    }

    /// Build the ResourceManager (std version)
    #[cfg(feature = "std")]
    pub fn build(self) -> super::ResourceManager {
        if self.use_optimized_memory {
            super::ResourceManager::new_with_config_and_optimized_memory(
                self.instance_id,
                64, // Default max resources
                self.default_memory_strategy,
                self.default_verification_level,
            )
        } else {
            super::ResourceManager::new_with_config(
                self.instance_id,
                64, // Default max resources
                self.default_memory_strategy,
                self.default_verification_level,
            )
        }
    }

    /// Build the ResourceManager (no_std version)
        pub fn build<'b>(self, table: &'b Mutex<super::ResourceTable>) -> super::ResourceManager<'b>
    where
        'a: 'b,
    {
        super::ResourceManager::new_with_config(
            table,
            self.instance_id,
            self.default_memory_strategy,
            self.default_verification_level,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_builder() {
        let (resource, strategy, level) = ResourceBuilder::new(1, "test".to_string())
            .with_name("test-resource")
            .with_memory_strategy(MemoryStrategy::ZeroCopy)
            .with_verification_level(VerificationLevel::Full)
            .build();

        assert_eq!(resource.type_idx, 1);
        assert_eq!(resource.name, Some("test-resource".to_string()));
        assert_eq!(strategy, MemoryStrategy::ZeroCopy);
        assert_eq!(level, VerificationLevel::Full);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_resource_table_builder() {
        let table = ResourceTableBuilder::new()
            .with_max_resources(100)
            .with_default_memory_strategy(MemoryStrategy::ZeroCopy)
            .with_default_verification_level(VerificationLevel::None)
            .with_optimized_memory()
            .build();

        // Create a resource to test the table was built correctly
        let data = Arc::new("test".to_string());
        let handle = table.create_resource(1, data).unwrap();
        assert_eq!(handle, 1);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_resource_manager_builder() {
        let manager = ResourceManagerBuilder::new()
            .with_instance_id("test-instance")
            .with_default_memory_strategy(MemoryStrategy::Copy)
            .with_default_verification_level(VerificationLevel::Critical)
            .build();

        assert_eq!(manager.instance_id(), "test-instance");
        assert_eq!(manager.default_memory_strategy(), MemoryStrategy::Copy);
        assert_eq!(manager.default_verification_level(), VerificationLevel::Critical);
    }

    #[test]
        fn test_resource_manager_builder_no_std() {
        let table = Mutex::new(ResourceTable::new());

        let manager = ResourceManagerBuilder::new()
            .with_instance_id("test-instance")
            .with_default_memory_strategy(MemoryStrategy::Copy)
            .with_default_verification_level(VerificationLevel::Critical)
            .build(&table);

        assert_eq!(manager.instance_id(), "test-instance");
        assert_eq!(manager.default_memory_strategy(), MemoryStrategy::Copy);
        assert_eq!(manager.default_verification_level(), VerificationLevel::Critical);
    }
}
