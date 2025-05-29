//! Template for External Platform Implementation
//!
//! Copy this template to create your own external platform crate.
//! Replace "MyOs" with your platform name throughout.

use wrt_platform::{PageAllocator, FutexLike, WASM_PAGE_SIZE};
use wrt_error::{Error, ErrorKind};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicU32, Ordering};
use core::time::Duration;

#[cfg(feature = "alloc")]
use alloc::{vec::Vec, string::String, boxed::Box};

/// TODO: Replace with your platform name
/// Platform configuration
#[derive(Clone, Debug)]
pub struct MyOsConfig {
    pub max_memory_pages: usize,
    // TODO: Add platform-specific configuration fields
    // pub enable_feature_x: bool,
    // pub custom_setting: u32,
}

impl Default for MyOsConfig {
    fn default() -> Self {
        Self {
            max_memory_pages: 1024,
            // TODO: Set default values for your platform
        }
    }
}

/// TODO: Replace with your platform name  
/// Platform memory allocator
pub struct MyOsAllocator {
    config: MyOsConfig,
    allocated_pages: usize,
    // TODO: Add platform-specific fields
    // native_heap: PlatformHeapHandle,
    // allocation_map: HashMap<NonNull<u8>, usize>,
}

impl MyOsAllocator {
    pub fn new(config: MyOsConfig) -> Result<Self, Error> {
        // TODO: Initialize your platform's memory system
        Ok(Self {
            config,
            allocated_pages: 0,
            // TODO: Initialize platform-specific fields
        })
    }
    
    // TODO: Implement platform-specific allocation
    fn platform_allocate(&self, size: usize) -> Result<*mut u8, Error> {
        #[cfg(target_os = "your_platform")]
        {
            // TODO: Call your platform's allocation API
            // extern "C" {
            //     fn your_platform_alloc(size: usize, align: usize) -> *mut u8;
            // }
            // 
            // let ptr = unsafe { your_platform_alloc(size, WASM_PAGE_SIZE) };
            // if ptr.is_null() {
            //     return Err(Error::new(ErrorKind::Memory, "Platform allocation failed"));
            // }
            // Ok(ptr)
            
            // Placeholder for template
            Err(Error::new(ErrorKind::Platform, "Not implemented"))
        }
        
        #[cfg(not(target_os = "your_platform"))]
        {
            // Development fallback using standard allocator
            use core::alloc::{alloc, Layout};
            
            let layout = Layout::from_size_align(size, WASM_PAGE_SIZE)
                .map_err(|_| Error::new(ErrorKind::Memory, "Invalid layout"))?;
            
            let ptr = unsafe { alloc(layout) };
            if ptr.is_null() {
                return Err(Error::new(ErrorKind::Memory, "Allocation failed"));
            }
            
            // Zero memory for security
            unsafe { core::ptr::write_bytes(ptr, 0, size) };
            
            Ok(ptr)
        }
    }
    
    // TODO: Implement platform-specific deallocation
    fn platform_deallocate(&self, ptr: *mut u8, size: usize) -> Result<(), Error> {
        #[cfg(target_os = "your_platform")]
        {
            // TODO: Call your platform's deallocation API
            // extern "C" {
            //     fn your_platform_free(ptr: *mut u8, size: usize);
            // }
            // 
            // unsafe { your_platform_free(ptr, size) };
            // Ok(())
            
            // Placeholder for template
            Ok(())
        }
        
        #[cfg(not(target_os = "your_platform"))]
        {
            // Development fallback
            use core::alloc::{dealloc, Layout};
            
            let layout = Layout::from_size_align(size, WASM_PAGE_SIZE)
                .map_err(|_| Error::new(ErrorKind::Memory, "Invalid layout"))?;
            
            unsafe { dealloc(ptr, layout) };
            Ok(())
        }
    }
}

impl PageAllocator for MyOsAllocator {
    fn allocate_pages(&mut self, pages: usize) -> Result<NonNull<u8>, Error> {
        if self.allocated_pages + pages > self.config.max_memory_pages {
            return Err(Error::new(ErrorKind::Memory, "Page limit exceeded"));
        }
        
        let size = pages * WASM_PAGE_SIZE;
        let ptr = self.platform_allocate(size)?;
        
        self.allocated_pages += pages;
        
        NonNull::new(ptr).ok_or_else(|| 
            Error::new(ErrorKind::Memory, "Null pointer"))
    }
    
