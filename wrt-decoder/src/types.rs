//! Type aliases for no_std compatibility in wrt-decoder

use wrt_foundation::{BoundedVec, NoStdProvider};

use crate::prelude::*;

// Module parsing limits based on WebAssembly spec
pub const MAX_MODULE_TYPES: usize = 1024;
pub const MAX_MODULE_FUNCTIONS: usize = 1024;
pub const MAX_MODULE_IMPORTS: usize = 512;
pub const MAX_MODULE_EXPORTS: usize = 512;
pub const MAX_MODULE_GLOBALS: usize = 512;
pub const MAX_MODULE_TABLES: usize = 64;
pub const MAX_MODULE_MEMORIES: usize = 64;
pub const MAX_MODULE_ELEMENTS: usize = 512;
pub const MAX_MODULE_DATA: usize = 512;
pub const MAX_MODULE_CUSTOMS: usize = 64;

// Instruction parsing
pub const MAX_INSTRUCTIONS: usize = 65536;
pub const MAX_LOCALS: usize = 50000; // WebAssembly spec limit
pub const MAX_BR_TABLE_TARGETS: usize = 1024;

// Name section limits
pub const MAX_NAME_ENTRIES: usize = 1024;
pub const MAX_LOCAL_NAMES: usize = 256;
pub const MAX_NAME_LENGTH: usize = 256;

// Producer section limits
pub const MAX_PRODUCERS: usize = 64;
pub const MAX_PRODUCER_FIELDS: usize = 16;
pub const MAX_PRODUCER_VALUES: usize = 16;

// CFI metadata limits
pub const MAX_CFI_FEATURES: usize = 32;
pub const MAX_CFI_REQUIREMENTS: usize = 64;
pub const MAX_INDIRECT_CALLS: usize = 1024;
pub const MAX_RETURN_SITES: usize = 1024;
pub const MAX_LANDING_PADS: usize = 512;
pub const MAX_CONTROL_FLOW: usize = 2048;

// Type aliases for Vec
#[cfg(feature = "alloc")]
pub type TypesVec = Vec<FuncType>;
#[cfg(not(feature = "alloc"))]
pub type TypesVec =
    BoundedVec<FuncType, MAX_MODULE_TYPES, NoStdProvider<{ MAX_MODULE_TYPES * 64 }>>;

#[cfg(feature = "alloc")]
pub type FunctionsVec = Vec<u32>;
#[cfg(not(feature = "alloc"))]
pub type FunctionsVec =
    BoundedVec<u32, MAX_MODULE_FUNCTIONS, NoStdProvider<{ MAX_MODULE_FUNCTIONS * 4 }>>;

#[cfg(feature = "alloc")]
pub type ImportsVec<T> = Vec<T>;
#[cfg(not(feature = "alloc"))]
pub type ImportsVec<T> =
    BoundedVec<T, MAX_MODULE_IMPORTS, NoStdProvider<{ MAX_MODULE_IMPORTS * 128 }>>;

#[cfg(feature = "alloc")]
pub type ExportsVec<T> = Vec<T>;
#[cfg(not(feature = "alloc"))]
pub type ExportsVec<T> =
    BoundedVec<T, MAX_MODULE_EXPORTS, NoStdProvider<{ MAX_MODULE_EXPORTS * 64 }>>;

#[cfg(feature = "alloc")]
pub type GlobalsVec<T> = Vec<T>;
#[cfg(not(feature = "alloc"))]
pub type GlobalsVec<T> =
    BoundedVec<T, MAX_MODULE_GLOBALS, NoStdProvider<{ MAX_MODULE_GLOBALS * 32 }>>;

#[cfg(feature = "alloc")]
pub type TablesVec<T> = Vec<T>;
#[cfg(not(feature = "alloc"))]
pub type TablesVec<T> = BoundedVec<T, MAX_MODULE_TABLES, NoStdProvider<{ MAX_MODULE_TABLES * 32 }>>;

#[cfg(feature = "alloc")]
pub type MemoriesVec<T> = Vec<T>;
#[cfg(not(feature = "alloc"))]
pub type MemoriesVec<T> =
    BoundedVec<T, MAX_MODULE_MEMORIES, NoStdProvider<{ MAX_MODULE_MEMORIES * 32 }>>;

