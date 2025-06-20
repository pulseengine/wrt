//! Simplified bounded types for WebAssembly parsing
//!
//! This module provides bounded collection types that don't require
//! complex trait implementations from stored types.

use wrt_error::{Error, ErrorCategory, Result, codes};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
compile_error!("Either 'std' or 'alloc' feature must be enabled");

/// A simplified bounded vector that enforces capacity limits
pub struct SimpleBoundedVec<T, const N: usize> {
    data: Vec<T>,
}

impl<T, const N: usize> SimpleBoundedVec<T, N> {
    /// Create a new empty bounded vector
    pub fn new() -> Self {
        SimpleBoundedVec {
            data: Vec::new(),
        }
    }
    
    /// Push an element to the vector
    pub fn push(&mut self, value: T) -> Result<()> {
        if self.data.len() >= N {
            return Err(Error::new(
                ErrorCategory::Resource,
                codes::RESOURCE_EXHAUSTED,
                "Bounded vector capacity exceeded"
            ));
        }
        
        self.data.push(value);
        Ok(())
    }
    
    /// Get the length
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    /// Get an element by index
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }
    
    /// Get a mutable element by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }
    
    /// Iterate over elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }
}

impl<T: Clone, const N: usize> Clone for SimpleBoundedVec<T, N> {
    fn clone(&self) -> Self {
        SimpleBoundedVec {
            data: self.data.clone(),
        }
    }
}

impl<T: core::fmt::Debug, const N: usize> core::fmt::Debug for SimpleBoundedVec<T, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.data.iter()).finish()
    }
}

impl<T, const N: usize> Default for SimpleBoundedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for bounded byte vector
pub type BoundedBytes<const N: usize> = SimpleBoundedVec<u8, N>;