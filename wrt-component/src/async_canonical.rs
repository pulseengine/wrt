//! Async canonical built-ins for WebAssembly Component Model
//!
//! This module implements the async canonical built-ins required by the
//! Component Model MVP specification for stream, future, and task operations.

#[cfg(not(feature = "std"))]
use core::{fmt, mem};
#[cfg(feature = "std")]
use std::{fmt, mem};

#[cfg(any(feature = "std", feature = "alloc"))]
use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};

use wrt_foundation::{
    bounded::BoundedVec, component_value::ComponentValue, prelude::*, resource::ResourceHandle,
};

use crate::{
    async_types::{
        AsyncReadResult, ErrorContext, ErrorContextHandle, Future, FutureHandle, FutureState,
        Stream, StreamHandle, StreamState, Waitable, WaitableSet,
    },
    canonical::CanonicalAbi,
    task_manager::{TaskId, TaskManager, TaskType},
    types::{ValType, Value},
    WrtResult,
};

/// Maximum number of streams/futures in no_std environments
const MAX_ASYNC_RESOURCES: usize = 256;

/// Async canonical ABI implementation
pub struct AsyncCanonicalAbi {
    /// Base canonical ABI
    canonical_abi: CanonicalAbi,

    /// Task manager
    task_manager: TaskManager,

    /// Stream registry
    #[cfg(any(feature = "std", feature = "alloc"))]
    streams: BTreeMap<StreamHandle, Box<dyn StreamValue>>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    streams: BoundedVec<(StreamHandle, StreamValueEnum), MAX_ASYNC_RESOURCES>,

    /// Future registry
    #[cfg(any(feature = "std", feature = "alloc"))]
    futures: BTreeMap<FutureHandle, Box<dyn FutureValue>>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    futures: BoundedVec<(FutureHandle, FutureValueEnum), MAX_ASYNC_RESOURCES>,

    /// Error context registry
    #[cfg(any(feature = "std", feature = "alloc"))]
    error_contexts: BTreeMap<ErrorContextHandle, ErrorContext>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    error_contexts: BoundedVec<(ErrorContextHandle, ErrorContext), MAX_ASYNC_RESOURCES>,

    /// Next handle IDs
    next_stream_handle: u32,
    next_future_handle: u32,
    next_error_context_handle: u32,
}

/// Stream value trait for type erasure
#[cfg(any(feature = "std", feature = "alloc"))]
pub trait StreamValue: fmt::Debug {
    fn read(&mut self) -> WrtResult<AsyncReadResult>;
    fn write(&mut self, values: &[Value]) -> WrtResult<()>;
    fn cancel_read(&mut self) -> WrtResult<()>;
    fn cancel_write(&mut self) -> WrtResult<()>;
    fn close_readable(&mut self) -> WrtResult<()>;
    fn close_writable(&mut self) -> WrtResult<()>;
    fn element_type(&self) -> &ValType;
    fn is_readable(&self) -> bool;
    fn is_writable(&self) -> bool;
}

/// Future value trait for type erasure
#[cfg(any(feature = "std", feature = "alloc"))]
pub trait FutureValue: fmt::Debug {
    fn read(&mut self) -> WrtResult<AsyncReadResult>;
    fn write(&mut self, value: &Value) -> WrtResult<()>;
    fn cancel_read(&mut self) -> WrtResult<()>;
    fn cancel_write(&mut self) -> WrtResult<()>;
    fn close_readable(&mut self) -> WrtResult<()>;
    fn close_writable(&mut self) -> WrtResult<()>;
    fn value_type(&self) -> &ValType;
    fn is_readable(&self) -> bool;
    fn is_writable(&self) -> bool;
}

/// Enum for stream values in no_std environments
#[cfg(not(any(feature = "std", feature = "alloc")))]
#[derive(Debug)]
pub enum StreamValueEnum {
    Values(Stream<Value>),
    // Add more variants as needed for different types
}

/// Enum for future values in no_std environments
#[cfg(not(any(feature = "std", feature = "alloc")))]
#[derive(Debug)]
pub enum FutureValueEnum {
    Value(Future<Value>),
    // Add more variants as needed for different types
}

/// Concrete stream implementation
#[derive(Debug)]
pub struct ConcreteStream<T> {
    inner: Stream<T>,
}

/// Concrete future implementation
#[derive(Debug)]
pub struct ConcreteFuture<T> {
    inner: Future<T>,
}

