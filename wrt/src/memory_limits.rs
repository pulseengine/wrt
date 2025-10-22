//! Runtime Memory Budget Limits for Safety-Critical Operations
//!
//! This module re-exports runtime limit constants from `wrt-foundation`.
//! The original implementation has been migrated to
//! `wrt-foundation/src/runtime_limits.rs` for better architectural separation.
//!
//! For ASIL-compliant runtime configurations, these limits ensure deterministic
//! memory behavior and prevent resource exhaustion.

// Re-export all runtime limits from foundation
pub use wrt_foundation::runtime_limits::*;
