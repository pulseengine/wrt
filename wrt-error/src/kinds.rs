//! Defines specific error kinds used within the WRT runtime.

use super::source::ErrorSource;
use core::fmt::{self, Debug, Display};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::String;

// --- Common WebAssembly Error Types ---

/// Error when label stack underflows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelStackUnderflowError;
impl Display for LabelStackUnderflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Label stack underflow")
    }
}
impl ErrorSource for LabelStackUnderflowError {}

/// Error for unaligned memory access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnalignedMemoryAccessError;
impl Display for UnalignedMemoryAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unaligned memory access")
    }
}
impl ErrorSource for UnalignedMemoryAccessError {}

/// Error for invalid memory access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidMemoryAccessError;
impl Display for InvalidMemoryAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid memory access")
    }
}
impl ErrorSource for InvalidMemoryAccessError {}

/// Error for invalid module instance index
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidInstanceIndexError(pub usize);
impl Display for InvalidInstanceIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid module instance index: {}", self.0)
    }
}
impl ErrorSource for InvalidInstanceIndexError {}

/// General execution error with a message
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ExecutionError(pub String);
#[cfg(feature = "alloc")]
impl Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Execution error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ExecutionError {}

/// Error for not implemented features (replacing UnimplementedError for backward compatibility)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct NotImplementedError(pub String);
#[cfg(feature = "alloc")]
impl Display for NotImplementedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Not implemented: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for NotImplementedError {}

/// Error for memory access issues
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct MemoryAccessError(pub String);
#[cfg(feature = "alloc")]
impl Display for MemoryAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Memory access error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for MemoryAccessError {}

/// Error for initialization failures
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct InitializationError(pub String);
#[cfg(feature = "alloc")]
impl Display for InitializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Initialization error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for InitializationError {}

/// Error for type mismatches
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct TypeMismatchError(pub String);
#[cfg(feature = "alloc")]
impl Display for TypeMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type mismatch: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for TypeMismatchError {}

/// Error for invalid local index
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidLocalIndexError(pub u32);
impl Display for InvalidLocalIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid local index: {}", self.0)
    }
}
impl ErrorSource for InvalidLocalIndexError {}

/// Error for invalid data segment index
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidDataSegmentIndexError(pub u32);
impl Display for InvalidDataSegmentIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid data segment index: {}", self.0)
    }
}
impl ErrorSource for InvalidDataSegmentIndexError {}

/// Error for invalid branch target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidBranchTargetError {
    pub depth: u32,
}
impl Display for InvalidBranchTargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid branch target with depth {}", self.depth)
    }
}
impl ErrorSource for InvalidBranchTargetError {}

/// Error for memory access out of bounds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryAccessOutOfBoundsError {
    pub address: u64,
    pub length: u64,
}
impl Display for MemoryAccessOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Memory access out of bounds: address {}, length {}",
            self.address, self.length
        )
    }
}
impl ErrorSource for MemoryAccessOutOfBoundsError {}

/// Error for unaligned memory access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryAccessUnalignedError {
    pub address: u64,
    pub length: u64,
}
impl Display for MemoryAccessUnalignedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unaligned memory access: address {}, requested alignment {}",
            self.address, self.length
        )
    }
}
impl ErrorSource for MemoryAccessUnalignedError {}

/// Error for export not found
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ExportNotFoundError(pub String);
#[cfg(feature = "alloc")]
impl Display for ExportNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Export not found: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ExportNotFoundError {}

/// Error for fuel exhaustion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuelExhaustedError;
impl Display for FuelExhaustedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fuel exhausted for execution")
    }
}
impl ErrorSource for FuelExhaustedError {}

/// Error for invalid function index
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidFunctionIndexError(pub usize);
impl Display for InvalidFunctionIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid function index: {}", self.0)
    }
}
impl ErrorSource for InvalidFunctionIndexError {}

/// Error for invalid function type
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct InvalidFunctionTypeError(pub String);
#[cfg(feature = "alloc")]
impl Display for InvalidFunctionTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid function type: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for InvalidFunctionTypeError {}

/// WebAssembly trap
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct Trap(pub String);
#[cfg(feature = "alloc")]
impl Display for Trap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WebAssembly trap: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for Trap {}

/// Generic memory out of bounds error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryOutOfBoundsError;
impl Display for MemoryOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Memory access out of bounds")
    }
}
impl ErrorSource for MemoryOutOfBoundsError {}

/// Generic table access out of bounds error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableAccessOutOfBounds;
impl Display for TableAccessOutOfBounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Table access out of bounds")
    }
}
impl ErrorSource for TableAccessOutOfBounds {}

/// Detailed table access out of bounds error with table and element indices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableAccessOutOfBoundsError {
    pub table_idx: u32,
    pub element_idx: usize,
}
impl Display for TableAccessOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Table access out of bounds: table {} at index {}",
            self.table_idx, self.element_idx
        )
    }
}
impl ErrorSource for TableAccessOutOfBoundsError {}