impl AsyncCanonicalAbi {
    /// Create a new async canonical ABI
    pub fn new() -> Self {
        Self {
            canonical_abi: CanonicalAbi::new(),
            task_manager: TaskManager::new(),
            #[cfg(any(feature = "std", feature = "alloc"))]
            streams: BTreeMap::new(),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            streams: BoundedVec::new(),
            #[cfg(any(feature = "std", feature = "alloc"))]
            futures: BTreeMap::new(),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            futures: BoundedVec::new(),
            #[cfg(any(feature = "std", feature = "alloc"))]
            error_contexts: BTreeMap::new(),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            error_contexts: BoundedVec::new(),
            next_stream_handle: 0,
            next_future_handle: 0,
            next_error_context_handle: 0,
        }
    }

    /// Create a new stream
    pub fn stream_new(&mut self, element_type: &ValType) -> WrtResult<StreamHandle> {
        let handle = StreamHandle(self.next_stream_handle);
        self.next_stream_handle += 1;

        let stream = Stream::new(handle, element_type.clone());

        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let concrete = ConcreteStream { inner: stream };
            self.streams.insert(handle, Box::new(concrete));
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            let stream_enum = StreamValueEnum::Values(stream);
            self.streams.push((handle, stream_enum)).map_err(|_| {
                wrt_foundation::WrtError::ResourceExhausted("Too many streams".into())
            })?;
        }

