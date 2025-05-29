//! Complete External Platform Implementation Example
//!
//! This example demonstrates a full external platform crate implementation,
//! showing how external developers can create their own platform support
//! without modifying core WRT.

// This simulates what would be in an external crate like "wrt-platform-myos"
mod wrt_platform_myos {
    use wrt_platform::{PageAllocator, FutexLike, WASM_PAGE_SIZE};
    use wrt_error::{Error, ErrorKind};
    use core::ptr::NonNull;
    use core::sync::atomic::{AtomicU32, Ordering};
    use core::time::Duration;
    
    #[cfg(feature = "alloc")]
    use alloc::{vec::Vec, string::String, boxed::Box};
    
    /// Platform configuration
    #[derive(Clone, Debug)]
    pub struct MyOsConfig {
        pub max_memory_pages: usize,
        pub enable_large_pages: bool,
        pub enable_memory_protection: bool,
        pub thread_stack_size: usize,
        pub priority_inheritance: bool,
    }
    
    impl Default for MyOsConfig {
        fn default() -> Self {
            Self {
                max_memory_pages: 1024,
                enable_large_pages: false,
                enable_memory_protection: true,
                thread_stack_size: 64 * 1024,
                priority_inheritance: true,
            }
        }
    }
    
    /// Platform capabilities detection
    #[derive(Debug, Clone)]
    pub struct PlatformCapabilities {
        pub os_name: &'static str,
        pub os_version: String,
        pub cpu_cores: usize,
        pub total_memory: usize,
        pub page_sizes: Vec<usize>,
        pub has_memory_protection: bool,
        pub has_large_page_support: bool,
        pub max_threads: usize,
        pub supports_priority_inheritance: bool,
    }
    
    /// MyOS memory allocator
    pub struct MyOsAllocator {
        config: MyOsConfig,
        allocated_pages: usize,
        allocations: Vec<(NonNull<u8>, usize)>,
        heap_base: usize,
        heap_size: usize,
        heap_offset: usize,
    }
    
    impl MyOsAllocator {
        fn new(config: MyOsConfig) -> Result<Self, Error> {
            // Simulate platform-specific heap initialization
            let heap_size = config.max_memory_pages * WASM_PAGE_SIZE;
            let heap_base = Self::allocate_heap(heap_size)?;
            
            Ok(Self {
                config,
                allocated_pages: 0,
                allocations: Vec::new(),
                heap_base,
                heap_size,
                heap_offset: 0,
            })
        }
        
        #[cfg(target_os = "myos")]
        fn allocate_heap(size: usize) -> Result<usize, Error> {
            // This would call actual MyOS APIs
            extern "C" {
                fn myos_heap_create(size: usize, flags: u32) -> *mut u8;
            }
            
            let ptr = unsafe {
                myos_heap_create(
                    size,
                    if self.config.enable_large_pages { 0x1 } else { 0x0 }
                )
            };
            
            if ptr.is_null() {
                return Err(Error::new(ErrorKind::Platform, "MyOS heap creation failed"));
            }
            
            Ok(ptr as usize)
        }
        
        #[cfg(not(target_os = "myos"))]
        fn allocate_heap(size: usize) -> Result<usize, Error> {
            // Development fallback using system allocator
            use core::alloc::{alloc, Layout};
            
            let layout = Layout::from_size_align(size, WASM_PAGE_SIZE)
                .map_err(|_| Error::new(ErrorKind::Memory, "Invalid layout"))?;
            
            let ptr = unsafe { alloc(layout) };
            if ptr.is_null() {
                return Err(Error::new(ErrorKind::Memory, "System allocation failed"));
            }
            
            Ok(ptr as usize)
        }
        
        fn allocate_from_heap(&mut self, size: usize) -> Result<NonNull<u8>, Error> {
            // Align to WASM page boundary
            let aligned_offset = (self.heap_offset + WASM_PAGE_SIZE - 1) & !(WASM_PAGE_SIZE - 1);
            
            if aligned_offset + size > self.heap_size {
                return Err(Error::new(ErrorKind::Memory, "Heap exhausted"));
            }
            
            let ptr = (self.heap_base + aligned_offset) as *mut u8;
            self.heap_offset = aligned_offset + size;
            
            // Zero memory for security
            unsafe { core::ptr::write_bytes(ptr, 0, size) };
            
            // Apply memory protection if enabled
            if self.config.enable_memory_protection {
                self.apply_memory_protection(ptr, size)?;
            }
            
            NonNull::new(ptr).ok_or_else(|| 
                Error::new(ErrorKind::Memory, "Null pointer"))
        }
        
