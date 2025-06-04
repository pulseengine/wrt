// WRT - wrt-foundation
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Prelude module for wrt-foundation
//!
//! This module provides a unified set of imports for both std and `no_std`
//! environments. It re-exports commonly used types and traits to ensure
//! consistency across all crates in the WRT project and simplify imports in
//! individual modules.

// Core imports for both std and no_std environments
// Re-export from alloc when no_std but alloc is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet, BTreeMap as HashMap},
    format,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};
// Consumers must explicitly use core::* or bounded types.

// Explicitly re-export common core traits and types
pub use core::any::Any;
pub use core::{
    clone::Clone,
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    convert::{TryFrom, TryInto},
    default::Default,
    fmt::{self, Debug, Display, Write},
    hash::Hash,
    marker::{Copy, PhantomData, Sized},
    mem,
    ops::{Deref, DerefMut},
    slice, str,
};
// Re-export from std when the std feature is enabled
// Only include these imports when std feature is enabled
#[cfg(feature = "std")]
pub use std::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet, HashSet, BTreeMap as HashMap},
    format,
    string::{String, ToString},
    sync::{Arc, Mutex, RwLock},
    vec,
    vec::Vec,
};

#[cfg(feature = "use-hashbrown")]
pub use hashbrown::HashMap as BHashMap;
// If only no_std (and not alloc) is active, common collections like Vec, String, Box, HashMap,
// HashSet, Arc are NOT exported by this prelude. Users should use bounded types or core types
// directly.

// Re-export from wrt_error
pub use wrt_error::prelude::*;
pub use wrt_error::{codes, kinds, Error, ErrorCategory, Result};

// Feature-gated re-exports that can't be included in the main use block
#[cfg(feature = "alloc")]
pub use crate::component_builder::{
    ComponentTypeBuilder, ExportBuilder, ImportBuilder, NamespaceBuilder,
};
// Re-export from wrt_sync, only if the feature is active
// #[cfg(feature = "wrt-sync")] // Or a more specific feature if wrt-sync is always a dep

// Re-export platform-specific memory builders if the feature is enabled
#[cfg(feature = "platform-memory")]
pub use crate::memory_builder::{LinearMemoryBuilder, PalMemoryProviderBuilder};
// When neither std nor alloc is available, we provide a pure no_std SimpleHashMap
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub use crate::no_std_hashmap::SimpleHashMap;
// Re-export from this crate
pub use crate::{
    // Atomic memory operations
    atomic_memory::{AtomicMemoryExt, AtomicMemoryOps},
    // Bounded collections
    bounded::{BoundedStack, BoundedString, BoundedVec, CapacityError, WasmName},
    bounded_collections::{BoundedDeque, BoundedMap, BoundedQueue, BoundedSet},
    // Builder patterns
    builder::{
        BoundedBuilder, MemoryBuilder, NoStdProviderBuilder, ResourceBuilder, ResourceItemBuilder,
        StringBuilder,
    },
    // Builtin types
    builtin::BuiltinType,
    // Component model types
    component::{
        ComponentType,
        ExternType,
        InstanceType, // Limits,
        Namespace,
        ResourceType,
    },
    // Resource types
    resource::ResourceOperation,
    // Safe memory types (SafeMemoryHandler, SafeSlice, SafeStack are already here from direct
    // re-exports) Sections (SectionId, SectionType, Section are usually handled by decoder)
    // Import NoStdProvider for no_alloc type aliases
    safe_memory::NoStdProvider,
    // Validation traits (moved to traits module to break circular dependency)
    traits::{
        BoundedCapacity, Checksummed,
        Validatable, /* ValidationContext,
                     * ValidationError and ValidationResult will be re-added when validation
                     * module is restored */
    },
    // Traits
    traits::{FromFormat, ToFormat},
    // Common types (BlockType, FuncType, GlobalType, MemoryType, TableType, ValueType are already
    // here)
    types::{
        // BlockType,
        FuncType,
        GlobalType,
        Limits,
        MemoryType,
        RefType,
        TableType,
        ValueType,
    },
    // Value representations
    values::{FloatBits32, FloatBits64, Value},
    // Verification types
    verification::{Checksum, VerificationLevel},
    // Direct re-exports for convenience (original list)
    // ResourceType, // Already covered by component::* above
    SafeMemoryHandler,
    SafeSlice,
};

// Conversion utilities (only available with alloc/std)
#[cfg(any(feature = "alloc", feature = "std"))]
pub use crate::conversion::{ref_type_to_val_type, val_type_to_ref_type};

// Alloc-dependent re-exports
#[cfg(feature = "alloc")]
pub use crate::{
    // Component builders
    component_value::{ComponentValue, ValType},
    component_value_store::{ComponentValueStore, ValueRef},
    component_value_store_builder::ComponentValueStoreBuilder,
};

// Type aliases for no_std/no_alloc compatibility
/// Maximum number of arguments/results for WebAssembly functions
pub const MAX_WASM_FUNCTION_PARAMS: usize = 128;

/// Type alias for function argument vectors in no_alloc environments
#[cfg(not(feature = "alloc"))]
pub type ArgVec<T> =
    BoundedVec<T, MAX_WASM_FUNCTION_PARAMS, NoStdProvider<{ MAX_WASM_FUNCTION_PARAMS * 16 }>>;

/// Type alias for function argument vectors in alloc environments
#[cfg(feature = "alloc")]
pub type ArgVec<T> = Vec<T>;
