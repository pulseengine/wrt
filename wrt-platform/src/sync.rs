// WRT - wrt-platform
// Module: Platform Synchronization Primitives
// SW-REQ-ID: REQ_PLATFORM_001
//
// Copyright (c) 2025 The WRT Project Developers
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Provides traits and implementations for platform-specific synchronization.

#![cfg_attr(not(feature = "std"), no_std)]

use core::{
    fmt::Debug,
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};
#[cfg(feature = "std")]
use std::sync::{Condvar, Mutex};

use wrt_error::codes; // Import error codes

use crate::prelude::{Error, ErrorCategory, Result};

/// A trait abstracting futex-like operations.
///
/// This trait provides a minimal set of operations similar to those offered by
/// futexes on Linux, allowing for thread synchronization based on an atomic U32
/// value.
pub trait FutexLike: Send + Sync + Debug {
    /// Atomically checks if the futex value is `expected`, and if so, sleeps
    /// until woken by `wake` or until a timeout occurs.
    ///
    /// # Arguments
    ///
    /// * `expected`: The value the futex is expected to hold.
    /// * `timeout`: An optional duration to wait. If `None`, waits
    ///   indefinitely.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the wait was successful (either woken or value changed).
    /// * `Err(Error::Concurrency(...))` if a lock was poisoned.
    /// * `Err(Error::System(...))` if a timeout occurred.
    fn wait(&self, expected: u32, timeout: Option<Duration>) -> Result<()>;

    /// Wakes `count` waiters blocked on this futex.
    ///
    /// # Arguments
    ///
    /// * `count`: The number of waiters to wake. `u32::MAX` can be used to wake
    ///   all.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the wake signal was sent.
    /// * `Err(Error::Concurrency(...))` if a lock was poisoned (relevant for
    ///   fallback).
    fn wake(&self, count: u32) -> Result<()>;
}

/// A fallback `FutexLike` implementation using `std::sync::{Mutex, Condvar}`.
///
/// This is suitable for platforms where `std` is available but specialized
/// futex operations are not, or for testing.
#[cfg(feature = "std")]
#[derive(Debug, Default)]
pub struct FallbackFutex {
    value: AtomicU32,
    mutex: Mutex<()>, // Mutex for Condvar
    condvar: Condvar,
}

#[cfg(feature = "std")]
impl FallbackFutex {
    /// Creates a new `FallbackFutex` initialized with `initial_value`.
    #[must_use]
    pub fn new(initial_value: u32) -> Self {
        Self {
            value: AtomicU32::new(initial_value),
            mutex: Mutex::new(()),
            condvar: Condvar::new(),
        }
    }
}

#[cfg(feature = "std")]
impl FutexLike for FallbackFutex {
    fn wait(&self, expected: u32, timeout: Option<Duration>) -> Result<()> {
        let mut guard = self.mutex.lock().map_err(|_| {
            Error::new(
                ErrorCategory::Concurrency,
                codes::POISONED_LOCK,
                "Mutex poisoned during futex wait",
            )
        })?;

        // Check the condition *after* acquiring the lock.
        if self.value.load(Ordering::Relaxed) != expected {
            // Value changed before or during lock acquisition, don't wait.
            return Ok(());
        }

        match timeout {
            Some(duration) => {
                let (_new_guard, wait_result) =
                    self.condvar.wait_timeout(guard, duration).map_err(|_| {
                        Error::new(
                            ErrorCategory::Concurrency,
                            codes::POISONED_LOCK,
                            "Mutex poisoned during timed futex wait",
                        )
                    })?;
                if wait_result.timed_out() {
                    Err(Error::new(
                        // Use Error::new for timeout as well
                        ErrorCategory::System,    // System for timeout
                        codes::EXECUTION_TIMEOUT, // Specific code for timeout
                        "Futex wait timed out",
                    ))
                } else {
                    Ok(())
                }
            }
            None => {
                self.condvar.wait(guard).map_err(|_| {
                    Error::new(
                        ErrorCategory::Concurrency,
                        codes::POISONED_LOCK,
                        "Mutex poisoned during indefinite futex wait",
                    )
                })?;
                Ok(())
            }
        }
    }

