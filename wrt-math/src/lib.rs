// Copyright (c) 2025 R T
// SPDX-License-Identifier: MIT
// Project: WRT
// Module: wrt-math (SW-REQ-ID-TBD)

//! Mathematical operations and types for WRT.
//! Provides implementations for WebAssembly numeric instructions.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::todo, clippy::unimplemented)]
#![warn(clippy::pedantic)]
// Allow specific lints necessary for low-level math/Wasm ops, matching Cargo.toml
#![allow(clippy::float_arithmetic, clippy::cast_possible_truncation, clippy::cast_sign_loss)]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std with allocation
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Modules
pub mod float_bits;
pub mod ops;
pub mod prelude;
pub mod traits;

// Re-export key types and potentially functions for easier access
pub use float_bits::{FloatBits32, FloatBits64};
// Re-export all operations from the ops module
pub use ops::*; // Consider selectively exporting if API needs to be controlled
// Re-export error type from wrt-error for convenience
pub use wrt_error::Error as WrtMathError; // Alias specific to this crate context
pub use wrt_error::Result as WrtMathResult; // Alias specific to this crate context