    fn deallocate_pages(&mut self, ptr: NonNull<u8>, pages: usize) -> Result<(), Error> {
        let size = pages * WASM_PAGE_SIZE;
        self.platform_deallocate(ptr.as_ptr(), size)?;
        
        self.allocated_pages = self.allocated_pages.saturating_sub(pages);
        Ok(())
    }
    
    fn grow_pages(&mut self, old_ptr: NonNull<u8>, old_pages: usize, new_pages: usize) 
        -> Result<NonNull<u8>, Error> {
        if new_pages <= old_pages {
            return Ok(old_ptr);
        }
        
        // Simple implementation: allocate new and copy
        // TODO: Optimize using platform-specific reallocation if available
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
        self.allocated_pages
    }
    
    fn max_pages(&self) -> usize {
        self.config.max_memory_pages
    }
}

/// TODO: Replace with your platform name
/// Platform synchronization primitive
pub struct MyOsFutex {
    value: AtomicU32,
    // TODO: Add platform-specific synchronization fields
    // native_semaphore: PlatformSemaphoreHandle,
    // wait_queue: PlatformWaitQueue,
}

impl MyOsFutex {
    pub fn new(initial: u32) -> Result<Self, Error> {
        // TODO: Initialize platform-specific synchronization
        Ok(Self {
            value: AtomicU32::new(initial),
            // TODO: Initialize platform-specific fields
        })
    }
    
    // TODO: Implement platform-specific wait
    fn platform_wait(&self, timeout: Option<Duration>) -> Result<(), Error> {
        #[cfg(target_os = "your_platform")]
        {
            // TODO: Call your platform's wait API
            // extern "C" {
            //     fn your_platform_sem_wait(handle: usize, timeout_ms: u32) -> i32;
            // }
            // 
            // let timeout_ms = timeout.map_or(0xFFFFFFFF, |d| d.as_millis() as u32);
            // let result = unsafe { your_platform_sem_wait(self.native_semaphore, timeout_ms) };
            // if result != 0 {
            //     return Err(Error::new(ErrorKind::Platform, "Wait failed"));
            // }
            // Ok(())
            
            // Placeholder for template
            Ok(())
        }
        
        #[cfg(not(target_os = "your_platform"))]
        {
            // Development fallback - no actual waiting
            Ok(())
        }
    }
    
    // TODO: Implement platform-specific wake
    fn platform_wake_one(&self) -> Result<u32, Error> {
        #[cfg(target_os = "your_platform")]
        {
            // TODO: Call your platform's signal API
            // extern "C" {
            //     fn your_platform_sem_signal(handle: usize) -> i32;
            // }
            // 
            // let result = unsafe { your_platform_sem_signal(self.native_semaphore) };
            // if result != 0 {
            //     return Err(Error::new(ErrorKind::Platform, "Signal failed"));
            // }
            // Ok(1)
            
            // Placeholder for template
            Ok(1)
        }
        
        #[cfg(not(target_os = "your_platform"))]
        {
            Ok(1)
        }
    }
    
    // TODO: Implement platform-specific broadcast
    fn platform_wake_all(&self) -> Result<u32, Error> {
        #[cfg(target_os = "your_platform")]
        {
            // TODO: Call your platform's broadcast API
            // extern "C" {
            //     fn your_platform_sem_broadcast(handle: usize) -> i32;
            // }
            // 
            // let result = unsafe { your_platform_sem_broadcast(self.native_semaphore) };
            // if result < 0 {
            //     return Err(Error::new(ErrorKind::Platform, "Broadcast failed"));
            // }
            // Ok(result as u32)
            
            // Placeholder for template
            Ok(u32::MAX)
        }
        
        #[cfg(not(target_os = "your_platform"))]
        {
            Ok(u32::MAX)
        }
    }
}

impl FutexLike for MyOsFutex {
    fn wait(&self, expected: u32, timeout: Option<Duration>) -> Result<(), Error> {
        if self.value.load(Ordering::Acquire) != expected {
            return Ok(());
        }
        
        self.platform_wait(timeout)
    }
    
    fn wake_one(&self) -> Result<u32, Error> {
        self.platform_wake_one()
    }
    
