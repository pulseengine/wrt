//! Prelude module for wrt
//!
//! This module provides a unified set of imports for both std and no_std environments.
//! It re-exports commonly used types and traits to ensure consistency across all crates
//! in the WRT project and simplify imports in individual modules.

// Core imports for both std and no_std environments
pub use core::{
    any::Any,
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    convert::{TryFrom, TryInto},
    fmt,
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

// Sync primitives from wrt-sync for both std and no_std
// Import the WrtMutex, WrtMutexGuard from wrt_sync
pub use wrt_sync::{
    WrtMutex as Mutex, WrtMutexGuard as MutexGuard, WrtRwLock as RwLock,
    WrtRwLockReadGuard as RwLockReadGuard, WrtRwLockWriteGuard as RwLockWriteGuard,
};

// Re-export from wrt-error with clear prefixes
pub use wrt_error::{kinds as error_kinds, Error as WrtError, ErrorCategory, Result as WrtResult};

// Re-export from wrt-types with clear prefixes
pub use wrt_types::{
    bounded::{BoundedHashMap, BoundedStack, BoundedVec, CapacityError},
    builtin::BuiltinType,
    component::{
        ComponentType as TypesComponentType, ExternType as TypesExternType,
        GlobalType as TypesGlobalType, InstanceType as TypesInstanceType,
        MemoryType as TypesMemoryType, TableType as TypesTableType,
    },
    component_value::ComponentValue,
    conversion::{binary_to_val_type, val_type_to_binary},
    operations::{OperationSummary, OperationTracking, OperationType},
    // ResourceId and ResourceMap are in our local resource module
    safe_memory::{
        MemoryProvider, MemorySafety, MemoryStats, SafeMemoryHandler, SafeSlice, SafeStack,
    },
    sections::{Section as TypesSection, SectionId, SectionType},
    traits::{FromFormat, ToFormat},
    // When you actually need the TypesXXX prefix for disambiguation
    types::{
        BlockType as TypesBlockType, FuncType as TypesFuncType, Limits as TypesLimits, RefType,
        ValueType as TypesValueType,
    },
    validation::{BoundedCapacity, Checksummed, Validatable as TypesValidatable},
    values::{v128, Value as TypesValue, V128},
    verification::{Checksum, VerificationLevel},
    // Re-export types using aliases only - direct imports should use these names
    BlockType,
    ExternType,
    FuncType,
    GlobalType,
    Limits,
    MemoryType,
    TableType,
    ValueType,
};

// Re-export from wrt-format with clear prefixes
pub use wrt_format::{
    binary,
    component::{Component as FormatComponent, ComponentType as FormatComponentType},
    compression::{rle_decode, rle_encode, CompressionType},
    error::{parse_error, runtime_error, type_error, validation_error},
    module::Module as FormatModule,
    section::{
        ComponentSectionHeader, ComponentSectionType, CustomSection as FormatCustomSection,
        Section as FormatSection, SectionId as FormatSectionId,
    },
    types::{FormatBlockType, Limits as FormatLimits},
    validation::Validatable as FormatValidatable,
    FuncType as FormatFuncType, RefType as FormatRefType, ValueType as FormatValueType,
};

// Re-export from wrt-instructions with clear prefixes
pub use wrt_instructions::{
    arithmetic_ops::{ArithmeticContext, ArithmeticOp},
    comparison_ops::ComparisonOp,
    control_ops::{
        Block as InstructionsBlock, BranchTarget, ControlBlockType, ControlContext, ControlOp,
    },
    conversion_ops::ConversionOp,
    instruction_traits::PureInstruction,
    memory_ops::{MemoryLoad, MemoryStore},
    table_ops::{RefValue, TableContext, TableOp},
    variable_ops::VariableOp,
    Error as InstructionsError, Result as InstructionsResult,
};

// Re-export from wrt-decoder with clear prefixes
pub use wrt_decoder::{
    component,
    conversion::{
        byte_to_value_type, component_limits_to_format_limits, convert_to_wrt_error,
        format_limits_to_component_limits, format_limits_to_types_limits,
        format_value_type_to_value_type, section_code_to_section_type,
        section_type_to_section_code, types_limits_to_format_limits,
        value_type_to_format_value_type,
    },
    decoder_core, from_binary as decode_binary,
    module::{
        CodeSection, Data as DecoderData, DataMode, Element as DecoderElement,
        Export as DecoderExport, Module as DecoderModule,
    },
    parse as parse_module,
    parser::{Parser, Payload},
    section_error::SectionError,
    section_reader::SectionReader,
    CustomSection as DecoderCustomSection, Error as DecoderError, Result as DecoderResult,
};

// Re-export from wrt-runtime with clear prefixes
pub use wrt_runtime::{
    component_traits::{ComponentInstance, ComponentRuntime, HostFunction},
    func::FuncType as RuntimeFuncType,
    global::Global as RuntimeGlobal,
    memory::Memory as RuntimeMemory,
    table::Table as RuntimeTable,
    types::{
        GlobalType as RuntimeGlobalType, MemoryType as RuntimeMemoryType,
        TableType as RuntimeTableType,
    },
};

// Only selectively re-export from this crate to avoid conflicts
pub use crate::{
    behavior::{
        ControlFlow, ControlFlowBehavior, EngineBehavior, FrameBehavior, InstructionExecutor,
        Label, ModuleBehavior, StackBehavior,
    },
    execution::ExecutionStats,
    instructions::instruction_type::Instruction as InstructionType,
    module::{
        CustomSection, Data, Element, ExportKind, Function, Import, Module, OtherExport, TableAddr,
    },
    stackless::StacklessEngine,
    stackless_frame::StacklessFrame,
};

// Already re-exported above, this line is redundant
// pub use crate::instructions::instruction_type::Instruction as InstructionType;

// Use this function to convert between error types
pub use crate::error::convert_instructions_error;
