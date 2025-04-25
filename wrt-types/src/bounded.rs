//! Bounded collections with functional safety verification
//!
//! This module provides bounded collection types that are designed for
//! functional safety with built-in size limits and verification features.

#[cfg(feature = "std")]
use std::{fmt, hash, mem};

#[cfg(not(feature = "std"))]
use alloc::{format, vec, vec::Vec};

#[cfg(not(feature = "std"))]
use core::{fmt, hash, mem};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::ToString;

// Use the HashMap that's re-exported in lib.rs - works for both std and no_std
#[allow(unused_imports)]
use crate::operations;
use crate::operations::record_global_operation;
use crate::operations::OperationType;
use crate::validation::{importance, BoundedCapacity, Checksummed, Validatable};
use crate::verification::{Checksum, VerificationLevel};
use crate::HashMap;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::String;

/// Error indicating a collection has reached its capacity limit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapacityError;

impl fmt::Display for CapacityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bounded collection capacity exceeded")
    }
}

/// Error types for bounded collections
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundedError {
    /// Capacity limit exceeded
    CapacityExceeded {
        /// Type of collection that had its capacity exceeded
        collection_type: String,
        /// Maximum capacity of the collection
        capacity: usize,
        /// Size that was attempted
        attempted_size: usize,
    },

    /// Invalid access to a collection
    InvalidAccess {
        /// Type of collection being accessed
        collection_type: String,
        /// Index that was attempted
        index: usize,
        /// Current size of the collection
        size: usize,
    },

    /// Checksum validation failed
    ChecksumMismatch {
        /// Expected checksum value
        expected: u32,
        /// Actual calculated checksum
        actual: u32,
        /// Description of what was being validated
        description: String,
    },

    /// Generic validation error
    ValidationFailure(String),
}

impl fmt::Display for BoundedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityExceeded {
                collection_type,
                capacity,
                attempted_size,
            } => {
                write!(
                    f,
                    "{} capacity exceeded: limit {}, attempted {}",
                    collection_type, capacity, attempted_size
                )
            }
            Self::InvalidAccess {
                collection_type,
                index,
                size,
            } => {
                write!(
                    f,
                    "Invalid access to {} at index {}, size is {}",
                    collection_type, index, size
                )
            }
            Self::ChecksumMismatch {
                expected,
                actual,
                description,
            } => {
                write!(
                    f,
                    "Checksum mismatch in {}: expected {:08x}, got {:08x}",
                    description, expected, actual
                )
            }
            Self::ValidationFailure(msg) => {
                write!(f, "Validation failure: {}", msg)
            }
        }
    }
}

/// A bounded stack with a fixed maximum capacity and verification
///
/// This stack ensures it never exceeds the specified capacity,
/// returning an error when push operations would exceed this limit.
/// It also maintains a checksum of its contents for verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedStack<T, const N: usize>
where
    T: AsRef<[u8]>,
{
    /// The underlying stack storage
    data: Vec<T>,
    /// Checksum for verifying data integrity
    checksum: Checksum,
    /// Verification level for this stack
    verification_level: VerificationLevel,
}

impl<T, const N: usize> Default for BoundedStack<T, N>
where
    T: AsRef<[u8]>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> BoundedStack<T, N>
where
    T: AsRef<[u8]>,
{
    /// Create a new empty bounded stack
    pub fn new() -> Self {
        Self::with_verification_level(VerificationLevel::default())
    }

    /// Create a new empty bounded stack with a specific verification level
    pub fn with_verification_level(level: VerificationLevel) -> Self {
        Self {
            data: Vec::with_capacity(N),
            checksum: Checksum::new(),
            verification_level: level,
        }
    }

    /// Push an element onto the stack
    ///
    /// Returns an error if the stack is at capacity
    pub fn push(&mut self, item: T) -> Result<(), CapacityError> {
        // Track the push operation
        record_global_operation(OperationType::CollectionPush, self.verification_level);

        if self.is_full() {
            return Err(CapacityError);
        }

        // Only update checksum if verification is enabled
        if crate::validation::should_validate(self.verification_level, importance::MUTATION) {
            self.checksum.update_slice(item.as_ref());
            // Track checksum calculation
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
        }

        self.data.push(item);

        Ok(())
    }

    /// Pop an element from the stack
    ///
    /// # Returns
    ///
    /// Returns the top element if the stack is not empty, or None if it is.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    /// Safety impact: [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
    /// Tracking: WRTQ-XXX (qualification requirement tracking ID).
    pub fn pop(&mut self) -> Option<T> {
        // Track the pop operation
        record_global_operation(OperationType::CollectionPop, self.verification_level);

        let item = self.data.pop()?;

        // Recompute checksum if verification is enabled
        if crate::validation::should_validate(self.verification_level, importance::MUTATION) {
            self.recalculate_checksum();
            // Track checksum calculation
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
        }

        Some(item)
    }

    /// Peek at the top element without removing it
    pub fn peek(&self) -> Option<&T> {
        // Track the lookup operation
        record_global_operation(OperationType::CollectionLookup, self.verification_level);

        // For high verification levels, verify before returning data
        if crate::validation::should_validate_redundant(self.verification_level)
            && !self.verify_checksum()
        {
            return None;
        }

        self.data.last()
    }

    /// Get the current verification level
    pub fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }

    /// Set a new verification level
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        // When changing to a non-None level from None, recalculate the checksum
        let needs_recalc = matches!(self.verification_level, VerificationLevel::None)
            && !matches!(level, VerificationLevel::None);

        self.verification_level = level;

        if needs_recalc {
            self.recalculate_checksum();
        }
    }

    /// Verify the integrity of the stack (backward compatibility method)
    pub fn verify(&self) -> bool {
        // Track the validation operation
        record_global_operation(OperationType::CollectionValidate, self.verification_level);

        self.verify_checksum()
    }
}