#[cfg(feature = "alloc")]
pub type ElementsVec<T> = Vec<T>;
#[cfg(not(feature = "alloc"))]
pub type ElementsVec<T> =
    BoundedVec<T, MAX_MODULE_ELEMENTS, NoStdProvider<{ MAX_MODULE_ELEMENTS * 128 }>>;

#[cfg(feature = "alloc")]
pub type DataVec<T> = Vec<T>;
#[cfg(not(feature = "alloc"))]
pub type DataVec<T> = BoundedVec<T, MAX_MODULE_DATA, NoStdProvider<{ MAX_MODULE_DATA * 128 }>>;

#[cfg(feature = "alloc")]
pub type CustomSectionsVec<T> = Vec<T>;
#[cfg(not(feature = "alloc"))]
pub type CustomSectionsVec<T> =
    BoundedVec<T, MAX_MODULE_CUSTOMS, NoStdProvider<{ MAX_MODULE_CUSTOMS * 1024 }>>;

// Instruction vectors
#[cfg(feature = "alloc")]
pub type InstructionVec = Vec<crate::instructions::Instruction>;
#[cfg(not(feature = "alloc"))]
pub type InstructionVec = BoundedVec<
    crate::instructions::Instruction,
    MAX_INSTRUCTIONS,
    NoStdProvider<{ MAX_INSTRUCTIONS * 8 }>,
>;

#[cfg(feature = "alloc")]
pub type LocalsVec = Vec<wrt_foundation::types::LocalEntry>;
#[cfg(not(feature = "alloc"))]
pub type LocalsVec =
    BoundedVec<wrt_foundation::types::LocalEntry, MAX_LOCALS, NoStdProvider<{ MAX_LOCALS * 16 }>>;

#[cfg(feature = "alloc")]
pub type BrTableTargetsVec = Vec<u32>;
#[cfg(not(feature = "alloc"))]
pub type BrTableTargetsVec =
    BoundedVec<u32, MAX_BR_TABLE_TARGETS, NoStdProvider<{ MAX_BR_TABLE_TARGETS * 4 }>>;

// Name section types
#[cfg(feature = "alloc")]
pub type NameMapVec = Vec<(u32, String)>;
#[cfg(not(feature = "alloc"))]
pub type NameMapVec = BoundedVec<
    (u32, wrt_foundation::BoundedString<MAX_NAME_LENGTH, NoStdProvider<MAX_NAME_LENGTH>>),
    MAX_NAME_ENTRIES,
    NoStdProvider<{ MAX_NAME_ENTRIES * (4 + MAX_NAME_LENGTH) }>,
>;

#[cfg(feature = "alloc")]
pub type LocalNamesVec = Vec<(u32, Vec<(u32, String)>)>;
#[cfg(not(feature = "alloc"))]
pub type LocalNamesVec = BoundedVec<
    (
        u32,
        BoundedVec<
            (u32, wrt_foundation::BoundedString<MAX_NAME_LENGTH, NoStdProvider<MAX_NAME_LENGTH>>),
            MAX_LOCAL_NAMES,
            NoStdProvider<{ MAX_LOCAL_NAMES * (4 + MAX_NAME_LENGTH) }>,
        >,
    ),
    MAX_NAME_ENTRIES,
    NoStdProvider<{ MAX_NAME_ENTRIES * MAX_LOCAL_NAMES * (4 + MAX_NAME_LENGTH) }>,
>;

// Producer section types
#[cfg(feature = "alloc")]
pub type ProducerFieldVec = Vec<crate::producers_section::ProducerField>;
#[cfg(not(feature = "alloc"))]
pub type ProducerFieldVec = BoundedVec<
    crate::producers_section::ProducerField,
    MAX_PRODUCER_FIELDS,
    NoStdProvider<{ MAX_PRODUCER_FIELDS * 512 }>,
>;

// CFI metadata types
#[cfg(feature = "alloc")]
pub type CfiFeatureVec = Vec<crate::cfi_metadata::CfiFeature>;
#[cfg(not(feature = "alloc"))]
pub type CfiFeatureVec = BoundedVec<
    crate::cfi_metadata::CfiFeature,
    MAX_CFI_FEATURES,
    NoStdProvider<{ MAX_CFI_FEATURES * 32 }>,
>;

