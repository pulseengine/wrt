//! Async-enhanced component execution engine
//!
//! This module extends the component execution engine with async support,
//! integrating task management and async canonical built-ins.

#[cfg(not(feature = "std"))]
use core::{fmt, mem};
#[cfg(feature = "std")]
use std::{fmt, mem};

#[cfg(any(feature = "std", feature = "alloc"))]
use alloc::{boxed::Box, vec::Vec};

use wrt_foundation::{
    bounded::BoundedVec,
    component_value::ComponentValue,
    prelude::*,
};

use crate::{
    async_canonical::AsyncCanonicalAbi,
    async_types::{
        AsyncReadResult, ErrorContextHandle, FutureHandle, StreamHandle,
        Waitable, WaitableSet
    },
    canonical::CanonicalAbi,
    execution_engine::{ComponentExecutionEngine, HostFunction},
    task_manager::{TaskId, TaskManager, TaskType},
    types::{Value, ValType},
    WrtResult,
};

/// Async-enhanced component execution engine
pub struct AsyncComponentExecutionEngine {
    /// Base execution engine
    base_engine: ComponentExecutionEngine,
    
    /// Async canonical ABI
    async_abi: AsyncCanonicalAbi,
    
    /// Current execution mode
    execution_mode: ExecutionMode,
    
    /// Async operation timeout (in milliseconds)
    async_timeout_ms: u32,
}

/// Execution mode for the engine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Synchronous execution only
    Sync,
    /// Async execution enabled
    Async,
    /// Mixed mode (sync and async)
    Mixed,
}

/// Async execution result
#[derive(Debug, Clone)]
pub enum AsyncExecutionResult {
    /// Execution completed synchronously
    Completed(Value),
    /// Execution is suspended waiting for async operation
    Suspended {
        task_id: TaskId,
        waitables: WaitableSet,
    },
    /// Execution failed
    Failed(ErrorContextHandle),
    /// Execution was cancelled
    Cancelled,
}

/// Async function call parameters
#[derive(Debug, Clone)]
pub struct AsyncCallParams {
    /// Whether to enable async execution
    pub enable_async: bool,
    /// Timeout for async operations
    pub timeout_ms: Option<u32>,
    /// Maximum number of async operations
    pub max_async_ops: Option<u32>,
}

impl AsyncComponentExecutionEngine {
    /// Create a new async execution engine
    pub fn new() -> Self {
        Self {
            base_engine: ComponentExecutionEngine::new(),
            async_abi: AsyncCanonicalAbi::new(),
            execution_mode: ExecutionMode::Mixed,
            async_timeout_ms: 5000, // 5 second default
        }
    }

    /// Set execution mode
    pub fn set_execution_mode(&mut self, mode: ExecutionMode) {
        self.execution_mode = mode;
    }

    /// Set async timeout
    pub fn set_async_timeout(&mut self, timeout_ms: u32) {
        self.async_timeout_ms = timeout_ms;
    }

    /// Call a component function with async support
    pub fn call_function_async(
        &mut self,
        instance_id: u32,
        function_index: u32,
        args: &[Value],
        params: AsyncCallParams,
    ) -> WrtResult<AsyncExecutionResult> {
        // Check execution mode
        if !params.enable_async && self.execution_mode == ExecutionMode::Async {
            return Err(wrt_foundation::WrtError::InvalidInput(
                "Async execution required but not enabled".into()
            ));
        }

        // Create task for this function call
        let task_id = self.async_abi.task_manager_mut().spawn_task(
            TaskType::ComponentFunction,
            instance_id,
            Some(function_index),
        )?;

        // Switch to the task
        self.async_abi.task_manager_mut().switch_to_task(task_id)?;

        // Execute the function
        let result = self.execute_function_with_async(
            instance_id,
            function_index,
            args,
            &params,
        );

        match result {
            Ok(value) => {
                // Complete the task
                self.async_abi.task_manager_mut().task_return(vec![value.clone()])?;
                Ok(AsyncExecutionResult::Completed(value))
            }
            Err(err) => {
                // Handle async suspension or error
                if let Some(current_task) = self.async_abi.task_manager().current_task_id() {
                    if let Some(task) = self.async_abi.task_manager().get_task(current_task) {
                        if let Some(waitables) = &task.waiting_on {
                            return Ok(AsyncExecutionResult::Suspended {
                                task_id: current_task,
                                waitables: waitables.clone(),
                            });
                        }
                    }
                }
                
                // Create error context
                let error_handle = self.async_abi.error_context_new(&format!("{:?}", err))?;
                Ok(AsyncExecutionResult::Failed(error_handle))
            }
        }
    }

