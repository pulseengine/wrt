//! Example demonstrating VxWorks platform usage with WRT
//!
//! This example shows how to use the VxWorks platform implementation
//! for both RTP (Real-Time Process) and LKM (Loadable Kernel Module) contexts.

use wrt_platform::{
    PageAllocator, FutexLike, WASM_PAGE_SIZE,
    vxworks_memory::{VxWorksAllocator, VxWorksAllocatorBuilder, VxWorksContext},
    vxworks_sync::{VxWorksFutex, VxWorksFutexBuilder},
    vxworks_threading::{VxWorksThread, VxWorksThreadBuilder, VxWorksThreadConfig},
};
use core::sync::atomic::Ordering;
use core::time::Duration;

#[cfg(not(target_os = "vxworks"))]
fn main() {
    println!("This example is designed to run on VxWorks. Showing mock behavior.");
    
    // Example 1: RTP Memory Allocation
    example_rtp_memory_allocation();
    
    // Example 2: LKM Memory Allocation with Partitions
    example_lkm_memory_allocation();
    
    // Example 3: Synchronization Primitives
    example_synchronization();
    
    // Example 4: Threading
    example_threading();
    
    // Example 5: Complete Integration
    example_complete_integration();
}

#[cfg(target_os = "vxworks")]
fn main() {
    println!("Running VxWorks platform examples...");
    
    // Detect execution context
    let context = if is_kernel_context() {
        VxWorksContext::Lkm
    } else {
        VxWorksContext::Rtp
    };
    
    println!("Detected context: {:?}", context);
    
    match context {
        VxWorksContext::Rtp => {
            example_rtp_memory_allocation();
            example_synchronization();
            example_threading();
        }
        VxWorksContext::Lkm => {
            example_lkm_memory_allocation();
            example_synchronization();
        }
    }
    
    example_complete_integration();
}

/// Example 1: RTP Memory Allocation
fn example_rtp_memory_allocation() {
    println!("\n=== Example 1: RTP Memory Allocation ===");
    
    // Create RTP allocator
    let mut allocator = VxWorksAllocatorBuilder::new()
        .context(VxWorksContext::Rtp)
        .max_pages(100)
        .enable_guard_pages(false) // RTP doesn't typically use guard pages
        .build()
        .expect("Failed to create RTP allocator");
    
    println!("Created RTP allocator with max {} pages", allocator.max_pages());
    
    // Allocate some WASM pages
    let pages_to_allocate = 10;
    let ptr = allocator.allocate_pages(pages_to_allocate)
        .expect("Failed to allocate pages");
    
    println!("Allocated {} pages at {:?}", pages_to_allocate, ptr);
    println!("Total allocated: {} pages", allocator.allocated_pages());
    
    // Use the memory
    unsafe {
        let slice = core::slice::from_raw_parts_mut(
            ptr.as_ptr(),
            pages_to_allocate * WASM_PAGE_SIZE
        );
        
        // Write some data
        slice[0] = 0x42;
        slice[WASM_PAGE_SIZE] = 0x43; // Second page
        
        println!("Wrote data to allocated memory");
    }
    
    // Grow the allocation
    let new_pages = 15;
    let new_ptr = allocator.grow_pages(ptr, pages_to_allocate, new_pages)
        .expect("Failed to grow allocation");
    
    println!("Grew allocation from {} to {} pages", pages_to_allocate, new_pages);
    
    // Deallocate
    allocator.deallocate_pages(new_ptr, new_pages)
        .expect("Failed to deallocate");
    
    println!("Deallocated all pages");
    println!("Final allocated: {} pages", allocator.allocated_pages());
}

/// Example 2: LKM Memory Allocation with Partitions
fn example_lkm_memory_allocation() {
    println!("\n=== Example 2: LKM Memory Allocation ===");
    
    // Create LKM allocator with dedicated partition
    let mut allocator = VxWorksAllocatorBuilder::new()
        .context(VxWorksContext::Lkm)
        .max_pages(50)
        .use_dedicated_partition(true)
        .partition_size(50 * WASM_PAGE_SIZE)
        .enable_guard_pages(true)
        .build()
        .expect("Failed to create LKM allocator");
    
    println!("Created LKM allocator with dedicated partition");
    
    // Allocate pages from partition
    let pages = 5;
    let ptr = allocator.allocate_pages(pages)
        .expect("Failed to allocate from partition");
    
    println!("Allocated {} pages from partition", pages);
    
    // Demonstrate deterministic allocation
    let start = std::time::Instant::now();
    for i in 0..10 {
        let temp_ptr = allocator.allocate_pages(1)
            .expect("Failed to allocate");
        allocator.deallocate_pages(temp_ptr, 1)
            .expect("Failed to deallocate");
    }
    let elapsed = start.elapsed();
    
    println!("10 allocate/deallocate cycles took {:?} (deterministic)", elapsed);
    
    // Clean up
    allocator.deallocate_pages(ptr, pages)
        .expect("Failed to deallocate");
}

