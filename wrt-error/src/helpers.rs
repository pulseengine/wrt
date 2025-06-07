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
//! This module provides helper functions for creating common error types,
//! including foundation-specific errors for the new unified type system,
//! memory providers, and safety primitives.

use crate::{codes, Error, ErrorCategory};

// Re-export error kind creation functions
pub use crate::kinds::*;

/// Create a safety violation error
pub const fn safety_violation_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Safety, codes::SAFETY_VIOLATION, message)
}

/// Create a safety ASIL violation error
pub const fn safety_asil_violation_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Safety, codes::SAFETY_ASIL_VIOLATION, message)
}

/// Create a memory corruption detected error
pub const fn memory_corruption_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Safety, codes::MEMORY_CORRUPTION_DETECTED, message)
}

/// Create a verification failed error
pub const fn verification_failed_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Safety, codes::VERIFICATION_FAILED, message)
}

/// Create a unified type configuration error
pub const fn unified_type_config_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Type, codes::UNIFIED_TYPE_CONFIG_ERROR, message)
}

/// Create a platform capacity mismatch error
pub const fn platform_capacity_mismatch_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Capacity, codes::PLATFORM_CAPACITY_MISMATCH, message)
}

/// Create a memory provider creation error
pub const fn memory_provider_creation_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Memory, codes::MEMORY_PROVIDER_CREATION_ERROR, message)
}

/// Create a memory allocation failed error
pub const fn memory_allocation_failed_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Memory, codes::MEMORY_ALLOCATION_FAILED, message)
}

/// Create a memory provider capacity exceeded error
pub const fn memory_provider_capacity_exceeded_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Capacity, codes::MEMORY_PROVIDER_CAPACITY_EXCEEDED, message)
}

/// Create a bounded collection capacity exceeded error
pub const fn bounded_collection_capacity_exceeded_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Capacity, codes::BOUNDED_COLLECTION_CAPACITY_EXCEEDED, message)
}

/// Create a bounded collection invalid capacity error
pub const fn bounded_collection_invalid_capacity_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Validation, codes::BOUNDED_COLLECTION_INVALID_CAPACITY, message)
}

/// Create a bounded collection conversion error
pub const fn bounded_collection_conversion_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Type, codes::BOUNDED_COLLECTION_CONVERSION_ERROR, message)
}

/// Create an invalid value error
pub const fn invalid_value_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Validation, codes::INVALID_VALUE, message)
}

/// Create an unimplemented feature error
pub const fn unimplemented_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::NotSupported, codes::UNIMPLEMENTED, message)
}

/// Create a conversion error
pub const fn conversion_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Type, codes::CONVERSION_ERROR, message)
}

// Agent B helper stubs
/// Create a platform detection failed error
pub const fn platform_detection_failed_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::System, codes::PLATFORM_DETECTION_FAILED, message)
}

/// Create a memory limit exceeded error
pub const fn memory_limit_exceeded_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Capacity, codes::MEMORY_LIMIT_EXCEEDED, message)
}

/// Create a stack limit exceeded error
pub const fn stack_limit_exceeded_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Capacity, codes::STACK_LIMIT_EXCEEDED, message)
}

// Agent C helper stubs
/// Create a WIT input too large error
pub const fn wit_input_too_large_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Parse, codes::WIT_INPUT_TOO_LARGE, message)
}

/// Create an insufficient memory error
pub const fn insufficient_memory_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Memory, codes::INSUFFICIENT_MEMORY, message)
}

/// Create a resource type limit exceeded error
pub const fn resource_type_limit_exceeded_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Resource, codes::RESOURCE_TYPE_LIMIT_EXCEEDED, message)
}

// Agent D helper stubs
/// Create a CFI validation failed error
pub const fn cfi_validation_failed_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Safety, codes::CFI_VALIDATION_FAILED, message)
}

/// Create a CFI unsupported error
pub const fn cfi_unsupported_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::NotSupported, codes::CFI_UNSUPPORTED, message)
}

/// Create an execution engine error
pub const fn execution_engine_error(message: &'static str) -> Error {
    Error::new(ErrorCategory::Runtime, codes::EXECUTION_ENGINE_ERROR, message)
}