    fn wake(&self, count: u32) -> Result<()> {
        let _guard = self.mutex.lock().map_err(|_| {
            Error::new(
                ErrorCategory::Concurrency,
                codes::POISONED_LOCK,
                "Mutex poisoned during futex wake",
            )
        })?;

        if count == 0 {
            // Do nothing if count is 0
        } else if count == u32::MAX || count > 1 {
            // Simplified: if count is not 1, wake all.
            // More precise would be to loop `count` times for notify_one,
            // but condvar doesn't guarantee waking distinct threads.
            // Waking all is safer for count > 1.
            self.condvar.notify_all();
        } else {
            // count == 1
            self.condvar.notify_one();
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;
    use crate::prelude::sync::Arc; // For Arc in multithreaded tests
    use crate::prelude::thread; // For thread::spawn

    #[test]
    fn futex_new() {
        let futex = FallbackFutex::new(5);
        assert_eq!(futex.value.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn futex_wait_value_mismatch_returns_ok() {
        let futex = FallbackFutex::new(0);
        // Try to wait for value 1 when it's 0
        let result = futex.wait(1, Some(Duration::from_millis(10)));
        assert!(result.is_ok(), "Wait should return Ok if value mismatches, not block");
    }

    #[test]
    fn futex_wait_timeout() {
        let futex = FallbackFutex::new(0);
        // Wait for value 0, should timeout
        let result = futex.wait(0, Some(Duration::from_millis(10)));
        match result {
            Err(e) if e.category == ErrorCategory::System && e.code == codes::EXECUTION_TIMEOUT => {
                // Expected timeout
            }
            _ => panic!("Expected timeout error, got {:?}", result),
        }
    }

    #[test]
    fn futex_wake_one() {
        let futex = Arc::new(FallbackFutex::new(0));
        let futex_clone = Arc::clone(&futex);

        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10)); // Give time for main thread to wait
            futex_clone.value.store(1, Ordering::Relaxed);
            let wake_res = futex_clone.wake(1);
            assert!(wake_res.is_ok());
        });

        // Wait for value 0, should be woken when value becomes 1
        let wait_res = futex.wait(0, Some(Duration::from_secs(1)));
        assert!(wait_res.is_ok(), "Wait failed: {:?}", wait_res.err());

        handle.join().unwrap();
        assert_eq!(futex.value.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn futex_wake_all() {
        let futex = Arc::new(FallbackFutex::new(0));
        let mut handles = vec![];

        for _ in 0..3 {
            let futex_clone = Arc::clone(&futex);
            handles.push(thread::spawn(move || {
                let wait_res = futex_clone.wait(0, Some(Duration::from_secs(1)));
                assert!(wait_res.is_ok(), "Thread wait failed: {:?}", wait_res.err());
            }));
        }

        thread::sleep(Duration::from_millis(50)); // Give threads time to wait
        futex.value.store(1, Ordering::Relaxed); // Change value (optional, good practice)
        let wake_res = futex.wake(u32::MAX); // Wake all
        assert!(wake_res.is_ok());

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn futex_wake_zero_does_nothing() {
        let futex = Arc::new(FallbackFutex::new(0));
        let futex_clone = Arc::clone(&futex);

        let handle = thread::spawn(move || {
            // This thread will wait, but should timeout as wake(0) does nothing
            let wait_res = futex_clone.wait(0, Some(Duration::from_millis(50)));
            match wait_res {
                Err(e)
                    if e.category == ErrorCategory::System
                        && e.code == codes::EXECUTION_TIMEOUT =>
                {
                    // Expected timeout
                }
                _ => {
                    panic!("Expected timeout error as wake(0) should not wake, got {:?}", wait_res)
                }
            }
        });

        thread::sleep(Duration::from_millis(10));
        let wake_res = futex.wake(0);
        assert!(wake_res.is_ok());

        handle.join().unwrap();
    }
}

#[cfg(feature = "std")]
pub use fallback_std::FallbackFutex;
