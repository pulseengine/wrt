//! Asynchronous Component Model implementation
//!
//! This module contains all async-related functionality for the WebAssembly
//! Component Model, including async runtimes, execution engines, and async
//! canonical ABI implementations.

// Advanced sync primitives require Arc/Weak which need std or alloc
#[cfg(any(feature = "std", feature = "bounded-allocation", feature = "managed-allocation"))]
pub mod advanced_sync_primitives;
pub mod async_builtins;
pub mod async_canonical;
pub mod async_canonical_abi_support;
pub mod async_canonical_lifting;
pub mod async_combinators;
pub mod async_context_builtins;
pub mod async_execution_engine;
pub mod async_resource_cleanup;
pub mod async_runtime;
pub mod async_runtime_bridge;
pub mod async_task_executor;
pub mod async_types;
pub mod component_async_bridge;
pub mod component_model_async_ops;
pub mod fuel_async_bridge;
pub mod fuel_async_channels;
pub mod fuel_async_executor;
pub mod fuel_async_scheduler;
pub mod fuel_aware_waker;
pub mod fuel_deadline_scheduler;
pub mod fuel_debt_credit;
pub mod fuel_dynamic_manager;
pub mod fuel_error_context;
pub mod fuel_future_combinators;
pub mod fuel_handle_table;
pub mod fuel_preemption_support;
pub mod fuel_preemptive_scheduler;
pub mod fuel_priority_inheritance;
pub mod fuel_resource_cleanup;
pub mod fuel_resource_lifetime;
pub mod fuel_stream_handler;
pub mod fuel_wcet_analyzer;
pub mod optimized_async_channels;
pub mod resource_async_operations;
pub mod task_manager_async_bridge;
pub mod timer_integration;

#[cfg(any(feature = "std", feature = "bounded-allocation", feature = "managed-allocation"))]
pub use advanced_sync_primitives::*;
pub use async_builtins::*;
pub use async_canonical::*;
pub use async_canonical_abi_support::*;
pub use async_canonical_lifting::*;
pub use async_combinators::*;
pub use async_context_builtins::*;
pub use async_execution_engine::*;
pub use async_resource_cleanup::*;
pub use async_runtime::*;
pub use async_runtime_bridge::*;
pub use async_task_executor::*;
pub use async_types::*;
pub use component_async_bridge::*;
pub use component_model_async_ops::*;
pub use fuel_async_bridge::*;
pub use fuel_async_channels::*;
pub use fuel_async_executor::*;
pub use fuel_async_scheduler::*;
pub use fuel_aware_waker::*;
pub use fuel_deadline_scheduler::*;
pub use fuel_debt_credit::*;
pub use fuel_dynamic_manager::*;
pub use fuel_error_context::*;
pub use fuel_future_combinators::*;
pub use fuel_handle_table::*;
pub use fuel_preemption_support::*;
pub use fuel_preemptive_scheduler::*;
pub use fuel_priority_inheritance::*;
pub use fuel_resource_cleanup::*;
pub use fuel_resource_lifetime::*;
pub use fuel_stream_handler::*;
pub use fuel_wcet_analyzer::*;
pub use optimized_async_channels::*;
pub use resource_async_operations::*;
pub use task_manager_async_bridge::*;
pub use timer_integration::*;
