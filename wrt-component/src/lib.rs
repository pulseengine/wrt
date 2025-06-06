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

// Binary std/no_std choice
#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

// Note: Panic handler should be defined by the final binary, not library crates

// Note about functionality with different features
// - std: Full functionality
// Binary std/no_std choice
// Binary std/no_std choice

// Export our prelude module for consistent imports
pub mod prelude;

// Temporary stubs for independent development (Agent C)
pub mod foundation_stubs;
pub mod platform_stubs;
pub mod runtime_stubs;

// Agent C deliverables - Component Model & Integration
pub mod platform_component;
pub mod bounded_resource_management;

// Unified execution agent - consolidates all execution capabilities
pub mod unified_execution_agent;
pub mod unified_execution_agent_stubs;

// Agent registry for managing execution agents
pub mod agent_registry;

// Export modules - organized in subdirectories
pub mod adapter;
pub mod async_;
pub mod canonical_abi;
pub mod components;
pub mod threading;
pub mod borrowed_handles;
pub mod builtins;
pub mod streaming_canonical;
#[cfg(test)]
pub mod canonical_abi_tests;
pub mod component_value_no_std;
pub mod cross_component_calls;
pub mod cross_component_resource_sharing;
pub mod component_communication;
pub mod call_context;
pub mod cross_component_communication;
pub mod error_format;
pub mod error_context_builtins;
pub mod fixed_length_lists;
pub mod execution_engine;
pub mod generative_types;
pub mod handle_representation;
pub mod host_integration;
pub mod memory_layout;
pub mod memory_table_management;
pub mod post_return;
#[cfg(test)]
pub mod resource_management_tests;
pub mod start_function_validation;
pub mod string_encoding;
pub mod type_bounds;
pub mod virtualization;
pub mod wit_integration;
// Enhanced WIT component integration for lowering/lifting
#[cfg(feature = "std")]
pub mod wit_component_integration;
// Binary std/no_std choice
pub mod execution;
pub mod export;
pub mod export_map;
pub mod factory;
pub mod host;
pub mod import;
pub mod import_map;
#[cfg(feature = "std")]
pub mod instance;
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
// Re-export component instantiation and linking
pub use component_instantiation::{
    create_component_export, create_component_import, create_function_signature, ComponentExport,
    ComponentFunction, ComponentImport, ComponentInstance, ComponentMemory, ExportType,
    FunctionHandle, FunctionImplementation, FunctionSignature, ImportType, InstanceConfig,
    InstanceId, InstanceMetadata, InstanceState, MemoryConfig, ResolvedImport,
};
pub use component_linker::{
    CircularDependencyMode, ComponentDefinition, ComponentId, ComponentLinker, ComponentMetadata,
    GraphEdge, GraphNode, LinkGraph, LinkerConfig, LinkingStats,
};
// Re-export component types based on feature flags
#[cfg(feature = "std")]
pub use component::{Component, ExternValue, FunctionValue, GlobalValue, MemoryValue, TableValue};
pub use component_no_std::{
    BuiltinRequirements, Component, ComponentBuilder, ExternValue, FunctionValue, GlobalValue,
    MemoryValue, RuntimeInstance, TableValue, WrtComponentType, WrtComponentTypeBuilder,
    MAX_COMPONENT_EXPORTS, MAX_COMPONENT_IMPORTS, MAX_COMPONENT_INSTANCES,
};
// Re-export common constants
pub use component_no_std::{
    MAX_BINARY_SIZE, MAX_COMPONENT_EXPORTS, MAX_COMPONENT_IMPORTS, MAX_COMPONENT_INSTANCES,
    MAX_FUNCTION_REF_SIZE, MAX_LINKED_COMPONENTS, MAX_MEMORY_SIZE, MAX_TABLE_SIZE,
};
// Re-export component registry based on feature flags
#[cfg(feature = "std")]
pub use component_registry::ComponentRegistry;
pub use component_registry_no_std::ComponentRegistry;
pub use component_value_no_std::deserialize_component_value_no_std as deserialize_component_value;
// Re-export component value utilities for no_std
pub use adapter::{
    AdaptationMode, CoreFunctionSignature, CoreModuleAdapter, CoreValType, FunctionAdapter,
    GlobalAdapter, MemoryAdapter, MemoryLimits, TableAdapter, TableLimits,
};
pub use async_canonical::{
    AsyncCanonicalAbi, AsyncLiftResult, AsyncLowerResult, AsyncOperation, AsyncOperationState,
    AsyncOperationType,
};
pub use async_runtime::{
    AsyncRuntime, EventHandler, FutureEntry, FutureOperation, ReactorEvent, ReactorEventType,
    RuntimeConfig, RuntimeStats, ScheduledTask, StreamEntry, StreamOperation, TaskExecutionResult,
    TaskFunction, TaskScheduler, WaitCondition, WaitingTask,
};
pub use async_execution_engine::{
    AsyncExecution, AsyncExecutionEngine, AsyncExecutionState, AsyncExecutionOperation, CallFrame,
    ExecutionContext, ExecutionId, ExecutionResult, ExecutionStats as AsyncExecutionStats,
    FrameAsyncState, MemoryPermissions, MemoryRegion, MemoryViews, StepResult, WaitSet,
};
pub use streaming_canonical::{
    BackpressureConfig, BackpressureState, StreamDirection, StreamStats, StreamingCanonicalAbi,
    StreamingContext, StreamingLiftResult, StreamingLowerResult, StreamingResult,
};
pub use resource_lifecycle_management::{
    ComponentId, DropHandler, DropHandlerId, DropHandlerFunction, DropResult, GarbageCollectionState,
    GcResult, LifecyclePolicies, LifecycleStats, ResourceCreateRequest, ResourceEntry, ResourceId,
    ResourceLifecycleManager, ResourceMetadata, ResourceState, ResourceType,
};
pub use resource_representation::{
    ResourceRepresentationManager, ResourceRepresentation, RepresentationValue, ResourceEntry as ResourceRepresentationEntry,
    ResourceMetadata as ResourceRepresentationMetadata, RepresentationStats, FileHandleRepresentation,
    MemoryBufferRepresentation, NetworkConnectionRepresentation, NetworkConnection, ConnectionState,
    FileHandle, MemoryBuffer, NetworkHandle, canon_resource_rep, canon_resource_new, canon_resource_drop,
};
pub use borrowed_handles::{
    BorrowHandle, BorrowId, BorrowValidation, HandleConversionError, HandleLifetimeTracker,
    LifetimeScope, LifetimeStats, OwnHandle, OwnedHandleEntry, BorrowedHandleEntry,
    LifetimeScopeEntry, with_lifetime_scope,
};
pub use async_types::{
    AsyncReadResult, ErrorContext, ErrorContextHandle, Future, FutureHandle, FutureState, Stream,
    StreamHandle, StreamState, Waitable, WaitableSet,
};
pub use async_context_builtins::{
    AsyncContext, AsyncContextManager, AsyncContextScope, ContextKey, ContextValue,
    canonical_builtins as async_context_canonical_builtins,
};
pub use component_value_no_std::{
    convert_format_to_valtype, convert_valtype_to_format, serialize_component_value_no_std,
};
// Legacy execution engines (deprecated - use UnifiedExecutionAgent instead)
pub use execution_engine::{ComponentExecutionEngine, ExecutionContext, ExecutionState};