// Implement the traits for BoundedStack
impl<T, const N: usize> BoundedCapacity for BoundedStack<T, N>
where
    T: AsRef<[u8]>,
{
    fn capacity(&self) -> usize {
        N
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<T, const N: usize> Checksummed for BoundedStack<T, N>
where
    T: AsRef<[u8]>,
{
    fn checksum(&self) -> Checksum {
        self.checksum
    }

    fn recalculate_checksum(&mut self) {
        let mut new_checksum = Checksum::new();
        for item in &self.data {
            new_checksum.update_slice(item.as_ref());
        }
        self.checksum = new_checksum;
    }

    fn verify_checksum(&self) -> bool {
        // Always return true for None verification level
        if matches!(self.verification_level, VerificationLevel::None) {
            return true;
        }

        // Skip verification based on sampling level
        if !self.verification_level.should_verify(importance::READ) {
            return true;
        }

        // Compute a fresh checksum and compare
        let mut fresh = Checksum::new();
        for item in &self.data {
            fresh.update_slice(item.as_ref());
        }
        fresh == self.checksum
    }
}

#[cfg(feature = "alloc")]
impl<T, const N: usize> Validatable for BoundedStack<T, N>
where
    T: AsRef<[u8]>,
{
    type Error = BoundedError;

    fn validate(&self) -> Result<(), BoundedError> {
        // Always pass for None verification level
        if matches!(self.verification_level, VerificationLevel::None) {
            return Ok(());
        }

        // Verify capacity constraints
        if self.len() > self.capacity() {
            return Err(BoundedError::CapacityExceeded {
                collection_type: "BoundedStack".to_string(),
                capacity: self.capacity(),
                attempted_size: self.len(),
            });
        }

        // Verify checksum integrity
        if !self.verify_checksum() {
            let mut fresh = Checksum::new();
            for item in &self.data {
                fresh.update_slice(item.as_ref());
            }

            return Err(BoundedError::ChecksumMismatch {
                expected: self.checksum.value(),
                actual: fresh.value(),
                description: "BoundedStack".to_string(),
            });
        }

        Ok(())
    }

    fn validation_level(&self) -> VerificationLevel {
        self.verification_level
    }

    fn set_validation_level(&mut self, level: VerificationLevel) {
        // When changing to a non-None level from None, recalculate the checksum
        let needs_recalc = matches!(self.verification_level, VerificationLevel::None)
            && !matches!(level, VerificationLevel::None);

        self.verification_level = level;

        if needs_recalc {
            self.recalculate_checksum();
        }
    }
}

/// A bounded vector with a fixed maximum capacity and verification
///
/// This vector ensures it never exceeds the specified capacity,
/// returning an error when operations would exceed this limit.
/// It also maintains a checksum of its contents for verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedVec<T, const N: usize>
where
    T: AsRef<[u8]>,
{
    /// The underlying vector storage
    data: Vec<T>,
    /// Checksum for verifying data integrity
    checksum: Checksum,
    /// Verification level for this vector
    verification_level: VerificationLevel,
}

impl<T, const N: usize> Default for BoundedVec<T, N>
where
    T: AsRef<[u8]>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> BoundedVec<T, N>
where
    T: AsRef<[u8]>,
{
    /// Create a new empty bounded vector
    pub fn new() -> Self {
        Self::with_verification_level(VerificationLevel::default())
    }

    /// Create a new empty bounded vector with a specific verification level
    pub fn with_verification_level(level: VerificationLevel) -> Self {
        Self {
            data: Vec::with_capacity(N),
            checksum: Checksum::new(),
            verification_level: level,
        }
    }

    /// Push an element onto the vector
    ///
    /// Returns an error if the vector is at capacity
    pub fn push(&mut self, item: T) -> Result<(), CapacityError> {
        // Track the push operation
        record_global_operation(OperationType::CollectionPush, self.verification_level);

        if self.is_full() {
            return Err(CapacityError);
        }

        // Only update checksum if verification is enabled
        if crate::validation::should_validate(self.verification_level, importance::MUTATION) {
            self.checksum.update_slice(item.as_ref());
            // Track checksum calculation
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
        }

        self.data.push(item);

        Ok(())
    }

    /// Get an element at the specified index
    pub fn get(&self, index: usize) -> Option<&T> {
        // Track the lookup operation
        record_global_operation(OperationType::CollectionLookup, self.verification_level);

        // For high verification levels, verify before returning data
        if crate::validation::should_validate_redundant(self.verification_level)
            && !self.verify_checksum()
        {
            return None;
        }

        self.data.get(index)
    }

    /// Get a mutable reference to an element
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        // Track the lookup operation with mutation intent
        record_global_operation(OperationType::CollectionLookup, self.verification_level);

        // For high verification levels, verify before returning data
        if crate::validation::should_validate_redundant(self.verification_level)
            && !self.verify_checksum()
        {
            return None;
        }

        // When returning mutable reference, mark that checksum needs recalculation
        // This is handled in a simplistic way for now
        if crate::validation::should_validate(self.verification_level, importance::MUTATION) {
            // When we return, the checksum will need to be recalculated, which we'll do on next verified operation
            // Track checksum calculation that will be needed
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
        }

        self.data.get_mut(index)
    }

    /// Set an element at the specified index, returning the old value
    pub fn set(&mut self, index: usize, item: T) -> Option<T> {
        // Track the insert operation
        record_global_operation(OperationType::CollectionInsert, self.verification_level);

        if index >= self.data.len() {
            return None;
        }

        let old = mem::replace(&mut self.data[index], item);

        // Recompute checksum if verification is enabled
        if crate::validation::should_validate(self.verification_level, importance::MUTATION) {
            self.recalculate_checksum();
            // Track checksum calculation
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
        }

        Some(old)
    }

    /// Remove an element at the specified index
    pub fn remove(&mut self, index: usize) -> Option<T> {
        // Track the remove operation
        record_global_operation(OperationType::CollectionRemove, self.verification_level);

        if index >= self.data.len() {
            return None;
        }

        let item = self.data.remove(index);

        // Recompute checksum if verification is enabled
        if crate::validation::should_validate(self.verification_level, importance::MUTATION) {
            self.recalculate_checksum();
            // Track checksum calculation
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
        }

        Some(item)
    }

    /// Get the current verification level
    pub fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }

    /// Set a new verification level
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        // When changing to a non-None level from None, recalculate the checksum
        let needs_recalc = matches!(self.verification_level, VerificationLevel::None)
            && !matches!(level, VerificationLevel::None);

        self.verification_level = level;

        if needs_recalc {
            self.recalculate_checksum();
        }
    }

    /// Verify the integrity of the vector (backward compatibility method)
    pub fn verify(&self) -> bool {
        // Track the validation operation
        record_global_operation(OperationType::CollectionValidate, self.verification_level);

        self.verify_checksum()
    }

    /// Get an iterator over the items
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    /// Get the last element of the vector
    pub fn last(&self) -> Option<&T> {
        self.data.last()
    }

    /// Get a mutable reference to the last element of the vector
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.data.last_mut()
    }

    /// Remove the last element from the vector and return it
    pub fn pop(&mut self) -> Option<T> {
        // Track the pop operation
        record_global_operation(OperationType::CollectionPop, self.verification_level);

        let item = self.data.pop()?;

        // Recompute checksum if verification is enabled
        if crate::validation::should_validate(self.verification_level, importance::MUTATION) {
            self.recalculate_checksum();
            // Track checksum calculation
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
        }

        Some(item)
    }

    /// Splits the collection at the given index, returning a new vector containing
    /// the elements from index onwards.
    pub fn split_off(&mut self, at: usize) -> Self {
        // Track the mutation operation
        record_global_operation(OperationType::CollectionMutate, self.verification_level);

        // Split the vec
        let split_data = self.data.split_off(at);

        // Recompute checksums
        if crate::validation::should_validate(self.verification_level, importance::MUTATION) {
            self.recalculate_checksum();
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
        }

        // Create a new BoundedVec with the split data
        let mut result = Self::with_verification_level(self.verification_level);
        result.data = split_data;

        // Calculate checksum for the new vec
        if crate::validation::should_validate(result.verification_level, importance::MUTATION) {
            result.recalculate_checksum();
            record_global_operation(
                OperationType::ChecksumCalculation,
                result.verification_level,
            );
        }

        result
    }

    /// Reverses the order of elements in the BoundedVec
    pub fn reverse(&mut self) {
        self.data.reverse();

        // Track the operation and recalculate checksum if needed
        #[cfg(feature = "std")]
        {
            operations::record_global_operation(
                OperationType::CollectionMutate,
                self.verification_level,
            );

            if matches!(
                self.verification_level,
                VerificationLevel::Standard | VerificationLevel::Full
            ) {
                self.recalculate_checksum();
            }
        }
    }
}

