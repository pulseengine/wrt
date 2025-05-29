//! Bridge between Component Model async and Rust async/await
//!
//! This module provides the glue between Component Model async primitives
//! (stream, future, error-context) and Rust's Future trait when using
//! the pluggable executor system.

#![cfg_attr(not(feature = "std"), no_std)]

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::async_executor::{current_executor, ExecutorError, WrtExecutor};
#[cfg(feature = "component-model-async")]
use crate::async_types::{ComponentFuture, ComponentStream, StreamHandle, ComponentFutureStatus, FutureHandle};
use crate::types::ValType;
use crate::values::Value;

#[cfg(feature = "component-model-async")]
/// Bridge a Component Model future to a Rust future
pub struct ComponentFutureBridge<T> {
    component_future: ComponentFuture<T>,
}

#[cfg(feature = "component-model-async")]
impl<T> ComponentFutureBridge<T> {
    pub fn new(component_future: ComponentFuture<T>) -> Self {
        Self { component_future }
    }
}

#[cfg(feature = "component-model-async")]
impl<T: Clone + Send + 'static> Future for ComponentFutureBridge<T> {
    type Output = Result<T, ExecutorError>;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Check Component Model future status
        match self.component_future.poll_status() {
            Ok(ComponentFutureStatus::Ready(value)) => {
                Poll::Ready(Ok(value))
            }
            Ok(ComponentFutureStatus::Pending) => {
                // Register waker to be notified when Component Model future completes
                self.component_future.set_waker(cx.waker().clone());
                Poll::Pending
            }
            Err(_) => Poll::Ready(Err(ExecutorError::TaskPanicked)),
        }
    }
}

#[cfg(feature = "component-model-async")]
/// Bridge a Component Model stream to a Rust Stream (when available)
pub struct ComponentStreamBridge<T> {
    component_stream: ComponentStream<T>,
}

#[cfg(feature = "component-model-async")]
impl<T> ComponentStreamBridge<T> {
    pub fn new(component_stream: ComponentStream<T>) -> Self {
        Self { component_stream }
    }
    
    /// Poll for the next value from the stream
    pub fn poll_next(&mut self, _cx: &mut Context<'_>) -> Poll<Option<T>> {
        match self.component_stream.try_read() {
            Ok(Some(value)) => Poll::Ready(Some(value)),
            Ok(None) if self.component_stream.is_closed() => Poll::Ready(None),
            _ => Poll::Pending,
        }
    }
}

/// Async-aware runtime that bridges Component Model and Rust async
pub struct AsyncRuntime {
    executor: &'static dyn WrtExecutor,
}

impl AsyncRuntime {
    /// Create runtime using the current executor
    pub fn new() -> Self {
        Self {
            executor: current_executor(),
        }
    }
    
    #[cfg(feature = "component-model-async")]
    /// Execute a Component Model future using Rust async
    pub async fn execute_component_future<T>(&self, future: ComponentFuture<T>) -> Result<T, ExecutorError>
    where
        T: Clone + Send + 'static,
    {
        let bridge = ComponentFutureBridge::new(future);
        bridge.await
    }
    
    #[cfg(feature = "component-model-async")]
    /// Spawn a Rust future that will complete a Component Model future
    pub fn spawn_for_component<F, T>(&self, rust_future: F, component_future: &mut ComponentFuture<T>) -> Result<(), ExecutorError>
    where
        F: Future<Output = T> + Send + 'static,
        T: Clone + Send + 'static,
    {
        let future_id = component_future.id();
        
        // Spawn the Rust future
        self.executor.spawn(Box::pin(async move {
            let result = rust_future.await;
            
            // When complete, update the Component Model future
            // In a real implementation, this would notify the Component Model runtime
            // For now, we just store the result
            let _ = result; // Component Model integration would happen here
        }))?;
        
        Ok(())
    }
}

#[cfg(feature = "component-model-async")]
/// Extension trait for Component Model values to work with async
pub trait ComponentAsyncExt {
    /// Convert a Component Model async value to a Rust future
    fn into_future<T>(self) -> Result<ComponentFutureBridge<T>, ExecutorError>
    where
        T: Clone + Send + 'static;
        
    /// Convert a Component Model stream to a bridged stream
    fn into_stream<T>(self) -> Result<ComponentStreamBridge<T>, ExecutorError>
    where
        T: Clone + Send + 'static;
}

#[cfg(feature = "component-model-async")]
impl ComponentAsyncExt for Value {
    fn into_future<T>(self) -> Result<ComponentFutureBridge<T>, ExecutorError>
    where
        T: Clone + Send + 'static,
    {
        // Note: In a real implementation, Value would have Future and Stream variants
        // For now, we simulate with U32 values
        match self {
            Value::U32(future_handle) => {
                // In a real implementation, we'd look up the Component Model future
                // For now, we create a placeholder
                let component_future = ComponentFuture::new(
                    FutureHandle(future_handle),
                    ValType::I32, // Placeholder type
                );
                Ok(ComponentFutureBridge::new(component_future))
            }
            _ => Err(ExecutorError::Custom("Value is not a future")),
        }
    }
    
    fn into_stream<T>(self) -> Result<ComponentStreamBridge<T>, ExecutorError>
    where
        T: Clone + Send + 'static,
    {
        match self {
            Value::U32(stream_handle) => {
                // In a real implementation, we'd look up the Component Model stream
                // For now, we create a placeholder
                let component_stream = ComponentStream::new(
                    StreamHandle(stream_handle),
                    ValType::I32, // Placeholder type
                );
                Ok(ComponentStreamBridge::new(component_stream))
            }
            _ => Err(ExecutorError::Custom("Value is not a stream")),
        }
    }
}

/// Helper to run async code in a Component Model context
pub fn with_async<F, T>(f: F) -> Result<T, ExecutorError>
where
    F: Future<Output = T>,
{
    let executor = current_executor();
    executor.block_on(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_async_runtime_creation() {
        let runtime = AsyncRuntime::new();
        assert!(runtime.executor.is_running());
    }
}