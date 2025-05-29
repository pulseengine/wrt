#![allow(unsafe_code)]
// Allow unsafe syscalls for sync operations
// WRT - wrt-platform
// Module: macOS Synchronization Primitives (No libc)
// SW-REQ-ID: REQ_PLATFORM_001
//
// Copyright (c) 2025 The WRT Project Developers
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! macOS-specific `FutexLike` implementation using direct syscalls without
//! libc.

use core::{fmt, marker::PhantomData, ptr::NonNull, sync::atomic::AtomicU32};

use wrt_error::{codes, Error, ErrorCategory, Result};

use crate::sync::{FutexLike, TimeoutResult};

// Darwin / macOS specific ulock operations syscall numbers
const SYSCALL_ULOCK_WAIT: u64 = 515; // __MAC_OS_X_VERSION_MIN_REQUIRED >= 101200
const SYSCALL_ULOCK_WAKE: u64 = 516; // __MAC_OS_X_VERSION_MIN_REQUIRED >= 101200

// Constants for ulf_wait operations
const ULF_WAIT_ABSTIME: u32 = 0x0000_0008; // ULF_WAIT_FLAG_ABSTIME
const ULF_WAIT_TIMEOUT: u32 = 0x0000_0004; // ULF_WAIT_FLAG_TIMEOUT
const ULF_WAIT: u32 = 0x0000_0001; // ULF_WAIT
const ULF_WAKE: u32 = 0x00000002; // ULF_WAKE
const ULF_WAKE_SHARED: u32 = 0x0000_0100; // ULF_WAKE_FLAG_SHARED

// Additional constant for wake operations
const ULF_WAKE_ALL: u64 = 0x0000_0100; // Wake all waiting threads

// For _ulock_wait timeout parameter
const ULOCK_WAIT_TIMEOUT_INFINITE: u32 = u32::MAX;

/// A `FutexLike` implementation for macOS using direct syscalls.
///
/// This implementation provides wait/notify synchronization using macOS
/// `_ulock_wait` and `_ulock_wake` syscalls.
#[derive(Debug)]
pub struct MacOsFutex {
    /// The atomic value used for synchronization
    value: AtomicU32,
    /// Padding to ensure the value is on its own cache line
    _padding: [u8; 60], // 64 - sizeof(AtomicU32)
    /// Safety: We're carefully managing the lifetime of the value
    _marker: PhantomData<*mut ()>,
}

impl MacOsFutex {
    /// Creates a new `MacOsFutex` with the given initial value.
    pub fn new(initial_value: u32) -> Self {
        Self { value: AtomicU32::new(initial_value), _padding: [0; 60], _marker: PhantomData }
    }

    /// Direct syscall implementation of _ulock_wait
    unsafe fn ulock_wait(operation: u32, addr: *const u32, value: u64, timeout: u32) -> i32 {
        let mut result: i32;

        // x86_64 syscall ABI: rax=syscall number, args in rdi, rsi, rdx, r10
        core::arch::asm!(
            "syscall",
            inout("rax") SYSCALL_ULOCK_WAIT => _,
            in("rdi") operation,
            in("rsi") addr,
            in("rdx") value,
            in("r10") timeout,
            lateout("rax") result,
            out("rcx") _,
            out("r11") _,
        );

        result
    }

    /// Direct syscall implementation of _ulock_wake
    unsafe fn ulock_wake(operation: u32, addr: *const u32, wake_flags: u64) -> i32 {
        let mut result: i32;

        // x86_64 syscall ABI: rax=syscall number, args in rdi, rsi, rdx
        core::arch::asm!(
            "syscall",
            inout("rax") SYSCALL_ULOCK_WAKE => _,
            in("rdi") operation,
            in("rsi") addr,
            in("rdx") wake_flags,
            lateout("rax") result,
            out("rcx") _,
            out("r11") _,
        );

        result
    }
}

/// Builder for `MacOsFutex`.
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

impl FutexLike for MacOsFutex {
    fn wait(&self, expected: u32, timeout_us: Option<u32>) -> Result<TimeoutResult> {
        let addr = &self.value as *const AtomicU32 as *const u32;
        let operation = ULF_WAIT;

        // Convert timeout to the format required by _ulock_wait
        let timeout_param = match timeout_us {
            Some(0) => 0, // Immediate timeout (try once)
            Some(timeout) => {
                // Add the ULF_WAIT_TIMEOUT flag to indicate a timeout is provided
                ULF_WAIT_TIMEOUT | timeout
            }
            None => ULOCK_WAIT_TIMEOUT_INFINITE, // Infinite timeout
        };

        // Call the syscall
        // SAFETY: We're calling _ulock_wait, which requires a pointer to the atomic
        // value. addr points to self.value, which is valid for the lifetime of
        // self.
        let result = unsafe { Self::ulock_wait(operation, addr, expected as u64, timeout_param) };

        match result {
            0 => Ok(TimeoutResult::Notified), // Woken up by a notify call
            -1 => {
                // Check errno - in a real implementation we'd use the real errno value
                // For now, assume ETIMEDOUT (60) if a timeout was specified
                if timeout_us.is_some() && timeout_us != Some(ULOCK_WAIT_TIMEOUT_INFINITE) {
                    Ok(TimeoutResult::TimedOut)
                } else {
                    // Other error
                    Err(Error::new(
                        ErrorCategory::System, 1,
                        
                        "Failed to wait on futex",
                    ))
                }
            }
            _ => Err(Error::new(
                ErrorCategory::System, 1,
                
                "Unexpected error waiting on futex",
            )),
        }
    }

    fn notify_one(&self) -> Result<()> {
        let addr = &self.value as *const AtomicU32 as *const u32;
        let operation = ULF_WAKE;

        // Call the syscall
        // SAFETY: We're calling _ulock_wake, which requires a pointer to the atomic
        // value. addr points to self.value, which is valid for the lifetime of
        // self.
        let result = unsafe {
            Self::ulock_wake(operation, addr, 0) // Wake one waiter
        };

        if result >= 0 {
            // Success, return number of waiters woken (>= 0)
            Ok(())
        } else {
            // Error
            Err(Error::new(ErrorCategory::System, 1, "Failed to notify one"))
        }
    }

    fn notify_all(&self) -> Result<()> {
        let addr = &self.value as *const AtomicU32 as *const u32;
        let operation = ULF_WAKE;

        // Call the syscall
        // SAFETY: We're calling _ulock_wake, which requires a pointer to the atomic
        // value. addr points to self.value, which is valid for the lifetime of
        // self.
        let result = unsafe {
            Self::ulock_wake(operation, addr, ULF_WAKE_ALL) // Wake all waiters
        };

        if result >= 0 {
            // Success, return number of waiters woken (>= 0)
            Ok(())
        } else {
            // Error
            Err(Error::new(ErrorCategory::System, 1, "Failed to notify all"))
        }
    }
}
