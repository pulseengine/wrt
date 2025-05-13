// WRT - wrt-platform
// Module: Crate Prelude
// SW-REQ-ID: REQ_PLATFORM_001
//
// Copyright (c) 2025 The WRT Project Developers
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Public prelude for the `wrt-platform` crate.
//!
//! This module re-exports the core traits and types for convenient use.

// Re-export core error type (already re-exported in lib.rs, but good practice)
pub use wrt_error::Error;

// Re-export memory allocator trait and fallback implementation
#[cfg(feature = "std")]
pub use crate::memory::FallbackAllocator;
// Re-export sync trait and fallback implementation
#[cfg(feature = "std")]
pub use crate::sync::FallbackFutex;
pub use crate::{
    memory::{PageAllocator, WASM_PAGE_SIZE},
    sync::FutexLike,
};