// Unified execution agent - recommended for new development
pub use unified_execution_agent::{
    UnifiedExecutionAgent, AgentConfiguration, CoreExecutionState, UnifiedExecutionState, 
    ExecutionMode, HybridModeFlags, UnifiedExecutionStatistics, UnifiedCallFrame,
};

// Agent registry for managing execution agents
pub use agent_registry::{
    AgentRegistry, AgentId, AgentCreationOptions, PreferredAgentType, AgentInfo, AgentType,
    AgentMigrationStatus, RegistryStatistics, MigrationStatus, MigrationWarning, WarningType,
};
pub use generative_types::{BoundKind, GenerativeResourceType, GenerativeTypeRegistry, TypeBound};
pub use task_manager::{Task, TaskContext, TaskId, TaskManager, TaskState, TaskType};
pub use task_cancellation::{
    CancellationHandler, CancellationHandlerFn, CancellationScope, CancellationToken,
    CompletionHandler, CompletionHandlerFn, HandlerId, ScopeId, SubtaskEntry, SubtaskManager,
    SubtaskResult, SubtaskState, SubtaskStats, with_cancellation_scope,
};
pub use task_builtins::{
    Task as TaskBuiltinTask, TaskBuiltins, TaskId as TaskBuiltinId, TaskRegistry, TaskReturn,
    TaskStatus, task_helpers,
};
pub use waitable_set_builtins::{
    WaitableSetBuiltins, WaitableSetId, WaitableSetImpl, WaitableSetRegistry, WaitResult,
    WaitableEntry, WaitableId, waitable_set_helpers,
};
pub use error_context_builtins::{
    ErrorContextBuiltins, ErrorContextId, ErrorContextImpl, ErrorContextRegistry, ErrorSeverity,
    StackFrame, error_context_helpers,
};
pub use advanced_threading_builtins::{
    AdvancedThreadingBuiltins, AdvancedThreadId, AdvancedThread, AdvancedThreadRegistry,
    AdvancedThreadState, FunctionReference, IndirectCall, ThreadLocalEntry,
    advanced_threading_helpers,
};
pub use fixed_length_lists::{
    FixedLengthList, FixedLengthListType, FixedLengthListTypeRegistry,
    component_integration::{ExtendedValueType}, fixed_list_utils,
};
pub use type_bounds::{
    RelationConfidence, RelationKind, RelationResult, TypeBoundsChecker, TypeRelation,
};
// Re-export WIT parser types from wrt-format
pub use canonical_options::{
    CanonicalLiftContext, CanonicalLowerContext, CanonicalOptions, CanonicalOptionsBuilder,
};
pub use canonical_realloc::{
    helpers as realloc_helpers, CanonicalOptionsWithRealloc, ReallocManager,
    StringEncoding as ReallocStringEncoding,
};
pub use component_linker::{
    Binding, ComponentLinker, CompositeComponent, ExternalExport, ExternalImport,
    LinkageDescriptor, TypeConstraint,
};
pub use component_resolver::{
    ComponentResolver, ComponentValue, ExportValue as ResolverExportValue,
    ImportValue as ResolverImportValue,
};
pub use cross_component_calls::{
    CallPermissions, CallStatistics, CallTarget, CrossCallResult, CrossComponentCallManager,
    ResourceTransferPolicy,
};
pub use cross_component_resource_sharing::{
    create_basic_sharing_policy, create_component_pair_policy, AuditAction, AuditEntry,
    CrossComponentResourceSharingManager, PolicyRule, PolicyScope, ResourceSharingError,
    ResourceSharingResult, ResourceTransferRequest, SharedResource, SharingAgreement,
    SharingLifetime, SharingMetadata, SharingPolicy, SharingRestriction, SharingStatistics,
    TransferPolicy, TransferType,
};
pub use export::Export;
pub use factory::ComponentFactory;
pub use handle_representation::{
    create_access_rights, AccessRights, HandleAccessPolicy, HandleMetadata, HandleOperation,
    HandleRepresentation, HandleRepresentationError, HandleRepresentationManager,
    HandleRepresentationResult, TypedHandle,
};
pub use host::Host;
pub use host_integration::{
    ComponentEvent, EventHandler, EventType, HostFunctionPermissions, HostFunctionRegistry,
    HostIntegrationManager, HostResource, HostResourceManager, HostResourceType, SecurityPolicy,
};
pub use import::{Import, ImportType};
#[cfg(feature = "std")]
pub use instance::InstanceValue;
pub use instance_no_std::{InstanceCollection, InstanceValue, InstanceValueBuilder};
pub use instantiation::{
    ExportValue, FunctionExport, FunctionImport, ImportValue, ImportValues, InstanceImport,
    InstantiationContext, ResolvedExport, ResolvedImport,
};
pub use memory_table_management::{
    ComponentMemory, ComponentMemoryManager, ComponentTable, ComponentTableManager, MemoryLimits,
    MemoryPermissions, SharingMode, TableElement, TableLimits,
};
pub use namespace::Namespace;
pub use parser::get_required_builtins;
pub use parser_integration::{
    CanonicalOptions, ComponentLoader, ExportKind, ImportKind, ParsedComponent, ParsedExport,
    ParsedImport, StringEncoding, ValidationLevel,
};
pub use post_return::{
    CleanupTask, CleanupTaskType, CleanupData, PostReturnFunction, PostReturnMetrics, 
    PostReturnRegistry, PostReturnContext, helpers as post_return_helpers,
};
pub use resources::{
    BoundedBufferPool, MemoryStrategy, Resource, ResourceArena, ResourceManager,
    ResourceOperationNoStd, ResourceStrategyNoStd, ResourceTable, VerificationLevel,
};
pub use start_function_validation::{
    create_start_function_descriptor, create_start_function_param, SideEffect, SideEffectSeverity,
    SideEffectType, StartFunctionDescriptor, StartFunctionError, StartFunctionExecutionResult,
    StartFunctionParam, StartFunctionResult, StartFunctionValidation, StartFunctionValidator,
    ValidationLevel, ValidationState, ValidationSummary,
};
pub use thread_spawn::{
    create_default_thread_config, create_thread_config_with_priority,
    create_thread_config_with_stack_size, ComponentThreadManager, ThreadConfiguration,
    ThreadHandle, ThreadId, ThreadResult, ThreadSpawnBuiltins, ThreadSpawnError,
    ThreadSpawnRequest, ThreadSpawnResult,
};
pub use thread_spawn_fuel::{
    create_fuel_thread_config, create_unlimited_fuel_thread_config, FuelAwareExecution,
    FuelThreadConfiguration, FuelTrackedThreadContext, FuelTrackedThreadManager,
    FuelTrackedThreadResult, GlobalFuelStatus, ThreadFuelStatus,
};
pub use thread_builtins::{
    ThreadBuiltins, ParallelismInfo, ThreadSpawnConfig, ComponentFunction,
    FunctionSignature, ValueType, ThreadJoinResult, ThreadError,
};
pub use virtualization::{
    Capability, CapabilityGrant, ExportVisibility, IsolationLevel, LogLevel, MemoryPermissions,
    ResourceLimits, ResourceUsage, SandboxState, VirtualComponent, VirtualExport, VirtualImport,
    VirtualMemoryRegion, VirtualSource, VirtualizationError, VirtualizationManager,
    VirtualizationResult,
};
pub use wit_integration::{
    AsyncInterfaceFunction, AsyncTypedResult, ComponentInterface, InterfaceFunction, TypedParam,
    TypedResult, WitComponentBuilder,
};
#[cfg(feature = "std")]
pub use wit_component_integration::{
    ComponentConfig, ComponentLowering, ComponentType, WitComponentContext,
    InterfaceMapping, TypeMapping, FunctionMapping, RecordType, VariantType,
    EnumType, FlagsType, ResourceType, FunctionType, FieldType, CaseType,
};
pub use wrt_format::wit_parser::{
    WitEnum, WitExport, WitFlags, WitFunction, WitImport, WitInterface, WitItem, WitParam,
    WitParseError, WitParser, WitRecord, WitResult, WitType, WitTypeDef, WitVariant, WitWorld,
};
// Re-export resource types based on feature flags
#[cfg(feature = "std")]
pub use resources::{
    BufferPool, MemoryStrategy, Resource, ResourceArena, ResourceManager, ResourceTable,
    VerificationLevel,
};
// Re-export resource management system
pub use resource_management::{
    create_resource_data_bytes, create_resource_data_custom, create_resource_data_external,
    create_resource_type, Resource as ComponentResource, ResourceData, ResourceError,
    ResourceHandle, ResourceManager as ComponentResourceManager, ResourceManagerConfig,
    ResourceManagerStats, ResourceOwnership, ResourceState,
    ResourceTable as ComponentResourceTable, ResourceTableStats, ResourceType, ResourceTypeId,
    ResourceTypeMetadata, ResourceValidationLevel, INVALID_HANDLE,
};
// Re-export component communication system
pub use component_communication::{
    CallContext, CallFrame, CallId, CallMetadata, CallRouter, CallRouterConfig, CallStack,
    CallState, CallStatistics, CommunicationError, MemoryContext, MemoryIsolationLevel,
    MemoryProtectionFlags, ParameterBridge, ParameterCopyStrategy, ResourceBridge,
    ResourceTransfer, ResourceTransferPolicy, ResourceTransferType,
};
pub use call_context::{
    CallContextConfig, CallContextManager, CallMetrics, CallValidator, ManagedCallContext,
    MarshalingConfig as CallMarshalingConfig, MarshalingMetadata, MarshalingState,
    ParameterMarshaler, PerformanceMonitor, ResourceCoordinator, ResourceState as CallResourceState,
    ValidationResults, ValidationStatus,
};
// Re-export cross-component communication integration
pub use cross_component_communication::{
    ComponentCommunicationConfig, ComponentCommunicationStrategy, ComponentSecurityPolicy,
    CommunicationStats, create_communication_strategy, create_communication_strategy_with_config,
    create_default_security_policy, create_permissive_security_policy,
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

// Re-export Agent C deliverables
pub use platform_component::{
    AllocationType, ComponentInstance, ComponentMemoryBudget, ComponentMetadata, ComponentRequirements,
    ComponentResultExt, ComponentState, ExportKind, ExportRequirement, ImportKind, ImportRequirement,
    MemoryAllocation, PlatformComponentRuntime, RuntimeStatistics,
};
pub use bounded_resource_management::{
    BoundedResourceManager, BoundedResourceTable, Resource, ResourceDestructor, ResourceHandle,
    ResourceId, ResourceLimits, ResourceManagerStatistics, ResourceOwnership, ResourceSharingEntry,
    ResourceState, ResourceType, ResourceTypeId,
};

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

// Panic handler disabled to avoid conflicts with other crates
// // Provide a panic handler only when wrt-component is being tested in isolation
// #[cfg(all(not(feature = "std"), not(test), not(feature = "disable-panic-handler")))]
// #[panic_handler]
// fn panic(_info: &core::panic::PanicInfo) -> ! {
//     loop {}
// }
