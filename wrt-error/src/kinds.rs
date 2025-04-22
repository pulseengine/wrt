//! Defines specific error kinds used within the WRT runtime.

use super::source::ErrorSource;
use core::fmt::{self, Debug, Display};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::String;

// Error codes definitions
// Core related errors
pub const LABEL_STACK_UNDERFLOW_ERROR: u16 = 1000;
pub const UNALIGNED_MEMORY_ACCESS_ERROR: u16 = 1001;
pub const INVALID_MEMORY_ACCESS_ERROR: u16 = 1002;
pub const INVALID_INSTANCE_INDEX_ERROR: u16 = 1003;
pub const EXECUTION_ERROR: u16 = 1004;
pub const NOT_IMPLEMENTED_ERROR: u16 = 1005;
pub const MEMORY_ACCESS_ERROR: u16 = 1006;
pub const INITIALIZATION_ERROR: u16 = 1007;
pub const TYPE_MISMATCH_ERROR: u16 = 1008;

// Runtime related errors
pub const INVALID_LOCAL_INDEX_ERROR: u16 = 2000;
pub const INVALID_DATA_SEGMENT_INDEX_ERROR: u16 = 2001;
pub const INVALID_BRANCH_TARGET_ERROR: u16 = 2002;
pub const MEMORY_ACCESS_OUT_OF_BOUNDS_ERROR: u16 = 2003;
pub const MEMORY_ACCESS_UNALIGNED_ERROR: u16 = 2004;
pub const EXPORT_NOT_FOUND_ERROR: u16 = 2005;
pub const FUEL_EXHAUSTED_ERROR: u16 = 2006;
pub const INVALID_FUNCTION_INDEX_ERROR: u16 = 2007;
pub const INVALID_FUNCTION_TYPE_ERROR: u16 = 2008;
pub const TRAP_ERROR: u16 = 2009;
pub const MEMORY_OUT_OF_BOUNDS_ERROR: u16 = 2010;
pub const TABLE_ACCESS_OUT_OF_BOUNDS_ERROR: u16 = 2011;
pub const PARSE_ERROR: u16 = 2012;
pub const VALIDATION_ERROR: u16 = 2013;
pub const MEMORY_GROW_ERROR: u16 = 2014;
pub const POISONED_LOCK_ERROR: u16 = 2015;
pub const UNSUPPORTED_ERROR: u16 = 2016;
pub const STACK_UNDERFLOW_ERROR: u16 = 2017;
pub const FUNCTION_NOT_FOUND_ERROR: u16 = 2018;
pub const INVALID_MEMORY_INDEX_ERROR: u16 = 2019;
pub const INVALID_GLOBAL_INDEX_ERROR: u16 = 2020;
pub const INVALID_TYPE_ERROR: u16 = 2021;

// Component related errors
pub const INVALID_FUNCTION_INDEX: u16 = 3000;
pub const TYPE_MISMATCH: u16 = 3001;
pub const ENCODING_ERROR: u16 = 3002;
pub const EXECUTION_LIMIT_EXCEEDED: u16 = 3003;
pub const RESOURCE_ERROR: u16 = 3004;
pub const COMPONENT_INSTANTIATION_ERROR: u16 = 3005;
pub const CANONICAL_ABI_ERROR: u16 = 3006;
pub const COMPONENT_LINKING_ERROR: u16 = 3007;
pub const MEMORY_ACCESS_ERROR_COMPONENT: u16 = 3008;
pub const CONVERSION_ERROR: u16 = 3009;
pub const RESOURCE_LIMIT_EXCEEDED: u16 = 3010;
pub const RESOURCE_ACCESS_ERROR: u16 = 3011;
pub const OUT_OF_BOUNDS_ACCESS: u16 = 3012;
pub const INVALID_VALUE_ERROR: u16 = 3013;

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

/// Resource access error for component model resources
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
impl ErrorSource for ResourceAccessError {
    fn code(&self) -> u16 {
        RESOURCE_ACCESS_ERROR
    }
}

/// Out of bounds access error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct OutOfBoundsAccess(pub String);
#[cfg(feature = "alloc")]
impl Display for OutOfBoundsAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Out of bounds access: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for OutOfBoundsAccess {
    fn code(&self) -> u16 {
        OUT_OF_BOUNDS_ACCESS
    }
}

/// Out of bounds error
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
impl ErrorSource for OutOfBoundsError {
    fn code(&self) -> u16 {
        OUT_OF_BOUNDS_ACCESS
    }
}

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
impl ErrorSource for ResourceLimitExceeded {
    fn code(&self) -> u16 {
        RESOURCE_LIMIT_EXCEEDED
    }
}

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

// --- Bounded Collection and Verification Error Types ---

/// Error for validation failures in bounded collections
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ValidationFailureError(pub String);
#[cfg(feature = "alloc")]
impl Display for ValidationFailureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation failure: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ValidationFailureError {}

