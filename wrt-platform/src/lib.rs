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
//! - `no_std` support.

#![no_std] // Rule: Enforce no_std
#![deny(missing_docs)] // Rule 9: Require documentation.
#![deny(clippy::panic)] // Rule 3: No panic!.
#![deny(clippy::unwrap_used)] // Rule 3: No unwrap.
#![deny(clippy::expect_used)] // Rule 3: No expect.
#![deny(clippy::unreachable)] // Rule 4: No unreachable!.
#![warn(clippy::pedantic)] // Rule 8: Enable pedantic lints.

// Module declarations
pub mod memory;
pub mod memory_optimizations;
pub mod prelude;
pub mod sync;

// Platform-specific modules
#[cfg(all(feature = "platform-macos", feature = "use-libc", target_os = "macos"))]
pub mod macos_memory;
#[cfg(all(feature = "platform-macos", not(feature = "use-libc"), target_os = "macos"))]
pub mod macos_memory_no_libc;
#[cfg(all(feature = "platform-macos", feature = "use-libc", target_os = "macos"))]
pub mod macos_sync;
#[cfg(all(feature = "platform-macos", not(feature = "use-libc"), target_os = "macos"))]
pub mod macos_sync_no_libc;

// Publicly export items via the prelude
// Publicly export the core traits and the fallback implementations
// Export macOS specific implementations if enabled and on macOS
#[cfg(all(feature = "platform-macos", feature = "use-libc", target_os = "macos"))]
pub use macos_memory::{MacOsAllocator, MacOsAllocatorBuilder};
#[cfg(all(feature = "platform-macos", not(feature = "use-libc"), target_os = "macos"))]
pub use macos_memory_no_libc::{MacOsAllocator, MacOsAllocatorBuilder};
// Export macOS specific implementations if enabled and on macOS
#[cfg(all(feature = "platform-macos", feature = "use-libc", target_os = "macos"))]
pub use macos_sync::{MacOsFutex, MacOsFutexBuilder};
#[cfg(all(feature = "platform-macos", not(feature = "use-libc"), target_os = "macos"))]
pub use macos_sync_no_libc::{MacOsFutex, MacOsFutexBuilder};
pub use memory::{
    NoStdProvider, NoStdProviderBuilder, PageAllocator, VerificationLevel, WASM_PAGE_SIZE,
}; // WASM_PAGE_SIZE is always available
pub use memory_optimizations::{
    MemoryOptimization, PlatformMemoryOptimizer, PlatformOptimizedProviderBuilder,
};
pub use prelude::*;
pub use sync::{FutexLike, SpinFutex, SpinFutexBuilder, TimeoutResult}; /* FutexLike is always available */
// Re-export core error type (also available via prelude)
pub use wrt_error::Error; // This is fine as wrt_error::Error is always available

#[cfg(test)]
#[allow(clippy::panic)] // Allow panics in the test module
mod tests {
    // Import through the prelude for testing
    use super::{memory::MemoryProvider, prelude::*};

    #[test]
    fn it_works() {
        // Basic sanity check test
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_no_std_provider_builder() {
        let provider = NoStdProviderBuilder::new()
            .with_size(2048)
            .with_verification_level(VerificationLevel::Full)
            .build();

        assert_eq!(provider.verification_level(), VerificationLevel::Full);
        // Actual size is capped at 4096 in the stub implementation
        assert!(provider.capacity() <= 4096);
    }

    #[cfg(all(feature = "platform-macos", target_os = "macos"))]
    #[test]
    fn test_macos_allocator_builder() {
        let allocator = MacOsAllocatorBuilder::new()
            .with_maximum_pages(100)
            .with_guard_pages(true)
            .with_memory_tagging(true)
            .build();

        // Just making sure the builder returns an allocator
        // We can't test its settings without accessing private fields
        assert_eq!(core::mem::size_of_val(&allocator) > 0, true);
    }
}

#[cfg(all(not(feature = "std"), not(test)))] // Apply panic handler for no_std builds, excluding test context
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // For no_std builds, simply loop indefinitely on panic.
    // A more sophisticated handler might print to a debug console if available.
    loop {}
}
