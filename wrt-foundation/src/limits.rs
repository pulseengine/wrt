//! Platform-specific WebAssembly limits for bounded memory allocation.
//!
//! This module defines compile-time limits for WebAssembly module decoding and
//! execution. Different platform profiles are supported via feature flags:
//!
//! - `embedded-small`: MCU targets with ~64KB RAM
//! - `embedded-medium`: Embedded Linux with ~1MB RAM
//! - Default: Desktop/Server with ample memory
//!
//! # Design Rationale
//!
//! WebAssembly's section format provides counts BEFORE data, enabling single-pass
//! streaming decode with bounded memory. These limits enable early rejection of
//! modules that exceed platform capacity, preventing OOM conditions.
//!
//! # Usage
//!
//! ```rust
//! use wrt_foundation::limits;
//!
//! // Check before allocation
//! if param_count > limits::MAX_FUNCTION_PARAMS {
//!     return Err(Error::module_exceeds_platform_limits());
//! }
//! ```
//!
//! # Safety
//!
//! These limits are designed to ensure deterministic memory usage for
//! ASIL-D and other safety-critical configurations.

/// WebAssembly specification constants (not platform-dependent)
pub mod spec {
    /// WebAssembly page size: 64 KiB
    pub const WASM_PAGE_SIZE: usize = 65536;

    /// Maximum linear memory pages per spec (4 GiB / 64 KiB = 65536)
    pub const MAX_WASM_MEMORY_PAGES: u32 = 65536;

    /// Maximum tables per module per spec
    pub const MAX_WASM_TABLES: u32 = 100;

    /// Maximum memories per module (multi-memory proposal)
    pub const MAX_WASM_MEMORIES: u32 = 100;
}

/// Platform profile for embedded microcontrollers (~64KB RAM)
#[cfg(feature = "embedded-small")]
pub mod platform {
    /// Maximum types in a module's type section
    pub const MAX_TYPES: usize = 64;

    /// Maximum functions in a module
    pub const MAX_FUNCTIONS: usize = 256;

    /// Maximum tables in a module
    pub const MAX_TABLES: usize = 4;

    /// Maximum memories in a module (typically 1)
    pub const MAX_MEMORIES: usize = 1;

    /// Maximum globals in a module
    pub const MAX_GLOBALS: usize = 128;

    /// Maximum imports in a module
    pub const MAX_IMPORTS: usize = 64;

    /// Maximum exports in a module
    pub const MAX_EXPORTS: usize = 64;

    /// Maximum elements in an element segment
    pub const MAX_ELEMENT_ITEMS: usize = 256;

    /// Maximum data segments in a module
    pub const MAX_DATA_SEGMENTS: usize = 64;

    /// Maximum parameters per function type
    pub const MAX_FUNCTION_PARAMS: usize = 16;

    /// Maximum results per function type
    pub const MAX_FUNCTION_RESULTS: usize = 8;

    /// Maximum locals per function (not counting params)
    pub const MAX_FUNCTION_LOCALS: usize = 128;

    /// Maximum code size per function in bytes
    pub const MAX_FUNCTION_CODE_SIZE: usize = 16384; // 16 KiB

    /// Maximum total module size in bytes
    pub const MAX_MODULE_SIZE: usize = 65536; // 64 KiB

    /// Maximum value stack depth during execution
    pub const MAX_VALUE_STACK: usize = 1024;

    /// Maximum call stack depth during execution
    pub const MAX_CALL_STACK: usize = 64;

    /// Arena size for decode-phase temporary allocation
    pub const DECODE_ARENA_SIZE: usize = 64 * 1024; // 64 KiB

    /// Maximum memory pages for this platform
    pub const MAX_MEMORY_PAGES: u32 = 16; // 1 MiB max

    /// Maximum custom section name length
    pub const MAX_CUSTOM_SECTION_NAME: usize = 64;

    /// Maximum import/export name length
    pub const MAX_NAME_LENGTH: usize = 128;
}

/// Platform profile for embedded Linux (~1MB RAM)
/// Note: embedded-small takes priority if both features are enabled
#[cfg(all(feature = "embedded-medium", not(feature = "embedded-small")))]
pub mod platform {
    /// Maximum types in a module's type section
    pub const MAX_TYPES: usize = 1024;

    /// Maximum functions in a module
    pub const MAX_FUNCTIONS: usize = 4096;

    /// Maximum tables in a module
    pub const MAX_TABLES: usize = 16;

    /// Maximum memories in a module
    pub const MAX_MEMORIES: usize = 4;

    /// Maximum globals in a module
    pub const MAX_GLOBALS: usize = 1024;

    /// Maximum imports in a module
    pub const MAX_IMPORTS: usize = 512;

    /// Maximum exports in a module
    pub const MAX_EXPORTS: usize = 512;

    /// Maximum elements in an element segment
    pub const MAX_ELEMENT_ITEMS: usize = 4096;

    /// Maximum data segments in a module
    pub const MAX_DATA_SEGMENTS: usize = 256;

    /// Maximum parameters per function type
    pub const MAX_FUNCTION_PARAMS: usize = 64;

    /// Maximum results per function type
    pub const MAX_FUNCTION_RESULTS: usize = 32;

    /// Maximum locals per function (not counting params)
    pub const MAX_FUNCTION_LOCALS: usize = 2048;

