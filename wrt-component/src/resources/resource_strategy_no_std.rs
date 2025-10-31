// WRT - wrt-component
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#[cfg(feature = "std")]
use std::vec::Vec;

use wrt_error::{
    codes,
    Error,
    ErrorCategory,
    Result,
};
use wrt_foundation::{
    collections::StaticVec as BoundedVec,
    bounded::MAX_BUFFER_SIZE,
    resource::ResourceOperation,
};

use super::{
    resource_strategy::ResourceStrategy,
    MemoryStrategy,
};

/// No-std version of ResourceStrategy implementation
/// This struct provides resource access strategies for no_std environments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceStrategyNoStd {
    /// The memory strategy to use for this resource
    strategy: MemoryStrategy,
}

impl ResourceStrategyNoStd {
    /// Create a new ResourceStrategyNoStd with the given memory strategy
    pub fn new(strategy: MemoryStrategy) -> Self {
        Self { strategy }
    }
}

impl ResourceStrategy for ResourceStrategyNoStd {
    fn memory_strategy_type(&self) -> MemoryStrategy {
        self.strategy
    }

    #[cfg(feature = "std")]
    fn process_memory(
        &self,
        data: &[u8],
        operation: ResourceOperation,
    ) -> Result<Vec<u8>> {
        match self.strategy {
            // Zero-copy strategy - returns a view without copying for reads, a copy for writes
            MemoryStrategy::ZeroCopy => match operation {
                ResourceOperation::Read => {
                    Ok(data.to_vec())
                },
                ResourceOperation::Write => {
                    Ok(data.to_vec())
                },
                _ => Ok(data.to_vec()), // Default for other operations
            },
            // Bounded-copy strategy - copies with size limit checks
            MemoryStrategy::BoundedCopy => {
                if data.len() > MAX_BUFFER_SIZE {
                    return Err(Error::new(
                        ErrorCategory::Memory,
                        codes::MEMORY_OUT_OF_BOUNDS,
                        "Data exceeds maximum buffer size for bounded copy",
                    ));
                }
                Ok(data.to_vec())
            },
            // Full isolation - creates independent copies
            MemoryStrategy::FullIsolation => {
                Ok(data.to_vec())
            },
            // Handle other memory strategies
            _ => {
                Ok(data.to_vec())
            },
        }
    }

    #[cfg(not(feature = "std"))]
    fn process_memory(
        &self,
        data: &[u8],
        operation: ResourceOperation,
    ) -> core::result::Result<
        wrt_foundation::bounded::BoundedVec<u8, MAX_BUFFER_SIZE, wrt_foundation::safe_memory::NoStdProvider<{MAX_BUFFER_SIZE}>>,
        wrt_error::Error,
    > {
        use wrt_foundation::{safe_managed_alloc, CrateId};
        let provider = safe_managed_alloc!(MAX_BUFFER_SIZE, CrateId::Component)?;

        match self.strategy {
            // Zero-copy strategy - returns a view without copying for reads, a copy for writes
            MemoryStrategy::ZeroCopy => match operation {
                ResourceOperation::Read => {
                    let mut result = wrt_foundation::bounded::BoundedVec::new(provider)?;

                    for &byte in data {
                        result.push(byte).map_err(|e| Error::memory_error("Error occurred"))?;
                    }
                    Ok(result)
                },
                ResourceOperation::Write => {
                    let mut result = wrt_foundation::bounded::BoundedVec::new(provider)?;

                    for &byte in data {
                        result.push(byte).map_err(|e| Error::memory_error("Error occurred"))?;
                    }
                    Ok(result)
                },
                _ => Err(Error::not_supported("Error occurred")),
            },

            // Bounded-copy strategy - always copies but reuses buffers
            MemoryStrategy::BoundedCopy => {
                let mut result = wrt_foundation::bounded::BoundedVec::new(provider)?;

                for &byte in data {
                    result.push(byte).map_err(|e| Error::memory_error("Error occurred"))?;
                }
                Ok(result)
            },

            // Isolated strategy - always copies and validates
            MemoryStrategy::Isolated => {
                let mut result = wrt_foundation::bounded::BoundedVec::new(provider)?;

                // In a real implementation this would include validation
                for &byte in data {
                    result.push(byte).map_err(|e| Error::memory_error("Error occurred"))?;
                }
                Ok(result)
            },

            // Copy strategy - always copies the data
            MemoryStrategy::Copy => {
                let mut result = wrt_foundation::bounded::BoundedVec::new(provider)?;

                for &byte in data {
                    result.push(byte).map_err(|e| Error::memory_error("Error occurred"))?;
                }
                Ok(result)
            },

            // Reference strategy - returns a view without copying
            MemoryStrategy::Reference => {
                let mut result = wrt_foundation::bounded::BoundedVec::new(provider)?;

                // In a real implementation, this would return a reference
                // For now, we'll still return a BoundedVec
                for &byte in data {
                    result.push(byte).map_err(|e| Error::memory_error("Error occurred"))?;
                }
                Ok(result)
            },

            // Full isolation strategy - copies and performs full validation
            MemoryStrategy::FullIsolation => {
                let mut result = wrt_foundation::bounded::BoundedVec::new(provider)?;

                // In a real implementation this would include more extensive validation
                for &byte in data {
                    result.push(byte).map_err(|e| Error::memory_error("Error occurred"))?;
                }
                Ok(result)
            },

            // FixedBuffer strategy - uses fixed size buffer
            MemoryStrategy::FixedBuffer => {
                let mut result = wrt_foundation::bounded::BoundedVec::new(provider)?;

                for &byte in data {
                    result.push(byte).map_err(|e| Error::memory_error("Error occurred"))?;
                }
                Ok(result)
            },

            // BoundedCollections strategy - uses bounded collections
            MemoryStrategy::BoundedCollections => {
                let mut result = wrt_foundation::bounded::BoundedVec::new(provider)?;

                for &byte in data {
                    result.push(byte).map_err(|e| Error::memory_error("Error occurred"))?;
                }
                Ok(result)
            },
        }
    }

    // We're using the default implementation for allows_operation
    // fn allows_operation(&self, operation: ResourceOperation) -> bool {
    //     true // Default implementation allows all operations
    // }

    // We're using the default implementation for reset
    // fn reset(&mut self) {
    //     // Default is no-op
    // }
}

// Implementation-specific constants
/// Maximum buffer size for bounded vectors in no_std environments
pub const MAX_RESOURCE_BUFFER_SIZE: usize = wrt_foundation::bounded::MAX_BUFFER_SIZE;
