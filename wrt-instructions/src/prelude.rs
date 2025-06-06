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

// no_std alternatives using bounded collections
#[cfg(not(feature = "std"))]
pub use wrt_foundation::bounded::{BoundedVec, BoundedString};

// Type alias for Vec in no_std mode to match wrt-runtime behavior
#[cfg(not(feature = "std"))]
pub type Vec<T> = wrt_foundation::bounded::BoundedVec<T, 256, wrt_foundation::NoStdProvider<1024>>;


// Binary std/no_std choice
#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => {{
        // Binary std/no_std choice
        // Return a static string or use write! to a fixed buffer
        "formatted string not available in no_std without alloc"
    }};
}

// Binary std/no_std choice
#[cfg(not(feature = "std"))]
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
    bounded::{BoundedStack},
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
    memory_ops::{MemoryLoad, MemoryStore, MemoryFill, MemoryCopy, MemoryInit, DataDrop, MemoryOperations, DataSegmentOperations},
    multi_memory::{MultiMemoryLoad, MultiMemoryStore, MultiMemoryBulk, MultiMemoryCrossCopy,
                   MultiMemorySize, MultiMemoryGrow, MultiMemoryValidation, MAX_MEMORIES},
    reference_ops::{RefNull, RefIsNull, RefFunc, RefAsNonNull, ReferenceOp, ReferenceOperations},
    table_ops::TableOp,
    validation::{ValidationContext, ControlFrame, ControlKind, Validate, 
                  validate_arithmetic_op, validate_memory_op, validate_control_op,
                  validate_branch, validate_call, validate_local_op, validate_global_op,
                  validate_comparison_op, validate_conversion_op, validate_ref_op},
    variable_ops::VariableOp,
};

// Re-export constant expression types
pub use crate::const_expr::{ConstExpr, ConstExprContext, ConstExprSequence};