        #[cfg(target_os = "myos")]
        fn apply_memory_protection(&self, ptr: *mut u8, size: usize) -> Result<(), Error> {
            extern "C" {
                fn myos_memory_protect(addr: *mut u8, size: usize, prot: u32) -> i32;
            }
            
            const PROT_READ_WRITE: u32 = 0x3;
            let result = unsafe { myos_memory_protect(ptr, size, PROT_READ_WRITE) };
            
            if result != 0 {
                return Err(Error::new(ErrorKind::Platform, "Memory protection failed"));
            }
            
            Ok(())
        }
        
        #[cfg(not(target_os = "myos"))]
        fn apply_memory_protection(&self, _ptr: *mut u8, _size: usize) -> Result<(), Error> {
            // No-op on development platforms
            Ok(())
        }
    }
    
    impl PageAllocator for MyOsAllocator {
        fn allocate_pages(&mut self, pages: usize) -> Result<NonNull<u8>, Error> {
            if self.allocated_pages + pages > self.config.max_memory_pages {
                return Err(Error::new(ErrorKind::Memory, "Page limit exceeded"));
            }
            
            let size = pages * WASM_PAGE_SIZE;
            let ptr = self.allocate_from_heap(size)?;
            
            self.allocated_pages += pages;
            self.allocations.push((ptr, pages));
            
            Ok(ptr)
        }
        
        fn deallocate_pages(&mut self, ptr: NonNull<u8>, pages: usize) -> Result<(), Error> {
            // Find and remove allocation record
            let index = self.allocations.iter()
                .position(|(p, s)| *p == ptr && *s == pages)
                .ok_or_else(|| Error::new(ErrorKind::Memory, "Invalid deallocation"))?;
            
            self.allocations.remove(index);
            self.allocated_pages = self.allocated_pages.saturating_sub(pages);
            
            // In a real allocator, you might need to actually free memory
            // For this simple allocator, we just track it
            
            Ok(())
        }
        
        fn grow_pages(&mut self, old_ptr: NonNull<u8>, old_pages: usize, new_pages: usize) 
            -> Result<NonNull<u8>, Error> {
            if new_pages <= old_pages {
                return Ok(old_ptr);
            }
            
            // Allocate new memory
            let new_ptr = self.allocate_pages(new_pages)?;
            
            // Copy existing data
            unsafe {
                core::ptr::copy_nonoverlapping(
                    old_ptr.as_ptr(),
                    new_ptr.as_ptr(),
                    old_pages * WASM_PAGE_SIZE
                );
            }
            
            // Free old memory
            self.deallocate_pages(old_ptr, old_pages)?;
            
            Ok(new_ptr)
        }
        
        fn allocated_pages(&self) -> usize {
            self.allocated_pages
        }
        
        fn max_pages(&self) -> usize {
            self.config.max_memory_pages
        }
    }
    
    /// MyOS synchronization primitive
    pub struct MyOsFutex {
        value: AtomicU32,
        #[cfg(target_os = "myos")]
        semaphore: MyOsSemaphore,
        priority_inheritance: bool,
    }
    
    #[cfg(target_os = "myos")]
    struct MyOsSemaphore {
        handle: u32, // Platform semaphore handle
    }
    
    impl MyOsFutex {
        pub fn new(initial: u32, priority_inheritance: bool) -> Result<Self, Error> {
            #[cfg(target_os = "myos")]
            {
                let semaphore = MyOsSemaphore::create(priority_inheritance)?;
                Ok(Self {
                    value: AtomicU32::new(initial),
                    semaphore,
                    priority_inheritance,
                })
            }
            
            #[cfg(not(target_os = "myos"))]
            {
                Ok(Self {
                    value: AtomicU32::new(initial),
                    priority_inheritance,
                })
            }
        }
    }
    
