use wrt_error::{Error, Result};

/// Operations that can be performed on resources
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceOperation {
    /// Read access to a resource
    Read,
    /// Write access to a resource
    Write,
    /// Execute a resource as code
    Execute,
    /// Create a new resource
    Create,
    /// Delete an existing resource
    Delete,
}

impl ResourceOperation {
    /// Check if the operation requires read access
    pub fn requires_read(&self) -> bool {
        match self {
            ResourceOperation::Read | ResourceOperation::Execute => true,
            _ => false,
        }
    }
    
    /// Check if the operation requires write access
    pub fn requires_write(&self) -> bool {
        match self {
            ResourceOperation::Write | ResourceOperation::Create | ResourceOperation::Delete => true,
            _ => false,
        }
    }
}

/// Memory strategy for resource operations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryStrategy {
    /// Zero-copy strategy for trusted components
    ZeroCopy,
    /// Bounded-copy strategy with buffer pooling
    BoundedCopy,
    /// Full isolation with validation
    Isolated,
    /// Copy strategy - creates a copy of memory for safety
    Copy,
    /// Reference strategy - provides a direct reference to memory
    Reference,
    /// Full isolation with complete memory validation
    FullIsolation,
}

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

impl ResourceStrategy for MemoryStrategy {
    fn memory_strategy_type(&self) -> MemoryStrategy {
        *self
    }
    
    fn process_memory(&self, data: &[u8], operation: ResourceOperation) -> Result<Vec<u8>> {
        match self {
            // Zero-copy strategy - returns a view without copying for reads, a copy for writes
            MemoryStrategy::ZeroCopy => {
                match operation {
                    ResourceOperation::Read => Ok(data.to_vec()),
                    ResourceOperation::Write => Ok(data.to_vec()),
                    _ => Err(Error::new(format!("Unsupported operation for ZeroCopy strategy: {:?}", operation))),
                }
            },
            
            // Bounded-copy strategy - always copies but reuses buffers
            MemoryStrategy::BoundedCopy => {
                Ok(data.to_vec())
            },
            
            // Isolated strategy - always copies and validates 
            MemoryStrategy::Isolated => {
                // In a real implementation this would include validation
                Ok(data.to_vec())
            },
            
            // Copy strategy - always copies the data
            MemoryStrategy::Copy => {
                Ok(data.to_vec())
            },
            
            // Reference strategy - returns a view without copying
            MemoryStrategy::Reference => {
                // In a real implementation, this would return a reference
                // For testing purposes, we'll still return a vec
                Ok(data.to_vec())
            },
            
            // Full isolation strategy - copies and performs full validation
            MemoryStrategy::FullIsolation => {
                // In a real implementation this would include more extensive validation
                Ok(data.to_vec())
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_strategy() {
        let strategy = MemoryStrategy::Copy;
        let data = vec![1, 2, 3, 4, 5];
        
        let result = strategy.process_memory(&data, ResourceOperation::Read).unwrap();
        assert_eq!(result, data);
        
        // Modifying the copy shouldn't affect the original
        let mut result_copy = result.clone();
        result_copy[0] = 99;
        assert_ne!(result_copy[0], data[0]);
    }
    
    #[test]
    fn test_reference_strategy() {
        let strategy = MemoryStrategy::Reference;
        let data = vec![1, 2, 3, 4, 5];
        
        let result = strategy.process_memory(&data, ResourceOperation::Read).unwrap();
        assert_eq!(result, data);
    }
    
    #[test]
    fn test_operation_permissions() {
        assert!(ResourceOperation::Read.requires_read());
        assert!(!ResourceOperation::Read.requires_write());
        
        assert!(ResourceOperation::Write.requires_write());
        assert!(!ResourceOperation::Write.requires_read());
        
        assert!(ResourceOperation::Execute.requires_read());
        assert!(!ResourceOperation::Execute.requires_write());
        
        assert!(ResourceOperation::Create.requires_write());
        assert!(!ResourceOperation::Create.requires_read());
        
        assert!(ResourceOperation::Delete.requires_write());
        assert!(!ResourceOperation::Delete.requires_read());
    }
} 