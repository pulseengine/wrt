// Synchronization primitives for WRT.
// Re-exports synchronization primitives from the wrt-sync crate.

// Re-export the mutex and rwlock types from wrt-sync
pub use wrt_sync::{
    WrtMutex as Mutex, WrtMutexGuard as MutexGuard, WrtRwLock as RwLock,
    WrtRwLockReadGuard as RwLockReadGuard, WrtRwLockWriteGuard as RwLockWriteGuard,
};

// Optional std-only exports when running on std
#[cfg(feature = "std")]
pub use wrt_sync::{
    WrtParkingRwLock as ParkingRwLock, WrtParkingRwLockReadGuard as ParkingRwLockReadGuard,
    WrtParkingRwLockWriteGuard as ParkingRwLockWriteGuard,
};

// Also re-export directly from the root for convenience in other files
#[cfg(feature = "std")]
pub use wrt_sync::WrtParkingRwLock;

#[cfg(feature = "std")]
pub use wrt_sync::{WrtParkingRwLockReadGuard, WrtParkingRwLockWriteGuard};