/// Error for checksum mismatches in verified data structures
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ChecksumMismatchError {
    pub expected: u32,
    pub actual: u32,
    pub description: String,
}
#[cfg(feature = "alloc")]
impl Display for ChecksumMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Checksum mismatch in {}: expected {:08x}, got {:08x}",
            self.description, self.expected, self.actual
        )
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ChecksumMismatchError {}

/// Error for bounded collection capacity exceeded
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct BoundedCapacityExceededError {
    pub collection_type: String,
    pub capacity: usize,
    pub attempted_size: usize,
}
#[cfg(feature = "alloc")]
impl Display for BoundedCapacityExceededError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} capacity exceeded: limit {}, attempted {}",
            self.collection_type, self.capacity, self.attempted_size
        )
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for BoundedCapacityExceededError {}

/// Error for invalid access to a bounded collection
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct BoundedCollectionAccessError {
    pub collection_type: String,
    pub index: usize,
    pub size: usize,
}
#[cfg(feature = "alloc")]
impl Display for BoundedCollectionAccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid access to {} at index {}, size is {}",
            self.collection_type, self.index, self.size
        )
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for BoundedCollectionAccessError {}

/// Error for critical integrity violations
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct IntegrityViolationError(pub String);
#[cfg(feature = "alloc")]
impl Display for IntegrityViolationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Critical integrity violation: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for IntegrityViolationError {}

/// Error for verification level violations (attempting unsafe operations)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct VerificationLevelViolationError {
    pub operation: String,
    pub required_level: String,
    pub current_level: String,
}
#[cfg(feature = "alloc")]
impl Display for VerificationLevelViolationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Operation '{}' requires verification level '{}', but current level is '{}'",
            self.operation, self.required_level, self.current_level
        )
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for VerificationLevelViolationError {}

/// Decoding error for binary format parsing
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct DecodingError(pub String);
#[cfg(feature = "alloc")]
impl Display for DecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Decoding error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for DecodingError {}

/// Execution timeout error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ExecutionTimeoutError(pub String);
#[cfg(feature = "alloc")]
impl Display for ExecutionTimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Execution timeout: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ExecutionTimeoutError {}

/// Invalid function index error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct InvalidFunctionIndex(pub usize);
#[cfg(feature = "alloc")]
impl Display for InvalidFunctionIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid function index: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for InvalidFunctionIndex {
    fn code(&self) -> u16 {
        INVALID_FUNCTION_INDEX
    }
}

/// Type mismatch error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct TypeMismatch(pub String);
#[cfg(feature = "alloc")]
impl Display for TypeMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type mismatch: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for TypeMismatch {
    fn code(&self) -> u16 {
        TYPE_MISMATCH
    }
}

/// Encoding error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct EncodingError(pub String);
#[cfg(feature = "alloc")]
impl Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Encoding error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for EncodingError {
    fn code(&self) -> u16 {
        ENCODING_ERROR
    }
}

/// Execution limit exceeded error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ExecutionLimitExceeded(pub String);
#[cfg(feature = "alloc")]
impl Display for ExecutionLimitExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Execution limit exceeded: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ExecutionLimitExceeded {
    fn code(&self) -> u16 {
        EXECUTION_LIMIT_EXCEEDED
    }
}

/// Resource error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ResourceError(pub String);
#[cfg(feature = "alloc")]
impl Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Resource error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ResourceError {
    fn code(&self) -> u16 {
        RESOURCE_ERROR
    }
}

/// Component instantiation error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ComponentInstantiationError(pub String);
#[cfg(feature = "alloc")]
impl Display for ComponentInstantiationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Component instantiation error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ComponentInstantiationError {
    fn code(&self) -> u16 {
        COMPONENT_INSTANTIATION_ERROR
    }
}

/// Canonical ABI error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct CanonicalABIError(pub String);
#[cfg(feature = "alloc")]
impl Display for CanonicalABIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Canonical ABI error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for CanonicalABIError {
    fn code(&self) -> u16 {
        CANONICAL_ABI_ERROR
    }
}

/// Component linking error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ComponentLinkingError(pub String);
#[cfg(feature = "alloc")]
impl Display for ComponentLinkingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Component linking error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ComponentLinkingError {
    fn code(&self) -> u16 {
        COMPONENT_LINKING_ERROR
    }
}

/// Conversion error
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct ConversionError(pub String);
#[cfg(feature = "alloc")]
impl Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Conversion error: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for ConversionError {
    fn code(&self) -> u16 {
        CONVERSION_ERROR
    }
}

/// Invalid value
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct InvalidValue(pub String);
#[cfg(feature = "alloc")]
impl Display for InvalidValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid value: {}", self.0)
    }
}
#[cfg(feature = "alloc")]
impl ErrorSource for InvalidValue {
    fn code(&self) -> u16 {
        INVALID_VALUE_ERROR
    }
}
