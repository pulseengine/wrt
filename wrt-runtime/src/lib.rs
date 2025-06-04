// WRT - wrt-runtime
// Module: Core WebAssembly Runtime
// SW-REQ-ID: REQ_001
// SW-REQ-ID: REQ_002
// SW-REQ-ID: REQ_MEM_SAFETY_001
//
// Copyright (c) 2024 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! WebAssembly Runtime (WRT) - Runtime Implementation
//!
//! This crate provides the core runtime types and implementations for
//! WebAssembly, shared between both the core WebAssembly and Component Model
//! implementations.
//!
//! # Safety
//!
//! Most modules forbid unsafe code. Only specific modules that require direct
//! memory access (atomic operations, wait queues) allow unsafe code with
//! documented safety invariants.

#![cfg_attr(not(feature = "std"), no_std)]
// Note: unsafe_code is allowed selectively in specific modules that need it
#![warn(missing_docs)]
#![warn(clippy::missing_panics_doc)]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Panic handler is provided by wrt-platform when needed

// Re-export prelude module publicly
pub use prelude::*;

// Core modules
pub mod atomic_execution;
pub mod atomic_memory_model;
pub mod branch_prediction;
pub mod cfi_engine;
pub mod execution;
pub mod func;
pub mod global;
pub mod interpreter_optimization;
pub mod memory;
pub mod memory_adapter;
pub mod memory_helpers;
pub mod module;
pub mod module_builder;
pub mod module_instance;
pub mod prelude;
pub mod stackless;
pub mod table;
pub mod thread_manager;
pub mod types;
pub mod wait_queue;
pub mod wit_debugger_integration;

// Re-export commonly used types
pub use atomic_execution::{AtomicMemoryContext, AtomicExecutionStats};
pub use atomic_memory_model::{
    AtomicMemoryModel, MemoryOrderingPolicy, ConsistencyValidationResult,
    MemoryModelPerformanceMetrics, DataRaceReport, OrderingViolationReport,
};
pub use branch_prediction::{
    BranchLikelihood, BranchPrediction, FunctionBranchPredictor, ModuleBranchPredictor,
    PredictiveExecutionContext, PredictionStats,
};
pub use cfi_engine::{
    CfiEngineStatistics, CfiExecutionEngine, CfiExecutionResult, CfiViolationPolicy,
    CfiViolationType, ExecutionResult,
};
pub use execution::{ExecutionContext, ExecutionStats};
pub use interpreter_optimization::{
    OptimizedInterpreter, OptimizationStrategy, OptimizationMetrics, 
    BranchOptimizationResult, ExecutionPath,
};
pub use thread_manager::{
    ThreadManager, ThreadConfig, ThreadInfo, ThreadState, ThreadExecutionContext,
    ThreadExecutionStats, ThreadManagerStats, ThreadId,
};
pub use wait_queue::{
    WaitQueueManager, WaitQueue, WaitQueueId, WaitResult, WaitQueueStats,
    WaitQueueGlobalStats, pause,
};
#[cfg(feature = "wit-debug-integration")]
pub use wit_debugger_integration::{
    WrtRuntimeState, WrtDebugMemory, DebuggableWrtRuntime,
    create_wit_enabled_runtime, create_component_metadata, 
    create_function_metadata, create_type_metadata,
    ComponentMetadata, FunctionMetadata, TypeMetadata, WitTypeKind,
    Breakpoint, BreakpointCondition,
};
pub use func::FuncType;
pub use global::Global;
pub use memory::Memory;
pub use memory_adapter::{MemoryAdapter, SafeMemoryAdapter, StdMemoryProvider};
pub use memory_helpers::ArcMemoryExt;
pub use module::{
    Data, Element, Export, ExportItem, ExportKind, Function, Import, Module, OtherExport,
};
pub use module_builder::{load_module_from_binary, ModuleBuilder};
pub use module_instance::ModuleInstance;
pub use stackless::{
    StacklessCallbackRegistry, StacklessEngine, StacklessExecutionState, StacklessFrame,
};
pub use table::Table;

/// The WebAssembly memory page size (64KiB)
pub const PAGE_SIZE: usize = 65536;

/// Component Model implementations of runtime interfaces
pub mod component_impl;
/// Component Model trait definitions for runtime interfaces
pub mod component_traits;

// Internal modules
#[cfg(test)]
mod tests;

// Re-export trait definitions
// Re-export implementations
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub use component_impl::no_alloc::MinimalComponent;
#[cfg(any(feature = "std", feature = "alloc"))]
pub use component_impl::{ComponentRuntimeImpl, DefaultHostFunctionFactory};
pub use component_traits::{
    ComponentInstance, ComponentRuntime, HostFunction, HostFunctionFactory,
};

// Panic handler is provided by the main binary crate to avoid conflicts
