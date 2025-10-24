//! Component management and lifecycle
//!
//! This module handles component instantiation, communication, linking,
//! and registry management for the WebAssembly Component Model.

pub mod component;
pub mod component_communication;
pub mod component_instantiation;
pub mod component_linker;
pub mod component_no_std;
pub mod component_registry;
pub mod component_registry_no_std;
pub mod component_resolver;

// Re-export based on feature flags to avoid ambiguous imports
#[cfg(feature = "std")]
pub use component::*;
#[cfg(not(feature = "std"))]
pub use component_no_std::*;

pub use component_communication::{
    CallContext,
    CallFrame,
    CallId,
    CallMetadata,
    CallRouter,
    CallRouterConfig,
    CallStack,
    CallState,
    CallStatistics,
    CommunicationError,
    MemoryContext,
    MemoryIsolationLevel,
    MemoryProtectionFlags,
    MarshalingConfig,
    ParameterBridge,
    ParameterCopyStrategy,
    ResourceBridge,
    ResourceTransfer,
    ResourceTransferPolicy,
    ResourceTransferType,
    create_default_transfer_policy,
    create_memory_context,
    create_parameter_bridge,
};
pub use component_instantiation::{
    ComponentExport,
    ComponentFunction,
    ComponentImport,
    ComponentInstanceImpl,
    ComponentMemory,
    ExportType,
    FunctionHandle,
    FunctionImplementation,
    FunctionSignature,
    ImportType,
    InstanceConfig,
    InstanceId,
    InstanceMetadata,
    InstanceState,
    InstantiationError,
    MemoryConfig,
    MemoryHandle,
    ResolvedImport,
    create_component_export,
    create_component_import,
    create_function_signature,
};
pub use component_linker::{
    CircularDependencyMode,
    ComponentDefinition,
    ComponentId,
    ComponentLinker,
    ComponentMetadata,
    GraphEdge,
    GraphNode,
    LinkGraph,
    LinkerConfig,
    LinkingStats,
};

#[cfg(feature = "std")]
pub use component_registry::*;
#[cfg(not(feature = "std"))]
pub use component_registry_no_std::*;

pub use component_resolver::{
    ComponentResolver,
    ExportResolution,
    ExportValue,
    ImportResolution,
    ImportValue,
    ResolvedExport,
    ResolvedImport as ResolvedImportFromResolver,
};