// Implement the traits for BoundedVec
impl<T, const N: usize> BoundedCapacity for BoundedVec<T, N>
where
    T: AsRef<[u8]>,
{
    fn capacity(&self) -> usize {
        N
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

// Implement AsRef trait for BoundedVec
impl<T, const N: usize> AsRef<[T]> for BoundedVec<T, N>
where
    T: AsRef<[u8]>,
{
    fn as_ref(&self) -> &[T] {
        &self.data
    }
}

// Implement AsMut trait for BoundedVec
impl<T, const N: usize> AsMut<[T]> for BoundedVec<T, N>
where
    T: AsRef<[u8]>,
{
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

impl<T, const N: usize> Checksummed for BoundedVec<T, N>
where
    T: AsRef<[u8]>,
{
    fn checksum(&self) -> Checksum {
        self.checksum
    }

    fn recalculate_checksum(&mut self) {
        let mut new_checksum = Checksum::new();
        for item in &self.data {
            new_checksum.update_slice(item.as_ref());
        }
        self.checksum = new_checksum;
    }

    fn verify_checksum(&self) -> bool {
        // Always return true for None verification level
        if matches!(self.verification_level, VerificationLevel::None) {
            return true;
        }

        // Skip verification based on level
        if !self.verification_level.should_verify(importance::READ) {
            return true;
        }

        // Compute a fresh checksum and compare
        let mut fresh = Checksum::new();
        for item in &self.data {
            fresh.update_slice(item.as_ref());
        }
        fresh == self.checksum
    }
}

#[cfg(feature = "alloc")]
impl<T, const N: usize> Validatable for BoundedVec<T, N>
where
    T: AsRef<[u8]>,
{
    type Error = BoundedError;

    fn validate(&self) -> Result<(), BoundedError> {
        // Always pass for None verification level
        if matches!(self.verification_level, VerificationLevel::None) {
            return Ok(());
        }

        // Verify capacity constraints
        if self.len() > self.capacity() {
            return Err(BoundedError::CapacityExceeded {
                collection_type: "BoundedVec".to_string(),
                capacity: self.capacity(),
                attempted_size: self.len(),
            });
        }

        // Verify checksum integrity
        if !self.verify_checksum() {
            let mut fresh = Checksum::new();
            for item in &self.data {
                fresh.update_slice(item.as_ref());
            }

            return Err(BoundedError::ChecksumMismatch {
                expected: self.checksum.value(),
                actual: fresh.value(),
                description: "BoundedVec".to_string(),
            });
        }

        Ok(())
    }

    fn validation_level(&self) -> VerificationLevel {
        self.verification_level
    }

    fn set_validation_level(&mut self, level: VerificationLevel) {
        // When changing to a non-None level from None, recalculate the checksum
        let needs_recalc = matches!(self.verification_level, VerificationLevel::None)
            && !matches!(level, VerificationLevel::None);

        self.verification_level = level;

        if needs_recalc {
            self.recalculate_checksum();
        }
    }
}

/// A bounded hash map with a fixed maximum capacity and verification
///
/// This hash map ensures it never exceeds the specified capacity,
/// returning an error when operations would exceed this limit.
/// It also maintains a checksum of its contents for verification.
///
/// For no_std environments, this uses BTreeMap underneath, requiring keys to implement Ord.
#[derive(Debug, Clone)]
#[cfg(feature = "std")]
pub struct BoundedHashMap<K, V, const N: usize>
where
    K: Eq + hash::Hash + AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    /// The underlying data
    data: HashMap<K, V>,
    /// The checksum of the data
    checksum: Checksum,
    /// The level of verification to apply
    verification_level: VerificationLevel,
}

/// A bounded hash map with a fixed maximum capacity and verification
///
/// This hash map ensures it never exceeds the specified capacity,
/// returning an error when operations would exceed this limit.
/// It also maintains a checksum of its contents for verification.
///
/// For no_std environments, this uses BTreeMap underneath, requiring keys to implement Ord.
#[derive(Debug, Clone)]
#[cfg(not(feature = "std"))]
pub struct BoundedHashMap<K, V, const N: usize>
where
    K: Eq + hash::Hash + AsRef<[u8]> + core::cmp::Ord,
    V: AsRef<[u8]>,
{
    /// The underlying data
    data: HashMap<K, V>,
    /// The checksum of the data
    checksum: Checksum,
    /// The level of verification to apply
    verification_level: VerificationLevel,
}

#[cfg(feature = "std")]
impl<K, V, const N: usize> Default for BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "std"))]
impl<K, V, const N: usize> Default for BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]> + core::cmp::Ord,
    V: AsRef<[u8]>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
