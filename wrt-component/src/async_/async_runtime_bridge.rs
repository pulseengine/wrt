//! Bridge between Component Model async and Rust async (if needed)
//!
//! The WebAssembly Component Model defines its own async primitives (stream,
//! future, error-context) that are different from Rust's async/await. This
//! module provides optional bridges between them.

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::{String, ToString};
#[cfg(feature = "std")]
use std::string::{String, ToString};
#[cfg(not(any(feature = "std", feature = "alloc")))]
type String = wrt_foundation::bounded::BoundedString<256>;

use core::{
    pin::Pin,
    task::{
        Context,
        Poll,
        Waker,
    },
};

use wrt_error::Error;
#[cfg(feature = "std")]
use wrt_foundation::BoundedVec;
use wrt_runtime::{Checksummable, ToBytes, FromBytes};

use super::async_types::{
    Future as WasmFuture,
    FutureHandle,
    FutureState,
    Stream as WasmStream,
    StreamHandle,
};
#[cfg(feature = "component-model-threading")]
use crate::threading::task_manager::{
    TaskId,
    TaskManager,
    TaskState,
};
// Use Value for all component values (non-generic)
use crate::types::Value as ComponentValue;
use crate::ComponentInstanceId;

// Placeholder types when threading is not available
#[cfg(not(feature = "component-model-threading"))]
pub type TaskId = u32;
#[cfg(not(feature = "component-model-threading"))]
pub type TaskManager = ();
#[cfg(not(feature = "component-model-threading"))]
pub type TaskState = ();

// ValType import placeholder - need to check where this should come from
pub type ValType = u32;

/// The Component Model async primitives DO NOT require Rust's Future trait.
/// They work through their own polling/waiting mechanisms via the task manager.
///
/// However, if you want to integrate with Rust async runtimes (tokio,
/// async-std), this module provides adapters.

#[cfg(feature = "std")]
pub mod rust_async_bridge {
    use std::{
        future::Future as RustFuture,
        string::String,
        sync::{
            Arc,
            Mutex,
        },
        task::Wake,
    };

    use super::*;

    /// Adapter to use a Component Model Future in Rust async code
    pub struct ComponentFutureAdapter<T> {
        wasm_future:  Arc<Mutex<WasmFuture<T>>>,
        task_manager: Arc<Mutex<TaskManager>>,
        task_id:      TaskId,
    }

    impl<T: Clone + Send + 'static> RustFuture for ComponentFutureAdapter<T> {
        type Output = core::result::Result<T, String>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let future = self.wasm_future.lock().unwrap();

            match future.state {
                FutureState::Ready => {
                    if let Some(ref value) = future.value {
                        Poll::Ready(Ok(value.clone()))
                    } else {
                        Poll::Ready(Err(String::from("Future ready but no value")))
                    }
                },
                FutureState::Error | FutureState::Failed => {
                    Poll::Ready(Err(String::from("Future failed")))
                },
                FutureState::Cancelled => Poll::Ready(Err(String::from("Future cancelled"))),
                FutureState::Pending => {
                    // Register waker with task manager
                    // In a real implementation, this would notify the task manager
                    // to wake this future when the Component Model future completes
                    cx.waker().wake_by_ref();
                    Poll::Pending
                },
            }
        }
    }

    /// Adapter to use Rust futures in Component Model
    pub fn rust_future_to_component<F, T>(
        future: F,
        task_manager: &mut TaskManager,
        component_id: ComponentInstanceId,
    ) -> core::result::Result<FutureHandle, String>
    where
        F: RustFuture<Output = T> + Send + 'static,
        T: Into<ComponentValue>,
    {
        // This would spawn the Rust future and create a Component Model future
        // that completes when the Rust future completes
        // For now, this is a placeholder
        Err(String::from("Not implemented"))
    }
}

/// Pure Component Model async - no Rust Future trait needed
pub mod component_async {
    use super::*;

