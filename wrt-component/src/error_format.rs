//! Error formatting utilities for no_std compatibility
//!
//! This module provides alternatives to the format! macro for error messages
//! in no_std environments.

use wrt_error::{Error, ErrorCategory};

/// Error context for canonical ABI operations
#[derive(Debug, Clone, Copy)]
pub enum CanonicalErrorContext {
    OutOfBounds { addr: u32, size: usize },
    InvalidUtf8,
    InvalidCodePoint { code_point: u32 },
    InvalidDiscriminant { discriminant: u32 },
    NotImplemented(&'static str),
    TypeMismatch,
    ResourceNotFound { handle: u32 },
    InvalidAlignment { addr: u32, align: u32 },
    InvalidSize { expected: usize, actual: usize },
}

/// Format an error message for the given context
#[cfg(feature = "alloc")]
pub fn format_error(category: ErrorCategory, code: u32, context: CanonicalErrorContext) -> Error {
    use alloc::format;
    
    let message = match context {
        CanonicalErrorContext::OutOfBounds { addr, size } => {
            format!("Address {} out of bounds for memory of size {}", addr, size)
        }
        CanonicalErrorContext::InvalidUtf8 => {
            "Invalid UTF-8 string".to_string()
        }
        CanonicalErrorContext::InvalidCodePoint { code_point } => {
            format!("Invalid UTF-8 code point: {}", code_point)
        }
        CanonicalErrorContext::InvalidDiscriminant { discriminant } => {
            format!("Invalid variant discriminant: {}", discriminant)
        }
        CanonicalErrorContext::NotImplemented(feature) => {
            format!("{} not yet implemented", feature)
        }
        CanonicalErrorContext::TypeMismatch => {
            "Type mismatch".to_string()
        }
        CanonicalErrorContext::ResourceNotFound { handle } => {
            format!("Resource not found: {}", handle)
        }
        CanonicalErrorContext::InvalidAlignment { addr, align } => {
            format!("Address {} not aligned to {}", addr, align)
        }
        CanonicalErrorContext::InvalidSize { expected, actual } => {
            format!("Invalid size: expected {}, got {}", expected, actual)
        }
    };
    
    Error::new(category, code, message)
}

/// Format an error message for the given context (no_std version)
#[cfg(not(feature = "alloc"))]
pub fn format_error(category: ErrorCategory, code: u32, context: CanonicalErrorContext) -> Error {
    let message = match context {
        CanonicalErrorContext::OutOfBounds { .. } => {
            "Address out of bounds"
        }
        CanonicalErrorContext::InvalidUtf8 => {
            "Invalid UTF-8 string"
        }
        CanonicalErrorContext::InvalidCodePoint { .. } => {
            "Invalid UTF-8 code point"
        }
        CanonicalErrorContext::InvalidDiscriminant { .. } => {
            "Invalid variant discriminant"
        }
        CanonicalErrorContext::NotImplemented(feature) => {
            feature
        }
        CanonicalErrorContext::TypeMismatch => {
            "Type mismatch"
        }
        CanonicalErrorContext::ResourceNotFound { .. } => {
            "Resource not found"
        }
        CanonicalErrorContext::InvalidAlignment { .. } => {
            "Invalid alignment"
        }
        CanonicalErrorContext::InvalidSize { .. } => {
            "Invalid size"
        }
    };
    
    Error::new(category, code, message)
}

/// Component error context
#[derive(Debug, Clone, Copy)]
pub enum ComponentErrorContext {
    ImportNotFound(&'static str),
    ExportNotFound(&'static str),
    InvalidComponentType,
    LinkingFailed,
    InstantiationFailed,
    ResourceLimitExceeded,
}

/// Format a component error
#[cfg(feature = "alloc")]
pub fn format_component_error(category: ErrorCategory, code: u32, context: ComponentErrorContext) -> Error {
    use alloc::format;
    
    let message = match context {
        ComponentErrorContext::ImportNotFound(name) => {
            format!("Import not found: {}", name)
        }
        ComponentErrorContext::ExportNotFound(name) => {
            format!("Export not found: {}", name)
        }
        ComponentErrorContext::InvalidComponentType => {
            "Invalid component type".to_string()
        }
        ComponentErrorContext::LinkingFailed => {
            "Component linking failed".to_string()
        }
        ComponentErrorContext::InstantiationFailed => {
            "Component instantiation failed".to_string()
        }
        ComponentErrorContext::ResourceLimitExceeded => {
            "Resource limit exceeded".to_string()
        }
    };
    
    Error::new(category, code, message)
}

/// Format a component error (no_std version)
#[cfg(not(feature = "alloc"))]
pub fn format_component_error(category: ErrorCategory, code: u32, context: ComponentErrorContext) -> Error {
    let message = match context {
        ComponentErrorContext::ImportNotFound(name) => name,
        ComponentErrorContext::ExportNotFound(name) => name,
        ComponentErrorContext::InvalidComponentType => "Invalid component type",
        ComponentErrorContext::LinkingFailed => "Component linking failed",
        ComponentErrorContext::InstantiationFailed => "Component instantiation failed",
        ComponentErrorContext::ResourceLimitExceeded => "Resource limit exceeded",
    };
    
    Error::new(category, code, message)
}

/// Helper macro to create errors with context
#[macro_export]
macro_rules! canonical_error {
    ($category:expr, $code:expr, $context:expr) => {
        $crate::error_format::format_error($category, $code, $context)
    };
}

/// Helper macro to create component errors
#[macro_export]
macro_rules! component_error {
    ($category:expr, $code:expr, $context:expr) => {
        $crate::error_format::format_component_error($category, $code, $context)
    };
}