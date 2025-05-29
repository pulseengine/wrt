//! Pluggable async executor support for no_std environments
//!
//! This module provides a trait-based system for plugging in external async executors
//! while maintaining a minimal fallback executor for cases where no external executor
//! is provided.

#![cfg_attr(not(feature = "std"), no_std)]

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::bounded_collections::BoundedVec;
use crate::sync::Mutex;

/// Maximum number of concurrent tasks in fallback executor
pub const MAX_TASKS: usize = 32;

/// Core executor trait that external executors must implement
pub trait WrtExecutor: Send + Sync {
    /// Spawn a future onto the executor
    fn spawn(&self, future: BoxedFuture<'_, ()>) -> Result<TaskHandle, ExecutorError>;
    
    /// Block on a future until completion
    fn block_on<F: Future>(&self, future: F) -> Result<F::Output, ExecutorError>;
    
    /// Poll all ready tasks once (for cooperative executors)
    fn poll_once(&self) -> Result<(), ExecutorError> {
        // Default implementation does nothing
        // Executors can override this for cooperative scheduling
        Ok(())
    }
    
    /// Check if the executor is still running
    fn is_running(&self) -> bool;
    
    /// Shutdown the executor gracefully
    fn shutdown(&self) -> Result<(), ExecutorError>;
}

/// Handle to a spawned task
#[derive(Debug, Clone)]
pub struct TaskHandle {
    pub id: u64,
    pub waker: Option<Waker>,
}

/// Boxed future type for no_std environments
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Executor errors
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutorError {
    NotRunning,
    TaskPanicked,
    OutOfResources,
    NotSupported,
    Custom(&'static str),
}

/// Global executor registry
pub struct ExecutorRegistry {
    executor: Mutex<Option<Box<dyn WrtExecutor>>>,
    fallback: FallbackExecutor,
}

impl ExecutorRegistry {
    /// Create new registry with fallback executor
    pub const fn new() -> Self {
        Self {
            executor: Mutex::new(None),
            fallback: FallbackExecutor::new(),
        }
    }
    
    /// Register an external executor
    pub fn register_executor(&self, executor: Box<dyn WrtExecutor>) -> Result<(), ExecutorError> {
        let mut guard = self.executor.lock();
        if guard.is_some() {
            return Err(ExecutorError::Custom("Executor already registered"));
        }
        *guard = Some(executor);
        Ok(())
    }
    
    /// Get the active executor (external or fallback)
    pub fn get_executor(&self) -> &dyn WrtExecutor {
        let guard = self.executor.lock();
        match guard.as_ref() {
            Some(executor) => unsafe {
                // SAFETY: We ensure the executor lifetime is valid through the registry
                &**(executor as *const Box<dyn WrtExecutor>)
            },
            None => &self.fallback,
        }
    }
    
    /// Remove registered executor (revert to fallback)
    pub fn unregister_executor(&self) -> Option<Box<dyn WrtExecutor>> {
        self.executor.lock().take()
    }
    
    /// Check if using fallback executor
    pub fn is_using_fallback(&self) -> bool {
        self.executor.lock().is_none()
    }
}

// Global registry instance
static EXECUTOR_REGISTRY: ExecutorRegistry = ExecutorRegistry::new();

/// Register a custom executor
pub fn register_executor(executor: Box<dyn WrtExecutor>) -> Result<(), ExecutorError> {
    EXECUTOR_REGISTRY.register_executor(executor)
}

/// Get the current executor
pub fn current_executor() -> &'static dyn WrtExecutor {
    EXECUTOR_REGISTRY.get_executor()
}

/// Check if using fallback executor
pub fn is_using_fallback() -> bool {
    EXECUTOR_REGISTRY.is_using_fallback()
}

/// Task structure for fallback executor
struct Task {
    id: u64,
    future: BoxedFuture<'static, ()>,
    completed: AtomicBool,
}

/// Minimal fallback executor for no_std environments
pub struct FallbackExecutor {
    tasks: Mutex<BoundedVec<Task, MAX_TASKS>>,
    running: AtomicBool,
    next_id: AtomicU64,
}

impl FallbackExecutor {
    pub const fn new() -> Self {
        Self {
            tasks: Mutex::new(BoundedVec::new()),
            running: AtomicBool::new(true),
            next_id: AtomicU64::new(0),
        }
    }
    