        Ok(handle)
    }

    /// Read from a stream
    pub fn stream_read(&mut self, stream_handle: StreamHandle) -> WrtResult<AsyncReadResult> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            if let Some(stream) = self.streams.get_mut(&stream_handle) {
                stream.read()
            } else {
                Err(wrt_foundation::WrtError::InvalidInput("Stream not found".into()))
            }
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            for (handle, stream) in &mut self.streams {
                if *handle == stream_handle {
                    return match stream {
                        StreamValueEnum::Values(ref mut s) => {
                            if s.buffer.is_empty() {
                                if s.writable_closed {
                                    Ok(AsyncReadResult::Closed)
                                } else {
                                    Ok(AsyncReadResult::Blocked)
                                }
                            } else {
                                // Read one value
                                let value = s.buffer.remove(0);
                                Ok(AsyncReadResult::Values(vec![value]))
                            }
                        }
                    };
                }
            }
            Err(wrt_foundation::WrtError::InvalidInput("Stream not found".into()))
        }
    }

    /// Write to a stream
    pub fn stream_write(&mut self, stream_handle: StreamHandle, values: &[Value]) -> WrtResult<()> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            if let Some(stream) = self.streams.get_mut(&stream_handle) {
                stream.write(values)
            } else {
                Err(wrt_foundation::WrtError::InvalidInput("Stream not found".into()))
            }
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            for (handle, stream) in &mut self.streams {
                if *handle == stream_handle {
                    return match stream {
                        StreamValueEnum::Values(ref mut s) => {
                            if s.writable_closed {
                                return Err(wrt_foundation::WrtError::InvalidState(
                                    "Stream write end is closed".into(),
                                ));
                            }
                            for value in values {
                                s.buffer.push(value.clone()).map_err(|_| {
                                    wrt_foundation::WrtError::ResourceExhausted(
                                        "Stream buffer full".into(),
                                    )
                                })?;
                            }
                            s.state = StreamState::Ready;
                            Ok(())
                        }
                    };
                }
            }
            Err(wrt_foundation::WrtError::InvalidInput("Stream not found".into()))
        }
    }

    /// Cancel read operation on a stream
    pub fn stream_cancel_read(&mut self, stream_handle: StreamHandle) -> WrtResult<()> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            if let Some(stream) = self.streams.get_mut(&stream_handle) {
                stream.cancel_read()
            } else {
                Err(wrt_foundation::WrtError::InvalidInput("Stream not found".into()))
            }
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            for (handle, stream) in &mut self.streams {
                if *handle == stream_handle {
                    return match stream {
                        StreamValueEnum::Values(ref mut s) => {
                            s.close_readable();
                            Ok(())
                        }
                    };
                }
            }
            Err(wrt_foundation::WrtError::InvalidInput("Stream not found".into()))
        }
    }

    /// Cancel write operation on a stream
    pub fn stream_cancel_write(&mut self, stream_handle: StreamHandle) -> WrtResult<()> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            if let Some(stream) = self.streams.get_mut(&stream_handle) {
                stream.cancel_write()
            } else {
                Err(wrt_foundation::WrtError::InvalidInput("Stream not found".into()))
            }
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            for (handle, stream) in &mut self.streams {
                if *handle == stream_handle {
                    return match stream {
                        StreamValueEnum::Values(ref mut s) => {
                            s.close_writable();
                            Ok(())
                        }
                    };
                }
            }
            Err(wrt_foundation::WrtError::InvalidInput("Stream not found".into()))
        }
    }

    /// Close readable end of a stream
    pub fn stream_close_readable(&mut self, stream_handle: StreamHandle) -> WrtResult<()> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            if let Some(stream) = self.streams.get_mut(&stream_handle) {
                stream.close_readable()
            } else {
                Err(wrt_foundation::WrtError::InvalidInput("Stream not found".into()))
            }
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            for (handle, stream) in &mut self.streams {
                if *handle == stream_handle {
                    return match stream {
                        StreamValueEnum::Values(ref mut s) => {
                            s.close_readable();
                            Ok(())
                        }
                    };
                }
            }
            Err(wrt_foundation::WrtError::InvalidInput("Stream not found".into()))
        }
    }

    /// Close writable end of a stream
    pub fn stream_close_writable(&mut self, stream_handle: StreamHandle) -> WrtResult<()> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            if let Some(stream) = self.streams.get_mut(&stream_handle) {
                stream.close_writable()
            } else {
                Err(wrt_foundation::WrtError::InvalidInput("Stream not found".into()))
            }
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            for (handle, stream) in &mut self.streams {
                if *handle == stream_handle {
                    return match stream {
                        StreamValueEnum::Values(ref mut s) => {
                            s.close_writable();
                            Ok(())
                        }
                    };
                }
            }
            Err(wrt_foundation::WrtError::InvalidInput("Stream not found".into()))
        }
    }

    /// Create a new future
    pub fn future_new(&mut self, value_type: &ValType) -> WrtResult<FutureHandle> {
        let handle = FutureHandle(self.next_future_handle);
        self.next_future_handle += 1;

        let future = Future::new(handle, value_type.clone());

        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let concrete = ConcreteFuture { inner: future };
            self.futures.insert(handle, Box::new(concrete));
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            let future_enum = FutureValueEnum::Value(future);
            self.futures.push((handle, future_enum)).map_err(|_| {
                wrt_foundation::WrtError::ResourceExhausted("Too many futures".into())
            })?;
        }

        Ok(handle)
    }

    /// Read from a future
    pub fn future_read(&mut self, future_handle: FutureHandle) -> WrtResult<AsyncReadResult> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            if let Some(future) = self.futures.get_mut(&future_handle) {
                future.read()
            } else {
                Err(wrt_foundation::WrtError::InvalidInput("Future not found".into()))
            }
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            for (handle, future) in &mut self.futures {
                if *handle == future_handle {
                    return match future {
                        FutureValueEnum::Value(ref mut f) => match f.state {
                            FutureState::Ready => {
                                if let Some(value) = f.value.take() {
                                    Ok(AsyncReadResult::Values(vec![value]))
                                } else {
                                    Ok(AsyncReadResult::Closed)
                                }
                            }
                            FutureState::Cancelled => Ok(AsyncReadResult::Closed),
                            FutureState::Error => Ok(AsyncReadResult::Closed),
                            FutureState::Pending => Ok(AsyncReadResult::Blocked),
                        },
                    };
                }
            }
            Err(wrt_foundation::WrtError::InvalidInput("Future not found".into()))
        }
    }

    /// Write to a future
    pub fn future_write(&mut self, future_handle: FutureHandle, value: &Value) -> WrtResult<()> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            if let Some(future) = self.futures.get_mut(&future_handle) {
                future.write(value)
            } else {
                Err(wrt_foundation::WrtError::InvalidInput("Future not found".into()))
            }
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            for (handle, future) in &mut self.futures {
                if *handle == future_handle {
                    return match future {
                        FutureValueEnum::Value(ref mut f) => f.set_value(value.clone()),
                    };
                }
            }
            Err(wrt_foundation::WrtError::InvalidInput("Future not found".into()))
        }
    }

    /// Create a new error context
    pub fn error_context_new(&mut self, message: &str) -> WrtResult<ErrorContextHandle> {
        let handle = ErrorContextHandle(self.next_error_context_handle);
        self.next_error_context_handle += 1;

        #[cfg(any(feature = "std", feature = "alloc"))]
        let error_context = ErrorContext::new(handle, message.to_string());
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        let error_context =
            ErrorContext::new(handle, BoundedString::from_str(message).unwrap_or_default());

        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            self.error_contexts.insert(handle, error_context);
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            self.error_contexts.push((handle, error_context)).map_err(|_| {
                wrt_foundation::WrtError::ResourceExhausted("Too many error contexts".into())
            })?;
        }

        Ok(handle)
    }

    /// Get debug string from error context
    pub fn error_context_debug_string(
        &self,
        handle: ErrorContextHandle,
    ) -> WrtResult<BoundedString<2048>> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            if let Some(error_context) = self.error_contexts.get(&handle) {
                Ok(error_context.debug_string())
            } else {
                Err(wrt_foundation::WrtError::InvalidInput("Error context not found".into()))
            }
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            for (ctx_handle, error_context) in &self.error_contexts {
                if *ctx_handle == handle {
                    return Ok(error_context.debug_string());
                }
            }
            Err(wrt_foundation::WrtError::InvalidInput("Error context not found".into()))
        }
    }

    /// Drop an error context
    pub fn error_context_drop(&mut self, handle: ErrorContextHandle) -> WrtResult<()> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            self.error_contexts.remove(&handle);
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            self.error_contexts.retain(|(h, _)| *h != handle);
        }
        Ok(())
    }

    /// Task return operation
    pub fn task_return(&mut self, values: Vec<Value>) -> WrtResult<()> {
        self.task_manager.task_return(values)
    }

    /// Task wait operation
    pub fn task_wait(&mut self, waitables: WaitableSet) -> WrtResult<u32> {
        self.task_manager.task_wait(waitables)
    }

    /// Task poll operation
    pub fn task_poll(&self, waitables: &WaitableSet) -> WrtResult<Option<u32>> {
        self.task_manager.task_poll(waitables)
    }

    /// Task yield operation
    pub fn task_yield(&mut self) -> WrtResult<()> {
        self.task_manager.task_yield()
    }

    /// Task cancel operation
    pub fn task_cancel(&mut self, task_id: TaskId) -> WrtResult<()> {
        self.task_manager.task_cancel(task_id)
    }

    /// Task backpressure operation
    pub fn task_backpressure(&mut self) -> WrtResult<()> {
        self.task_manager.task_backpressure()
    }

    /// Get the underlying task manager
    pub fn task_manager(&self) -> &TaskManager {
        &self.task_manager
    }

    /// Get mutable task manager
    pub fn task_manager_mut(&mut self) -> &mut TaskManager {
        &mut self.task_manager
    }

    /// Get the underlying canonical ABI
    pub fn canonical_abi(&self) -> &CanonicalAbi {
        &self.canonical_abi
    }

    /// Get mutable canonical ABI
    pub fn canonical_abi_mut(&mut self) -> &mut CanonicalAbi {
        &mut self.canonical_abi
    }
}