    /// Execute function with async capabilities
    fn execute_function_with_async(
        &mut self,
        instance_id: u32,
        function_index: u32,
        args: &[Value],
        params: &AsyncCallParams,
    ) -> WrtResult<Value> {
        // Check for async operations in arguments
        let has_async_args = args.iter().any(|arg| self.is_async_value(arg));

        if has_async_args && params.enable_async {
            // Handle async execution
            self.execute_async_function(instance_id, function_index, args, params)
        } else {
            // Fall back to synchronous execution
            self.base_engine.call_function(instance_id, function_index, args)
        }
    }

    /// Execute function with async operations
    fn execute_async_function(
        &mut self,
        instance_id: u32,
        function_index: u32,
        args: &[Value],
        _params: &AsyncCallParams,
    ) -> WrtResult<Value> {
        // Process async arguments
        let processed_args = self.process_async_args(args)?;

        // Execute with base engine but with async context
        self.base_engine.call_function(instance_id, function_index, &processed_args)
    }

    /// Process arguments that may contain async values
    fn process_async_args(&mut self, args: &[Value]) -> WrtResult<Vec<Value>> {
        let mut processed = Vec::new();

        for arg in args {
            match arg {
                Value::Own(handle) | Value::Borrow(handle) => {
                    // Check if this is an async resource
                    if self.is_async_resource(*handle) {
                        let processed_value = self.resolve_async_resource(*handle)?;
                        processed.push(processed_value);
                    } else {
                        processed.push(arg.clone());
                    }
                }
                _ => processed.push(arg.clone()),
            }
        }

        Ok(processed)
    }

    /// Check if a value contains async operations
    fn is_async_value(&self, value: &Value) -> bool {
        match value {
            Value::Own(handle) | Value::Borrow(handle) => {
                self.is_async_resource(*handle)
            }
            Value::List(values) => {
                values.iter().any(|v| self.is_async_value(v))
            }
            Value::Record(values) => {
                values.iter().any(|v| self.is_async_value(v))
            }
            Value::Tuple(values) => {
                values.iter().any(|v| self.is_async_value(v))
            }
            _ => false,
        }
    }

    /// Check if a resource handle refers to an async resource
    fn is_async_resource(&self, _handle: u32) -> bool {
        // In a real implementation, would check if the handle refers to
        // a stream, future, or other async resource
        false
    }