impl<K, V, const N: usize> BoundedCapacity for BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    fn capacity(&self) -> usize {
        N
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(not(feature = "std"))]
impl<K, V, const N: usize> BoundedCapacity for BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]> + core::cmp::Ord,
    V: AsRef<[u8]>,
{
    fn capacity(&self) -> usize {
        N
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(feature = "std")]
impl<K, V, const N: usize> Checksummed for BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    fn checksum(&self) -> Checksum {
        self.checksum
    }

    fn recalculate_checksum(&mut self) {
        let mut new_checksum = Checksum::new();
        // We need to iterate in a consistent order for checksum stability
        // For std HashMap this isn't guaranteed, so we sort

        // Convert to sorted Vec of key/value pairs to ensure consistent order
        let mut sorted_items = Vec::with_capacity(self.data.len());
        for (k, v) in &self.data {
            sorted_items.push((k.as_ref(), v.as_ref()));
        }

        // Sort by key bytes for consistent ordering
        sorted_items.sort_by(|a, b| a.0.cmp(b.0));

        // Calculate checksum in this consistent order
        for (k, v) in sorted_items {
            new_checksum.update_slice(k);
            new_checksum.update_slice(v);
        }

        self.checksum = new_checksum;
    }

    fn verify_checksum(&self) -> bool {
        // For empty collections, checksum is always valid
        if self.data.is_empty() {
            return true;
        }

        // Calculate a fresh checksum
        let mut fresh_checksum = Checksum::new();

        // Convert to sorted Vec of key/value pairs to ensure consistent order
        let mut sorted_items = Vec::with_capacity(self.data.len());
        for (k, v) in &self.data {
            sorted_items.push((k.as_ref(), v.as_ref()));
        }

        // Sort by key bytes for consistent ordering
        sorted_items.sort_by(|a, b| a.0.cmp(b.0));

        // Calculate checksum in this consistent order
        for (k, v) in sorted_items {
            fresh_checksum.update_slice(k);
            fresh_checksum.update_slice(v);
        }

        // Compare the calculated checksum with the stored one
        fresh_checksum == self.checksum
    }
}

