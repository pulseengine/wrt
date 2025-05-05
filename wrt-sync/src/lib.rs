#![no_std]
#![cfg_attr(feature = "std", allow(unused_imports))]
#![doc = include_str!("../README.md")]
#![warn(clippy::missing_panics_doc)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Allow `alloc` crate usage when no_std
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Conditionally use `std` for tests or specific features
#[cfg(feature = "std")]
extern crate std;

// Verify required features
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
compile_error!("The 'alloc' feature must be enabled when using no_std");

/// Mutex implementation for both std and no_std environments.
///
/// This module provides a synchronization primitive that enforces mutual exclusion,
/// allowing only one thread at a time to access the protected data. The implementation
/// transparently adapts to both standard library and no_std environments based on
/// feature flags.
///
/// # Examples
///
/// ```
/// use wrt_sync::prelude::*;
///
/// let mutex = Mutex::new(42);
/// let mut guard = mutex.lock();
/// *guard += 1;
/// assert_eq!(*guard, 43);
/// ```
pub mod mutex;

/// Prelude module that provides unified imports for both std and no_std.
///
/// This module re-exports the appropriate implementations of synchronization primitives
/// based on the enabled feature flags, allowing downstream code to import from a single
/// location regardless of the target environment.
pub mod prelude;

/// ReadWrite lock implementation for both std and no_std environments.
///
/// This module provides a synchronization primitive that allows multiple readers
/// or a single writer to access the protected data at any given time. The implementation
/// transparently adapts to both standard library and no_std environments based on
/// feature flags.
///
/// # Examples
///
/// ```
/// use wrt_sync::prelude::*;
///
/// let rwlock = RwLock::new(42);
/// let read_guard = rwlock.read();
/// assert_eq!(*read_guard, 42);
/// ```
pub mod rwlock;

// Include verification module conditionally, but exclude during coverage builds
#[cfg(all(not(coverage), doc))]
#[cfg_attr(docsrs, doc(cfg(feature = "kani")))]
pub mod verify;

// Re-export types for convenience
pub use mutex::*;
pub use rwlock::*;
