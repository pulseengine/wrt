// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Prelude module for wrt-instructions
//!
//! This module provides a unified set of imports for both std and no_std
//! environments. It re-exports commonly used types and traits to ensure
//! consistency across all crates in the WRT project and simplify imports in
//! individual modules.

// Core imports for both std and no_std environments
pub use core::{
    any::Any,
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    convert::{TryFrom, TryInto},
    fmt,
    fmt::{Debug, Display},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    slice, str,
};

// Re-export from std when the std feature is enabled
#[cfg(feature = "std")]
pub use std::{
    boxed::Box,
    collections::{HashMap, HashSet},
    format,
    string::{String, ToString},
    sync::Arc,
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

// For no_std without alloc, use bounded collections
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub use wrt_foundation::bounded::{BoundedVec as Vec};

// Define format! macro for no_std without alloc
#[cfg(not(any(feature = "std", feature = "alloc")))]
#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => {{
        // In no_std without alloc, we can't allocate strings
        // Return a static string or use write! to a fixed buffer
        "formatted string not available in no_std without alloc"
    }};
}

// Define vec! macro for no_std without alloc
#[cfg(not(any(feature = "std", feature = "alloc")))]
#[macro_export]
macro_rules! vec {
    () => {
        {
            $crate::types::InstructionVec::new(wrt_foundation::NoStdProvider::<1024>::default())
                .unwrap_or_else(|_| panic!("Failed to create BoundedVec"))
        }
    };
    ($($x:expr),+ $(,)?) => {
        {
            let provider = wrt_foundation::NoStdProvider::<1024>::default();
            let mut temp_vec = $crate::types::InstructionVec::new(provider)
                .unwrap_or_else(|_| panic!("Failed to create BoundedVec"));
            $(
                temp_vec.push($x).unwrap_or_else(|_| panic!("Failed to push to BoundedVec"));
            )*
            temp_vec
        }
    };
}

// Re-export from wrt-error
pub use wrt_error::{codes, kinds, Error, ErrorCategory, Result};
// Re-export from wrt-foundation
pub use wrt_foundation::{
    bounded::{BoundedStack, BoundedVec},
    // SafeMemory types
    safe_memory::{NoStdMemoryProvider, SafeMemoryHandler, SafeSlice, SafeStack},
    // Common types
    types::{BlockType, FuncType, GlobalType, MemoryType, RefType, TableType, ValueType},
    values::{Value, FloatBits32, FloatBits64},
    // Verification types
    verification::VerificationLevel,
    // Traits
    traits::BoundedCapacity,
    // Result type
    Result as TypesResult,
};
// Import synchronization primitives for both std and no_std
pub use wrt_sync::{
    WrtMutex as Mutex, WrtMutexGuard as MutexGuard, WrtRwLock as RwLock,
    WrtRwLockReadGuard as RwLockReadGuard, WrtRwLockWriteGuard as RwLockWriteGuard,
};

// Re-export instruction specific types
pub use crate::{
    arithmetic_ops::ArithmeticOp,
    comparison_ops::ComparisonOp,
    control_ops::ControlOp,
    conversion_ops::ConversionOp,
    instruction_traits::PureInstruction,
    memory_ops::{MemoryLoad, MemoryStore},
    table_ops::TableOp,
    variable_ops::VariableOp,
};