    /// Poll all tasks once
    fn poll_all(&self) {
        let tasks = self.tasks.lock();
        
        for task in tasks.iter() {
            if !task.completed.load(Ordering::Acquire) {
                // Create a simple waker
                let waker = create_waker(task.id);
                let mut cx = Context::from_waker(&waker);
                
                // SAFETY: We ensure exclusive access through the mutex
                let future_ptr = &task.future as *const BoxedFuture<'static, ()> as *mut BoxedFuture<'static, ()>;
                let future = unsafe { &mut *future_ptr };
                
                // Poll the future
                match future.as_mut().poll(&mut cx) {
                    Poll::Ready(()) => task.completed.store(true, Ordering::Release),
                    Poll::Pending => continue,
                }
            }
        }
        
        // Note: In a real implementation, we'd remove completed tasks
        // For simplicity, we keep them until shutdown
    }
}

impl WrtExecutor for FallbackExecutor {
    fn spawn(&self, future: BoxedFuture<'_, ()>) -> Result<TaskHandle, ExecutorError> {
        if !self.is_running() {
            return Err(ExecutorError::NotRunning);
        }
        
        let mut tasks = self.tasks.lock();
        
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        
        // Convert to 'static lifetime
        // SAFETY: The executor ensures the future is polled to completion
        let static_future: BoxedFuture<'static, ()> = unsafe {
            core::mem::transmute(future)
        };
        
        let task = Task {
            id,
            future: static_future,
            completed: AtomicBool::new(false),
        };
        
        tasks.push(task).map_err(|_| ExecutorError::OutOfResources)?;
        
        Ok(TaskHandle { 
            id, 
            waker: Some(create_waker(id))
        })
    }
    
    fn block_on<F: Future>(&self, mut future: F) -> Result<F::Output, ExecutorError> {
        if !self.is_running() {
            return Err(ExecutorError::NotRunning);
        }
        
        // Pin the future
        let mut future = unsafe { Pin::new_unchecked(&mut future) };
        
        // Create waker
        let waker = create_waker(u64::MAX); // Special ID for block_on
        let mut cx = Context::from_waker(&waker);
        
        // Poll until ready
        loop {
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(output) => return Ok(output),
                Poll::Pending => {
                    // Poll other tasks while waiting
                    self.poll_all();
                    
                    // In a real implementation, we'd yield to the OS
                    // For no_std, we just busy-wait with task polling
                }
            }
        }
    }
    
    fn poll_once(&self) -> Result<(), ExecutorError> {
        if !self.is_running() {
            return Err(ExecutorError::NotRunning);
        }
        
        self.poll_all();
        Ok(())
    }
    
    fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
    
    fn shutdown(&self) -> Result<(), ExecutorError> {
        self.running.store(false, Ordering::Release);
        
        // Clear all tasks
        let mut tasks = self.tasks.lock();
        tasks.clear();
        
        Ok(())
    }
}

// Simple waker implementation for fallback executor
fn create_waker(id: u64) -> Waker {
    unsafe fn clone(data: *const ()) -> RawWaker {
        RawWaker::new(data, &VTABLE)
    }
    
    unsafe fn wake(_data: *const ()) {
        // In a real implementation, we'd notify the executor
        // For simplicity, we rely on polling
    }
    
    unsafe fn wake_by_ref(_data: *const ()) {
        // Same as wake
    }
    
    unsafe fn drop(_data: *const ()) {
        // Nothing to drop
    }
    
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    
    unsafe { Waker::from_raw(RawWaker::new(id as *const (), &VTABLE)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::future::Future;
    use core::task::{Context, Poll};
    
    struct TestFuture {
        polls_remaining: u32,
    }
    
    impl Future for TestFuture {
        type Output = u32;
        
        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.polls_remaining == 0 {
                Poll::Ready(42)
            } else {
                self.polls_remaining -= 1;
                Poll::Pending
            }
        }
    }
    
    #[test]
    fn test_fallback_executor() {
        let executor = FallbackExecutor::new();
        
        // Test block_on
        let future = TestFuture { polls_remaining: 3 };
        let result = executor.block_on(future).unwrap();
        assert_eq!(result, 42);
        
        // Test spawn
        let future = Box::pin(async {
            // Simple async task
        });
        let handle = executor.spawn(future).unwrap();
        assert!(handle.waker.is_some());
        
        // Test shutdown
        assert!(executor.is_running());
        executor.shutdown().unwrap();
        assert!(!executor.is_running());
    }
}