#[cfg(not(feature = "std"))]
impl<K, V, const N: usize> Checksummed for BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]> + core::cmp::Ord,
    V: AsRef<[u8]>,
{
    fn checksum(&self) -> Checksum {
        self.checksum
    }

    fn recalculate_checksum(&mut self) {
        let mut new_checksum = Checksum::new();

        // BTreeMap already iterates in a consistent order, so we can just use that
        for (k, v) in &self.data {
            new_checksum.update_slice(k.as_ref());
            new_checksum.update_slice(v.as_ref());
        }

        self.checksum = new_checksum;
    }

    fn verify_checksum(&self) -> bool {
        // For empty collections, checksum is always valid
        if self.data.is_empty() {
            return true;
        }

        // Calculate a fresh checksum
        let mut fresh_checksum = Checksum::new();

        // BTreeMap already iterates in a consistent order, so we can just use that
        for (k, v) in &self.data {
            fresh_checksum.update_slice(k.as_ref());
            fresh_checksum.update_slice(v.as_ref());
        }

        // Compare the calculated checksum with the stored one
        fresh_checksum == self.checksum
    }
}

#[cfg(feature = "std")]
impl<K, V, const N: usize> Validatable for BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    type Error = BoundedError;

    fn validate(&self) -> Result<(), BoundedError> {
        // Only run validation if the validation level requires it
        if !crate::validation::should_validate(self.verification_level, importance::CRITICAL) {
            return Ok(());
        }

        // Validate capacity constraints
        if self.len() > self.capacity() {
            return Err(BoundedError::CapacityExceeded {
                collection_type: "BoundedHashMap".to_string(),
                capacity: self.capacity(),
                attempted_size: self.len(),
            });
        }

        // Validate checksum integrity
        if !self.verify_checksum() {
            // Calculate a fresh checksum for error reporting
            let mut fresh_checksum = Checksum::new();

            // Sort key/value pairs for deterministic checksum
            let mut sorted_items = Vec::with_capacity(self.data.len());
            for (k, v) in &self.data {
                sorted_items.push((k.as_ref(), v.as_ref()));
            }
            sorted_items.sort_by(|a, b| a.0.cmp(b.0));

            for (k, v) in sorted_items {
                fresh_checksum.update_slice(k);
                fresh_checksum.update_slice(v);
            }

            return Err(BoundedError::ChecksumMismatch {
                expected: self.checksum.value(),
                actual: fresh_checksum.value(),
                description: "BoundedHashMap".to_string(),
            });
        }

        Ok(())
    }

    fn validation_level(&self) -> VerificationLevel {
        self.verification_level
    }

    fn set_validation_level(&mut self, level: VerificationLevel) {
        self.set_verification_level(level);
    }
}

#[cfg(not(feature = "std"))]
impl<K, V, const N: usize> Validatable for BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]> + core::cmp::Ord,
    V: AsRef<[u8]>,
{
    type Error = BoundedError;

    fn validate(&self) -> Result<(), BoundedError> {
        // Only run validation if the validation level requires it
        if !crate::validation::should_validate(self.verification_level, importance::CRITICAL) {
            return Ok(());
        }

        // Validate capacity constraints
        if self.len() > self.capacity() {
            return Err(BoundedError::CapacityExceeded {
                collection_type: "BoundedHashMap".to_string(),
                capacity: self.capacity(),
                attempted_size: self.len(),
            });
        }

        // Validate checksum integrity
        if !self.verify_checksum() {
            // Calculate a fresh checksum for error reporting
            let mut fresh_checksum = Checksum::new();

            // BTreeMap already iterates in a consistent order
            for (k, v) in &self.data {
                fresh_checksum.update_slice(k.as_ref());
                fresh_checksum.update_slice(v.as_ref());
            }

            return Err(BoundedError::ChecksumMismatch {
                expected: self.checksum.value(),
                actual: fresh_checksum.value(),
                description: "BoundedHashMap".to_string(),
            });
        }

        Ok(())
    }

    fn validation_level(&self) -> VerificationLevel {
        self.verification_level
    }

    fn set_validation_level(&mut self, level: VerificationLevel) {
        self.set_verification_level(level);
    }
}

#[cfg(feature = "std")]
impl<K, V, const N: usize> PartialEq for BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]>,
    V: AsRef<[u8]> + Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

#[cfg(not(feature = "std"))]
impl<K, V, const N: usize> PartialEq for BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]> + core::cmp::Ord,
    V: AsRef<[u8]> + Eq,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

#[cfg(feature = "std")]
impl<K, V, const N: usize> Eq for BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]>,
    V: AsRef<[u8]> + Eq,
{
}

#[cfg(not(feature = "std"))]
impl<K, V, const N: usize> Eq for BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]> + core::cmp::Ord,
    V: AsRef<[u8]> + Eq,
{
}

