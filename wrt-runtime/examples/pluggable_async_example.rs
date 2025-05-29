//! Example demonstrating how to use the pluggable async executor system
//!
//! This example shows:
//! 1. Using the fallback executor (default)
//! 2. Plugging in a custom executor
//! 3. Integrating with Component Model async

#![cfg(feature = "async-api")]

use wrt_foundation::{
    async_executor::{register_executor, current_executor, is_using_fallback, WrtExecutor, ExecutorError, TaskHandle, BoxedFuture},
    async_bridge::{AsyncRuntime, with_async},
};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// Example custom executor that just prints what it's doing
struct CustomExecutor {
    name: &'static str,
}

impl WrtExecutor for CustomExecutor {
    fn spawn(&self, future: BoxedFuture<'_, ()>) -> Result<TaskHandle, ExecutorError> {
        println!("[{}] Spawning a future", self.name);
        // In a real implementation, you'd actually spawn the future
        Ok(TaskHandle { id: 42, waker: None })
    }
    
    fn block_on<F: Future>(&self, future: F) -> Result<F::Output, ExecutorError> {
        println!("[{}] Blocking on a future", self.name);
        // In a real implementation, you'd run the future to completion
        // For this example, we'll return an error
        Err(ExecutorError::NotSupported)
    }
    
    fn is_running(&self) -> bool {
        true
    }
    
    fn shutdown(&self) -> Result<(), ExecutorError> {
        println!("[{}] Shutting down", self.name);
        Ok(())
    }
}

/// Simple async function for testing
async fn hello_async() -> &'static str {
    "Hello from async!"
}

/// Example future that yields a few times before completing
struct CountdownFuture {
    count: u32,
}

impl Future for CountdownFuture {
    type Output = &'static str;
    
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.count == 0 {
            Poll::Ready("Countdown complete!")
        } else {
            self.count -= 1;
            Poll::Pending
        }
    }
}

fn main() {
    println!("=== Pluggable Async Executor Example ===\n");
    
    // 1. Check initial state - should be using fallback
    println!("1. Initial state:");
    println!("   Using fallback executor: {}", is_using_fallback());
    
    // 2. Use the fallback executor
    println!("\n2. Using fallback executor:");
    let executor = current_executor();
    
    // Block on a simple future
    match executor.block_on(hello_async()) {
        Ok(result) => println!("   Result: {}", result),
        Err(e) => println!("   Error: {:?}", e),
    }
    
    // Block on a countdown future
    let countdown = CountdownFuture { count: 3 };
    match executor.block_on(countdown) {
        Ok(result) => println!("   Countdown result: {}", result),
        Err(e) => println!("   Countdown error: {:?}", e),
    }
    
    // 3. Register a custom executor
    println!("\n3. Registering custom executor:");
    let custom = Box::new(CustomExecutor { name: "MyExecutor" });
    match register_executor(custom) {
        Ok(()) => println!("   Successfully registered custom executor"),
        Err(e) => println!("   Failed to register: {:?}", e),
    }
    
    println!("   Using fallback executor: {}", is_using_fallback());
    
    // 4. Try using the custom executor
    println!("\n4. Using custom executor:");
    let executor = current_executor();
    
    match executor.block_on(hello_async()) {
        Ok(result) => println!("   Result: {}", result),
        Err(e) => println!("   Error: {:?}", e),
    }
    
    // 5. Using the AsyncRuntime helper
    println!("\n5. Using AsyncRuntime:");
    let runtime = AsyncRuntime::new();
    println!("   Created AsyncRuntime with current executor");
    
    // 6. Using the with_async helper
    println!("\n6. Using with_async helper:");
    match with_async(async {
        println!("   Inside async block!");
        "Async block result"
    }) {
        Ok(result) => println!("   Result: {}", result),
        Err(e) => println!("   Error: {:?}", e),
    }
    
    println!("\n=== Example Complete ===");
}

#[cfg(not(feature = "async-api"))]
fn main() {
    println!("This example requires the 'async-api' feature to be enabled.");
    println!("Try running with: cargo run --example pluggable_async_example --features async-api");
}