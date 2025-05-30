// WRT - wrt-component
// Module: WebAssembly Component Model Implementation
// SW-REQ-ID: REQ_019
// SW-REQ-ID: REQ_002
//
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)] // Rule 2

//! Component Model implementation for the WebAssembly Runtime (WRT).
//!
//! This crate provides an implementation of the WebAssembly Component Model,
//! enabling composition and interoperability between WebAssembly modules
//! with shared-nothing linking.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "kani", feature(kani))]
#![warn(clippy::missing_panics_doc)]

// When no_std but alloc is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Panic handler for no_std builds
#[cfg(not(feature = "std"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Note about functionality with different features
// - std: Full functionality
// - no_std + alloc: Full no_std functionality
// - no_std without alloc: Limited to validation and introspection

// Export our prelude module for consistent imports
pub mod prelude;

// Export modules - some are conditionally compiled
pub mod adapter;
pub mod async_canonical;
pub mod async_types;
pub mod async_runtime_bridge;
pub mod builtins;
pub mod canonical;
pub mod cross_component_calls;
pub mod host_integration;
pub mod task_manager;
pub mod generative_types;
pub mod type_bounds;
pub mod wit_integration;
pub mod component_linker;
pub mod component_resolver;
pub mod canonical_realloc;
pub mod canonical_options;
pub mod post_return;
pub mod virtualization;
pub mod thread_spawn;
pub mod thread_spawn_fuel;
pub mod start_function_validation;
pub mod handle_representation;
pub mod cross_component_resource_sharing;
#[cfg(feature = "std")]
pub mod component;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub mod component_no_std;
#[cfg(feature = "std")]
pub mod component_registry;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub mod component_registry_no_std;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub mod component_value_no_std;
pub mod error_format;
pub mod execution_engine;
pub mod memory_layout;
pub mod memory_table_management;
pub mod resource_lifecycle;
pub mod string_encoding;
// No-alloc module for pure no_std environments
pub mod execution;
pub mod export;
pub mod export_map;
pub mod factory;
pub mod host;
pub mod import;
pub mod import_map;
#[cfg(feature = "std")]
pub mod instance;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub mod instance_no_std;
pub mod instantiation;
pub mod modules;
pub mod namespace;
pub mod no_alloc;
pub mod parser;
pub mod parser_integration;
pub mod resources;
pub mod runtime;
pub mod strategies;
pub mod type_conversion;
pub mod types;
pub mod values;

// Include verification module when the kani feature is enabled
#[cfg(feature = "kani")]
pub mod verify;