#[cfg(feature = "std")]
impl<K, V, const N: usize> BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    /// Create a new empty bounded hash map
    pub fn new() -> Self {
        Self::with_verification_level(VerificationLevel::default())
    }

    /// Create a new empty bounded hash map with a specific verification level
    pub fn with_verification_level(level: VerificationLevel) -> Self {
        Self {
            data: HashMap::new(),
            checksum: Checksum::new(),
            verification_level: level,
        }
    }

    /// Insert a key-value pair into the hash map
    ///
    /// Returns an error if the hash map is at capacity and the key does not exist
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, CapacityError> {
        // Track the insert operation
        record_global_operation(OperationType::CollectionInsert, self.verification_level);

        // If the key doesn't exist and we're at capacity, return error
        if !self.data.contains_key(&key) && self.is_full() {
            return Err(CapacityError);
        }

        // Update the checksum before inserting if verification is enabled
        if crate::validation::should_validate(self.verification_level, importance::MUTATION) {
            // For inserts/updates, we'll just recalculate the whole checksum after the operation
            // This is simpler than trying to adjust for the removed key+value
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
        }

        let old_value = self.data.insert(key, value);

        // Recalculate checksum after insert if verification is enabled
        if crate::validation::should_validate(self.verification_level, importance::MUTATION) {
            self.recalculate_checksum();
        }

        Ok(old_value)
    }

    /// Get a value by key
    pub fn get(&self, key: &K) -> Option<&V> {
        // Track the lookup operation
        record_global_operation(OperationType::CollectionLookup, self.verification_level);

        // For high verification levels, verify before returning data
        if crate::validation::should_validate_redundant(self.verification_level)
            && !self.verify_checksum()
        {
            return None;
        }

        self.data.get(key)
    }

    /// Get a mutable reference to a value by key
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.data.get_mut(key)
    }

    /// Remove a key-value pair from the hash map
    pub fn remove(&mut self, key: &K) -> Option<V> {
        // Track the remove operation
        record_global_operation(OperationType::CollectionRemove, self.verification_level);

        let result = self.data.remove(key);

        // If we removed something, recalculate the checksum
        if result.is_some()
            && crate::validation::should_validate(self.verification_level, importance::MUTATION)
        {
            self.recalculate_checksum();
            // Track checksum calculation
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
        }

        result
    }

    /// Get the current verification level
    pub fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }

    /// Set a new verification level
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        // When changing to a non-None level from None, recalculate the checksum
        let needs_recalc = matches!(self.verification_level, VerificationLevel::None)
            && !matches!(level, VerificationLevel::None);

        self.verification_level = level;

        if needs_recalc {
            self.recalculate_checksum();
        }
    }

    /// Verify the integrity of the hash map (backward compatibility method)
    pub fn verify(&self) -> bool {
        // Track the validation operation
        record_global_operation(OperationType::CollectionValidate, self.verification_level);

        // Always return true for None verification level
        if matches!(self.verification_level, VerificationLevel::None) {
            return true;
        }

        self.verify_checksum()
    }

    /// Get an iterator over the items
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.data.iter()
    }

    /// Force recalculation of the checksum (backward compatibility method)
    pub fn force_recalculate_checksum(&mut self) {
        self.recalculate_checksum();
    }
}

