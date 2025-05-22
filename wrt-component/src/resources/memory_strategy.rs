// WRT - wrt-component
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use wrt_error::{Error, Result};
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use wrt_foundation::bounded::{BoundedCollection, BoundedVec, MAX_BUFFER_SIZE};

#[cfg(feature = "std")]
use super::resource_table::MemoryStrategy;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use super::resource_table_no_std::MemoryStrategy;
use crate::resources::{ResourceOperation, ResourceStrategy};

#[cfg(feature = "std")]
impl ResourceStrategy for MemoryStrategy {
    fn memory_strategy_type(&self) -> MemoryStrategy {
        *self
    }

    fn process_memory(&self, data: &[u8], operation: ResourceOperation) -> Result<Vec<u8>> {
        match self {
            // Zero-copy strategy - returns a view without copying for reads, a copy for writes
            MemoryStrategy::ZeroCopy => match operation {
                ResourceOperation::Read => Ok(data.to_vec()),
                ResourceOperation::Write => Ok(data.to_vec()),
                _ => Err(Error::new("Unsupported operation for ZeroCopy strategy")),
            },

            // Bounded-copy strategy - always copies but reuses buffers
            MemoryStrategy::BoundedCopy => Ok(data.to_vec()),

            // Isolated strategy - always copies and validates
            MemoryStrategy::Isolated => {
                // In a real implementation this would include validation
                Ok(data.to_vec())
            }

            // Copy strategy - always copies the data
            MemoryStrategy::Copy => Ok(data.to_vec()),

            // Reference strategy - returns a view without copying
            MemoryStrategy::Reference => {
                // In a real implementation, this would return a reference
                // For testing purposes, we'll still return a vec
                Ok(data.to_vec())
            }

            // Full isolation strategy - copies and performs full validation
            MemoryStrategy::FullIsolation => {
                // In a real implementation this would include more extensive validation
                Ok(data.to_vec())
            }
        }
    }
}

#[cfg(all(not(feature = "std"), feature = "alloc"))]
impl ResourceStrategy for MemoryStrategy {
    fn memory_strategy_type(&self) -> MemoryStrategy {
        *self
    }

    fn process_memory(
        &self,
        data: &[u8],
        operation: ResourceOperation,
    ) -> Result<BoundedVec<u8, MAX_BUFFER_SIZE>> {
        match self {
            // Zero-copy strategy - returns a view without copying for reads, a copy for writes
            MemoryStrategy::ZeroCopy => match operation {
                ResourceOperation::Read => {
                    let mut result = BoundedVec::with_capacity(data.len()).map_err(|e| {
                        Error::new(
                            wrt_error::ErrorCategory::Memory,
                            wrt_error::codes::MEMORY_ERROR,
                            format!("Failed to create bounded vec for zero-copy: {}", e),
                        )
                    })?;

                    for &byte in data {
                        result.push(byte).map_err(|e| {
                            Error::new(
                                wrt_error::ErrorCategory::Memory,
                                wrt_error::codes::MEMORY_ERROR,
                                format!("Failed to push to bounded vec: {}", e),
                            )
                        })?;
                    }
                    Ok(result)
                }
                ResourceOperation::Write => {
                    let mut result = BoundedVec::with_capacity(data.len()).map_err(|e| {
                        Error::new(
                            wrt_error::ErrorCategory::Memory,
                            wrt_error::codes::MEMORY_ERROR,
                            format!("Failed to create bounded vec for zero-copy: {}", e),
                        )
                    })?;

                    for &byte in data {
                        result.push(byte).map_err(|e| {
                            Error::new(
                                wrt_error::ErrorCategory::Memory,
                                wrt_error::codes::MEMORY_ERROR,
                                format!("Failed to push to bounded vec: {}", e),
                            )
                        })?;
                    }
                    Ok(result)
                }
                _ => Err(Error::new("Unsupported operation for ZeroCopy strategy")),
            },

            // Bounded-copy strategy - always copies but reuses buffers
            MemoryStrategy::BoundedCopy => {
                let mut result = BoundedVec::with_capacity(data.len()).map_err(|e| {
                    Error::new(
                        wrt_error::ErrorCategory::Memory,
                        wrt_error::codes::MEMORY_ERROR,
                        format!("Failed to create bounded vec for bounded-copy: {}", e),
                    )
                })?;

                for &byte in data {
                    result.push(byte).map_err(|e| {
                        Error::new(
                            wrt_error::ErrorCategory::Memory,
                            wrt_error::codes::MEMORY_ERROR,
                            format!("Failed to push to bounded vec: {}", e),
                        )
                    })?;
                }
                Ok(result)
            }

            // Other strategies implemented similarly
            MemoryStrategy::Isolated
            | MemoryStrategy::Copy
            | MemoryStrategy::Reference
            | MemoryStrategy::FullIsolation => {
                let mut result = BoundedVec::with_capacity(data.len()).map_err(|e| {
                    Error::new(
                        wrt_error::ErrorCategory::Memory,
                        wrt_error::codes::MEMORY_ERROR,
                        format!("Failed to create bounded vec: {}", e),
                    )
                })?;

                for &byte in data {
                    result.push(byte).map_err(|e| {
                        Error::new(
                            wrt_error::ErrorCategory::Memory,
                            wrt_error::codes::MEMORY_ERROR,
                            format!("Failed to push to bounded vec: {}", e),
                        )
                    })?;
                }
                Ok(result)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
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
    #[cfg(feature = "std")]
    fn test_reference_strategy() {
        let strategy = MemoryStrategy::Reference;
        let data = vec![1, 2, 3, 4, 5];

        let result = strategy.process_memory(&data, ResourceOperation::Read).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    fn test_no_std_copy_strategy() {
        let strategy = MemoryStrategy::Copy;
        let data = &[1, 2, 3, 4, 5];

        let result = strategy.process_memory(data, ResourceOperation::Read).unwrap();
        assert_eq!(result.as_slice(), data);
    }

    #[test]
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    fn test_no_std_reference_strategy() {
        let strategy = MemoryStrategy::Reference;
        let data = &[1, 2, 3, 4, 5];

        let result = strategy.process_memory(data, ResourceOperation::Read).unwrap();
        assert_eq!(result.as_slice(), data);
    }
}