/// Parse error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ParseError(pub String);
#[cfg(feature = "alloc")]
impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ParseError {}

/// Error for validation failures
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ValidationError(pub String);
#[cfg(feature = "alloc")]
impl Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ValidationError {}

/// Error for memory growth failures
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct MemoryGrowError(pub String);
#[cfg(feature = "alloc")]
impl Display for MemoryGrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Memory grow error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for MemoryGrowError {}

/// Error for poisoned locks
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct PoisonedLockError(pub String);
#[cfg(feature = "alloc")]
impl Display for PoisonedLockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Poisoned lock error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for PoisonedLockError {}

/// Error for unsupported features
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct UnsupportedError(pub String);
#[cfg(feature = "alloc")]
impl Display for UnsupportedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unsupported feature or operation: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for UnsupportedError {}

/// Stack underflow error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackUnderflowError;
impl Display for StackUnderflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stack underflow")
    }
}
impl ErrorSource for StackUnderflowError {}

/// Function not found error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct FunctionNotFoundError(pub String);
#[cfg(feature = "alloc")]
impl Display for FunctionNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Function not found: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for FunctionNotFoundError {}

/// Invalid memory index error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidMemoryIndexError(pub u32);
impl Display for InvalidMemoryIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid memory index: {}", self.0)
    }
}
impl ErrorSource for InvalidMemoryIndexError {}

/// Invalid global index error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidGlobalIndexError(pub u32);
impl Display for InvalidGlobalIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid global index: {}", self.0)
    }
}
impl ErrorSource for InvalidGlobalIndexError {}

/// Invalid type error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct InvalidTypeError(pub String);
#[cfg(feature = "alloc")]
impl Display for InvalidTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid type: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for InvalidTypeError {}

/// Division by zero error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DivisionByZeroError;
impl Display for DivisionByZeroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Division by zero")
    }
}
impl ErrorSource for DivisionByZeroError {}

/// Integer overflow error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntegerOverflowError;
impl Display for IntegerOverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Integer overflow")
    }
}
impl ErrorSource for IntegerOverflowError {}

/// Error for invalid table index
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTableIndexError(pub u32);
impl Display for InvalidTableIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid table index: {}", self.0)
    }
}
impl ErrorSource for InvalidTableIndexError {}

/// Error for invalid element index
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidElementIndexError(pub u32);
impl Display for InvalidElementIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid element index: {}", self.0)
    }
}
impl ErrorSource for InvalidElementIndexError {}

/// Element segment out of bounds error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElementSegmentOutOfBoundsError(pub u32);
impl Display for ElementSegmentOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Element segment out of bounds: {}", self.0)
    }
}
impl ErrorSource for ElementSegmentOutOfBoundsError {}

/// General function type error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct InvalidFunctionType(pub String);
#[cfg(feature = "alloc")]
impl Display for InvalidFunctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid function type: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for InvalidFunctionType {}

/// General type error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct InvalidType(pub String);
#[cfg(feature = "alloc")]
impl Display for InvalidType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid type: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for InvalidType {}

/// General function not found error with index
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionNotFound(pub u32);
impl Display for FunctionNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Function not found with index: {}", self.0)
    }
}
impl ErrorSource for FunctionNotFound {}

/// General execution error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct Execution(pub String);
#[cfg(feature = "alloc")]
impl Display for Execution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Execution error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for Execution {}

/// Stack underflow error (value)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackUnderflow;
impl Display for StackUnderflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Stack underflow")
    }
}
impl ErrorSource for StackUnderflow {}

/// Runtime error type
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct RuntimeError(pub String);
#[cfg(feature = "alloc")]
impl Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Runtime error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for RuntimeError {}

/// Error for out of bounds access with a message
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct OutOfBoundsError(pub String);
#[cfg(feature = "alloc")]
impl Display for OutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Out of bounds error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for OutOfBoundsError {}

/// Error for resource access issues
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ResourceAccessError(pub String);
#[cfg(feature = "alloc")]
impl Display for ResourceAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Resource access error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ResourceAccessError {}

/// Error for invalid resource handle
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct InvalidResourceHandle(pub String);
#[cfg(feature = "alloc")]
impl Display for InvalidResourceHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid resource handle: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for InvalidResourceHandle {}

/// Error for resource limit exceeded
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ResourceLimitExceeded(pub String);
#[cfg(feature = "alloc")]
impl Display for ResourceLimitExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Resource limit exceeded: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ResourceLimitExceeded {}

/// Error for unsupported operations
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct UnsupportedOperation(pub String);
#[cfg(feature = "alloc")]
impl Display for UnsupportedOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unsupported operation: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for UnsupportedOperation {}