    fn wake_all(&self) -> Result<u32, Error> {
        self.platform_wake_all()
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

/// TODO: Replace with your platform name
/// Platform capabilities
#[derive(Debug, Clone)]
pub struct PlatformCapabilities {
    pub os_name: &'static str,
    pub os_version: String,
    // TODO: Add platform-specific capabilities
    // pub has_feature_x: bool,
    // pub max_y: usize,
}

/// TODO: Replace with your platform name
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
                "Platform not available"
            ));
        }
        
        Ok(Self::new(MyOsConfig::default()))
    }
    
    pub fn capabilities(&self) -> &PlatformCapabilities {
        &self.capabilities
    }
    
    pub fn create_allocator(&self) -> Result<MyOsAllocator, Error> {
        MyOsAllocator::new(self.config.clone())
    }
    
    pub fn create_futex(&self) -> Result<MyOsFutex, Error> {
        MyOsFutex::new(0)
    }
    
    pub fn create_allocator_boxed(&self) -> Result<Box<dyn PageAllocator>, Error> {
        Ok(Box::new(self.create_allocator()?))
    }
    
    pub fn create_futex_boxed(&self) -> Result<Box<dyn FutexLike>, Error> {
        Ok(Box::new(self.create_futex()?))
    }
    
    // TODO: Implement platform detection
    pub fn is_platform_available() -> bool {
        #[cfg(target_os = "your_platform")]
        {
            // TODO: Check if your platform runtime is available
            // extern "C" {
            //     fn your_platform_get_version() -> u32;
            // }
            // 
            // unsafe { your_platform_get_version() != 0 }
            
            true // Placeholder
        }
        
        #[cfg(not(target_os = "your_platform"))]
        {
            false
        }
    }
    
    // TODO: Implement capability detection
    fn detect_capabilities() -> PlatformCapabilities {
        #[cfg(target_os = "your_platform")]
        {
            // TODO: Query actual platform capabilities
            PlatformCapabilities {
                os_name: "MyOS",
                os_version: "1.0".to_string(),
                // TODO: Detect actual capabilities
            }
        }
        
        #[cfg(not(target_os = "your_platform"))]
        {
            PlatformCapabilities {
                os_name: "MyOS (Development)",
                os_version: "Dev".to_string(),
                // TODO: Provide development defaults
            }
        }
    }
}

/// TODO: Replace with your platform name
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
    
    // TODO: Add builder methods for your platform-specific configuration
    // pub fn enable_feature_x(mut self, enable: bool) -> Self {
    //     self.config.enable_feature_x = enable;
    //     self
    // }
    
    pub fn build(self) -> MyOsPlatform {
        MyOsPlatform::new(self.config)
    }
}

impl Default for MyOsPlatformBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: Add unit tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_platform_detection() {
        // This should work on all platforms for development
        let available = MyOsPlatform::is_platform_available();
        
        #[cfg(target_os = "your_platform")]
        assert!(available);
        
        #[cfg(not(target_os = "your_platform"))]
        assert!(!available);
    }
    
    #[test]
    fn test_allocator_basic() {
        let platform = MyOsPlatformBuilder::new()
            .memory_pages(10)
            .build();
        
        let mut allocator = platform.create_allocator().unwrap();
        
        // Test basic allocation
        let ptr = allocator.allocate_pages(5).unwrap();
        assert_eq!(allocator.allocated_pages(), 5);
        
        // Test deallocation
        allocator.deallocate_pages(ptr, 5).unwrap();
        assert_eq!(allocator.allocated_pages(), 0);
    }
    
    #[test]
    fn test_futex_operations() {
        let platform = MyOsPlatformBuilder::new().build();
        let futex = platform.create_futex().unwrap();
        
        // Test atomic operations
        futex.store(42, Ordering::SeqCst);
        assert_eq!(futex.load(Ordering::SeqCst), 42);
        
        // Test compare-exchange
        let result = futex.compare_exchange_weak(
            42, 100, 
            Ordering::SeqCst, 
            Ordering::SeqCst
        );
        assert_eq!(result, Ok(42));
        assert_eq!(futex.load(Ordering::SeqCst), 100);
    }
    
    #[test]
    fn test_trait_objects() {
        let platform = MyOsPlatformBuilder::new().build();
        
        // Test that we can create trait objects
        let _allocator: Box<dyn PageAllocator> = platform.create_allocator_boxed().unwrap();
        let _futex: Box<dyn FutexLike> = platform.create_futex_boxed().unwrap();
    }
}

// TODO: Usage example in your crate's documentation
/// # Example
/// 
/// ```rust,no_run
/// use wrt_platform_myos::*;
/// 
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Detect platform
///     let platform = MyOsPlatformBuilder::new()
///         .memory_pages(1024)
///         .build();
///     
///     // Create WRT components
///     let allocator = platform.create_allocator_boxed()?;
///     let futex = platform.create_futex_boxed()?;
///     
///     // Use with WRT runtime
///     // let runtime = wrt::Runtime::builder()
///     //     .with_allocator(allocator)
///     //     .with_futex(futex)
///     //     .build()?;
///     
///     Ok(())
/// }
/// ```