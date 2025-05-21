// WRT - wrt-component
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use crate::prelude::*;
use wrt_error::Result;
use wrt_types::bounded::{BoundedVec, BoundedCollection};

use super::{
    memory_strategy::MemoryStrategy,
    resource_operation::ResourceOperation,
    resource_strategy::ResourceStrategy
};

/// Maximum size for buffer operations
pub const MAX_BUFFER_SIZE: usize = 4096;

/// ResourceStrategy implementation for no_std environments
///
/// This implementation uses bounded collections to avoid dynamic allocation
/// and is designed for environments without the standard library.
#[derive(Debug, Clone, Copy)]
pub struct ResourceStrategyNoStd {
    /// The type of memory strategy this resource strategy implements
    memory_strategy: MemoryStrategy,
}

impl ResourceStrategyNoStd {
    /// Create a new ResourceStrategyNoStd with the given memory strategy
    pub fn new(memory_strategy: MemoryStrategy) -> Self {
        Self { memory_strategy }
    }
}

impl ResourceStrategy for ResourceStrategyNoStd {
    fn memory_strategy_type(&self) -> MemoryStrategy {
        self.memory_strategy
    }

    fn process_memory(&self, data: &[u8], operation: ResourceOperation) -> Result<BoundedVec<u8, MAX_BUFFER_SIZE>> {
        match self.memory_strategy {
            // Zero-copy strategy - returns a view without copying for reads, a copy for writes
            MemoryStrategy::ZeroCopy => match operation {
                ResourceOperation::Read => {
                    let mut result = BoundedVec::with_capacity(data.len()).map_err(|e| {
                        Error::new(ErrorCategory::Memory, 
                                  codes::MEMORY_ERROR,
                                  format!("Failed to create bounded vec for zero-copy: {}", e))
                    })?;
                    
                    for &byte in data {
                        result.push(byte).map_err(|e| {
                            Error::new(ErrorCategory::Memory, 
                                      codes::MEMORY_ERROR,
                                      format!("Failed to push to bounded vec: {}", e))
                        })?;
                    }
                    Ok(result)
                },
                ResourceOperation::Write => {
                    let mut result = BoundedVec::with_capacity(data.len()).map_err(|e| {
                        Error::new(ErrorCategory::Memory, 
                                  codes::MEMORY_ERROR,
                                  format!("Failed to create bounded vec for zero-copy: {}", e))
                    })?;
                    
                    for &byte in data {
                        result.push(byte).map_err(|e| {
                            Error::new(ErrorCategory::Memory, 
                                      codes::MEMORY_ERROR,
                                      format!("Failed to push to bounded vec: {}", e))
                        })?;
                    }
                    Ok(result)
                },
                _ => Err(Error::new(ErrorCategory::Operation, 
                                  codes::OPERATION_ERROR,
                                  "Unsupported operation for ZeroCopy strategy")),
            },

            // Bounded-copy strategy - always copies but reuses buffers
            MemoryStrategy::BoundedCopy => {
                let mut result = BoundedVec::with_capacity(data.len()).map_err(|e| {
                    Error::new(ErrorCategory::Memory, 
                              codes::MEMORY_ERROR,
                              format!("Failed to create bounded vec for bounded-copy: {}", e))
                })?;
                
                for &byte in data {
                    result.push(byte).map_err(|e| {
                        Error::new(ErrorCategory::Memory, 
                                  codes::MEMORY_ERROR,
                                  format!("Failed to push to bounded vec: {}", e))
                    })?;
                }
                Ok(result)
            },

            // Isolated strategy - always copies and validates
            MemoryStrategy::Isolated => {
                // In a real implementation this would include validation
                let mut result = BoundedVec::with_capacity(data.len()).map_err(|e| {
                    Error::new(ErrorCategory::Memory, 
                              codes::MEMORY_ERROR,
                              format!("Failed to create bounded vec for isolated strategy: {}", e))
                })?;
                
                for &byte in data {
                    result.push(byte).map_err(|e| {
                        Error::new(ErrorCategory::Memory, 
                                  codes::MEMORY_ERROR,
                                  format!("Failed to push to bounded vec: {}", e))
                    })?;
                }
                Ok(result)
            },

            // Copy strategy - always copies the data
            MemoryStrategy::Copy => {
                let mut result = BoundedVec::with_capacity(data.len()).map_err(|e| {
                    Error::new(ErrorCategory::Memory, 
                              codes::MEMORY_ERROR,
                              format!("Failed to create bounded vec for copy strategy: {}", e))
                })?;
                
                for &byte in data {
                    result.push(byte).map_err(|e| {
                        Error::new(ErrorCategory::Memory, 
                                  codes::MEMORY_ERROR,
                                  format!("Failed to push to bounded vec: {}", e))
                    })?;
                }
                Ok(result)
            },

            // Reference strategy - returns a view without copying
            MemoryStrategy::Reference => {
                // In a real implementation, this would return a reference
                // For no_std compatibility, we'll still return a bounded vec
                let mut result = BoundedVec::with_capacity(data.len()).map_err(|e| {
                    Error::new(ErrorCategory::Memory, 
                              codes::MEMORY_ERROR,
                              format!("Failed to create bounded vec for reference strategy: {}", e))
                })?;
                
                for &byte in data {
                    result.push(byte).map_err(|e| {
                        Error::new(ErrorCategory::Memory, 
                                  codes::MEMORY_ERROR,
                                  format!("Failed to push to bounded vec: {}", e))
                    })?;
                }
                Ok(result)
            },

            // Full isolation strategy - copies and performs full validation
            MemoryStrategy::FullIsolation => {
                // In a real implementation this would include more extensive validation
                let mut result = BoundedVec::with_capacity(data.len()).map_err(|e| {
                    Error::new(ErrorCategory::Memory, 
                              codes::MEMORY_ERROR,
                              format!("Failed to create bounded vec for full isolation: {}", e))
                })?;
                
                for &byte in data {
                    result.push(byte).map_err(|e| {
                        Error::new(ErrorCategory::Memory, 
                                  codes::MEMORY_ERROR,
                                  format!("Failed to push to bounded vec: {}", e))
                    })?;
                }
                Ok(result)
            },
        }
    }