    /// Helper to create error strings compatible with all feature configurations
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn error_string(msg: &str) -> String {
        String::from(msg)
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn error_string(msg: &str) -> String {
        use wrt_foundation::{bounded::BoundedString, safe_memory::NoStdProvider};
        // Use a stack-allocated provider for error strings
        let provider1 = NoStdProvider::<1024>::default();
        BoundedString::from_str(msg)
            .unwrap_or_else(|_| {
                let provider2 = NoStdProvider::<1024>::default();
                BoundedString::from_str("Error").unwrap()
            })
    }

    /// Execute an async Component Model operation without Rust's async runtime
    pub fn execute_async_operation(
        task_manager: &mut TaskManager,
        operation: AsyncOperation,
    ) -> core::result::Result<TaskId, String> {
        #[cfg(feature = "component-model-threading")]
        {
            // Create a task for the async operation
            let task_id = task_manager
                .spawn_task(
                    crate::threading::task_manager::TaskType::AsyncOperation,
                    operation.component_id.0,
                    None,
                )
                .map_err(|_| Error::component_resource_lifecycle_error("Failed to spawn task"))?;

            // Start the task
            task_manager
                .switch_to_task(task_id)
                .map_err(|_| Error::component_resource_lifecycle_error("Failed to start task"))?;

            Ok(task_id)
        }
        #[cfg(not(feature = "component-model-threading"))]
        {
            // Return a dummy task ID when threading is not available
            // TaskId is just u32 when threading is disabled
            Ok(0)
        }
    }

    /// Poll a Component Model future manually
    pub fn poll_future<T>(
        future: &mut WasmFuture<T>,
        task_manager: &mut TaskManager,
    ) -> PollResult<T>
    where
        T: Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
    {
        match future.state {
            FutureState::Ready => {
                if let Some(ref value) = future.value {
                    PollResult::Ready(value.clone())
                } else {
                    PollResult::Error(error_string("Future ready but no value"))
                }
            },
            FutureState::Pending => PollResult::Pending,
            FutureState::Error | FutureState::Failed => {
                PollResult::Error(error_string("Future failed"))
            },
            FutureState::Cancelled => PollResult::Error(error_string("Future cancelled")),
        }
    }

    /// Poll a Component Model stream manually
    pub fn poll_stream<T>(
        stream: &mut WasmStream<T>,
        task_manager: &mut TaskManager,
    ) -> StreamPollResult<T>
    where
        T: Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
    {
        if !stream.buffer.is_empty() {
            // Return first item from buffer
            #[cfg(feature = "std")]
            {
                StreamPollResult::Item(stream.buffer.remove(0))
            }
            #[cfg(not(any(feature = "std",)))]
            {
                if !stream.buffer.is_empty() {
                    StreamPollResult::Item(stream.buffer.remove(0))
                } else {
                    StreamPollResult::Pending
                }
            }
        } else if stream.readable_closed {
            StreamPollResult::Closed
        } else {
            StreamPollResult::Pending
        }
    }

    #[derive(Debug, Clone)]
    pub struct AsyncOperation {
        pub component_id:   ComponentInstanceId,
        pub name:           String,
        pub operation_type: AsyncOperationType,
    }

    #[derive(Debug, Clone)]
    pub enum AsyncOperationType {
        StreamRead(StreamHandle),
        StreamWrite(StreamHandle, ComponentValue),
        FutureWait(FutureHandle),
        FutureComplete(FutureHandle, ComponentValue),
    }

    #[derive(Debug, Clone)]
    pub enum PollResult<T> {
        Ready(T),
        Pending,
        Error(String),
    }

    #[derive(Debug, Clone)]
    pub enum StreamPollResult<T> {
        Item(T),
        Pending,
        Closed,
    }
}

/// Example of using Component Model async WITHOUT Rust futures
#[cfg(test)]
mod tests {
    use super::{
        component_async::*,
        *,
    };

    #[test]
    #[cfg(feature = "component-model-threading")]
    fn test_component_model_async_without_rust_futures() {
        let mut task_manager = TaskManager::new().unwrap();
        let component_id = ComponentInstanceId::new(1);

        // Create a Component Model future - no Rust Future trait needed!
        let future_handle = FutureHandle(1);
        let mut wasm_future = WasmFuture::<i32>::new(future_handle, ValType::I32);

        // Poll it manually
        let result = poll_future(&mut wasm_future, &mut task_manager);
        assert!(matches!(result, PollResult::Pending));

        // Complete the future
        wasm_future.set_value(42).unwrap();

        // Poll again
        let result = poll_future(&mut wasm_future, &mut task_manager);
        assert!(matches!(result, PollResult::Ready(42)));
    }

    #[test]
    #[cfg(feature = "component-model-threading")]
    fn test_component_model_stream_without_rust_futures() {
        let mut task_manager = TaskManager::new().unwrap();

        // Create a Component Model stream - no Rust Stream trait needed!
        let stream_handle = StreamHandle(1);
        let mut wasm_stream = WasmStream::<String>::new(stream_handle, ValType::String);

        // Add some values
        #[cfg(feature = "std")]
        {
            wasm_stream.buffer.push("Hello".to_owned());
            wasm_stream.buffer.push("World".to_owned());
        }

        // Poll values manually
        let result1 = poll_stream(&mut wasm_stream, &mut task_manager);
        assert!(matches!(result1, StreamPollResult::Item(ref s) if s == "Hello"));

        let result2 = poll_stream(&mut wasm_stream, &mut task_manager);
        assert!(matches!(result2, StreamPollResult::Item(ref s) if s == "World"));

        // Now empty
        let result3 = poll_stream(&mut wasm_stream, &mut task_manager);
        assert!(matches!(result3, StreamPollResult::Pending));
    }
}

/// Summary: The WebAssembly Component Model async does NOT require the futures
/// crate or Rust's async/await. It has its own async primitives:
///
/// 1. `stream<T>` - for incremental value passing
/// 2. `future<T>` - for deferred single values
/// 3. `error-context` - for detailed error information
///
/// These are polled/waited on through the task manager and canonical built-ins
/// like:
/// - `task.wait` - wait for async operations
/// - `stream.read` / `stream.write` - stream operations
/// - `future.read` / `future.write` - future operations
///
/// The Rust Future trait is only needed if you want to integrate with Rust
/// async runtimes like tokio or async-std, which is optional.
pub struct ComponentModelAsyncSummary;
