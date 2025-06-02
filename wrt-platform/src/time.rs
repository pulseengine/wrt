//! Time utilities for WebAssembly runtime
//!
//! This module provides basic time functionality for tracking
//! thread execution times and durations.

/// Get current time in nanoseconds
/// 
/// In std environments, uses system time.
/// In no_std environments, returns a monotonic counter.
#[cfg(feature = "std")]
pub fn current_time_ns() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Get current time in nanoseconds (no_std version)
/// 
/// Returns a simple monotonic counter since we can't access
/// real time in no_std environments.
#[cfg(not(feature = "std"))]
pub fn current_time_ns() -> u64 {
    use core::sync::atomic::{AtomicU64, Ordering};
    
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1000000, Ordering::Relaxed) // Increment by 1ms equivalent
}