/// Example 3: Synchronization Primitives
fn example_synchronization() {
    println!("\n=== Example 3: Synchronization ===");
    
    // Create futex for current context
    let futex = VxWorksFutexBuilder::new(VxWorksContext::Rtp)
        .initial_value(0)
        .build()
        .expect("Failed to create futex");
    
    println!("Created VxWorks futex");
    
    // Test atomic operations
    futex.store(42, Ordering::Release);
    let value = futex.load(Ordering::Acquire);
    println!("Stored and loaded value: {}", value);
    
    // Test compare-exchange
    match futex.compare_exchange_weak(42, 100, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(old) => println!("Successfully changed {} to 100", old),
        Err(actual) => println!("Compare-exchange failed, actual value: {}", actual),
    }
    
    // Test wait/wake (in real scenario, would be cross-thread)
    println!("Testing wait with timeout...");
    let result = futex.wait(100, Some(Duration::from_millis(10)));
    match result {
        Ok(()) => println!("Wait completed (value changed or timeout)"),
        Err(e) => println!("Wait failed: {}", e),
    }
    
    // Wake operations
    let woken = futex.wake_one().expect("Failed to wake");
    println!("Woke {} waiters", woken);
}

/// Example 4: Threading (RTP only)
#[cfg(feature = "alloc")]
fn example_threading() {
    println!("\n=== Example 4: Threading ===");
    
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    // Shared state
    let counter = Arc::new(Mutex::new(0));
    
    // Spawn VxWorks thread
    let counter_clone = counter.clone();
    let thread = VxWorksThreadBuilder::new(VxWorksContext::Rtp)
        .stack_size(128 * 1024) // 128KB stack
        .name("worker_thread")
        .floating_point(true)
        .spawn(move || {
            println!("VxWorks thread started!");
            
            for i in 0..10 {
                let mut count = counter_clone.lock().unwrap();
                *count += 1;
                println!("Thread: count = {}", *count);
                drop(count);
                
                VxWorksThread::sleep_ms(10).unwrap();
            }
            
            println!("VxWorks thread finished!");
        })
        .expect("Failed to spawn thread");
    
    // Main thread work
    thread::sleep(Duration::from_millis(50));
    
    // Check final count
    let final_count = *counter.lock().unwrap();
    println!("Final count: {}", final_count);
    
    // Join thread
    thread.join().expect("Failed to join thread");
}

#[cfg(not(feature = "alloc"))]
fn example_threading() {
    println!("\n=== Example 4: Threading (requires alloc feature) ===");
}

/// Example 5: Complete Integration
fn example_complete_integration() {
    println!("\n=== Example 5: Complete WRT Integration ===");
    
    // This example shows how VxWorks platform integrates with WRT runtime
    
    // 1. Create platform-specific components
    let mut allocator = VxWorksAllocatorBuilder::new()
        .context(VxWorksContext::Rtp)
        .max_pages(1024)
        .build()
        .expect("Failed to create allocator");
    
    let futex = VxWorksFutexBuilder::new(VxWorksContext::Rtp)
        .build()
        .expect("Failed to create futex");
    
    println!("Created VxWorks platform components");
    
    // 2. Simulate WASM memory management
    struct WasmMemory<A: PageAllocator> {
        allocator: A,
        base: Option<core::ptr::NonNull<u8>>,
        pages: usize,
    }
    
    impl<A: PageAllocator> WasmMemory<A> {
        fn new(mut allocator: A, initial_pages: usize) -> Result<Self, wrt_error::Error> {
            let base = allocator.allocate_pages(initial_pages)?;
            Ok(Self {
                allocator,
                base: Some(base),
                pages: initial_pages,
            })
        }
        
        fn grow(&mut self, delta: usize) -> Result<(), wrt_error::Error> {
            if let Some(base) = self.base {
                let new_pages = self.pages + delta;
                let new_base = self.allocator.grow_pages(base, self.pages, new_pages)?;
                self.base = Some(new_base);
                self.pages = new_pages;
            }
            Ok(())
        }
        
        fn size(&self) -> usize {
            self.pages
        }
    }
    
    // Create WASM memory
    let mut wasm_memory = WasmMemory::new(allocator, 10)
        .expect("Failed to create WASM memory");
    
    println!("Created WASM memory with {} pages", wasm_memory.size());
    
    // Grow memory
    wasm_memory.grow(5).expect("Failed to grow memory");
    println!("Grew WASM memory to {} pages", wasm_memory.size());
    
    // 3. Demonstrate platform capabilities
    println!("\nPlatform Capabilities:");
    println!("- Execution contexts: RTP and LKM");
    println!("- Memory: partitions, guard pages, deterministic allocation");
    println!("- Sync: POSIX (RTP) and VxWorks (LKM) primitives");
    println!("- Threading: tasks (LKM) and pthreads (RTP)");
    println!("- Real-time: priority inheritance, deterministic behavior");
}

#[cfg(target_os = "vxworks")]
fn is_kernel_context() -> bool {
    // Check if we can access kernel-only functions
    extern "C" {
        fn kernelId() -> i32;
    }
    
    unsafe { kernelId() != 0 }
}

#[cfg(not(target_os = "vxworks"))]
fn is_kernel_context() -> bool {
    false
}