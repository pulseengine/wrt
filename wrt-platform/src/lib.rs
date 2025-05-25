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

#![cfg_attr(not(feature = "std"), no_std)] // Rule: Enforce no_std when std feature is not enabled
#![deny(missing_docs)] // Rule 9: Require documentation.
#![deny(clippy::panic)] // Rule 3: No panic!.
#![deny(clippy::unwrap_used)] // Rule 3: No unwrap.
#![deny(clippy::expect_used)] // Rule 3: No expect.
#![deny(clippy::unreachable)] // Rule 4: No unreachable!.
#![warn(clippy::pedantic)] // Rule 8: Enable pedantic lints.
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::extra_unused_type_parameters)]
#![allow(clippy::inline_always)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::unused_self)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::new_without_default)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ref_as_ptr)]
#![allow(clippy::ptr_eq)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::single_match)]
#![allow(clippy::single_match_else)]
#![allow(clippy::elidable_lifetime_names)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::ptr_cast_constness)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

// For no_std + alloc builds, we need a global allocator
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::alloc::{GlobalAlloc, Layout};

#[cfg(all(feature = "alloc", not(feature = "std")))]
struct DummyAllocator;

#[cfg(all(feature = "alloc", not(feature = "std")))]
unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Do nothing - this is just to satisfy the linker
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
#[global_allocator]
static GLOBAL: DummyAllocator = DummyAllocator;

// Module declarations
pub mod memory;
pub mod memory_optimizations;
pub mod performance_validation;
pub mod platform_abstraction;
pub mod prelude;
pub mod runtime_detection;
pub mod sync;

// Enhanced platform features
pub mod advanced_sync;
pub mod formal_verification;
pub mod hardware_optimizations;
pub mod side_channel_resistance;

// Platform-specific modules
#[cfg(all(feature = "platform-macos", feature = "use-libc", target_os = "macos"))]
pub mod macos_memory;
#[cfg(all(feature = "platform-macos", not(feature = "use-libc"), target_os = "macos"))]
pub mod macos_memory_no_libc;
#[cfg(all(feature = "platform-macos", feature = "use-libc", target_os = "macos"))]
pub mod macos_sync;
#[cfg(all(feature = "platform-macos", not(feature = "use-libc"), target_os = "macos"))]
pub mod macos_sync_no_libc;

// QNX-specific modules
#[cfg(all(feature = "platform-qnx", target_os = "nto"))]
pub mod qnx_arena;
#[cfg(all(feature = "platform-qnx", target_os = "nto"))]
pub mod qnx_memory;
#[cfg(all(feature = "platform-qnx", target_os = "nto"))]
pub mod qnx_partition;
#[cfg(all(feature = "platform-qnx", target_os = "nto"))]
pub mod qnx_sync;

// Linux-specific modules
#[cfg(all(feature = "platform-linux", target_os = "linux"))]
pub mod linux_memory;
#[cfg(all(
    feature = "platform-linux",
    feature = "linux-mte",
    target_arch = "aarch64",
    target_os = "linux"
))]
pub mod linux_memory_arm64_mte;
#[cfg(all(feature = "platform-linux", target_os = "linux"))]
pub mod linux_sync;

// Zephyr-specific modules
#[cfg(feature = "platform-zephyr")]
pub mod zephyr_memory;
#[cfg(feature = "platform-zephyr")]
pub mod zephyr_sync;

// Tock OS-specific modules
#[cfg(feature = "platform-tock")]
pub mod tock_memory;
#[cfg(feature = "platform-tock")]
pub mod tock_sync;

