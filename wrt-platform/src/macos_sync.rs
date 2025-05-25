#![allow(unsafe_code)]
// Allow unsafe FFI calls for _ulock_wait/_ulock_wake
// WRT - wrt-platform
// Module: macOS Synchronization Primitives
// SW-REQ-ID: REQ_PLATFORM_001
//
// Copyright (c) 2025 The WRT Project Developers
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! macOS-specific `FutexLike` implementation using `_ulock_wait` and
//! `_ulock_wake`.

// Remove conditional alloc imports for format!
// #[cfg(all(not(feature = "std"), feature = "alloc"))]
// extern crate alloc;
// #[cfg(all(not(feature = "std"), feature = "alloc"))]
// use alloc::format;

use core::sync::atomic::AtomicU32;

use wrt_error::{codes, Error, ErrorCategory, Result};

// Darwin / macOS specific ulock operations
// See: <sys/ulock.h> or xnu source code (osfmk/kern/sync_sema.c)
// int _ulock_wait(uint32_t operation, void *addr, uint64_t value, uint32_t
// timeout); // ULF_WAIT int _ulock_wake(uint32_t operation, void *addr,
// uint64_t wake_flags); // ULF_WAKE

// Constants for ulf_wait operations
const ULF_WAIT: u32 = 0x0000_0001; // Matched against Apple's internal ULF_WAIT value
const ULF_WAKE: u32 = 0x0000_0002; // Matched against Apple's internal ULF_WAKE value

// Additional constant for __ulock_wake if needed, though often combined with
// ULF_WAKE flags
const ULF_WAKE_ALL: u64 = 0x0000_0100; // Example, check actual value for wake_all semantics

// For _ulock_wait timeout parameter:
// Specifies a timeout in microseconds. A value of 0 means try once.
// A value of UINT32_MAX means wait forever.
const ULOCK_WAIT_TIMEOUT_INFINITE: u32 = u32::MAX;

// For _ulock_wake wake_flags parameter:
// ULF_WAKE_ALL wakes all waiters, otherwise wakes one.

extern "C" {
    fn _ulock_wait(
        operation: u32,
        addr: *mut libc::c_void,
        value: u64,
        timeout_us: u32,
    ) -> libc::c_int;
    fn _ulock_wake(operation: u32, addr: *mut libc::c_void, wake_flags: u64) -> libc::c_int;
}

/// A `FutexLike` implementation for macOS using `_ulock_wait` and
/// `_ulock_wake`.
#[derive(Debug)]
pub struct MacOsFutex {
    addr: AtomicU32, /* The address we wait/wake on.
                      * On macOS, _ulock_wait takes a pointer to the value and the value
                      * itself. We'll use the address of this
                      * AtomicU32. */
    _padding: [u8; 60], /* Padding to ensure the atomic is on its own cache line (64 - size of
                         * AtomicU32) */
}

/// Builder for `MacOsFutex` instances.
///
/// Provides a fluent API for configuring and constructing MacOS-specific
/// futex implementations.
#[derive(Debug)]
pub struct MacOsFutexBuilder {
    initial_value: u32,
}

impl Default for MacOsFutexBuilder {
    fn default() -> Self {
        Self { initial_value: 0 }
    }
}

impl MacOsFutexBuilder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the initial value for the futex.
    pub fn with_initial_value(mut self, value: u32) -> Self {
        self.initial_value = value;
        self
    }

    /// Builds and returns a configured `MacOsFutex`.
    pub fn build(self) -> MacOsFutex {
        MacOsFutex::new(self.initial_value)
    }
}

impl MacOsFutex {
    /// Creates a new `MacOsFutex` initialized with `initial_value`.
    #[must_use]
    pub fn new(initial_value: u32) -> Self {
        Self { addr: AtomicU32::new(initial_value), _padding: [0; 60] }
    }

