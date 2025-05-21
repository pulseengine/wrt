// WRT - wrt-types
// Module: No-std HashMap Implementation
//
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! A simple HashMap implementation for no_std environments without external dependencies.
//!
//! This module provides a basic hash map implementation that is suitable for
//! no_std/no_alloc environments. It has limited functionality compared to
//! the standard HashMap or external crates like hashbrown, but it provides
//! the core functionality needed for the WRT ecosystem.

use core::borrow::Borrow;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;

use crate::bounded::BoundedVec;
use crate::verification::Checksum;
use crate::MemoryProvider;

/// A simple fixed-capacity hash map implementation for no_std environments.
///
/// This implementation uses linear probing for collision resolution and
/// supports a fixed maximum number of entries specified by the N parameter.
#[derive(Debug)]
pub struct SimpleHashMap<K, V, const N: usize, P: MemoryProvider + Default + Clone + fmt::Debug + PartialEq + Eq> {
    entries: BoundedVec<Option<Entry<K, V>>, N, P>,
    len: usize,
    _phantom: PhantomData<(K, V, P)>,
}

/// A key-value pair entry in the hash map.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry<K, V> {
    key: K,
    value: V,
    hash: u64,
}

impl<K, V, const N: usize, P: MemoryProvider + Default + Clone + fmt::Debug + PartialEq + Eq> SimpleHashMap<K, V, N, P>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Creates a new empty SimpleHashMap with the given memory provider.
    pub fn new(provider: P) -> crate::WrtResult<Self> {
        let mut entries = BoundedVec::new(provider)?;
        
        // Pre-populate with None values to indicate empty slots
        for _ in 0..N {
            entries.push(None)?;
        }
        
        Ok(Self {
            entries,
            len: 0,
            _phantom: PhantomData,
        })
    }
    
    /// Returns the number of key-value pairs in the map.
    pub fn len(&self) -> usize {
        self.len
    }
    
    /// Returns true if the map contains no key-value pairs.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    /// Returns true if the map cannot hold any more key-value pairs.
    pub fn is_full(&self) -> bool {
        self.len == N
    }
    
    /// Returns the capacity of the map.
    pub fn capacity(&self) -> usize {
        N
    }
    
    /// Calculates the hash of a key.
    fn hash_key<Q: ?Sized + Hash>(&self, key: &Q) -> u64 {
        let mut hasher = Checksum::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Calculates the initial index for a key.
    fn initial_index<Q: ?Sized + Hash>(&self, hash: u64) -> usize {
        (hash as usize) % N
    }
    
    /// Inserts a key-value pair into the map.
    ///
    /// If the key already exists, the old value is replaced and returned.
    /// If the map is full and the key doesn't exist, returns an error.
    pub fn insert(&mut self, key: K, value: V) -> crate::WrtResult<Option<V>> 
    where
        K: Hash,
    {
        if self.is_full() {
            // Check if the key already exists
            let hash = self.hash_key(&key);
            let mut index = self.initial_index(hash);
            
            for i in 0..N {
                let actual_index = (index + i) % N;
                
                if let Some(entry) = self.entries.get(actual_index)? {
                    if entry.hash == hash && entry.key == key {
                        // Key exists, replace value
                        let old_value = entry.value.clone();
                        let mut entry = entry.clone();
                        entry.value = value;
                        self.entries.set(actual_index, Some(entry))?;
                        return Ok(Some(old_value));
                    }
                }
            }
            
            // Map is full and key doesn't exist
            return Err(crate::Error::capacity_error("SimpleHashMap is full"));
        }
        
        let hash = self.hash_key(&key);
        let mut index = self.initial_index(hash);
        
        // Find the slot for this key (either empty or matching key)
        for i in 0..N {
            let actual_index = (index + i) % N;
            
            match self.entries.get(actual_index)? {
                Some(entry) if entry.hash == hash && entry.key == key => {
                    // Key exists, replace value
                    let old_value = entry.value.clone();
                    let mut entry = entry.clone();
                    entry.value = value;
                    self.entries.set(actual_index, Some(entry))?;
                    return Ok(Some(old_value));
                },
                None => {
                    // Empty slot, insert new entry
                    let entry = Entry {
                        key,
                        value,
                        hash,
                    };
                    self.entries.set(actual_index, Some(entry))?;
                    self.len += 1;
                    return Ok(None);
                },
                _ => {
                    // Occupied by a different key, try next slot
                    continue;
                }
            }
        }
        
        // This should never happen as we checked if the map is full
        Err(crate::Error::internal_error("Failed to insert into SimpleHashMap"))
    }
    
    /// Gets a reference to the value associated with the key.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> crate::WrtResult<Option<V>> 
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let hash = self.hash_key(key);
        let mut index = self.initial_index(hash);
        
        for i in 0..N {
            let actual_index = (index + i) % N;
            
            match self.entries.get(actual_index)? {
                Some(entry) if entry.hash == hash && entry.key.borrow() == key => {
                    return Ok(Some(entry.value.clone()));
                },
                None => {
                    // Empty slot, key doesn't exist
                    return Ok(None);
                },
                _ => {
                    // Occupied by a different key, try next slot
                    continue;
                }
            }
        }
        
        // We searched all slots and didn't find the key
        Ok(None)
    }
    
    /// Removes a key from the map, returning the value if it was present.
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> crate::WrtResult<Option<V>> 
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let hash = self.hash_key(key);
        let mut index = self.initial_index(hash);
        
        for i in 0..N {
            let actual_index = (index + i) % N;
            
            match self.entries.get(actual_index)? {
                Some(entry) if entry.hash == hash && entry.key.borrow() == key => {
                    let value = entry.value.clone();
                    self.entries.set(actual_index, None)?;
                    self.len -= 1;
                    return Ok(Some(value));
                },
                None => {
                    // Empty slot, key doesn't exist
                    return Ok(None);
                },
                _ => {
                    // Occupied by a different key, try next slot
                    continue;
                }
            }
        }
        
        // We searched all slots and didn't find the key
        Ok(None)
    }
    
    /// Clears the map, removing all key-value pairs.
    pub fn clear(&mut self) -> crate::WrtResult<()> {
        for i in 0..N {
            self.entries.set(i, None)?;
        }
        self.len = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::safe_memory::NoStdProvider;
    
    #[test]
    fn test_simple_hashmap() {
        let provider = NoStdProvider::default();
        let mut map = SimpleHashMap::<&str, i32, 8, NoStdProvider>::new(provider).unwrap();
        
        // Test insertion
        assert!(map.insert("one", 1).unwrap().is_none());
        assert!(map.insert("two", 2).unwrap().is_none());
        assert!(map.insert("three", 3).unwrap().is_none());
        
        // Test get
        assert_eq!(map.get("one").unwrap(), Some(1));
        assert_eq!(map.get("two").unwrap(), Some(2));
        assert_eq!(map.get("three").unwrap(), Some(3));
        assert_eq!(map.get("four").unwrap(), None);
        
        // Test replacing a value
        assert_eq!(map.insert("one", 10).unwrap(), Some(1));
        assert_eq!(map.get("one").unwrap(), Some(10));
        
        // Test removing a value
        assert_eq!(map.remove("two").unwrap(), Some(2));
        assert_eq!(map.get("two").unwrap(), None);
        
        // Test len and is_empty
        assert_eq!(map.len(), 2);
        assert!(!map.is_empty());
        
        // Test clear
        map.clear().unwrap();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        assert_eq!(map.get("one").unwrap(), None);
    }
    
    #[test]
    fn test_full_map() {
        let provider = NoStdProvider::default();
        let mut map = SimpleHashMap::<i32, i32, 4, NoStdProvider>::new(provider).unwrap();
        
        // Fill the map
        assert!(map.insert(1, 10).unwrap().is_none());
        assert!(map.insert(2, 20).unwrap().is_none());
        assert!(map.insert(3, 30).unwrap().is_none());
        assert!(map.insert(4, 40).unwrap().is_none());
        
        // Map is full
        assert!(map.is_full());
        
        // Can replace existing keys
        assert_eq!(map.insert(1, 100).unwrap(), Some(10));
        
        // But can't add new keys
        assert!(map.insert(5, 50).is_err());
    }
}