    #[cfg(target_os = "myos")]
    impl MyOsSemaphore {
        fn create(priority_inheritance: bool) -> Result<Self, Error> {
            extern "C" {
                fn myos_sem_create(flags: u32) -> u32;
            }
            
            let flags = if priority_inheritance { 0x1 } else { 0x0 };
            let handle = unsafe { myos_sem_create(flags) };
            
            if handle == 0 {
                return Err(Error::new(ErrorKind::Platform, "Semaphore creation failed"));
            }
            
            Ok(Self { handle })
        }
        
        fn wait(&self, timeout_ms: u32) -> Result<(), Error> {
            extern "C" {
                fn myos_sem_wait(handle: u32, timeout: u32) -> i32;
            }
            
            let result = unsafe { myos_sem_wait(self.handle, timeout_ms) };
            if result != 0 {
                return Err(Error::new(ErrorKind::Platform, "Semaphore wait failed"));
            }
            
            Ok(())
        }
        
        fn signal(&self) -> Result<(), Error> {
            extern "C" {
                fn myos_sem_signal(handle: u32) -> i32;
            }
            
            let result = unsafe { myos_sem_signal(self.handle) };
            if result != 0 {
                return Err(Error::new(ErrorKind::Platform, "Semaphore signal failed"));
            }
            
            Ok(())
        }
        
        fn broadcast(&self) -> Result<u32, Error> {
            extern "C" {
                fn myos_sem_broadcast(handle: u32) -> i32;
            }
            
            let result = unsafe { myos_sem_broadcast(self.handle) };
            if result < 0 {
                return Err(Error::new(ErrorKind::Platform, "Semaphore broadcast failed"));
            }
            
            Ok(result as u32)
        }
    }
    
    #[cfg(target_os = "myos")]
    impl Drop for MyOsSemaphore {
        fn drop(&mut self) {
            extern "C" {
                fn myos_sem_destroy(handle: u32);
            }
            
            unsafe { myos_sem_destroy(self.handle) };
        }
    }
    
    impl FutexLike for MyOsFutex {
        fn wait(&self, expected: u32, timeout: Option<Duration>) -> Result<(), Error> {
            if self.value.load(Ordering::Acquire) != expected {
                return Ok(());
            }
            
            #[cfg(target_os = "myos")]
            {
                let timeout_ms = timeout
                    .map(|d| d.as_millis() as u32)
                    .unwrap_or(u32::MAX); // Infinite timeout
                
                self.semaphore.wait(timeout_ms)
            }
            
            #[cfg(not(target_os = "myos"))]
            {
                // Development fallback - just check if value still matches
                if self.value.load(Ordering::Acquire) == expected {
                    // Simulate brief wait
                    std::thread::sleep(Duration::from_millis(1));
                }
                Ok(())
            }
        }
        
        fn wake_one(&self) -> Result<u32, Error> {
            #[cfg(target_os = "myos")]
            {
                self.semaphore.signal()?;
                Ok(1)
            }
            
            #[cfg(not(target_os = "myos"))]
            {
                Ok(1)
            }
        }
        
        fn wake_all(&self) -> Result<u32, Error> {
            #[cfg(target_os = "myos")]
            {
                self.semaphore.broadcast()
            }
            
            #[cfg(not(target_os = "myos"))]
            {
                Ok(4) // Simulate waking multiple waiters
            }
        }
        
        fn load(&self, ordering: Ordering) -> u32 {
            self.value.load(ordering)
        }
        
        fn store(&self, value: u32, ordering: Ordering) {
            self.value.store(value, ordering);
        }
        
        fn compare_exchange_weak(
            &self,
            current: u32,
            new: u32,
            success: Ordering,
            failure: Ordering,
        ) -> Result<u32, u32> {
            self.value.compare_exchange_weak(current, new, success, failure)
        }
    }
    
    /// High-level platform interface
    pub struct MyOsPlatform {
        config: MyOsConfig,
        capabilities: PlatformCapabilities,
    }
    
    impl MyOsPlatform {
        pub fn new(config: MyOsConfig) -> Self {
            let capabilities = Self::detect_capabilities();
            Self { config, capabilities }
        }
        
        pub fn detect() -> Result<Self, Error> {
            if !Self::is_platform_available() {
                return Err(Error::new(
                    ErrorKind::Platform,
                    "MyOS platform not available"
                ));
            }
            
            let config = MyOsConfig::default();
            Ok(Self::new(config))
        }
        
        pub fn capabilities(&self) -> &PlatformCapabilities {
            &self.capabilities
        }
        
