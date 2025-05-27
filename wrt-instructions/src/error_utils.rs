//! Error formatting utilities for no_std compatibility

use wrt_error::{Error, ErrorCategory};

/// Error context for instruction operations
#[derive(Debug, Clone, Copy)]
pub enum InstructionErrorContext {
    /// Type mismatch in operation
    TypeMismatch { expected: &'static str, actual: &'static str },
    /// Stack underflow
    StackUnderflow { required: usize, available: usize },
    /// Invalid memory access
    InvalidMemoryAccess { offset: u32, size: u32 },
    /// Division by zero
    DivisionByZero,
    /// Integer overflow
    IntegerOverflow,
    /// Invalid conversion
    InvalidConversion { from: &'static str, to: &'static str },
    /// Table out of bounds
    TableOutOfBounds { index: u32, size: u32 },
    /// Invalid reference
    InvalidReference,
    /// Function not found
    FunctionNotFound { index: u32 },
    /// Invalid branch target
    InvalidBranchTarget { depth: u32 },
}

/// Format an error with context (with alloc)
#[cfg(feature = "alloc")]
pub fn format_error(category: ErrorCategory, code: u32, context: InstructionErrorContext) -> Error {
    use alloc::format;
    
    let _message = match context {
        InstructionErrorContext::TypeMismatch { expected, actual } => {
            format!("Expected {}, got {}", expected, actual)
        }
        InstructionErrorContext::StackUnderflow { required, available } => {
            format!("Stack underflow: required {}, available {}", required, available)
        }
        InstructionErrorContext::InvalidMemoryAccess { offset, size } => {
            format!("Invalid memory access at offset {} with size {}", offset, size)
        }
        InstructionErrorContext::DivisionByZero => {
            "Division by zero".into()
        }
        InstructionErrorContext::IntegerOverflow => {
            "Integer overflow".into()
        }
        InstructionErrorContext::InvalidConversion { from, to } => {
            format!("Invalid conversion from {} to {}", from, to)
        }
        InstructionErrorContext::TableOutOfBounds { index, size } => {
            format!("Table index {} out of bounds (size: {})", index, size)
        }
        InstructionErrorContext::InvalidReference => {
            "Invalid reference".into()
        }
        InstructionErrorContext::FunctionNotFound { index } => {
            format!("Function {} not found", index)
        }
        InstructionErrorContext::InvalidBranchTarget { depth } => {
            format!("Invalid branch target depth: {}", depth)
        }
    };
    
    // Use a static message since Error::new requires &'static str
    let static_message = match context {
        InstructionErrorContext::TypeMismatch { .. } => "Type mismatch",
        InstructionErrorContext::StackUnderflow { .. } => "Stack underflow", 
        InstructionErrorContext::InvalidMemoryAccess { .. } => "Invalid memory access",
        InstructionErrorContext::DivisionByZero => "Division by zero",
        InstructionErrorContext::IntegerOverflow => "Integer overflow",
        InstructionErrorContext::InvalidConversion { .. } => "Invalid conversion",
        InstructionErrorContext::TableOutOfBounds { .. } => "Table index out of bounds",
        InstructionErrorContext::InvalidReference => "Invalid reference",
        InstructionErrorContext::FunctionNotFound { .. } => "Function not found",
        InstructionErrorContext::InvalidBranchTarget { .. } => "Invalid branch target depth",
    };
    Error::new(category, code as u16, static_message)
}

/// Format an error with context (no alloc)
#[cfg(not(feature = "alloc"))]
pub fn format_error(category: ErrorCategory, code: u32, context: InstructionErrorContext) -> Error {
    let message = match context {
        InstructionErrorContext::TypeMismatch { expected, .. } => expected,
        InstructionErrorContext::StackUnderflow { .. } => "Stack underflow",
        InstructionErrorContext::InvalidMemoryAccess { .. } => "Invalid memory access",
        InstructionErrorContext::DivisionByZero => "Division by zero",
        InstructionErrorContext::IntegerOverflow => "Integer overflow",
        InstructionErrorContext::InvalidConversion { from, .. } => from,
        InstructionErrorContext::TableOutOfBounds { .. } => "Table index out of bounds",
        InstructionErrorContext::InvalidReference => "Invalid reference",
        InstructionErrorContext::FunctionNotFound { .. } => "Function not found",
        InstructionErrorContext::InvalidBranchTarget { .. } => "Invalid branch target",
    };
    
    // Use a static message since Error::new requires &'static str
    let static_message = match context {
        InstructionErrorContext::TypeMismatch { .. } => "Type mismatch",
        InstructionErrorContext::StackUnderflow { .. } => "Stack underflow", 
        InstructionErrorContext::InvalidMemoryAccess { .. } => "Invalid memory access",
        InstructionErrorContext::DivisionByZero => "Division by zero",
        InstructionErrorContext::IntegerOverflow => "Integer overflow",
        InstructionErrorContext::InvalidConversion { .. } => "Invalid conversion",
        InstructionErrorContext::TableOutOfBounds { .. } => "Table index out of bounds",
        InstructionErrorContext::InvalidReference => "Invalid reference",
        InstructionErrorContext::FunctionNotFound { .. } => "Function not found",
        InstructionErrorContext::InvalidBranchTarget { .. } => "Invalid branch target depth",
    };
    Error::new(category, code as u16, static_message)
}

/// Helper macro for creating instruction errors
#[macro_export]
macro_rules! instruction_error {
    ($category:expr, $code:expr, $context:expr) => {
        $crate::error_utils::format_error($category, $code, $context)
    };
}

/// Type name helper for error messages
pub fn type_name(value: &crate::prelude::Value) -> &'static str {
    match value {
        crate::prelude::Value::I32(_) => "I32",
        crate::prelude::Value::I64(_) => "I64",
        crate::prelude::Value::F32(_) => "F32",
        crate::prelude::Value::F64(_) => "F64",
        crate::prelude::Value::FuncRef(_) => "FuncRef",
        crate::prelude::Value::ExternRef(_) => "ExternRef",
        crate::prelude::Value::V128(_) => "V128",
        crate::prelude::Value::Ref(_) => "Ref",
        crate::prelude::Value::I16x8(_) => "I16x8",
        // Note: Void type removed from Value enum
    }
}