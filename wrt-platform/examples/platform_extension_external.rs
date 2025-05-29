//! Example: External Platform Extension
//!
//! This example demonstrates how external developers can create their own
//! platform implementations without modifying the core wrt-platform crate.

// Simulating an external crate that extends wrt-platform
mod external_platform {
    use wrt_platform::{PageAllocator, FutexLike, WASM_PAGE_SIZE};
    use wrt_error::{Error, ErrorKind};
    use core::ptr::NonNull;
    use core::sync::atomic::{AtomicU32, Ordering};
    use core::time::Duration;
    
    /// Example: Custom embedded RTOS platform
    pub struct CustomRtosAllocator {
        heap_start: usize,
        heap_size: usize,
        allocated: usize,
        max_pages: usize,
    }
    
    impl CustomRtosAllocator {
        pub fn new(heap_start: usize, heap_size: usize) -> Self {
            Self {
                heap_start,
                heap_size,
                allocated: 0,
                max_pages: heap_size / WASM_PAGE_SIZE,
            }
        }
    }
    
    impl PageAllocator for CustomRtosAllocator {
        fn allocate_pages(&mut self, pages: usize) -> Result<NonNull<u8>, Error> {
            let size = pages * WASM_PAGE_SIZE;
            
            if self.allocated + size > self.heap_size {
                return Err(Error::new(ErrorKind::Memory, "Heap exhausted"));
            }
            
            let ptr = (self.heap_start + self.allocated) as *mut u8;
            self.allocated += size;
            
            // Zero memory for security
            unsafe { core::ptr::write_bytes(ptr, 0, size) };
            
            NonNull::new(ptr).ok_or_else(|| 
                Error::new(ErrorKind::Memory, "Null pointer"))
        }
        
        fn deallocate_pages(&mut self, _ptr: NonNull<u8>, pages: usize) -> Result<(), Error> {
            // Simple allocator - just track the size
            let size = pages * WASM_PAGE_SIZE;
            self.allocated = self.allocated.saturating_sub(size);
            Ok(())
        }
        
        fn grow_pages(&mut self, old_ptr: NonNull<u8>, old_pages: usize, new_pages: usize) 
            -> Result<NonNull<u8>, Error> {
            // For simplicity, allocate new and copy
            let new_ptr = self.allocate_pages(new_pages)?;
            
            unsafe {
                core::ptr::copy_nonoverlapping(
                    old_ptr.as_ptr(),
                    new_ptr.as_ptr(),
                    old_pages * WASM_PAGE_SIZE
                );
            }
            
            self.deallocate_pages(old_ptr, old_pages)?;
            Ok(new_ptr)
        }
        
        fn allocated_pages(&self) -> usize {
            self.allocated / WASM_PAGE_SIZE
        }
        
        fn max_pages(&self) -> usize {
            self.max_pages
        }
    }
    
    /// Custom RTOS synchronization primitive
    pub struct CustomRtosFutex {
        value: AtomicU32,
        // In real implementation, would have RTOS-specific sync primitive
    }
    
    impl CustomRtosFutex {
        pub fn new(initial: u32) -> Self {
            Self {
                value: AtomicU32::new(initial),
            }
        }
    }
    
    impl FutexLike for CustomRtosFutex {
        fn wait(&self, expected: u32, timeout: Option<Duration>) -> Result<(), Error> {
            if self.value.load(Ordering::Acquire) != expected {
                return Ok(());
            }
            
            // In real implementation, would call RTOS wait function
            // For example: rtos_sem_wait(self.sem_handle, timeout)
            
            Ok(())
        }
        
        fn wake_one(&self) -> Result<u32, Error> {
            // In real implementation: rtos_sem_signal(self.sem_handle)
            Ok(1)
        }
        
        fn wake_all(&self) -> Result<u32, Error> {
            // In real implementation: rtos_sem_broadcast(self.sem_handle)
            Ok(u32::MAX)
        }
        
        fn load(&self, ordering: Ordering) -> u32 {
            self.value.load(ordering)
        }
        
        fn store(&self, value: u32, ordering: Ordering) {
            self.value.store(value, ordering);
        }
        
        fn compare_exchange_weak(&self, current: u32, new: u32, 
            success: Ordering, failure: Ordering) -> Result<u32, u32> {
            self.value.compare_exchange_weak(current, new, success, failure)
        }
    }
    
    /// High-level platform adapter
    pub struct CustomRtosPlatform {
        heap_start: usize,
        heap_size: usize,
    }
    
