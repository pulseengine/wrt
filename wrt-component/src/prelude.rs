//! Prelude module for wrt-component
//!
//! This module provides a unified set of imports for both std and no_std environments.
//! It re-exports commonly used types and traits to ensure consistency across all crates
//! in the WRT project and simplify imports in individual modules.

// Core imports for both std and no_std environments
pub use core::{
    any::Any,
    array,
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    convert::{From, Into, TryFrom, TryInto},
    fmt,
    fmt::{Debug, Display, Write as FmtWrite},
    iter,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    result, slice, str,
    time::Duration,
};

// Re-export from std when the std feature is enabled
#[cfg(feature = "std")]
pub use std::{
    boxed::Box,
    collections::{HashMap, HashSet},
    fmt as std_fmt, format, io,
    string::{String, ToString},
    sync::{Arc, Mutex, RwLock},
    time::Instant,
    vec,
    vec::Vec,
};

// Re-export from alloc when no_std but alloc is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::{
    boxed::Box,
    collections::{BTreeMap as HashMap, BTreeSet as HashSet},
    format,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};

// Import synchronization primitives for no_std
#[cfg(not(feature = "std"))]
pub use wrt_sync::{Mutex, RwLock};

// Re-export from wrt-error
pub use wrt_error::{codes, kinds, Error, ErrorCategory, Result};

// Re-export from wrt-types
pub use wrt_types::{
    bounded::{BoundedStack, BoundedVec},
    // Builtin types
    builtin::BuiltinType,
    component::ComponentType,
    component_value::{ComponentValue, ValType},
    // Resource types
    resource::{ResourceOperation, ResourceType},
    // SafeMemory types
    safe_memory::{BoundedCollection, SafeMemoryHandler, SafeSlice, SafeStack},
    types::{BlockType, FuncType, GlobalType, MemoryType, TableType, ValueType},
    values::Value,
    // Verification types
    verification::VerificationLevel,
    // Common types
    ExternType,
};

// Re-export from wrt-format
pub use wrt_format::component::ValType as FormatValType;

// Re-export from wrt-host
pub use wrt_host::{
    builder::HostBuilder,
    callback::{CallbackRegistry, CallbackType},
    function::{CloneableFn, HostFunctionHandler},
    host::BuiltinHost,
};

// Re-export from wrt-intercept
pub use wrt_intercept::{
    // Builtin interceptors
    builtins::{BeforeBuiltinResult, BuiltinInterceptor, BuiltinSerialization, InterceptContext},
    InterceptionResult,

    // Core interception types
    LinkInterceptor,
    LinkInterceptorStrategy,
    Modification,
};

// Re-export from wrt-decoder
pub use wrt_decoder::{
    component::decode::decode_component, component::decode::Component as DecodedComponent,
    component::parse, component::validation, sections,
};

// Re-export from this crate
pub use crate::{
    // Builtins
    builtins::{BuiltinHandler, BuiltinRegistry},
    // Canonical ABI
    canonical::CanonicalABI,
    // Component model core types
    component::{Component, ExternValue, FunctionValue, GlobalValue, MemoryValue, TableValue},
    component_registry::ComponentRegistry,
    // Execution context
    execution::{TimeBoundedConfig, TimeBoundedContext, TimeBoundedOutcome},
    // Export/Import
    export::Export,
    export_map::{ExportMap, SafeExportMap},
    // Factory and instance
    factory::ComponentFactory,
    // Host and namespace
    host::Host,
    import::Import,
    import_map::{ImportMap, SafeImportMap},
    instance::InstanceValue,
    namespace::Namespace,
    // Resources
    resources::{
        BufferPool, MemoryStrategy, ResourceManager, ResourceOperation as RuntimeResourceOperation,
        ResourceTable,
    },
    // Memory strategies
    strategies::memory::{
        BoundedCopyStrategy, FullIsolationStrategy, MemoryOptimizationStrategy, ZeroCopyStrategy,
    },
    // Type conversion
    type_conversion::{
        format_to_types_extern_type, format_val_type_to_value_type,
        format_valtype_to_types_valtype, types_to_format_extern_type,
        types_valtype_to_format_valtype, value_type_to_format_val_type, IntoFormatType,
        IntoRuntimeType,
    },
    // Types and values
    types::ComponentInstance,
    values::{
        component_to_core_value, core_to_component_value, deserialize_component_value,
        serialize_component_value,
    },
};

// Include debug logging macro
pub use crate::debug_println;