        pub fn create_allocator(&self) -> Result<MyOsAllocator, Error> {
            MyOsAllocator::new(self.config.clone())
        }
        
        pub fn create_futex(&self) -> Result<MyOsFutex, Error> {
            MyOsFutex::new(0, self.config.priority_inheritance)
        }
        
        pub fn create_allocator_boxed(&self) -> Result<Box<dyn PageAllocator>, Error> {
            Ok(Box::new(self.create_allocator()?))
        }
        
        pub fn create_futex_boxed(&self) -> Result<Box<dyn FutexLike>, Error> {
            Ok(Box::new(self.create_futex()?))
        }
        
        pub fn is_platform_available() -> bool {
            #[cfg(target_os = "myos")]
            {
                // Check if MyOS runtime is available
                extern "C" {
                    fn myos_get_version() -> u32;
                }
                
                unsafe { myos_get_version() != 0 }
            }
            
            #[cfg(not(target_os = "myos"))]
            {
                false
            }
        }
        
        fn detect_capabilities() -> PlatformCapabilities {
            #[cfg(target_os = "myos")]
            {
                extern "C" {
                    fn myos_get_version_string() -> *const i8;
                    fn myos_get_cpu_count() -> u32;
                    fn myos_get_total_memory() -> u64;
                    fn myos_get_max_threads() -> u32;
                }
                
                let version_ptr = unsafe { myos_get_version_string() };
                let version = if !version_ptr.is_null() {
                    unsafe {
                        std::ffi::CStr::from_ptr(version_ptr)
                            .to_string_lossy()
                            .into_owned()
                    }
                } else {
                    "Unknown".to_string()
                };
                
                PlatformCapabilities {
                    os_name: "MyOS",
                    os_version: version,
                    cpu_cores: unsafe { myos_get_cpu_count() as usize },
                    total_memory: unsafe { myos_get_total_memory() as usize },
                    page_sizes: vec![4096, 2 * 1024 * 1024], // 4KB, 2MB
                    has_memory_protection: true,
                    has_large_page_support: true,
                    max_threads: unsafe { myos_get_max_threads() as usize },
                    supports_priority_inheritance: true,
                }
            }
            
            #[cfg(not(target_os = "myos"))]
            {
                PlatformCapabilities {
                    os_name: "MyOS (Development)",
                    os_version: "Dev 1.0".to_string(),
                    cpu_cores: 4,
                    total_memory: 8 * 1024 * 1024 * 1024, // 8GB
                    page_sizes: vec![4096],
                    has_memory_protection: false,
                    has_large_page_support: false,
                    max_threads: 256,
                    supports_priority_inheritance: false,
                }
            }
        }
        
        pub fn recommended_config(&self) -> MyOsConfig {
            let memory_pages = core::cmp::min(
                self.capabilities.total_memory / WASM_PAGE_SIZE / 4, // 25% of RAM
                4096 // Cap at 256MB
            );
            
            MyOsConfig {
                max_memory_pages: memory_pages,
                enable_large_pages: self.capabilities.has_large_page_support,
                enable_memory_protection: self.capabilities.has_memory_protection,
                thread_stack_size: 64 * 1024,
                priority_inheritance: self.capabilities.supports_priority_inheritance,
            }
        }
    }
    
    /// Builder for platform configuration
    pub struct MyOsPlatformBuilder {
        config: MyOsConfig,
    }
    
    impl MyOsPlatformBuilder {
        pub fn new() -> Self {
            Self {
                config: MyOsConfig::default(),
            }
        }
        
        pub fn memory_pages(mut self, pages: usize) -> Self {
            self.config.max_memory_pages = pages;
            self
        }
        
        pub fn large_pages(mut self, enable: bool) -> Self {
            self.config.enable_large_pages = enable;
            self
        }
        
        pub fn memory_protection(mut self, enable: bool) -> Self {
            self.config.enable_memory_protection = enable;
            self
        }
        
        pub fn priority_inheritance(mut self, enable: bool) -> Self {
            self.config.priority_inheritance = enable;
            self
        }
        
        pub fn auto_detect(mut self) -> Self {
            let capabilities = MyOsPlatform::detect_capabilities();
            self.config = MyOsPlatform::new(self.config).recommended_config();
            self
        }
        
        pub fn build(self) -> MyOsPlatform {
            MyOsPlatform::new(self.config)
        }
    }
    