    fn allows_operation(&self, operation: ResourceOperation) -> bool {
        match self.memory_strategy {
            MemoryStrategy::ZeroCopy => {
                // ZeroCopy only allows read and write, not other operations
                matches!(operation, ResourceOperation::Read | ResourceOperation::Write)
            },
            MemoryStrategy::BoundedCopy => true, // Allows all operations
            MemoryStrategy::Isolated => true,    // Allows all operations
            MemoryStrategy::Copy => true,        // Allows all operations
            MemoryStrategy::Reference => {
                // Reference primarily allows read operations
                matches!(operation, ResourceOperation::Read | ResourceOperation::Reference)
            },
            MemoryStrategy::FullIsolation => {
                // Full isolation might restrict certain operations
                !matches!(operation, ResourceOperation::Reference)
            },
        }
    }

    fn reset(&mut self) {
        // No-op for this implementation
        // In a real implementation, this might clear any cached buffers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_strategy() {
        let strategy = ResourceStrategyNoStd::new(MemoryStrategy::Copy);
        let data = &[1, 2, 3, 4, 5];

        let result = strategy.process_memory(data, ResourceOperation::Read).unwrap();
        assert_eq!(result.as_slice(), data);

        // Modifying the copy shouldn't affect the original
        let mut result_vec = Vec::from(result.as_slice());
        result_vec[0] = 99;
        assert_ne!(result_vec[0], data[0]);
    }

    #[test]
    fn test_reference_strategy() {
        let strategy = ResourceStrategyNoStd::new(MemoryStrategy::Reference);
        let data = &[1, 2, 3, 4, 5];

        let result = strategy.process_memory(data, ResourceOperation::Read).unwrap();
        assert_eq!(result.as_slice(), data);
    }

    #[test]
    fn test_allows_operation() {
        // Test ZeroCopy strategy restrictions
        let zero_copy = ResourceStrategyNoStd::new(MemoryStrategy::ZeroCopy);
        assert!(zero_copy.allows_operation(ResourceOperation::Read));
        assert!(zero_copy.allows_operation(ResourceOperation::Write));
        assert!(!zero_copy.allows_operation(ResourceOperation::Execute));

        // Test Reference strategy restrictions
        let reference = ResourceStrategyNoStd::new(MemoryStrategy::Reference);
        assert!(reference.allows_operation(ResourceOperation::Read));
        assert!(reference.allows_operation(ResourceOperation::Reference));
        assert!(!reference.allows_operation(ResourceOperation::Write));

        // Test FullIsolation strategy
        let full_isolation = ResourceStrategyNoStd::new(MemoryStrategy::FullIsolation);
        assert!(full_isolation.allows_operation(ResourceOperation::Read));
        assert!(full_isolation.allows_operation(ResourceOperation::Write));
        assert!(!full_isolation.allows_operation(ResourceOperation::Reference));
    }
}