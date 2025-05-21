use wrt_format::component::ResourceOperation as FormatResourceOperation;
use wrt_error::Result;

use super::Resource;

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
    fn on_resource_operation(&self, handle: u32, operation: &FormatResourceOperation)
        -> Result<()>;

    /// Get memory strategy for a resource
    fn get_memory_strategy(&self, handle: u32) -> Option<u8> {
        None
    }

    /// Intercept a resource operation
    fn intercept_resource_operation(
        &self,
        handle: u32,
        operation: &FormatResourceOperation,
    ) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }
}