//! No-std, no-alloc hash map implementation
//!
//! This module provides a simple hash map implementation that works in
//! no_std and no_alloc environments. It uses a fixed-size array and linear
//! search, making it suitable for small maps but not efficient for large ones.

use core::{fmt, marker::PhantomData};

use wrt_error::{codes, Error, Result};
use wrt_foundation::{bounded::BoundedVec, MemoryProvider};

/// A simple key-value pair for use in bounded hash maps
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValuePair<K, V> {
    /// The key
    pub key: K,
    /// The value
    pub value: V,
}

/// A fixed-size hash map implementation for no_std/no_alloc environments
///
/// This implementation uses a bounded vector of key-value pairs and performs
/// linear search for lookups. It's suitable for small maps in constrained
/// environments but does not scale well to large maps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedHashMap<
    K,
    V,
    const N: usize,
    P: MemoryProvider + Default + Clone + PartialEq + Eq,
> {
    /// The storage for key-value pairs
    entries: BoundedVec<KeyValuePair<K, V>, N, P>,
    /// Marker for the key type
    _key_marker: PhantomData<K>,
    /// Marker for the value type
    _value_marker: PhantomData<V>,
}

impl<K, V, const N: usize, P: MemoryProvider + Default + Clone + PartialEq + Eq>
    BoundedHashMap<K, V, N, P>
where
    K: PartialEq + Eq + Clone,
    V: Clone,
{
    /// Creates a new, empty bounded hash map
    pub fn new(provider: P) -> Result<Self> {
        Ok(Self {
            entries: BoundedVec::new(provider)?,
            _key_marker: PhantomData,
            _value_marker: PhantomData,
        })
    }

    /// Returns the number of key-value pairs in the map
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the map contains no key-value pairs
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the maximum capacity of the map
    pub fn capacity(&self) -> usize {
        N
    }

    /// Inserts a key-value pair into the map
    ///
    /// If the key already exists, the value is updated and the old value is
    /// returned. If the key doesn't exist, the pair is inserted and None is
    /// returned.
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>> {
        // Check if the key already exists
        for i in 0..self.entries.len() {
            if self.entries[i].key == key {
                // Update existing entry
                let old_value = self.entries[i].value.clone();
                self.entries[i].value = value;
                return Ok(Some(old_value));
            }
        }

        // Key doesn't exist, insert new entry
        let pair = KeyValuePair { key, value };
        self.entries.push(pair)?;
        Ok(None)
    }

    /// Removes a key-value pair from the map by key
    ///
    /// If the key exists, the pair is removed and the value is returned.
    /// If the key doesn't exist, None is returned.
    pub fn remove(&mut self, key: &K) -> Result<Option<V>> {
        // Find the index of the key
        let mut index = None;
        for i in 0..self.entries.len() {
            if &self.entries[i].key == key {
                index = Some(i);
                break;
            }
        }

        // Remove the entry if found
        if let Some(idx) = index {
            let pair = self.entries.remove(idx)?;
            return Ok(Some(pair.value));
        }

        Ok(None)
    }

    /// Returns a reference to the value associated with the key
    pub fn get(&self, key: &K) -> Option<&V> {
        for i in 0..self.entries.len() {
            if &self.entries[i].key == key {
                return Some(&self.entries[i].value);
            }
        }
        None
    }

    /// Returns a mutable reference to the value associated with the key
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        for i in 0..self.entries.len() {
            if &self.entries[i].key == key {
                return Some(&mut self.entries[i].value);
            }
        }
        None
    }

    /// Returns true if the map contains the key
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Clears the map, removing all key-value pairs
    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear()?;
        Ok(())
    }

    /// Returns an iterator over the key-value pairs
    pub fn iter(&self) -> BoundedHashMapIter<K, V, N, P> {
        BoundedHashMapIter { map: self, index: 0 }
    }
}

/// An iterator over the key-value pairs in a bounded hash map
pub struct BoundedHashMapIter<
    'a,
    K,
    V,
    const N: usize,
    P: MemoryProvider + Default + Clone + PartialEq + Eq,
> {
    /// The map being iterated
    map: &'a BoundedHashMap<K, V, N, P>,
    /// The current index in the entries vector
    index: usize,
}

impl<'a, K, V, const N: usize, P: MemoryProvider + Default + Clone + PartialEq + Eq> Iterator
    for BoundedHashMapIter<'a, K, V, N, P>
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.map.entries.len() {
            let pair = &self.map.entries[self.index];
            self.index += 1;
            Some((&pair.key, &pair.value))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use wrt_foundation::safe_memory::NoStdProvider;

    use super::*;

    #[test]
    fn test_bounded_hashmap() {
        let provider = NoStdProvider::default();
        let mut map =
            BoundedHashMap::<u32, &str, 10, _>::new(provider).expect("Failed to create map");

        // Test insertion
        map.insert(1, "one").expect("Failed to insert");
        map.insert(2, "two").expect("Failed to insert");
        map.insert(3, "three").expect("Failed to insert");

        // Test get
        assert_eq!(map.get(&1), Some(&"one"));
        assert_eq!(map.get(&2), Some(&"two"));
        assert_eq!(map.get(&3), Some(&"three"));
        assert_eq!(map.get(&4), None);

        // Test update
        let old = map.insert(2, "TWO").expect("Failed to update");
        assert_eq!(old, Some("two"));
        assert_eq!(map.get(&2), Some(&"TWO"));

        // Test remove
        let removed = map.remove(&2).expect("Failed to remove");
        assert_eq!(removed, Some("TWO"));
        assert_eq!(map.get(&2), None);

        // Test len and is_empty
        assert_eq!(map.len(), 2);
        assert!(!map.is_empty());

        // Test clear
        map.clear().expect("Failed to clear");
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn test_bounded_hashmap_iter() {
        let provider = NoStdProvider::default();
        let mut map =
            BoundedHashMap::<u32, &str, 10, _>::new(provider).expect("Failed to create map");

        map.insert(1, "one").expect("Failed to insert");
        map.insert(2, "two").expect("Failed to insert");
        map.insert(3, "three").expect("Failed to insert");

        let mut seen_keys = 0;
        for (key, value) in map.iter() {
            seen_keys += 1;
            match *key {
                1 => assert_eq!(*value, "one"),
                2 => assert_eq!(*value, "two"),
                3 => assert_eq!(*value, "three"),
                _ => panic!("Unexpected key: {}", key),
            }
        }

        assert_eq!(seen_keys, 3);
    }
}