    /// Maximum code size per function in bytes
    pub const MAX_FUNCTION_CODE_SIZE: usize = 262144; // 256 KiB

    /// Maximum total module size in bytes
    pub const MAX_MODULE_SIZE: usize = 1024 * 1024; // 1 MiB

    /// Maximum value stack depth during execution
    pub const MAX_VALUE_STACK: usize = 8192;

    /// Maximum call stack depth during execution
    pub const MAX_CALL_STACK: usize = 256;

    /// Arena size for decode-phase temporary allocation
    pub const DECODE_ARENA_SIZE: usize = 512 * 1024; // 512 KiB

    /// Maximum memory pages for this platform
    pub const MAX_MEMORY_PAGES: u32 = 256; // 16 MiB max

    /// Maximum custom section name length
    pub const MAX_CUSTOM_SECTION_NAME: usize = 256;

    /// Maximum import/export name length
    pub const MAX_NAME_LENGTH: usize = 512;
}

/// Platform profile for desktop/server (default - ample memory)
#[cfg(not(any(feature = "embedded-small", feature = "embedded-medium")))]
pub mod platform {
    /// Maximum types in a module's type section
    pub const MAX_TYPES: usize = 65536;

    /// Maximum functions in a module
    pub const MAX_FUNCTIONS: usize = 65536;

    /// Maximum tables in a module
    pub const MAX_TABLES: usize = 64;

    /// Maximum memories in a module
    pub const MAX_MEMORIES: usize = 64;

    /// Maximum globals in a module
    pub const MAX_GLOBALS: usize = 4096;

    /// Maximum imports in a module
    pub const MAX_IMPORTS: usize = 16384;

    /// Maximum exports in a module
    pub const MAX_EXPORTS: usize = 16384;

    /// Maximum elements in an element segment
    pub const MAX_ELEMENT_ITEMS: usize = 65536;

    /// Maximum data segments in a module
    pub const MAX_DATA_SEGMENTS: usize = 4096;

    /// Maximum parameters per function type
    pub const MAX_FUNCTION_PARAMS: usize = 1000;

    /// Maximum results per function type (multi-value)
    pub const MAX_FUNCTION_RESULTS: usize = 1000;

    /// Maximum locals per function (WebAssembly allows up to 50,000)
    pub const MAX_FUNCTION_LOCALS: usize = 50000;

    /// Maximum code size per function in bytes
    pub const MAX_FUNCTION_CODE_SIZE: usize = 8 * 1024 * 1024; // 8 MiB

    /// Maximum total module size in bytes
    pub const MAX_MODULE_SIZE: usize = 128 * 1024 * 1024; // 128 MiB

    /// Maximum value stack depth during execution
    pub const MAX_VALUE_STACK: usize = 65536;

    /// Maximum call stack depth during execution
    pub const MAX_CALL_STACK: usize = 512;

    /// Arena size for decode-phase temporary allocation
    pub const DECODE_ARENA_SIZE: usize = 4 * 1024 * 1024; // 4 MiB

    /// Maximum memory pages for this platform (spec max)
    pub const MAX_MEMORY_PAGES: u32 = 65536; // 4 GiB max

    /// Maximum custom section name length
    pub const MAX_CUSTOM_SECTION_NAME: usize = 1024;

    /// Maximum import/export name length
    pub const MAX_NAME_LENGTH: usize = 4096;
}

// Re-export platform limits at module level for convenience
pub use platform::*;

/// Validates a count against a limit, returning an error message if exceeded.
///
/// This is a helper for decoder implementations to provide consistent error messages.
#[inline]
pub const fn check_limit(count: usize, limit: usize, what: &'static str) -> Option<&'static str> {
    if count > limit {
        Some(what)
    } else {
        None
    }
}

/// Compile-time assertions to ensure limits are reasonable
mod assertions {
    use super::platform::*;

    // Ensure params/results fit WebAssembly reasonable bounds
    const _: () = assert!(MAX_FUNCTION_PARAMS >= 16, "At least 16 params required");
    const _: () = assert!(MAX_FUNCTION_RESULTS >= 1, "At least 1 result required");

    // Ensure execution limits are reasonable
    const _: () = assert!(MAX_VALUE_STACK >= 256, "Value stack too small");
    const _: () = assert!(MAX_CALL_STACK >= 32, "Call stack too small");

    // Ensure memory limits don't exceed spec
    const _: () = assert!(
        MAX_MEMORY_PAGES as usize <= super::spec::MAX_WASM_MEMORY_PAGES as usize,
        "Memory pages exceed WebAssembly spec"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_limits_are_defined() {
        // Ensure all limits are accessible and non-zero
        // These are compile-time constant checks for sanity
        assert!(MAX_TYPES > 0);
        assert!(MAX_FUNCTIONS > 0);
        assert!(MAX_FUNCTION_PARAMS > 0);
        assert!(MAX_FUNCTION_RESULTS > 0);
        assert!(MAX_FUNCTION_LOCALS > 0);
    }

    #[test]
    fn test_check_limit() {
        assert!(check_limit(10, 100, "test").is_none());
        assert!(check_limit(100, 100, "test").is_none());
        assert!(check_limit(101, 100, "test").is_some());
    }

    #[test]
    fn test_spec_constants() {
        assert_eq!(spec::WASM_PAGE_SIZE, 65536);
        assert_eq!(spec::MAX_WASM_MEMORY_PAGES, 65536);
    }
}
