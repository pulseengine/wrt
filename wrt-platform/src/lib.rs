// WRT - wrt-platform
// Module: Library Root
// SW-REQ-ID: REQ_PLATFORM_001
//
// Copyright (c) 2025 The WRT Project Developers
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! WRT Platform Abstraction Layer (PAL).
//!
//! Provides traits and implementations for platform-specific operations like
//! memory allocation (`PageAllocator`) and low-level synchronization
//! (`FutexLike`).
//!
//! This crate adheres to safety-critical guidelines:
//! - Unsafe code is used minimally and with justification (see `memory.rs`).
//! - Error handling via `wrt_error::Error`.
//! - `panic = "abort"`.
//! - `no_std` support (conditionally uses `std` for fallback implementations).

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)] // Rule 9: Require documentation.
#![deny(clippy::panic)] // Rule 3: No panic!.
#![deny(clippy::unwrap_used)] // Rule 3: No unwrap.
#![deny(clippy::expect_used)] // Rule 3: No expect.
#![deny(clippy::unreachable)] // Rule 4: No unreachable!.
#![warn(clippy::pedantic)] // Rule 8: Enable pedantic lints.

// Module declarations
pub mod memory;
pub mod prelude;
pub mod sync;

// Publicly export items via the prelude
// Publicly export the core traits and the fallback implementations
pub use memory::{FallbackAllocator, PageAllocator};
pub use prelude::*;
pub use sync::{FallbackFutex, FutexLike};
// Re-export core error type (also available via prelude)
pub use wrt_error::Error;

#[cfg(test)]
mod tests {
    // Import through the prelude for testing
    use super::prelude::*;

    #[test]
    fn it_works() {
        // Basic sanity check test
        assert_eq!(2 + 2, 4);
    }

    // Add more tests, including cfg-gated tests for std/no_std
    #[cfg(feature = "std")]
    #[test]
    fn std_fallback_allocator_compiles() {
        // Now uses prelude imports
        let mut allocator = FallbackAllocator::new();
        // Basic compile check - functionality tested in memory module
        assert!(allocator.allocate(1, None).is_ok());
        // Need to deallocate to avoid panic in drop test
        let (ptr, size) = allocator.memory.expect("memory should exist");
        unsafe {
            allocator.deallocate(ptr, size).expect("dealloc failed");
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn std_fallback_futex_compiles() {
        // Now uses prelude imports
        let futex = FallbackFutex::new(0);
        // Basic compile check - functionality tested in sync module
        futex.wake(1);
    }
}
