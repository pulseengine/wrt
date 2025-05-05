// Synchronization primitives for WRT.
// Re-exports synchronization primitives from the wrt-sync crate through our prelude.

// Re-export the mutex and rwlock types from our prelude
pub use crate::prelude::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

// For std compatibility when needed
#[cfg(feature = "std")]
pub use std::sync::{Mutex as StdMutex, MutexGuard as StdMutexGuard};
#[cfg(feature = "std")]
pub use std::sync::{
    RwLock as StdRwLock, RwLockReadGuard as StdRwLockReadGuard,
    RwLockWriteGuard as StdRwLockWriteGuard,
};
