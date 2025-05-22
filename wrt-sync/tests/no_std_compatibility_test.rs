//! Test no_std compatibility for wrt-sync
//!
//! This file validates that the wrt-sync crate works correctly in no_std
//! environments.

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
    use wrt_sync::{WrtMutex as Mutex, WrtRwLock as RwLock};

    #[test]
    fn test_mutex_operations() {
        // Create a mutex
        let mutex = Mutex::new(42);

        // Lock the mutex
        {
            let mut lock = mutex.lock();
            assert_eq!(*lock, 42);

            // Modify the value
            *lock = 100;
        }

        // Verify the value changed
        let lock = mutex.lock();
        assert_eq!(*lock, 100);
    }

    #[test]
    fn test_rwlock_operations() {
        // Create a read-write lock
        let rwlock = RwLock::new(String::from("test"));

        // Acquire read lock
        {
            let read_lock = rwlock.read();
            assert_eq!(*read_lock, "test");
        }

        // Acquire write lock
        {
            let mut write_lock = rwlock.write();
            write_lock.push_str("_modified");
        }

        // Verify the value changed
        let read_lock = rwlock.read();
        assert_eq!(*read_lock, "test_modified");
    }

    // Note: The WrtMutex implementation doesn't currently have try_lock,
    // so we're using lock() instead
    #[test]
    fn test_mutex_locking() {
        // Create a mutex
        let mutex = Mutex::new(42);

        // Lock the mutex
        let lock = mutex.lock();
        assert_eq!(*lock, 42);
    }

    // Note: The WrtRwLock implementation doesn't currently have try_read/try_write,
    // so we're using read()/write() instead
    #[test]
    fn test_rwlock_read_write() {
        // Create a read-write lock
        let rwlock = RwLock::new(42);

        // Acquire read lock
        {
            let lock = rwlock.read();
            assert_eq!(*lock, 42);
        }

        // Acquire write lock
        {
            let mut lock = rwlock.write();
            *lock = 100;
            assert_eq!(*lock, 100);
        }

        // Verify the value changed
        let lock = rwlock.read();
        assert_eq!(*lock, 100);
    }
}
