// WRT - wrt-error
// Module: WRT Error Helpers
// SW-REQ-ID: REQ_004
// SW-REQ-ID: REQ_ERROR_001
//
// Copyright (c) 2024 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Error helper functions for common error patterns.
//!
//! This module primarily re-exports functionality from the kinds module
//! for backward compatibility with existing code.

// Re-export error kind creation functions
pub use crate::kinds::*;
