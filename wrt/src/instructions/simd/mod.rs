//! SIMD instruction implementations for WebAssembly
//!
//! This module contains implementations of WebAssembly SIMD instructions
//! that operate on 128-bit vectors with various lane configurations.

// Define modules - only include those that are implemented
mod common;
mod f32x4;
mod f64x2;

// Export only used modules
pub use f32x4::f32x4_splat;
pub use f64x2::f64x2_splat;