    /// Resolve an async resource to a concrete value
    fn resolve_async_resource(&mut self, handle: u32) -> WrtResult<Value> {
        // Try as stream first
        if let Ok(result) = self.async_abi.stream_read(StreamHandle(handle)) {
            match result {
                AsyncReadResult::Values(values) => {
                    if let Some(value) = values.first() {
                        return Ok(value.clone());
                    }
                }
                AsyncReadResult::Blocked => {
                    return Err(wrt_foundation::WrtError::InvalidState(
                        "Stream read would block".into()
                    ));
                }
                AsyncReadResult::Closed => {
                    return Err(wrt_foundation::WrtError::InvalidState(
                        "Stream is closed".into()
                    ));
                }
                AsyncReadResult::Error(error_handle) => {
                    return Err(wrt_foundation::WrtError::AsyncError(
                        format!("Stream error: {:?}", error_handle).into()
                    ));
                }
            }
        }

        // Try as future
        if let Ok(result) = self.async_abi.future_read(FutureHandle(handle)) {
            match result {
                AsyncReadResult::Values(values) => {
                    if let Some(value) = values.first() {
                        return Ok(value.clone());
                    }
                }
                AsyncReadResult::Blocked => {
                    return Err(wrt_foundation::WrtError::InvalidState(
                        "Future read would block".into()
                    ));
                }
                AsyncReadResult::Closed => {
                    return Err(wrt_foundation::WrtError::InvalidState(
                        "Future is closed".into()
                    ));
                }
                AsyncReadResult::Error(error_handle) => {
                    return Err(wrt_foundation::WrtError::AsyncError(
                        format!("Future error: {:?}", error_handle).into()
                    ));
                }
            }
        }

        Err(wrt_foundation::WrtError::InvalidInput(
            "Unable to resolve async resource".into()
        ))
    }

    /// Resume suspended execution
    pub fn resume_execution(&mut self, task_id: TaskId) -> WrtResult<AsyncExecutionResult> {
        // Make the task ready
        self.async_abi.task_manager_mut().make_ready(task_id)?;

        // Switch to the task
        self.async_abi.task_manager_mut().switch_to_task(task_id)?;

        // Check if the task can proceed
        if let Some(task) = self.async_abi.task_manager().get_task(task_id) {
            if let Some(waitables) = &task.waiting_on {
                if let Some(ready_index) = waitables.first_ready() {
                    // Process the ready waitable
                    let waitable = &waitables.waitables[ready_index as usize];
                    let result = self.process_ready_waitable(waitable)?;
                    
                    // Complete the task
                    self.async_abi.task_manager_mut().task_return(vec![result.clone()])?;
                    Ok(AsyncExecutionResult::Completed(result))
                } else {
                    // Still waiting
                    Ok(AsyncExecutionResult::Suspended {
                        task_id,
                        waitables: waitables.clone(),
                    })
                }
            } else {
                // Task is not waiting, try to continue execution
                // In a real implementation, would continue from where it left off
                Ok(AsyncExecutionResult::Completed(Value::U32(0)))
            }
        } else {
            Err(wrt_foundation::WrtError::InvalidInput("Task not found".into()))
        }
    }

    /// Process a ready waitable
    fn process_ready_waitable(&mut self, waitable: &Waitable) -> WrtResult<Value> {
        match waitable {
            Waitable::StreamReadable(stream_handle) => {
                let result = self.async_abi.stream_read(*stream_handle)?;
                match result {
                    AsyncReadResult::Values(values) => {
                        if let Some(value) = values.first() {
                            Ok(value.clone())
                        } else {
                            Ok(Value::U32(0)) // Empty result
                        }
                    }
                    _ => Ok(Value::U32(0)),
                }
            }
            Waitable::FutureReadable(future_handle) => {
                let result = self.async_abi.future_read(*future_handle)?;
                match result {
                    AsyncReadResult::Values(values) => {
                        if let Some(value) = values.first() {
                            Ok(value.clone())
                        } else {
                            Ok(Value::U32(0))
                        }
                    }
                    _ => Ok(Value::U32(0)),
                }
            }
            _ => Ok(Value::U32(0)), // Default result
        }
    }

    /// Create a new stream
    pub fn create_stream(&mut self, element_type: &ValType) -> WrtResult<StreamHandle> {
        self.async_abi.stream_new(element_type)
    }

    /// Create a new future
    pub fn create_future(&mut self, value_type: &ValType) -> WrtResult<FutureHandle> {
        self.async_abi.future_new(value_type)
    }

    /// Wait for multiple waitables
    pub fn wait_for_waitables(&mut self, waitables: WaitableSet) -> WrtResult<u32> {
        self.async_abi.task_wait(waitables)
    }

