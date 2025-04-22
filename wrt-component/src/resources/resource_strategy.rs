use wrt_error::Result;
use crate::resources::{MemoryStrategy, ResourceOperation};

/// Trait for resource access strategies
pub trait ResourceStrategy: Send + Sync {
    /// Get the type of memory strategy this implements
    fn memory_strategy_type(&self) -> MemoryStrategy;
    
    /// Process memory with this strategy
    fn process_memory(&self, data: &[u8], operation: ResourceOperation) -> Result<Vec<u8>>;
    
    /// Check if the strategy allows a certain operation
    fn allows_operation(&self, operation: ResourceOperation) -> bool {
        true // Default implementation allows all operations
    }
    
    /// Reset any internal state or buffers
    fn reset(&mut self) {
        // Default is no-op
    }
} 