    impl CustomRtosPlatform {
        pub fn new(heap_start: usize, heap_size: usize) -> Self {
            Self { heap_start, heap_size }
        }
        
        pub fn create_allocator(&self) -> impl PageAllocator {
            CustomRtosAllocator::new(self.heap_start, self.heap_size)
        }
        
        pub fn create_futex(&self) -> impl FutexLike {
            CustomRtosFutex::new(0)
        }
        
        /// Platform capability detection
        pub fn capabilities(&self) -> PlatformCapabilities {
            PlatformCapabilities {
                name: "Custom RTOS",
                has_mmu: false,
                has_mpu: true,
                page_size: 4096,
                max_tasks: 32,
                priority_levels: 16,
                supports_smp: false,
            }
        }
    }
    
    #[derive(Debug)]
    pub struct PlatformCapabilities {
        pub name: &'static str,
        pub has_mmu: bool,
        pub has_mpu: bool,
        pub page_size: usize,
        pub max_tasks: usize,
        pub priority_levels: u8,
        pub supports_smp: bool,
    }
}

// Example usage of the external platform
fn main() {
    use external_platform::*;
    
    println!("=== External Platform Extension Example ===\n");
    
    // Simulate embedded system memory layout
    const HEAP_START: usize = 0x2000_0000; // 512MB mark
    const HEAP_SIZE: usize = 16 * 1024 * 1024; // 16MB heap
    
    // Create platform instance
    let platform = CustomRtosPlatform::new(HEAP_START, HEAP_SIZE);
    
    // Show platform capabilities
    let caps = platform.capabilities();
    println!("Platform: {}", caps.name);
    println!("Capabilities:");
    println!("  - MMU: {}", caps.has_mmu);
    println!("  - MPU: {}", caps.has_mpu);
    println!("  - Page size: {} bytes", caps.page_size);
    println!("  - Max tasks: {}", caps.max_tasks);
    println!("  - Priority levels: {}", caps.priority_levels);
    println!("  - SMP support: {}", caps.supports_smp);
    
    // Create allocator
    let mut allocator = platform.create_allocator();
    println!("\nAllocator created with {} max pages", allocator.max_pages());
    
    // Test allocation
    match allocator.allocate_pages(10) {
        Ok(ptr) => {
            println!("Allocated 10 pages at {:?}", ptr);
            println!("Current allocation: {} pages", allocator.allocated_pages());
            
            // Test grow
            match allocator.grow_pages(ptr, 10, 20) {
                Ok(new_ptr) => {
                    println!("Grew allocation to 20 pages at {:?}", new_ptr);
                    
                    // Deallocate
                    allocator.deallocate_pages(new_ptr, 20).unwrap();
                    println!("Deallocated all pages");
                }
                Err(e) => println!("Failed to grow: {}", e),
            }
        }
        Err(e) => println!("Failed to allocate: {}", e),
    }
    
    // Create synchronization primitive
    let futex = platform.create_futex();
    futex.store(42, core::sync::atomic::Ordering::Release);
    let value = futex.load(core::sync::atomic::Ordering::Acquire);
    println!("\nFutex test: stored and loaded {}", value);
    
    // Demonstrate how this integrates with WRT types
    use wrt_platform::{PageAllocator, FutexLike};
    
    fn use_with_wrt<A: PageAllocator, F: FutexLike>(
        allocator: &mut A,
        futex: &F,
    ) -> Result<(), wrt_error::Error> {
        println!("\n=== Using with WRT traits ===");
        
        // The external platform works seamlessly with WRT trait bounds
        let pages = allocator.allocate_pages(5)?;
        println!("Allocated {} pages through WRT trait", 5);
        
        futex.store(100, core::sync::atomic::Ordering::SeqCst);
        println!("Set futex value through WRT trait");
        
        allocator.deallocate_pages(pages, 5)?;
        Ok(())
    }
    
    let mut allocator = platform.create_allocator();
    let futex = platform.create_futex();
    
    if let Err(e) = use_with_wrt(&mut allocator, &futex) {
        println!("Error using with WRT: {}", e);
    }
    
    println!("\n=== Example Complete ===");
    println!("\nThis demonstrates how external developers can:");
    println!("1. Implement PageAllocator and FutexLike traits");
    println!("2. Create platform-specific abstractions");
    println!("3. Integrate seamlessly with WRT's trait system");
    println!("4. Add platform capability detection");
    println!("5. Package as a separate crate for distribution");
}