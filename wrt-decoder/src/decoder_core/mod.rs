/// Core WebAssembly module handling
///
/// This module re-exports functionality for working with core WebAssembly modules.
// Re-export name section handling
pub use crate::name_section;

// Define module submodules
pub mod decode;
pub mod encode;
pub mod validate;

// Re-export decode functionality
pub use crate::module::decode_module_with_binary as decode_module;

// Re-export encode functionality
pub use crate::module::encode_module;

// Re-export validation functionality
pub use validate::validate_module;
pub use validate::validate_module_with_config;
pub use validate::ValidationConfig;

/// Configuration types for the decoder
pub mod config {
    use crate::prelude::*;

    /// Parser configuration for WebAssembly module parsing
    #[derive(Debug, Clone)]
    pub struct ParserConfig {
        /// Whether to validate the module during parsing
        pub validate: bool,
        /// Maximum nesting level for blocks
        pub max_nesting_level: u32,
        /// Whether to track the function count
        pub track_function_count: bool,
    }

    impl Default for ParserConfig {
        fn default() -> Self {
            Self {
                validate: true,
                max_nesting_level: 100,
                track_function_count: true,
            }
        }
    }

    /// Configuration for validation
    #[derive(Debug, Clone)]
    pub struct ValidationConfig {
        /// Maximum number of locals in a function
        pub max_locals: u32,
        /// Maximum number of functions in a module
        pub max_functions: u32,
        /// Maximum number of imports in a module
        pub max_imports: u32,
        /// Maximum number of exports in a module
        pub max_exports: u32,
        /// Maximum memory size in pages (64KiB each)
        pub max_memory_pages: u32,
        /// Maximum number of elements in a table
        pub max_table_elements: u32,
        /// Maximum number of globals
        pub max_globals: u32,
    }

    impl Default for ValidationConfig {
        fn default() -> Self {
            Self {
                max_locals: 50000,
                max_functions: 10000,
                max_imports: 1000,
                max_exports: 1000,
                max_memory_pages: 65536, // 4GiB
                max_table_elements: 100000,
                max_globals: 1000,
            }
        }
    }
}