#[cfg(feature = "alloc")]
pub type CfiRequirementVec = Vec<crate::cfi_metadata::ValidationRequirement>;
#[cfg(not(feature = "alloc"))]
pub type CfiRequirementVec = BoundedVec<
    crate::cfi_metadata::ValidationRequirement,
    MAX_CFI_REQUIREMENTS,
    NoStdProvider<{ MAX_CFI_REQUIREMENTS * 64 }>,
>;

#[cfg(feature = "alloc")]
pub type IndirectCallVec = Vec<crate::cfi_metadata::IndirectCallSite>;
#[cfg(not(feature = "alloc"))]
pub type IndirectCallVec = BoundedVec<
    crate::cfi_metadata::IndirectCallSite,
    MAX_INDIRECT_CALLS,
    NoStdProvider<{ MAX_INDIRECT_CALLS * 32 }>,
>;

#[cfg(feature = "alloc")]
pub type ReturnSiteVec = Vec<crate::cfi_metadata::ReturnSite>;
#[cfg(not(feature = "alloc"))]
pub type ReturnSiteVec = BoundedVec<
    crate::cfi_metadata::ReturnSite,
    MAX_RETURN_SITES,
    NoStdProvider<{ MAX_RETURN_SITES * 32 }>,
>;

#[cfg(feature = "alloc")]
pub type LandingPadVec = Vec<crate::cfi_metadata::LandingPadRequirement>;
#[cfg(not(feature = "alloc"))]
pub type LandingPadVec = BoundedVec<
    crate::cfi_metadata::LandingPadRequirement,
    MAX_LANDING_PADS,
    NoStdProvider<{ MAX_LANDING_PADS * 64 }>,
>;

#[cfg(feature = "alloc")]
pub type ControlFlowVec = Vec<crate::cfi_metadata::InternalControlFlow>;
#[cfg(not(feature = "alloc"))]
pub type ControlFlowVec = BoundedVec<
    crate::cfi_metadata::InternalControlFlow,
    MAX_CONTROL_FLOW,
    NoStdProvider<{ MAX_CONTROL_FLOW * 64 }>,
>;

// Additional CFI types
#[cfg(feature = "alloc")]
pub type FunctionCfiVec = Vec<crate::cfi_metadata::FunctionCfiInfo>;
#[cfg(not(feature = "alloc"))]
pub type FunctionCfiVec =
    BoundedVec<crate::cfi_metadata::FunctionCfiInfo, 1024, NoStdProvider<{ 1024 * 256 }>>;

#[cfg(feature = "alloc")]
pub type ImportCfiVec = Vec<crate::cfi_metadata::ImportCfiRequirement>;
#[cfg(not(feature = "alloc"))]
pub type ImportCfiVec =
    BoundedVec<crate::cfi_metadata::ImportCfiRequirement, 256, NoStdProvider<{ 256 * 512 }>>;

#[cfg(feature = "alloc")]
pub type ExportCfiVec = Vec<crate::cfi_metadata::ExportCfiRequirement>;
#[cfg(not(feature = "alloc"))]
pub type ExportCfiVec =
    BoundedVec<crate::cfi_metadata::ExportCfiRequirement, 256, NoStdProvider<{ 256 * 256 }>>;

#[cfg(feature = "alloc")]
pub type ValueTypeVec = Vec<wrt_format::types::ValueType>;
#[cfg(not(feature = "alloc"))]
pub type ValueTypeVec = BoundedVec<wrt_format::types::ValueType, 32, NoStdProvider<{ 32 * 4 }>>;

#[cfg(feature = "alloc")]
pub type ValidationRequirementVec = Vec<crate::cfi_metadata::ValidationRequirement>;
#[cfg(not(feature = "alloc"))]
pub type ValidationRequirementVec = BoundedVec<
    crate::cfi_metadata::ValidationRequirement,
    MAX_CFI_REQUIREMENTS,
    NoStdProvider<{ MAX_CFI_REQUIREMENTS * 64 }>,
>;

// Generic byte vector for raw data
#[cfg(feature = "alloc")]
pub type ByteVec = Vec<u8>;
#[cfg(not(feature = "alloc"))]
pub type ByteVec = BoundedVec<u8, 65536, NoStdProvider<65536>>;

// String type
#[cfg(feature = "alloc")]
pub type DecoderString = String;
#[cfg(not(feature = "alloc"))]
pub type DecoderString =
    wrt_foundation::BoundedString<MAX_NAME_LENGTH, NoStdProvider<MAX_NAME_LENGTH>>;