// Re-export core types and functionality for convenience
pub use builtins::{BuiltinHandler, BuiltinRegistry};
pub use canonical::CanonicalABI;
// Re-export component types based on feature flags
#[cfg(feature = "std")]
pub use component::{Component, ExternValue, FunctionValue, GlobalValue, MemoryValue, TableValue};
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use component_no_std::{
    BuiltinRequirements, Component, ComponentBuilder, ExternValue, FunctionValue, GlobalValue,
    MemoryValue, RuntimeInstance, TableValue, WrtComponentType, WrtComponentTypeBuilder,
    MAX_COMPONENT_EXPORTS, MAX_COMPONENT_IMPORTS, MAX_COMPONENT_INSTANCES,
};
// Re-export common constants
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use component_no_std::{
    MAX_BINARY_SIZE, MAX_COMPONENT_EXPORTS, MAX_COMPONENT_IMPORTS, MAX_COMPONENT_INSTANCES,
    MAX_FUNCTION_REF_SIZE, MAX_LINKED_COMPONENTS, MAX_MEMORY_SIZE, MAX_TABLE_SIZE,
};
// Re-export component registry based on feature flags
#[cfg(feature = "std")]
pub use component_registry::ComponentRegistry;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use component_registry_no_std::ComponentRegistry;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use component_value_no_std::deserialize_component_value_no_std as deserialize_component_value;
// Re-export component value utilities for no_std
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use component_value_no_std::{
    convert_format_to_valtype, convert_valtype_to_format, serialize_component_value_no_std,
};
pub use execution_engine::{ComponentExecutionEngine, ExecutionContext, ExecutionState};
pub use adapter::{
    CoreModuleAdapter, FunctionAdapter, MemoryAdapter, TableAdapter, GlobalAdapter,
    CoreFunctionSignature, CoreValType, AdaptationMode, MemoryLimits, TableLimits
};
pub use async_types::{
    Stream, Future, ErrorContext, StreamHandle, FutureHandle, ErrorContextHandle,
    StreamState, FutureState, AsyncReadResult, Waitable, WaitableSet
};
pub use async_canonical::AsyncCanonicalAbi;
pub use task_manager::{TaskManager, TaskId, Task, TaskState, TaskType, TaskContext};
pub use generative_types::{GenerativeTypeRegistry, GenerativeResourceType, TypeBound, BoundKind};
pub use type_bounds::{TypeBoundsChecker, TypeRelation, RelationKind, RelationConfidence, RelationResult};
// Re-export WIT parser types from wrt-format
pub use wrt_format::wit_parser::{
    WitParser, WitWorld, WitInterface, WitFunction, WitType, WitTypeDef, WitImport, WitExport,
    WitItem, WitParam, WitResult, WitRecord, WitVariant, WitEnum, WitFlags, WitParseError
};
pub use wit_integration::{
    WitComponentBuilder, ComponentInterface, InterfaceFunction, AsyncInterfaceFunction,
    TypedParam, TypedResult, AsyncTypedResult
};
pub use component_linker::{
    ComponentLinker, LinkageDescriptor, Binding, TypeConstraint, CompositeComponent,
    ExternalImport, ExternalExport
};
pub use component_resolver::{
    ComponentResolver, ImportValue as ResolverImportValue, ExportValue as ResolverExportValue, ComponentValue
};
pub use canonical_realloc::{
    ReallocManager, CanonicalOptionsWithRealloc, StringEncoding as ReallocStringEncoding,
    helpers as realloc_helpers
};
pub use canonical_options::{
    CanonicalOptions, CanonicalLiftContext, CanonicalLowerContext, CanonicalOptionsBuilder
};
pub use post_return::{
    PostReturnRegistry, PostReturnFunction, CleanupTask, CleanupTaskType, PostReturnMetrics
};
pub use virtualization::{
    VirtualizationManager, VirtualComponent, VirtualImport, VirtualExport, VirtualSource,
    Capability, CapabilityGrant, IsolationLevel, ResourceLimits, ResourceUsage, MemoryPermissions,
    ExportVisibility, VirtualMemoryRegion, SandboxState, LogLevel, VirtualizationError, VirtualizationResult
};
pub use thread_spawn::{
    ComponentThreadManager, ThreadSpawnBuiltins, ThreadHandle, ThreadConfiguration, ThreadSpawnRequest,
    ThreadResult, ThreadId, ThreadSpawnError, ThreadSpawnResult, create_default_thread_config,
    create_thread_config_with_stack_size, create_thread_config_with_priority
};
pub use thread_spawn_fuel::{
    FuelTrackedThreadManager, FuelTrackedThreadContext, FuelThreadConfiguration, ThreadFuelStatus,
    FuelTrackedThreadResult, GlobalFuelStatus, FuelAwareExecution, create_fuel_thread_config,
    create_unlimited_fuel_thread_config
};
pub use start_function_validation::{
    StartFunctionValidator, StartFunctionDescriptor, StartFunctionParam, StartFunctionValidation,
    StartFunctionExecutionResult, ValidationLevel, ValidationState, ValidationSummary,
    SideEffect, SideEffectType, SideEffectSeverity, StartFunctionError, StartFunctionResult,
    create_start_function_descriptor, create_start_function_param
};
pub use handle_representation::{
    HandleRepresentationManager, HandleRepresentation, AccessRights, HandleMetadata,
    HandleOperation, HandleAccessPolicy, TypedHandle, HandleRepresentationError,
    HandleRepresentationResult, create_access_rights
};
pub use cross_component_resource_sharing::{
    CrossComponentResourceSharingManager, SharingAgreement, TransferPolicy, SharingLifetime,
    SharingMetadata, SharingRestriction, SharedResource, ResourceTransferRequest, TransferType,
    SharingPolicy, PolicyScope, PolicyRule, AuditEntry, AuditAction, SharingStatistics,
    ResourceSharingError, ResourceSharingResult, create_basic_sharing_policy, create_component_pair_policy
};
pub use instantiation::{
    InstantiationContext, ImportValues, ImportValue, FunctionImport, InstanceImport, 
    ExportValue, FunctionExport, ResolvedImport, ResolvedExport
};
pub use parser_integration::{
    ComponentLoader, ParsedComponent, ParsedImport, ParsedExport, ValidationLevel,
    ImportKind, ExportKind, CanonicalOptions, StringEncoding
};
pub use memory_table_management::{
    ComponentMemoryManager, ComponentTableManager, ComponentMemory, ComponentTable,
    MemoryLimits, TableLimits, MemoryPermissions, SharingMode, TableElement
};
pub use cross_component_calls::{
    CrossComponentCallManager, CallTarget, CallPermissions, ResourceTransferPolicy,
    CrossCallResult, CallStatistics
};
pub use host_integration::{
    HostIntegrationManager, HostFunctionRegistry, HostFunctionPermissions, EventHandler,
    EventType, ComponentEvent, HostResourceManager, HostResource, HostResourceType,
    SecurityPolicy
};
pub use export::Export;
pub use factory::ComponentFactory;
pub use host::Host;
pub use import::{Import, ImportType};
#[cfg(feature = "std")]
pub use instance::InstanceValue;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use instance_no_std::{InstanceCollection, InstanceValue, InstanceValueBuilder};
pub use namespace::Namespace;
pub use parser::get_required_builtins;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use resources::{
    BoundedBufferPool, MemoryStrategy, Resource, ResourceArena, ResourceManager,
    ResourceOperationNoStd, ResourceStrategyNoStd, ResourceTable, VerificationLevel,
};
// Re-export resource types based on feature flags
#[cfg(feature = "std")]
pub use resources::{
    BufferPool, MemoryStrategy, Resource, ResourceArena, ResourceManager, ResourceTable,
    VerificationLevel,
};
pub use strategies::memory::{
    BoundedCopyStrategy, FullIsolationStrategy, MemoryOptimizationStrategy, ZeroCopyStrategy,
};
pub use type_conversion::{
    common_to_format_val_type, core_value_to_types_componentvalue, extern_type_to_func_type,
    format_constvalue_to_types_componentvalue, format_to_common_val_type,
    format_to_types_extern_type, format_val_type_to_value_type, format_valtype_to_types_valtype,
    types_componentvalue_to_core_value, types_componentvalue_to_format_constvalue,
    types_to_format_extern_type, types_valtype_to_format_valtype, value_type_to_format_val_type,
    value_type_to_types_valtype, FormatComponentType, FormatInstanceType, IntoFormatComponentType,
    IntoFormatInstanceType, IntoFormatType, IntoRuntimeComponentType, IntoRuntimeInstanceType,
    IntoRuntimeType, RuntimeComponentType, RuntimeInstanceType,
};
pub use types::ComponentInstance;
// Re-export value functions conditionally
#[cfg(feature = "std")]
pub use values::{component_to_core_value, core_to_component_value, deserialize_component_value};
pub use wrt_error::{codes, Error, ErrorCategory, Result};
pub use wrt_foundation::{
    builtin::BuiltinType, component::ComponentType, types::ValueType, values::Value,
};
pub use wrt_host::CallbackRegistry;

/// Debug logging macro - conditionally compiled
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(all(feature = "debug-log", feature = "std"))]
        {
            println!($($arg)*);
        }
        #[cfg(not(all(feature = "debug-log", feature = "std")))]
        {
            // Do nothing when debug logging is disabled or in no_std
        }
    };
}