    /// Poll waitables without blocking
    pub fn poll_waitables(&self, waitables: &WaitableSet) -> WrtResult<Option<u32>> {
        self.async_abi.task_poll(waitables)
    }

    /// Yield current task
    pub fn yield_task(&mut self) -> WrtResult<()> {
        self.async_abi.task_yield()
    }

    /// Cancel a task
    pub fn cancel_task(&mut self, task_id: TaskId) -> WrtResult<()> {
        self.async_abi.task_cancel(task_id)
    }

    /// Update async resources and wake waiting tasks
    pub fn update_async_state(&mut self) -> WrtResult<()> {
        self.async_abi.task_manager_mut().update_waitables()
    }

    /// Get the base execution engine
    pub fn base_engine(&self) -> &ComponentExecutionEngine {
        &self.base_engine
    }

    /// Get mutable base execution engine
    pub fn base_engine_mut(&mut self) -> &mut ComponentExecutionEngine {
        &mut self.base_engine
    }

    /// Get the async canonical ABI
    pub fn async_abi(&self) -> &AsyncCanonicalAbi {
        &self.async_abi
    }

    /// Get mutable async canonical ABI
    pub fn async_abi_mut(&mut self) -> &mut AsyncCanonicalAbi {
        &mut self.async_abi
    }

    /// Get current execution mode
    pub fn execution_mode(&self) -> ExecutionMode {
        self.execution_mode
    }

    /// Get async timeout
    pub fn async_timeout_ms(&self) -> u32 {
        self.async_timeout_ms
    }
}

impl Default for AsyncComponentExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AsyncCallParams {
    fn default() -> Self {
        Self {
            enable_async: true,
            timeout_ms: Some(5000),
            max_async_ops: Some(100),
        }
    }
}

impl fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionMode::Sync => write!(f, "sync"),
            ExecutionMode::Async => write!(f, "async"),
            ExecutionMode::Mixed => write!(f, "mixed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_engine_creation() {
        let engine = AsyncComponentExecutionEngine::new();
        assert_eq!(engine.execution_mode(), ExecutionMode::Mixed);
        assert_eq!(engine.async_timeout_ms(), 5000);
    }

    #[test]
    fn test_execution_mode_configuration() {
        let mut engine = AsyncComponentExecutionEngine::new();
        
        engine.set_execution_mode(ExecutionMode::Async);
        assert_eq!(engine.execution_mode(), ExecutionMode::Async);
        
        engine.set_async_timeout(10000);
        assert_eq!(engine.async_timeout_ms(), 10000);
    }

    #[test]
    fn test_async_call_params() {
        let params = AsyncCallParams::default();
        assert!(params.enable_async);
        assert_eq!(params.timeout_ms, Some(5000));
        assert_eq!(params.max_async_ops, Some(100));
    }

    #[test]
    fn test_stream_creation() {
        let mut engine = AsyncComponentExecutionEngine::new();
        let stream_handle = engine.create_stream(&ValType::U32).unwrap();
        assert_eq!(stream_handle.0, 0);
    }

    #[test]
    fn test_future_creation() {
        let mut engine = AsyncComponentExecutionEngine::new();
        let future_handle = engine.create_future(&ValType::String).unwrap();
        assert_eq!(future_handle.0, 0);
    }

    #[test]
    fn test_execution_mode_display() {
        assert_eq!(ExecutionMode::Sync.to_string(), "sync");
        assert_eq!(ExecutionMode::Async.to_string(), "async");
        assert_eq!(ExecutionMode::Mixed.to_string(), "mixed");
    }

    #[test]
    fn test_is_async_value() {
        let engine = AsyncComponentExecutionEngine::new();
        
        // Regular values should not be async
        assert!(!engine.is_async_value(&Value::U32(42)));
        assert!(!engine.is_async_value(&Value::String(BoundedString::from_str("test").unwrap())));
        
        // Resource handles might be async (depends on implementation)
        assert!(!engine.is_async_value(&Value::Own(1)));
    }
}