// Trait implementations for std environment
#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Clone + fmt::Debug> StreamValue for ConcreteStream<T>
where
    Value: From<T>,
    T: TryFrom<Value>,
{
    fn read(&mut self) -> WrtResult<AsyncReadResult> {
        if self.inner.buffer.is_empty() {
            if self.inner.writable_closed {
                Ok(AsyncReadResult::Closed)
            } else {
                Ok(AsyncReadResult::Blocked)
            }
        } else {
            let value = self.inner.buffer.remove(0);
            Ok(AsyncReadResult::Values(vec![Value::from(value)]))
        }
    }

    fn write(&mut self, values: &[Value]) -> WrtResult<()> {
        if self.inner.writable_closed {
            return Err(wrt_foundation::WrtError::InvalidState(
                "Stream write end is closed".into(),
            ));
        }

        for value in values {
            if let Ok(typed_value) = T::try_from(value.clone()) {
                self.inner.buffer.push(typed_value);
            } else {
                return Err(wrt_foundation::WrtError::TypeError(
                    "Value type mismatch for stream".into(),
                ));
            }
        }
        self.inner.state = StreamState::Ready;
        Ok(())
    }

    fn cancel_read(&mut self) -> WrtResult<()> {
        self.inner.close_readable();
        Ok(())
    }

    fn cancel_write(&mut self) -> WrtResult<()> {
        self.inner.close_writable();
        Ok(())
    }

    fn close_readable(&mut self) -> WrtResult<()> {
        self.inner.close_readable();
        Ok(())
    }

    fn close_writable(&mut self) -> WrtResult<()> {
        self.inner.close_writable();
        Ok(())
    }

    fn element_type(&self) -> &ValType {
        &self.inner.element_type
    }

    fn is_readable(&self) -> bool {
        self.inner.is_readable()
    }

    fn is_writable(&self) -> bool {
        self.inner.is_writable()
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<T: Clone + fmt::Debug> FutureValue for ConcreteFuture<T>
where
    Value: From<T>,
    T: TryFrom<Value>,
{
    fn read(&mut self) -> WrtResult<AsyncReadResult> {
        match self.inner.state {
            FutureState::Ready => {
                if let Some(value) = self.inner.value.take() {
                    Ok(AsyncReadResult::Values(vec![Value::from(value)]))
                } else {
                    Ok(AsyncReadResult::Closed)
                }
            }
            FutureState::Cancelled | FutureState::Error => Ok(AsyncReadResult::Closed),
            FutureState::Pending => Ok(AsyncReadResult::Blocked),
        }
    }

    fn write(&mut self, value: &Value) -> WrtResult<()> {
        if let Ok(typed_value) = T::try_from(value.clone()) {
            self.inner.set_value(typed_value)
        } else {
            Err(wrt_foundation::WrtError::TypeError("Value type mismatch for future".into()))
        }
    }

    fn cancel_read(&mut self) -> WrtResult<()> {
        self.inner.cancel();
        Ok(())
    }

    fn cancel_write(&mut self) -> WrtResult<()> {
        self.inner.cancel();
        Ok(())
    }

    fn close_readable(&mut self) -> WrtResult<()> {
        self.inner.readable_closed = true;
        Ok(())
    }

    fn close_writable(&mut self) -> WrtResult<()> {
        self.inner.writable_closed = true;
        Ok(())
    }

    fn value_type(&self) -> &ValType {
        &self.inner.value_type
    }

    fn is_readable(&self) -> bool {
        self.inner.is_readable()
    }

    fn is_writable(&self) -> bool {
        self.inner.is_writable()
    }
}

impl Default for AsyncCanonicalAbi {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_canonical_abi_creation() {
        let abi = AsyncCanonicalAbi::new();
        assert_eq!(abi.streams.len(), 0);
        assert_eq!(abi.futures.len(), 0);
        assert_eq!(abi.error_contexts.len(), 0);
    }

    #[test]
    fn test_stream_lifecycle() {
        let mut abi = AsyncCanonicalAbi::new();

        // Create stream
        let stream_handle = abi.stream_new(&ValType::U32).unwrap();

        // Write to stream
        let values = vec![Value::U32(42), Value::U32(24)];
        abi.stream_write(stream_handle, &values).unwrap();

        // Read from stream
        let result = abi.stream_read(stream_handle).unwrap();
        match result {
            AsyncReadResult::Values(read_values) => {
                assert_eq!(read_values.len(), 1);
                assert_eq!(read_values[0], Value::U32(42));
            }
            _ => panic!("Expected values"),
        }

        // Close stream
        abi.stream_close_writable(stream_handle).unwrap();
        abi.stream_close_readable(stream_handle).unwrap();
    }

    #[test]
    fn test_future_lifecycle() {
        let mut abi = AsyncCanonicalAbi::new();

        // Create future
        let future_handle = abi.future_new(&ValType::String).unwrap();

        // Initially should block
        let result = abi.future_read(future_handle).unwrap();
        assert!(matches!(result, AsyncReadResult::Blocked));

        // Write value
        let value = Value::String(BoundedString::from_str("hello").unwrap());
        abi.future_write(future_handle, &value).unwrap();

        // Should be ready now
        let result = abi.future_read(future_handle).unwrap();
        match result {
            AsyncReadResult::Values(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values[0], value);
            }
            _ => panic!("Expected values"),
        }
    }

    #[test]
    fn test_error_context() {
        let mut abi = AsyncCanonicalAbi::new();

        let handle = abi.error_context_new("Test error").unwrap();
        let debug_string = abi.error_context_debug_string(handle).unwrap();
        assert!(debug_string.as_str().contains("Test error"));

        abi.error_context_drop(handle).unwrap();

        // Should be gone
        assert!(abi.error_context_debug_string(handle).is_err());
    }

    #[test]
    fn test_task_operations() {
        let mut abi = AsyncCanonicalAbi::new();

        // Test yield
        assert!(abi.task_yield().is_err()); // No current task

        // Test backpressure
        assert!(abi.task_backpressure().is_err()); // No current task
    }
}