    impl Default for MyOsPlatformBuilder {
        fn default() -> Self {
            Self::new()
        }
    }
}

// Example usage of the external platform
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use wrt_platform_myos::*;
    use wrt_platform::{PageAllocator, FutexLike};
    
    println!("=== External Platform Crate Example ===\n");
    
    // Check if platform is available
    if !MyOsPlatform::is_platform_available() {
        println!("MyOS platform not detected, running in development mode");
    }
    
    // Create platform with auto-detected settings
    let platform = MyOsPlatformBuilder::new()
        .auto_detect()
        .memory_pages(512)
        .large_pages(true)
        .build();
    
    // Show platform information
    let caps = platform.capabilities();
    println!("Platform: {} {}", caps.os_name, caps.os_version);
    println!("CPU Cores: {}", caps.cpu_cores);
    println!("Total Memory: {} GB", caps.total_memory / (1024 * 1024 * 1024));
    println!("Memory Protection: {}", caps.has_memory_protection);
    println!("Large Page Support: {}", caps.has_large_page_support);
    println!("Max Threads: {}", caps.max_threads);
    println!("Priority Inheritance: {}", caps.supports_priority_inheritance);
    
    // Create platform components
    let mut allocator = platform.create_allocator()?;
    let futex = platform.create_futex()?;
    
    println!("\n=== Testing Memory Allocation ===");
    
    // Test memory allocation
    let pages = 10;
    let ptr = allocator.allocate_pages(pages)?;
    println!("Allocated {} pages at {:?}", pages, ptr);
    println!("Total allocated: {} pages", allocator.allocated_pages());
    
    // Test memory growth
    let new_pages = 20;
    let new_ptr = allocator.grow_pages(ptr, pages, new_pages)?;
    println!("Grew allocation to {} pages at {:?}", new_pages, new_ptr);
    
    // Test deallocation
    allocator.deallocate_pages(new_ptr, new_pages)?;
    println!("Deallocated all pages");
    println!("Final allocated: {} pages", allocator.allocated_pages());
    
    println!("\n=== Testing Synchronization ===");
    
    // Test futex operations
    futex.store(42, core::sync::atomic::Ordering::Release);
    let value = futex.load(core::sync::atomic::Ordering::Acquire);
    println!("Futex value: {}", value);
    
    // Test compare-exchange
    match futex.compare_exchange_weak(
        42, 100, 
        core::sync::atomic::Ordering::SeqCst, 
        core::sync::atomic::Ordering::SeqCst
    ) {
        Ok(old) => println!("Changed {} to 100", old),
        Err(actual) => println!("CAS failed, actual: {}", actual),
    }
    
    // Test wake operations
    let woken = futex.wake_one()?;
    println!("Woke {} waiters", woken);
    
    println!("\n=== Integration with WRT Traits ===");
    
    // Demonstrate trait object usage
    let boxed_allocator: Box<dyn PageAllocator> = platform.create_allocator_boxed()?;
    let boxed_futex: Box<dyn FutexLike> = platform.create_futex_boxed()?;
    
    println!("Created trait objects successfully");
    
    // Function that works with any platform implementation
    fn use_platform_components<A: PageAllocator, F: FutexLike>(
        allocator: &mut A,
        futex: &F,
    ) -> Result<(), wrt_error::Error> {
        let ptr = allocator.allocate_pages(5)?;
        futex.store(123, core::sync::atomic::Ordering::SeqCst);
        let value = futex.load(core::sync::atomic::Ordering::SeqCst);
        allocator.deallocate_pages(ptr, 5)?;
        println!("Generic function used platform with futex value: {}", value);
        Ok(())
    }
    
    let mut allocator = platform.create_allocator()?;
    let futex = platform.create_futex()?;
    use_platform_components(&mut allocator, &futex)?;
    
    println!("\n=== Summary ===");
    println!("✓ Platform detection and capability querying");
    println!("✓ Memory allocation with platform-specific features");
    println!("✓ Synchronization with priority inheritance support");
    println!("✓ Seamless integration with WRT trait system");
    println!("✓ Builder pattern for configuration");
    println!("✓ Development fallbacks for non-target platforms");
    
    println!("\nThis external crate can be published independently and used by");
    println!("applications that need MyOS support without requiring changes to core WRT!");
    
    Ok(())
}