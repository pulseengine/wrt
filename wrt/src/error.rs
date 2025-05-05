// Re-export all public items from wrt-error
pub use wrt_error::*;

/// Error kinds for WebAssembly runtime.
pub mod kinds {
    // Error constants for common errors
    pub const EXECUTION_ERROR: &str = "execution_error";
    pub const STACK_UNDERFLOW: &str = "stack_underflow";
    pub const EXPORT_NOT_FOUND_ERROR: &str = "export_not_found";
    pub const INVALID_TYPE_ERROR: &str = "invalid_type";
    pub const INVALID_VALUE_ERROR: &str = "invalid_value";
    pub const INVALID_FUNCTION_TYPE_ERROR: &str = "invalid_function_type";
    pub const INVALID_INSTANCE_INDEX_ERROR: &str = "invalid_instance_index";
    pub const INVALID_FUNCTION_INDEX_ERROR: &str = "invalid_function_index";
    pub const INVALID_MEMORY_INDEX_ERROR: &str = "invalid_memory_index";
    pub const INVALID_TABLE_INDEX_ERROR: &str = "invalid_table_index";
    pub const INVALID_LOCAL_INDEX_ERROR: &str = "invalid_local_index";
    pub const INVALID_GLOBAL_INDEX_ERROR: &str = "invalid_global_index";
    pub const POISONED_LOCK_ERROR: &str = "poisoned_lock";
    pub const MEMORY_ACCESS_OUT_OF_BOUNDS_ERROR: &str = "memory_access_out_of_bounds";
    pub const UNALIGNED_MEMORY_ACCESS_ERROR: &str = "unaligned_memory_access";
    pub const MEMORY_ACCESS_ERROR: &str = "memory_access_error";
    pub const NOT_IMPLEMENTED_ERROR: &str = "not_implemented";
    pub const TRAP_ERROR: &str = "trap";
    pub const VALIDATION_ERROR: &str = "validation_error";
    pub const PARSE_ERROR: &str = "parse_error";
    pub const RUNTIME_ERROR: &str = "runtime_error";

    // Error kind constructors
    pub fn ExecutionError(msg: String) -> (String, String) {
        (EXECUTION_ERROR.to_string(), msg)
    }

    pub fn ExportNotFoundError(name: String) -> (String, String) {
        (EXPORT_NOT_FOUND_ERROR.to_string(), name)
    }

    pub fn InvalidTypeError(msg: String) -> (String, String) {
        (INVALID_TYPE_ERROR.to_string(), msg)
    }

    pub fn InvalidFunctionType(msg: String) -> (String, String) {
        (INVALID_FUNCTION_TYPE_ERROR.to_string(), msg)
    }

    pub fn MemoryAccessError(msg: String) -> (String, String) {
        (MEMORY_ACCESS_ERROR.to_string(), msg)
    }

    pub fn InvalidInstanceIndexError(index: u32) -> (String, String) {
        (
            INVALID_INSTANCE_INDEX_ERROR.to_string(),
            format!("Invalid instance index: {index}"),
        )
    }

    pub fn InvalidFunctionIndexError(index: u32) -> (String, String) {
        (
            INVALID_FUNCTION_INDEX_ERROR.to_string(),
            format!("Invalid function index: {index}"),
        )
    }

    pub fn InvalidMemoryIndexError(index: u32) -> (String, String) {
        (
            INVALID_MEMORY_INDEX_ERROR.to_string(),
            format!("Invalid memory index: {index}"),
        )
    }

    pub fn InvalidTableIndexError(index: u32) -> (String, String) {
        (
            INVALID_TABLE_INDEX_ERROR.to_string(),
            format!("Invalid table index: {index}"),
        )
    }

    pub fn InvalidLocalIndexError(index: u32) -> (String, String) {
        (
            INVALID_LOCAL_INDEX_ERROR.to_string(),
            format!("Invalid local index: {index}"),
        )
    }

    pub fn InvalidGlobalIndexError(index: u32) -> (String, String) {
        (
            INVALID_GLOBAL_INDEX_ERROR.to_string(),
            format!("Invalid global index: {index}"),
        )
    }

    pub fn NotImplementedError(msg: String) -> (String, String) {
        (NOT_IMPLEMENTED_ERROR.to_string(), msg)
    }

    pub fn PoisonedLockError(msg: String) -> (String, String) {
        (POISONED_LOCK_ERROR.to_string(), msg)
    }

    pub fn ValidationError(msg: String) -> (String, String) {
        (VALIDATION_ERROR.to_string(), msg)
    }

    pub fn ParseError(msg: String) -> (String, String) {
        (PARSE_ERROR.to_string(), msg)
    }

    pub fn TrapError(msg: String) -> (String, String) {
        (TRAP_ERROR.to_string(), msg)
    }

    pub fn RuntimeError(msg: String) -> (String, String) {
        (RUNTIME_ERROR.to_string(), msg)
    }

    pub struct MemoryAccessOutOfBoundsError {
        pub address: u64,
        pub length: u64,
    }

    pub fn StackUnderflowError() -> (String, String) {
        (STACK_UNDERFLOW.to_string(), "Stack underflow".to_string())
    }
}

// Helper function to convert wrt_instructions::Error to wrt_error::Error
pub fn convert_instructions_error(err: wrt_instructions::Error) -> Error {
    // wrt_instructions::Error already wraps wrt_error::Error
    // We can just use the inner error
    err.to_inner_error()
}
