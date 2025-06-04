//! Helper crate for WebAssembly Runtime
//!
//! This crate provides common utilities and helpers for the WebAssembly
//! Runtime.

#![cfg_attr(not(feature = "std"), no_std)]

// Import std when the feature is enabled
#[cfg(feature = "std")]
extern crate std;

// Import alloc when the feature is enabled
#[cfg(feature = "alloc")]
extern crate alloc;

/// Version of the helper crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Helper function to check if a feature is enabled
#[macro_export]
macro_rules! has_feature {
    ($feature:expr) => {
        cfg!(feature = $feature)
    };
}

// Panic handler disabled to avoid conflicts with other crates
// // Provide a panic handler only when wrt-helper is being tested in isolation
// #[cfg(all(not(feature = "std"), not(test), not(feature = "disable-panic-handler")))]
// #[panic_handler]
// fn panic(_info: &core::panic::PanicInfo) -> ! {
//     loop {}
// }
