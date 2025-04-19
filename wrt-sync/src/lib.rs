#![no_std]
#![cfg_attr(feature = "std", allow(unused_imports))]
#![doc = "no_std synchronization primitives (Mutex, RwLock) for the WRT project."]

// Allow `alloc` crate usage when no_std
#[cfg(not(feature = "std"))]
extern crate alloc;

// Conditionally use `std` for tests or specific features
#[cfg(feature = "std")]
extern crate std;

pub mod mutex;
pub mod rwlock;

pub use mutex::*;
pub use rwlock::*;