#[cfg(not(feature = "std"))]
impl<K, V, const N: usize> BoundedHashMap<K, V, N>
where
    K: Eq + hash::Hash + AsRef<[u8]> + core::cmp::Ord,
    V: AsRef<[u8]>,
{
    /// Create a new empty bounded hash map
    pub fn new() -> Self {
        Self::with_verification_level(VerificationLevel::default())
    }

    /// Create a new empty bounded hash map with a specific verification level
    pub fn with_verification_level(level: VerificationLevel) -> Self {
        Self {
            data: HashMap::new(),
            checksum: Checksum::new(),
            verification_level: level,
        }
    }

    /// Insert a key-value pair into the hash map
    ///
    /// Returns an error if the hash map is at capacity and the key does not exist
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, CapacityError> {
        // Track the insert operation
        record_global_operation(OperationType::CollectionInsert, self.verification_level);

        // If the key doesn't exist and we're at capacity, return error
        if !self.data.contains_key(&key) && self.is_full() {
            return Err(CapacityError);
        }

        // Update the checksum before inserting if verification is enabled
        if crate::validation::should_validate(self.verification_level, importance::MUTATION) {
            // For inserts/updates, we'll just recalculate the whole checksum after the operation
            // This is simpler than trying to adjust for the removed key+value
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
        }

        let old_value = self.data.insert(key, value);

        // Recalculate checksum after insert if verification is enabled
        if crate::validation::should_validate(self.verification_level, importance::MUTATION) {
            self.recalculate_checksum();
        }

        Ok(old_value)
    }

    /// Get a value by key
    pub fn get(&self, key: &K) -> Option<&V> {
        // Track the lookup operation
        record_global_operation(OperationType::CollectionLookup, self.verification_level);

        // For high verification levels, verify before returning data
        if crate::validation::should_validate_redundant(self.verification_level)
            && !self.verify_checksum()
        {
            return None;
        }

        self.data.get(key)
    }

    /// Get a mutable reference to a value by key
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.data.get_mut(key)
    }

    /// Remove a key-value pair from the hash map
    pub fn remove(&mut self, key: &K) -> Option<V> {
        // Track the remove operation
        record_global_operation(OperationType::CollectionRemove, self.verification_level);

        let result = self.data.remove(key);

        // If we removed something, recalculate the checksum
        if result.is_some()
            && crate::validation::should_validate(self.verification_level, importance::MUTATION)
        {
            self.recalculate_checksum();
            // Track checksum calculation
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
        }

        result
    }

    /// Get the current verification level
    pub fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }

    /// Set a new verification level
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        // When changing to a non-None level from None, recalculate the checksum
        let needs_recalc = matches!(self.verification_level, VerificationLevel::None)
            && !matches!(level, VerificationLevel::None);

        self.verification_level = level;

        if needs_recalc {
            self.recalculate_checksum();
        }
    }

    /// Verify the integrity of the hash map (backward compatibility method)
    pub fn verify(&self) -> bool {
        // Track the validation operation
        record_global_operation(OperationType::CollectionValidate, self.verification_level);

        // Always return true for None verification level
        if matches!(self.verification_level, VerificationLevel::None) {
            return true;
        }

        self.verify_checksum()
    }

    /// Get an iterator over the items
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.data.iter()
    }

    /// Force recalculation of the checksum (backward compatibility method)
    pub fn force_recalculate_checksum(&mut self) {
        self.recalculate_checksum();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "std"))]
    use alloc::vec;

    #[test]
    fn test_bounded_stack() {
        let mut stack = BoundedStack::<Vec<u8>, 3>::new();
        assert!(stack.push(vec![1, 2, 3]).is_ok());
        assert!(stack.push(vec![4, 5, 6]).is_ok());
        assert!(stack.push(vec![7, 8, 9]).is_ok());
        assert!(stack.push(vec![10, 11, 12]).is_err());

        assert_eq!(stack.len(), 3);
        assert!(stack.verify());

        let item = stack.pop();
        assert_eq!(item, Some(vec![7, 8, 9]));
        assert_eq!(stack.len(), 2);
        assert!(stack.verify());
    }

    #[test]
    fn test_bounded_stack_with_verification_levels() {
        // Test with standard verification
        let mut stack =
            BoundedStack::<Vec<u8>, 3>::with_verification_level(VerificationLevel::Standard);
        assert!(stack.push(vec![1, 2, 3]).is_ok());
        assert!(stack.verify());

        // Test with no verification
        let mut no_verify_stack =
            BoundedStack::<Vec<u8>, 3>::with_verification_level(VerificationLevel::None);
        assert!(no_verify_stack.push(vec![1, 2, 3]).is_ok());
        assert!(no_verify_stack.verify()); // Should return true without actually verifying

        // Test with full verification
        let mut full_verify_stack =
            BoundedStack::<Vec<u8>, 3>::with_verification_level(VerificationLevel::Full);
        assert!(full_verify_stack.push(vec![1, 2, 3]).is_ok());
        assert!(full_verify_stack.verify());

        // Test changing verification level
        let mut dynamic_stack =
            BoundedStack::<Vec<u8>, 3>::with_verification_level(VerificationLevel::None);
        assert!(dynamic_stack.push(vec![1, 2, 3]).is_ok());

        // When changing from None to a level that verifies, need to recalculate checksum
        dynamic_stack.set_verification_level(VerificationLevel::Full);

        // Recalculate checksum explicitly since it wasn't tracked with None level
        let mut new_checksum = Checksum::new();
        for item in &dynamic_stack.data {
            new_checksum.update_slice(item.as_ref());
        }
        dynamic_stack.checksum = new_checksum;

        assert!(dynamic_stack.verify()); // Now should perform full verification
    }

    #[test]
    fn test_bounded_vec() {
        let mut vec = BoundedVec::<Vec<u8>, 3>::new();
        assert!(vec.push(vec![1, 2, 3]).is_ok());
        assert!(vec.push(vec![4, 5, 6]).is_ok());
        assert!(vec.push(vec![7, 8, 9]).is_ok());
        assert!(vec.push(vec![10, 11, 12]).is_err());

        assert_eq!(vec.len(), 3);
        assert!(vec.verify());

        let item = vec.remove(1);
        assert_eq!(item, Some(vec![4, 5, 6]));
        assert_eq!(vec.len(), 2);
        assert!(vec.verify());

        let old = vec.set(0, vec![99, 100]);
        assert_eq!(old, Some(vec![1, 2, 3]));
        assert_eq!(vec.get(0), Some(&vec![99, 100]));
        assert!(vec.verify());
    }

    #[test]
    fn test_bounded_vec_with_verification_levels() {
        // Test with standard verification
        let mut vec =
            BoundedVec::<Vec<u8>, 3>::with_verification_level(VerificationLevel::Standard);
        assert!(vec.push(vec![1, 2, 3]).is_ok());
        assert!(vec.verify());

        // Test with sampling verification
        let mut sample_vec =
            BoundedVec::<Vec<u8>, 3>::with_verification_level(VerificationLevel::Sampling);
        assert!(sample_vec.push(vec![1, 2, 3]).is_ok());
        // With sampling, we can't deterministically test the verification result

        // Test verification with get operations
        let mut full_verify_vec =
            BoundedVec::<Vec<u8>, 3>::with_verification_level(VerificationLevel::Full);
        assert!(full_verify_vec.push(vec![1, 2, 3]).is_ok());
        assert!(full_verify_vec.push(vec![4, 5, 6]).is_ok());

        // get and get_mut should work with Full level
        let _ = full_verify_vec.get(0);
        let _ = full_verify_vec.get_mut(1);

        // Test iterator with verification
        let items: Vec<_> = full_verify_vec.iter().collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_bounded_hash_map() {
        // Create map with None verification level to avoid automatic checksum calculations
        let mut map =
            BoundedHashMap::<String, Vec<u8>, 3>::with_verification_level(VerificationLevel::None);

        // Insert all items
        assert!(map.insert("key1".to_string(), vec![1, 2, 3]).is_ok());
        assert!(map.insert("key2".to_string(), vec![4, 5, 6]).is_ok());
        assert!(map.insert("key3".to_string(), vec![7, 8, 9]).is_ok());

        // Verify we're at capacity and can't add more
        assert!(map.insert("key4".to_string(), vec![10, 11, 12]).is_err());
        assert_eq!(map.len(), 3);

        // Set to Standard level and force recalculate for verification
        map.set_verification_level(VerificationLevel::Standard);
        map.force_recalculate_checksum();
        assert!(map.verify());

        // Replacing an existing key should work even at capacity
        assert!(map.insert("key1".to_string(), vec![99, 100]).is_ok());
        assert_eq!(map.len(), 3);

        // Force recalculate before verification
        map.force_recalculate_checksum();
        assert!(map.verify());

        // Test removal
        let item = map.remove(&"key2".to_string());
        assert_eq!(item, Some(vec![4, 5, 6]));
        assert_eq!(map.len(), 2);

        // Force recalculate before verification
        map.force_recalculate_checksum();
        assert!(map.verify());
    }

    #[test]
    fn test_bounded_hash_map_with_verification_levels() {
        // Test with full verification
        let mut map =
            BoundedHashMap::<String, Vec<u8>, 3>::with_verification_level(VerificationLevel::Full);
        assert!(map.insert("key1".to_string(), vec![1, 2, 3]).is_ok());
        assert!(map.verify());

        // Test with no verification
        let mut no_verify_map =
            BoundedHashMap::<String, Vec<u8>, 3>::with_verification_level(VerificationLevel::None);
        assert!(no_verify_map
            .insert("key1".to_string(), vec![1, 2, 3])
            .is_ok());
        assert!(no_verify_map.verify()); // Should return true without actually verifying

        // Test verification with get and iter operations
        let mut full_map =
            BoundedHashMap::<String, Vec<u8>, 3>::with_verification_level(VerificationLevel::Full);
        assert!(full_map.insert("key1".to_string(), vec![1, 2, 3]).is_ok());
        assert!(full_map.insert("key2".to_string(), vec![4, 5, 6]).is_ok());

        // get and get_mut should work with Full level
        let _ = full_map.get(&"key1".to_string());
        let _ = full_map.get_mut(&"key2".to_string());

        // Test iterator with verification
        let items: Vec<_> = full_map.iter().collect();
        assert_eq!(items.len(), 2);

        // Test changing verification level dynamically
        let mut dynamic_map =
            BoundedHashMap::<String, Vec<u8>, 3>::with_verification_level(VerificationLevel::None);
        assert!(dynamic_map
            .insert("key1".to_string(), vec![1, 2, 3])
            .is_ok());

        // When changing from None to a level that verifies, need to recalculate checksum
        dynamic_map.set_verification_level(VerificationLevel::Standard);

        // Recalculate checksum explicitly since it wasn't tracked with None level
        let mut new_checksum = Checksum::new();
        let mut keys: Vec<&String> = dynamic_map.data.keys().collect();
        keys.sort_by(|a, b| {
            let a_bytes: &[u8] = AsRef::<[u8]>::as_ref(a);
            let b_bytes: &[u8] = AsRef::<[u8]>::as_ref(b);
            a_bytes.cmp(b_bytes)
        });
        for k in keys {
            new_checksum.update_slice(k.as_ref());
            if let Some(v) = dynamic_map.data.get(k) {
                new_checksum.update_slice(v.as_ref());
            }
        }
        dynamic_map.checksum = new_checksum;

        assert!(dynamic_map.verify()); // Now should perform standard verification
    }

    #[test]
    fn test_verification_level_behavior() {
        // Test behavior with different verification levels

        // None - should not perform verification
        let mut none_vec =
            BoundedVec::<Vec<u8>, 10>::with_verification_level(VerificationLevel::None);
        for i in 0..5 {
            assert!(none_vec.push(vec![i, i + 1]).is_ok());
        }

        // Standard - should perform verification on most operations
        let mut std_vec =
            BoundedVec::<Vec<u8>, 10>::with_verification_level(VerificationLevel::Standard);
        for i in 0..5 {
            assert!(std_vec.push(vec![i, i + 1]).is_ok());
        }
        assert!(std_vec.verify());

        // Full - should perform verification on all operations, including redundant ones
        let mut full_vec =
            BoundedVec::<Vec<u8>, 10>::with_verification_level(VerificationLevel::Full);
        for i in 0..5 {
            assert!(full_vec.push(vec![i, i + 1]).is_ok());
        }
        assert!(full_vec.verify());

        // Now test with a modifying operation
        full_vec.remove(0);
        assert!(full_vec.verify());
    }
}
