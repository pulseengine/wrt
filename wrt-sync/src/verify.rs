//! Formal verification for the synchronization primitives using Kani.
//!
//! This module contains proofs that verify core properties of the mutex and
//! rwlock implementations. These proofs only run with Kani and are isolated
//! from normal compilation and testing.

// Only compile Kani verification code when documentation is being generated
// or when explicitly running cargo kani. This prevents interference with
// coverage testing.
#[cfg(any(doc, kani))]
pub mod kani_verification {
    use crate::{prelude::*, *};

    // --- WrtMutex Verification ---

    /// Verify that mutex creation, locking and unlocking works correctly
    #[cfg_attr(kani, kani::proof)]
    #[cfg_attr(kani, kani::unwind(3))]
    pub fn verify_mutex_new_lock_unlock() {
        let m = WrtMutex::new(10);
        {
            let mut guard = m.lock();
            *guard += 5;
            assert_eq!(*guard, 15);
            // Test Debug impl while locked
            #[cfg(feature = "std")]
            let _ = format!("{:?}", m);
            #[cfg(not(feature = "std"))]
            let _ = core::fmt::Debug::fmt(&m, &mut core::fmt::Formatter::new());
        } // guard drops here

        {
            let guard = m.lock();
            assert_eq!(*guard, 15);
        } // guard drops here
          // Test Debug impl while unlocked
        #[cfg(feature = "std")]
        let _ = format!("{:?}", m);
        #[cfg(not(feature = "std"))]
        let _ = core::fmt::Debug::fmt(&m, &mut core::fmt::Formatter::new());
    }

    /// Verify that mutex guard deref works correctly
    #[cfg_attr(kani, kani::proof)]
    #[cfg_attr(kani, kani::unwind(3))]
    pub fn verify_mutex_guard_deref() {
        let m = WrtMutex::new(42);
        let guard = m.lock();
        assert_eq!(*guard, 42);
    }

    /// Verify that mutex guard deref_mut works correctly
    #[cfg_attr(kani, kani::proof)]
    #[cfg_attr(kani, kani::unwind(3))]
    pub fn verify_mutex_guard_deref_mut() {
        let m = WrtMutex::new(42);
        {
            let mut guard = m.lock();
            assert_eq!(*guard, 42);
            *guard = 99;
            assert_eq!(*guard, 99);
        }
        {
            let guard = m.lock();
            assert_eq!(*guard, 99);
        }
    }

    // --- WrtRwLock Verification ---

    /// Verify that rwlock creation, writing and reading works correctly
    #[cfg_attr(kani, kani::proof)]
    #[cfg_attr(kani, kani::unwind(3))]
    pub fn verify_rwlock_new_write_read() {
        let lock = WrtRwLock::new(0);
        {
            let mut writer = lock.write();
            *writer = 100;
            assert_eq!(*writer, 100);
            // Test Debug impl while write-locked
            #[cfg(feature = "std")]
            let _ = format!("{:?}", lock);
            #[cfg(not(feature = "std"))]
            let _ = core::fmt::Debug::fmt(&lock, &mut core::fmt::Formatter::new());
        } // writer drops
        {
            let data = lock.read();
            assert_eq!(*data, 100);
            // Test Debug impl while read-locked
            #[cfg(feature = "std")]
            let _ = format!("{:?}", lock);
            #[cfg(not(feature = "std"))]
            let _ = core::fmt::Debug::fmt(&lock, &mut core::fmt::Formatter::new());
        } // reader drops
          // Test Debug impl while unlocked
        #[cfg(feature = "std")]
        let _ = format!("{:?}", lock);
        #[cfg(not(feature = "std"))]
        let _ = core::fmt::Debug::fmt(&lock, &mut core::fmt::Formatter::new());
    }

    /// Verify that multiple read locks work correctly
    #[cfg_attr(kani, kani::proof)]
    #[cfg_attr(kani, kani::unwind(6))]
    pub fn verify_rwlock_new_read_read() {
        let lock = WrtRwLock::new(55);
        let r1 = lock.read();
        let r2 = lock.read();
        assert_eq!(*r1, 55);
        assert_eq!(*r2, 55);
        drop(r1);
        drop(r2);
        // Should be able to acquire write lock now
        {
            let mut writer = lock.write();
            *writer = 66;
        }
        // Verify write
        {
            let reader = lock.read();
            assert_eq!(*reader, 66);
        }
    }

    /// Verify that rwlock read guard deref works correctly
    #[cfg_attr(kani, kani::proof)]
    #[cfg_attr(kani, kani::unwind(3))]
    pub fn verify_rwlock_read_guard_deref() {
        let lock = WrtRwLock::new(123);
        let guard = lock.read();
        assert_eq!(*guard, 123);
    }

    /// Verify that rwlock write guard deref_mut works correctly
    #[cfg_attr(kani, kani::proof)]
    #[cfg_attr(kani, kani::unwind(3))]
    pub fn verify_rwlock_write_guard_derefmut() {
        let lock = WrtRwLock::new(123);
        {
            let mut guard = lock.write();
            assert_eq!(*guard, 123);
            *guard = 456;
            assert_eq!(*guard, 456);
        }
        {
            let guard = lock.read();
            assert_eq!(*guard, 456);
        }
    }
}

// Expose the verification module in docs but not for normal compilation
#[cfg(any(doc, kani))]
pub use kani_verification::*;
