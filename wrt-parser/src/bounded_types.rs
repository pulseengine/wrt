//! Simplified bounded types for WebAssembly parsing
//!
//! This module provides bounded collection types that don't require
//! complex trait implementations from stored types.

use wrt_error::{Error, ErrorCategory, Result, codes};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{vec::Vec, string::String};
#[cfg(feature = "std")]
use std::{vec::Vec, string::String};
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
compile_error!("Either 'std' or 'alloc' feature must be enabled");

/// A simplified bounded vector that enforces capacity limits
#[derive(PartialEq, Eq)]
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
    
    /// Iterate over elements mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut()
    }
    
    /// Get the capacity of the bounded vector
    pub fn capacity(&self) -> usize {
        N
    }
    
    /// Retain elements matching a predicate
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.data.retain(f);
    }
    
    /// Clear all elements from the vector
    pub fn clear(&mut self) {
        self.data.clear();
    }
    
    /// Remove and return the last element
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }
    
    /// Swap remove an element at index
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.data.swap_remove(index)
    }
    
    /// Get the first element
    pub fn first(&self) -> Option<&T> {
        self.data.first()
    }
    
    /// Get the last element
    pub fn last(&self) -> Option<&T> {
        self.data.last()
    }
    
    /// Get as a slice
    pub fn as_slice(&self) -> &[T] {
        &self.data
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

impl<T, const N: usize> core::ops::Index<usize> for SimpleBoundedVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T, const N: usize> core::ops::IndexMut<usize> for SimpleBoundedVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T, const N: usize> IntoIterator for SimpleBoundedVec<T, N> {
    type Item = T;
    type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a SimpleBoundedVec<T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut SimpleBoundedVec<T, N> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<T, const N: usize> Default for SimpleBoundedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for bounded byte vector
pub type BoundedBytes<const N: usize> = SimpleBoundedVec<u8, N>;

/// A simplified bounded string
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleBoundedString<const N: usize> {
    data: String,
}

impl<const N: usize> SimpleBoundedString<N> {
    /// Create a new empty bounded string
    pub fn new() -> Self {
        Self {
            data: String::new(),
        }
    }
    
    /// Create from a string slice, truncating if necessary
    pub fn from_str(s: &str) -> Self {
        let mut data = String::new();
        for (i, ch) in s.chars().enumerate() {
            if data.len() + ch.len_utf8() > N {
                break;
            }
            data.push(ch);
        }
        Self { data }
    }
    
    /// Get the string as a slice
    pub fn as_str(&self) -> &str {
        &self.data
    }
    
    /// Get the length in bytes
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    /// Get the capacity
    pub fn capacity(&self) -> usize {
        N
    }
}

impl<const N: usize> Default for SimpleBoundedString<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for bounded bytes with specific size
pub type SimpleBoundedBytes<const N: usize> = SimpleBoundedVec<u8, N>;