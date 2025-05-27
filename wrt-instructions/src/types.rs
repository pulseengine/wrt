//! Type aliases for no_std compatibility

use crate::prelude::*;
use wrt_foundation::NoStdProvider;

// CFI-specific types
pub const MAX_CFI_TARGETS: usize = 16;
pub const MAX_CFI_REQUIREMENTS: usize = 16;
pub const MAX_CFI_TARGET_TYPES: usize = 8;

#[cfg(feature = "alloc")]
pub type CfiTargetVec = Vec<u32>;

#[cfg(not(feature = "alloc"))]
pub type CfiTargetVec = BoundedVec<u32, MAX_CFI_TARGETS, NoStdProvider<1024>>;

#[cfg(feature = "alloc")]
pub type CfiRequirementVec = Vec<crate::cfi_control_ops::CfiValidationRequirement>;

#[cfg(not(feature = "alloc"))]
pub type CfiRequirementVec = BoundedVec<crate::cfi_control_ops::CfiValidationRequirement, MAX_CFI_REQUIREMENTS, NoStdProvider<1024>>;

#[cfg(feature = "alloc")]
pub type CfiTargetTypeVec = Vec<crate::cfi_control_ops::CfiTargetType>;

#[cfg(not(feature = "alloc"))]
pub type CfiTargetTypeVec = BoundedVec<crate::cfi_control_ops::CfiTargetType, MAX_CFI_TARGET_TYPES, NoStdProvider<1024>>;

// Additional CFI collection types
pub const MAX_SHADOW_STACK: usize = 1024;
pub const MAX_LANDING_PAD_EXPECTATIONS: usize = 64;
pub const MAX_CFI_EXPECTED_VALUES: usize = 16;

#[cfg(feature = "alloc")]
pub type ShadowStackVec = Vec<crate::cfi_control_ops::ShadowStackEntry>;

#[cfg(not(feature = "alloc"))]
pub type ShadowStackVec = BoundedVec<crate::cfi_control_ops::ShadowStackEntry, MAX_SHADOW_STACK, NoStdProvider<{ MAX_SHADOW_STACK * 64 }>>;

#[cfg(feature = "alloc")]
pub type LandingPadExpectationVec = Vec<crate::cfi_control_ops::LandingPadExpectation>;

#[cfg(not(feature = "alloc"))]
pub type LandingPadExpectationVec = BoundedVec<crate::cfi_control_ops::LandingPadExpectation, MAX_LANDING_PAD_EXPECTATIONS, NoStdProvider<{ MAX_LANDING_PAD_EXPECTATIONS * 64 }>>;

#[cfg(feature = "alloc")]
pub type CfiExpectedValueVec = Vec<crate::cfi_control_ops::CfiExpectedValue>;

#[cfg(not(feature = "alloc"))]
pub type CfiExpectedValueVec = BoundedVec<crate::cfi_control_ops::CfiExpectedValue, MAX_CFI_EXPECTED_VALUES, NoStdProvider<{ MAX_CFI_EXPECTED_VALUES * 32 }>>;

// Collection type aliases that work across all configurations
#[cfg(feature = "alloc")]
pub type InstructionVec<T> = Vec<T>;

#[cfg(not(feature = "alloc"))]
pub type InstructionVec<T> = BoundedVec<T, 256, NoStdProvider<{ 256 * 32 }>>;

// Stack type with reasonable size for WASM
pub const MAX_STACK_SIZE: usize = 1024;

#[cfg(feature = "alloc")]
pub type ValueStack = Vec<Value>;

#[cfg(not(feature = "alloc"))]
pub type ValueStack = BoundedStack<Value, MAX_STACK_SIZE, NoStdProvider<{ MAX_STACK_SIZE * 16 }>>;

// Table storage
pub const MAX_TABLES: usize = 16;
pub const MAX_TABLE_SIZE: usize = 65536;

#[cfg(feature = "alloc")]
pub type TableVec = Vec<Vec<RefValue>>;

#[cfg(not(feature = "alloc"))]
pub type TableVec = BoundedVec<BoundedVec<RefValue, MAX_TABLE_SIZE, NoStdProvider<{ MAX_TABLE_SIZE * 16 }>>, MAX_TABLES, NoStdProvider<{ MAX_TABLES * 256 }>>;

// Locals and globals storage
pub const MAX_LOCALS: usize = 1024;
pub const MAX_GLOBALS: usize = 1024;

#[cfg(feature = "alloc")]
pub type LocalsVec = Vec<Value>;

#[cfg(not(feature = "alloc"))]
pub type LocalsVec = BoundedVec<Value, MAX_LOCALS, NoStdProvider<{ MAX_LOCALS * 16 }>>;

#[cfg(feature = "alloc")]
pub type GlobalsVec = Vec<Value>;

#[cfg(not(feature = "alloc"))]
pub type GlobalsVec = BoundedVec<Value, MAX_GLOBALS, NoStdProvider<{ MAX_GLOBALS * 16 }>>;

// Reference value type (for tables)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RefValue {
    /// Function reference
    FuncRef(Option<u32>),
    /// External reference  
    ExternRef(Option<u32>),
}

// Helper to create vectors in both modes
#[cfg(feature = "alloc")]
#[macro_export]
macro_rules! make_vec {
    () => { Vec::new() };
    ($($elem:expr),*) => { vec![$($elem),*] };
}

#[cfg(not(feature = "alloc"))]
#[macro_export]
macro_rules! make_vec {
    () => { BoundedVec::new(NoStdProvider::default()).unwrap() };
    ($($elem:expr),*) => {{
        let mut v = BoundedVec::new(NoStdProvider::default()).unwrap();
        $(v.push($elem).unwrap();)*
        v
    }};
}