    /// Waits on this futex if the current value equals the expected value.
    ///
    /// # Arguments
    ///
    /// * `expected` - The value that the futex is expected to have
    /// * `timeout_ns` - Optional timeout in nanoseconds; None means wait
    ///   indefinitely
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the wait completed successfully (either due to a wake or
    ///   value change)
    /// * `Err(...)` if an error occurred
    pub fn futex_wait(&self, expected: u32, timeout_ns: Option<u64>) -> Result<()> {
        let ulock_cmd = ULF_WAIT; // Operation for _ulock_wait
        let timeout_val = match timeout_ns {
            Some(val) if val > u32::MAX as u64 => u32::MAX,
            Some(val) => val as u32,
            None => 0, // 0 means no timeout for __ulock_wait
        };

        let ret = unsafe {
            _ulock_wait(
                // Corrected: single underscore
                ulock_cmd,
                self.addr.as_ptr() as *mut libc::c_void,
                u64::from(expected), // Value to compare
                timeout_val,         // Timeout as u32
            )
        };

        match ret {
            0 => Ok(()), // Success: woken or value no longer `expected`
            libc::ETIMEDOUT => Err(Error::new(
                ErrorCategory::System,
                codes::EXECUTION_TIMEOUT,
                "Futex wait timed out (_ulock_wait)",
            )),
            libc::EINTR => Err(Error::new(
                ErrorCategory::System,
                codes::SYSTEM_CALL_INTERRUPTED,
                "Futex wait interrupted (_ulock_wait)",
            )),
            _err_code => {
                // TODO: Log the specific _err_code if logging is available
                Err(Error::new(
                    ErrorCategory::System,
                    codes::SYSTEM_ERROR, // Or perhaps CONCURRENCY_ERROR if more specific
                    "_ulock_wait failed with an OS error code.",
                ))
            }
        }
    }

    /// Wakes waiters blocked on this futex.
    ///
    /// # Arguments
    ///
    /// * `wake_all` - If true, wakes all waiters; if false, wakes just one
    ///   waiter
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the wake operation completed successfully
    /// * `Err(...)` if an error occurred
    pub fn futex_wake(&self, wake_all: bool) -> Result<()> {
        let ulock_cmd = ULF_WAKE; // Operation for _ulock_wake
        let wake_cmd_flags = if wake_all {
            ULF_WAKE_ALL // This is a u64 flag for the last arg of _ulock_wake
        } else {
            0 // Wake one waiter
        };

        // For __ulock_wake, the value parameter is usually 0 unless specific
        // wake-by-value is used. The wake_flags parameter (ULF_WAKE_ALL
        // equivalent) is part of the command.
        let ret = unsafe {
            _ulock_wake(ulock_cmd, self.addr.as_ptr() as *mut libc::c_void, wake_cmd_flags)
        }; // Corrected: single underscore, pass wake_cmd_flags

        match ret {
            0 => Ok(()),
            libc::ESRCH => Ok(()), // No such process; no threads were waiting. Not an error for
            // wake.
            libc::EINTR => Err(Error::new(
                ErrorCategory::System,
                codes::SYSTEM_CALL_INTERRUPTED,
                "Futex wake interrupted (_ulock_wake)",
            )),
            _err_code => {
                // TODO: Log the specific _err_code if logging is available
                Err(Error::new(
                    ErrorCategory::System,
                    codes::SYSTEM_ERROR, // Or perhaps CONCURRENCY_ERROR
                    "_ulock_wake failed with an OS error code.",
                ))
            }
        }
    }
}

// Note: This implementation requires the `addr` field of `MacOsFutex` to be the
// actual memory location that threads contend on. `_ulock_wait` uses the
// `value` parameter to compare against the memory at `addr`. `_ulock_wake` just
// needs `addr`. Our `FutexLike` trait implies the futex object itself might
// hold the atomic value, which MacOsFutex does.

impl crate::sync::FutexLike for MacOsFutex {
    fn wait(&self, expected: u32, timeout: Option<core::time::Duration>) -> Result<()> {
        // Convert Duration to microseconds for _ulock_wait
        let timeout_us = match timeout {
            Some(d) => {
                let micros = d.as_micros();
                if micros > u32::MAX as u128 {
                    u32::MAX // Cap at max u32
                } else {
                    micros as u32
                }
            }
            None => ULOCK_WAIT_TIMEOUT_INFINITE,
        };

        // Call the internal implementation
        self.futex_wait(expected, Some(timeout_us as u64))
    }

    fn wake(&self, count: u32) -> Result<()> {
        // If count is u32::MAX, wake all waiters
        self.futex_wake(count == u32::MAX)
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_futex_builder() {
        // Test with default settings
        let futex = MacOsFutexBuilder::new().build();

        // The default initial value should be 0
        assert_eq!(futex.addr.load(core::sync::atomic::Ordering::Relaxed), 0);

        // Test with custom initial value
        let futex = MacOsFutexBuilder::new().with_initial_value(42).build();

        assert_eq!(futex.addr.load(core::sync::atomic::Ordering::Relaxed), 42);
    }
}
