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
    collections::{BTreeMap, BTreeSet},
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
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    format,
    string::{String, ToString},
    sync::{Arc, Mutex, RwLock},
    vec,
    vec::Vec,
};

#[cfg(feature = "use-hashbrown")]
pub use hashbrown::HashMap;
// If only no_std (and not alloc) is active, common collections like Vec, String, Box, HashMap,
// HashSet, Arc are NOT exported by this prelude. Users should use bounded types or core types
// directly.

// Re-export from wrt_error
pub use wrt_error::prelude::*;
pub use wrt_error::{codes, kinds, Error, ErrorCategory, Result};

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
    // Component builders (alloc-dependent)
    component_builder,
    component_value::{ComponentValue, ValType},
    component_value_store::{ComponentValueStore, ValueRef},
    component_value_store_builder::ComponentValueStoreBuilder,
    // Conversion utilities
    conversion::{ref_type_to_val_type, val_type_to_ref_type},
    // Resource types
    resource::ResourceOperation,
    // Safe memory types (SafeMemoryHandler, SafeSlice, SafeStack are already here from direct
    // re-exports) Sections (SectionId, SectionType, Section are usually handled by decoder)
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
    // Validation traits
    validation::{
        BoundedCapacity,
        Checksummed,
        Validatable, // ValidationContext,
        ValidationError,
        ValidationResult, // ValidOutput
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
