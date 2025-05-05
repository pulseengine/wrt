// Synchronization primitives for WRT.
// Direct re-exports from wrt-sync crate
// This is a thin wrapper around the wrt-sync crate primitives

// Re-export directly from wrt-sync for both std and no_std environments
pub use wrt_sync::{
    WrtMutex as Mutex, WrtMutexGuard as MutexGuard, WrtRwLock as RwLock,
    WrtRwLockReadGuard as RwLockReadGuard, WrtRwLockWriteGuard as RwLockWriteGuard,
};
