//! SIMD instruction implementations for WebAssembly
//!
//! This module contains implementations of WebAssembly SIMD instructions
//! that operate on 128-bit vectors with various lane configurations.

// Define modules - only include those that are implemented
mod common;
mod f32x4;
mod f64x2;
mod i16x8;
mod i32x4;
mod i64x2;
mod i8x16;

// Export only used modules
pub use f32x4::f32x4_splat;
pub use f64x2::f64x2_splat;
pub use i16x8::*;
pub use i32x4::{
    i32x4_extadd_pairwise_i16x8_s, i32x4_extadd_pairwise_i16x8_u, i32x4_extract_lane,
    i32x4_replace_lane, i32x4_splat,
};
pub use i64x2::*;
pub use i8x16::*;

// Re-export helper functions needed by other modules
pub use i16x8::get_i16_lane;
pub use i32x4::set_i32_lane;
