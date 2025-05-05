//! Test no_std compatibility for wrt-sync
//!
//! This file validates that the wrt-sync crate works correctly in no_std environments.

// For testing in a no_std environment
#![cfg_attr(not(feature = "std"), no_std)]

// External crate imports
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(test)]
mod tests {
    // Import necessary types for no_std environment
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::{format, string::String};

    #[cfg(feature = "std")]
    use std::string::String;

    // Import from wrt-sync
    use wrt_sync::{Mutex, RwLock};

    #[test]
    fn test_mutex_operations() {
        // Create a mutex
        let mutex = Mutex::new(42);

        // Lock the mutex
        {
            let mut lock = mutex.lock().unwrap();
            assert_eq!(*lock, 42);

            // Modify the value
            *lock = 100;
        }

        // Verify the value changed
        let lock = mutex.lock().unwrap();
        assert_eq!(*lock, 100);
    }

    #[test]
    fn test_rwlock_operations() {
        // Create a read-write lock
        let rwlock = RwLock::new(String::from("test"));

        // Acquire read lock
        {
            let read_lock = rwlock.read().unwrap();
            assert_eq!(*read_lock, "test");
        }

        // Acquire write lock
        {
            let mut write_lock = rwlock.write().unwrap();
            write_lock.push_str("_modified");
        }

        // Verify the value changed
        let read_lock = rwlock.read().unwrap();
        assert_eq!(*read_lock, "test_modified");
    }

    #[test]
    fn test_mutex_try_lock() {
        // Create a mutex
        let mutex = Mutex::new(42);

        // Try to lock the mutex
        match mutex.try_lock() {
            Ok(lock) => {
                assert_eq!(*lock, 42);
            }
            Err(_) => {
                panic!("try_lock should succeed when mutex is not locked");
            }
        }
    }

    #[test]
    fn test_rwlock_try_read_write() {
        // Create a read-write lock
        let rwlock = RwLock::new(42);

        // Try to acquire read lock
        match rwlock.try_read() {
            Ok(lock) => {
                assert_eq!(*lock, 42);
            }
            Err(_) => {
                panic!("try_read should succeed when rwlock is not locked");
            }
        }

        // Try to acquire write lock
        match rwlock.try_write() {
            Ok(mut lock) => {
                *lock = 100;
                assert_eq!(*lock, 100);
            }
            Err(_) => {
                panic!("try_write should succeed when rwlock is not locked for writing");
            }
        }
    }
}