// Publicly export items via the prelude
// Publicly export the core traits and the fallback implementations
// Export macOS specific implementations if enabled and on macOS
// Export Linux specific implementations if enabled and on Linux
pub use advanced_sync::{
    AdvancedRwLock, LockFreeAllocator, Priority, PriorityInheritanceMutex, MAX_PRIORITY,
    MIN_PRIORITY,
};
#[cfg(feature = "alloc")]
pub use advanced_sync::{LockFreeMpscQueue, WaitFreeSpscQueue};
pub use formal_verification::{
    annotations, concurrency_verification, integration_verification, memory_verification,
    realtime_verification, security_verification,
};
// Export specific CFI/BTI types for easy access
pub use hardware_optimizations::arm::{BranchTargetIdentification, BtiExceptionLevel, BtiMode};
pub use hardware_optimizations::riscv::{CfiExceptionMode, ControlFlowIntegrity};
// Export enhanced platform features
pub use hardware_optimizations::{
    arm, compile_time, intel, riscv, HardwareOptimization, HardwareOptimizer, SecurityLevel,
};
#[cfg(all(
    feature = "platform-linux",
    target_os = "linux",
    not(all(feature = "linux-mte", target_arch = "aarch64"))
))]
pub use linux_memory::{LinuxAllocator, LinuxAllocatorBuilder};
#[cfg(all(
    feature = "platform-linux",
    feature = "linux-mte",
    target_arch = "aarch64",
    target_os = "linux"
))]
pub use linux_memory_arm64_mte::{LinuxArm64MteAllocator, LinuxArm64MteAllocatorBuilder, MteMode};
#[cfg(all(feature = "platform-linux", target_os = "linux"))]
pub use linux_sync::{LinuxFutex, LinuxFutexBuilder};
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
// Export performance validation (for testing and benchmarking)
pub use performance_validation::{BenchmarkResult, CompileTimeValidator, PerformanceValidator}; /* This is fine as wrt_error::Error is always available */
// Export hybrid platform abstraction
pub use platform_abstraction::{
    paradigm, platform_select, BaremetalPlatform, IsolationLevel, PlatformAbstraction,
    PlatformConfig, PosixPlatform, RealtimePlatform, SecurityPlatform, UnifiedPlatform,
};
pub use prelude::*;
#[cfg(all(feature = "platform-qnx", target_os = "nto"))]
pub use qnx_arena::{QnxArenaAllocator, QnxArenaAllocatorBuilder, QnxMallocOption};
// Export QNX specific implementations if enabled and on QNX
#[cfg(all(feature = "platform-qnx", target_os = "nto"))]
pub use qnx_memory::{QnxAllocator, QnxAllocatorBuilder, QnxMapFlags, QnxProtFlags};
#[cfg(all(feature = "platform-qnx", target_os = "nto"))]
pub use qnx_partition::{
    PartitionGuard, QnxMemoryPartition, QnxMemoryPartitionBuilder, QnxPartitionFlags,
};
#[cfg(all(feature = "platform-qnx", target_os = "nto"))]
pub use qnx_sync::{QnxFutex, QnxFutexBuilder, QnxSyncPriority};
// Export runtime detection
pub use runtime_detection::{
    MemoryCapabilities, PlatformCapabilities, PlatformDetector, RealtimeCapabilities,
    SecurityCapabilities, SyncCapabilities,
};
pub use side_channel_resistance::{
    access_obfuscation, cache_aware_allocation, constant_time, platform_integration, AttackVector,
    ResistanceLevel,
};
pub use sync::{FutexLike, SpinFutex, SpinFutexBuilder, TimeoutResult}; /* FutexLike is always available */
// Export Tock OS specific implementations if enabled
#[cfg(feature = "platform-tock")]
pub use tock_memory::{TockAllocator, TockAllocatorBuilder};
#[cfg(feature = "platform-tock")]
pub use tock_sync::{TockFutex, TockFutexBuilder, TockSemaphoreFutex};
// Re-export core error type (also available via prelude)
pub use wrt_error::Error;
// Export Zephyr specific implementations if enabled
#[cfg(feature = "platform-zephyr")]
pub use zephyr_memory::{ZephyrAllocator, ZephyrAllocatorBuilder, ZephyrMemoryFlags};
#[cfg(feature = "platform-zephyr")]
pub use zephyr_sync::{ZephyrFutex, ZephyrFutexBuilder, ZephyrSemaphoreFutex};

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

    #[cfg(all(feature = "platform-linux", target_os = "linux"))]
    #[test]
    fn test_linux_allocator_builder() {
        let allocator =
            LinuxAllocatorBuilder::new().with_maximum_pages(100).with_guard_pages(true).build();

        // Just making sure the builder returns an allocator
        // We can't test its settings without accessing private fields
        assert_eq!(core::mem::size_of_val(&allocator) > 0, true);
    }

    #[cfg(all(
        feature = "platform-linux",
        feature = "linux-mte",
        target_arch = "aarch64",
        target_os = "linux"
    ))]
    #[test]
    fn test_linux_arm64_mte_allocator_builder() {
        let allocator = LinuxArm64MteAllocatorBuilder::new()
            .with_maximum_pages(100)
            .with_guard_pages(true)
            .with_mte_mode(MteMode::Synchronous)
            .build();

        // Just making sure the builder returns an allocator
        // We can't test its settings without accessing private fields
        assert_eq!(core::mem::size_of_val(&allocator) > 0, true);
    }

    #[cfg(all(feature = "platform-linux", target_os = "linux"))]
    #[test]
    fn test_linux_futex_builder() {
        let futex = LinuxFutexBuilder::new().with_initial_value(42).build();

        // Test that the futex was created successfully
        assert_eq!(core::mem::size_of_val(&futex) > 0, true);
    }

    #[cfg(feature = "platform-zephyr")]
    #[test]
    fn test_zephyr_allocator_builder() {
        let allocator = ZephyrAllocatorBuilder::new()
            .with_maximum_pages(100)
            .with_memory_domains(true)
            .with_guard_regions(true)
            .build();

        // Test that the allocator was created successfully
        assert_eq!(core::mem::size_of_val(&allocator) > 0, true);
    }

    #[cfg(feature = "platform-zephyr")]
    #[test]
    fn test_zephyr_futex_builder() {
        let futex = ZephyrFutexBuilder::new().with_initial_value(42).build();

        // Test that the futex was created successfully
        assert_eq!(core::mem::size_of_val(&futex) > 0, true);
    }

    #[cfg(feature = "platform-zephyr")]
    #[test]
    fn test_zephyr_semaphore_futex() {
        let futex = ZephyrSemaphoreFutex::new(0);

        // Test that the semaphore futex was created successfully
        assert_eq!(core::mem::size_of_val(&futex) > 0, true);
    }

    #[cfg(feature = "platform-tock")]
    #[test]
    fn test_tock_allocator_builder() {
        let builder = TockAllocatorBuilder::new()
            .with_maximum_pages(64)
            .with_verification_level(VerificationLevel::Full);

        assert_eq!(builder.maximum_pages, 64);
        assert_eq!(builder.verification_level, VerificationLevel::Full);
    }

    #[cfg(feature = "platform-tock")]
    #[test]
    fn test_tock_futex_builder() {
        let futex = TockFutexBuilder::new().with_initial_value(123).with_ipc(true).build().unwrap();

        assert_eq!(futex.load(), 123);
    }

    #[test]
    fn test_hybrid_platform_abstraction() {
        use crate::platform_abstraction::*;

        // Test configuration creation for different paradigms
        let posix_config =
            PlatformConfig::<paradigm::Posix>::new().with_max_pages(1024).with_guard_pages(true);
        assert_eq!(posix_config.max_pages, 1024);
        assert!(posix_config.guard_pages);

        let security_config = PlatformConfig::<paradigm::SecurityFirst>::new()
            .with_max_pages(512)
            .with_static_allocation(64 * 1024)
            .with_isolation_level(IsolationLevel::Hardware);
        assert_eq!(security_config.max_pages, 512);
        assert_eq!(security_config.static_allocation_size, Some(64 * 1024));
        assert_eq!(security_config.isolation_level, Some(IsolationLevel::Hardware));

        let realtime_config =
            PlatformConfig::<paradigm::RealTime>::new().with_max_pages(256).with_rt_priority(10);
        assert_eq!(realtime_config.max_pages, 256);
        assert_eq!(realtime_config.rt_priority, Some(10));
    }

    #[test]
    fn test_platform_auto_selection() {
        use crate::platform_abstraction::platform_select;

        // Test that we can create auto-selected platform
        let _platform = platform_select::create_auto_platform();

        // The actual type will depend on which features are enabled
        // This test mainly ensures the compilation works
    }

    #[test]
    fn test_runtime_detection() {
        use crate::runtime_detection::PlatformDetector;

        let mut detector = PlatformDetector::new();
        let capabilities = detector.detect().unwrap();

        // Test that we can detect basic capabilities
        assert!(capabilities.memory.allocation_granularity > 0);
        assert!(capabilities.supports_wasm_runtime());

        let paradigm = capabilities.recommended_paradigm();
        assert!(paradigm == "SecurityFirst" || paradigm == "RealTime" || paradigm == "Posix");

        // Test caching works
        let capabilities2 = detector.detect().unwrap();
        assert_eq!(
            capabilities.memory.allocation_granularity,
            capabilities2.memory.allocation_granularity
        );
    }
}

#[cfg(all(not(feature = "std"), not(test)))] // Apply panic handler for no_std builds, excluding test context
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // For no_std builds, simply loop indefinitely on panic.
    // A more sophisticated handler might print to a debug console if available.
    loop {}
}
