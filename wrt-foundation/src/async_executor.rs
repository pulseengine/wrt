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

#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

#[cfg(any(feature = "std", feature = "alloc"))]
use alloc::boxed::Box;

use crate::bounded::BoundedVec;
use crate::NoStdProvider;
use wrt_sync::Mutex;

/// Maximum number of concurrent tasks in fallback executor
pub const MAX_TASKS: usize = 32;

/// Core executor trait that external executors must implement
pub trait WrtExecutor: Send + Sync {
    /// Spawn a future onto the executor
    fn spawn(&self, future: BoxedFuture<'_, ()>) -> Result<TaskHandle, ExecutorError>;
    
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

/// Boxed future type for environments with allocation
#[cfg(any(feature = "std", feature = "alloc"))]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// For pure no_std environments, we use a simpler approach
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub type BoxedFuture<'a, T> = Pin<&'a mut dyn Future<Output = T>>;

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
    #[cfg(any(feature = "std", feature = "alloc"))]
    executor: Mutex<Option<Box<dyn WrtExecutor>>>,
    fallback: FallbackExecutor,
}

impl ExecutorRegistry {
    /// Create new registry with fallback executor
    pub fn new() -> Self {
        Self {
            #[cfg(any(feature = "std", feature = "alloc"))]
            executor: Mutex::new(None),
            fallback: FallbackExecutor::new(),
        }
    }
    
    /// Register an external executor
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn register_executor(&self, executor: Box<dyn WrtExecutor>) -> Result<(), ExecutorError> {
        let mut guard = self.executor.lock();
        if guard.is_some() {
            return Err(ExecutorError::Custom("Executor already registered"));
        }
        *guard = Some(executor);
        Ok(())
    }
    
    /// Register an external executor (no-op in pure no_std)
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub fn register_executor(&self, _executor: ()) -> Result<(), ExecutorError> {
        Err(ExecutorError::Custom("External executors require alloc feature"))
    }
    
    /// Get the active executor (external or fallback)
    pub fn get_executor(&self) -> &dyn WrtExecutor {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let guard = self.executor.lock();
            match guard.as_ref() {
                Some(executor) => unsafe {
                    // SAFETY: We ensure the executor lifetime is valid through the registry
                    &**(executor as *const Box<dyn WrtExecutor>)
                },
                None => &self.fallback,
            }
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            &self.fallback
        }
    }
    
    /// Remove registered executor (revert to fallback)
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn unregister_executor(&self) -> Option<Box<dyn WrtExecutor>> {
        self.executor.lock().take()
    }
    
    /// Remove registered executor (no-op in pure no_std)
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub fn unregister_executor(&self) -> Option<()> {
        None
    }
    
    /// Check if using fallback executor
    pub fn is_using_fallback(&self) -> bool {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            self.executor.lock().is_none()
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            true
        }
    }
}

use core::sync::atomic::{AtomicPtr, Ordering as AtomicOrdering};
use core::ptr;

// Global registry instance using atomic pointer for thread safety
static EXECUTOR_REGISTRY_PTR: AtomicPtr<ExecutorRegistry> = AtomicPtr::new(ptr::null_mut());

fn get_or_init_registry() -> &'static ExecutorRegistry {
    let ptr = EXECUTOR_REGISTRY_PTR.load(AtomicOrdering::Acquire);
    if ptr.is_null() {
        // Initialize registry - this is safe for single-threaded and no_std environments
        let registry = Box::leak(Box::new(ExecutorRegistry::new()));
        let expected = ptr::null_mut();
        match EXECUTOR_REGISTRY_PTR.compare_exchange_weak(
            expected,
            registry as *mut ExecutorRegistry,
            AtomicOrdering::Release,
            AtomicOrdering::Relaxed,
        ) {
            Ok(_) => registry,
            Err(_) => {
                // Another thread beat us, use their registry
                unsafe { &*EXECUTOR_REGISTRY_PTR.load(AtomicOrdering::Acquire) }
            }
        }
    } else {
        unsafe { &*ptr }
    }
}

/// Register a custom executor
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn register_executor(executor: Box<dyn WrtExecutor>) -> Result<(), ExecutorError> {
    get_or_init_registry().register_executor(executor)
}

/// Register a custom executor (no-op in pure no_std)
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub fn register_executor(_executor: ()) -> Result<(), ExecutorError> {
    get_or_init_registry().register_executor(())
}

/// Get the current executor
pub fn current_executor() -> &'static dyn WrtExecutor {
    get_or_init_registry().get_executor()
}

/// Check if using fallback executor
pub fn is_using_fallback() -> bool {
    get_or_init_registry().is_using_fallback()
}

/// Block on a future using the current executor
pub fn block_on<F: Future>(future: F) -> Result<F::Output, ExecutorError> {
    let registry = get_or_init_registry();
    // For now, we'll implement this using the fallback executor directly
    // In a real implementation, this would be more sophisticated
    let fallback = &registry.fallback;
    fallback.block_on_impl(future)
}

/// Task structure for fallback executor
struct Task {
    id: u64,
    future: BoxedFuture<'static, ()>,
    completed: AtomicBool,
}

/// Minimal fallback executor for no_std environments
pub struct FallbackExecutor {
    tasks: Mutex<BoundedVec<Task, MAX_TASKS, NoStdProvider>>,
    running: AtomicBool,
    next_id: AtomicU64,
}

impl FallbackExecutor {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(BoundedVec::new(NoStdProvider).unwrap()),
            running: AtomicBool::new(true),
            next_id: AtomicU64::new(0),
        }
    }
    
    /// Block on a future until completion (internal implementation)
    pub fn block_on_impl<F: Future>(&self, mut future: F) -> Result<F::Output, ExecutorError> {
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