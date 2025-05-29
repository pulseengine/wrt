//! VxWorks-specific memory allocators
//!
//! This module provides external implementations of memory allocators for VxWorks
//! that demonstrate how to extend wrt-platform with custom platform support.

use core::ptr::NonNull;
use wrt_platform::{PageAllocator, WASM_PAGE_SIZE};
use wrt_error::{Error, ErrorKind};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// VxWorks RTP (Real-Time Process) memory allocator
///
/// This allocator uses standard C memory allocation functions available
/// in VxWorks RTP user-space applications.
pub struct VxWorksRtpAllocator {
    max_pages: usize,
    allocated_pages: usize,
    heap_size: usize,
    stack_size: usize,
    #[cfg(feature = "alloc")]
    allocations: Vec<(NonNull<u8>, usize)>,
}

impl VxWorksRtpAllocator {
    /// Create a new RTP allocator builder
    pub fn new() -> VxWorksRtpAllocatorBuilder {
        VxWorksRtpAllocatorBuilder::new()
    }

    /// Allocate memory using VxWorks RTP APIs
    fn allocate_memory(&self, size: usize) -> Result<NonNull<u8>, Error> {
        // In a real implementation, this would call VxWorks APIs like:
        // malloc(), memalign(), or posix_memalign()
        
        #[cfg(target_os = "vxworks")]
        {
            extern "C" {
                fn posix_memalign(memptr: *mut *mut core::ffi::c_void, alignment: usize, size: usize) -> i32;
            }
            
            let mut ptr: *mut core::ffi::c_void = core::ptr::null_mut();
            let result = unsafe { posix_memalign(&mut ptr, WASM_PAGE_SIZE, size) };
            
            if result != 0 || ptr.is_null() {
                return Err(Error::new(
                    ErrorKind::Memory,
                    "VxWorks RTP memory allocation failed"
                ));
            }
            
            NonNull::new(ptr as *mut u8).ok_or_else(|| {
                Error::new(ErrorKind::Memory, "Allocated null pointer")
            })
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation for non-VxWorks platforms
            use core::alloc::{alloc, Layout};
            
            let layout = Layout::from_size_align(size, WASM_PAGE_SIZE)
                .map_err(|_| Error::new(ErrorKind::Memory, "Invalid layout"))?;
            
            let ptr = unsafe { alloc(layout) };
            if ptr.is_null() {
                return Err(Error::new(
                    ErrorKind::Memory,
                    "Mock RTP allocation failed"
                ));
            }
            
            // Zero the memory for security
            unsafe { core::ptr::write_bytes(ptr, 0, size) };
            
            NonNull::new(ptr).ok_or_else(|| {
                Error::new(ErrorKind::Memory, "Allocated null pointer")
            })
        }
    }

    /// Free memory using VxWorks RTP APIs
    fn free_memory(&self, ptr: NonNull<u8>) -> Result<(), Error> {
        #[cfg(target_os = "vxworks")]
        {
            extern "C" {
                fn free(ptr: *mut core::ffi::c_void);
            }
            
            unsafe { free(ptr.as_ptr() as *mut core::ffi::c_void) };
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation for non-VxWorks platforms
            use core::alloc::{dealloc, Layout};
            
            // In a real implementation, we'd track the size
            // For now, we'll use a reasonable default
            let layout = Layout::from_size_align(WASM_PAGE_SIZE, WASM_PAGE_SIZE)
                .map_err(|_| Error::new(ErrorKind::Memory, "Invalid layout"))?;
            
            unsafe { dealloc(ptr.as_ptr(), layout) };
        }
        
        Ok(())
    }
}

impl PageAllocator for VxWorksRtpAllocator {
    fn allocate_pages(&mut self, pages: usize) -> Result<NonNull<u8>, Error> {
        if self.allocated_pages + pages > self.max_pages {
            return Err(Error::new(
                ErrorKind::Memory,
                "VxWorks RTP allocator page limit exceeded"
            ));
        }

        let size = pages * WASM_PAGE_SIZE;
        let ptr = self.allocate_memory(size)?;
        
        self.allocated_pages += pages;
        
        #[cfg(feature = "alloc")]
        {
            self.allocations.push((ptr, pages));
        }

        Ok(ptr)
    }

    fn deallocate_pages(&mut self, ptr: NonNull<u8>, pages: usize) -> Result<(), Error> {
        if pages > self.allocated_pages {
            return Err(Error::new(
                ErrorKind::Memory,
                "Attempting to deallocate more pages than allocated"
            ));
        }

        self.free_memory(ptr)?;
        self.allocated_pages -= pages;

        #[cfg(feature = "alloc")]
        {
            self.allocations.retain(|(p, _)| *p != ptr);
        }

        Ok(())
    }

    fn grow_pages(&mut self, old_ptr: NonNull<u8>, old_pages: usize, new_pages: usize) -> Result<NonNull<u8>, Error> {
        if new_pages <= old_pages {
            return Ok(old_ptr);
        }

        // VxWorks doesn't have a reliable realloc for aligned memory
        // So we allocate new memory and copy
        let new_ptr = self.allocate_pages(new_pages)?;
        
        let copy_size = old_pages * WASM_PAGE_SIZE;
        unsafe {
            core::ptr::copy_nonoverlapping(old_ptr.as_ptr(), new_ptr.as_ptr(), copy_size);
        }

        self.deallocate_pages(old_ptr, old_pages)?;
        Ok(new_ptr)
    }

    fn allocated_pages(&self) -> usize {
        self.allocated_pages
    }

    fn max_pages(&self) -> usize {
        self.max_pages
    }
}

/// Builder for VxWorks RTP allocator
pub struct VxWorksRtpAllocatorBuilder {
    max_pages: usize,
    heap_size: usize,
    stack_size: usize,
}

impl VxWorksRtpAllocatorBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            max_pages: 1024,
            heap_size: 1024 * 1024, // 1MB
            stack_size: 64 * 1024,  // 64KB
        }
    }

    /// Set maximum pages
    pub fn with_max_pages(mut self, max_pages: usize) -> Self {
        self.max_pages = max_pages;
        self
    }

    /// Set heap size
    pub fn with_heap_size(mut self, heap_size: usize) -> Self {
        self.heap_size = heap_size;
        self
    }

    /// Set stack size
    pub fn with_stack_size(mut self, stack_size: usize) -> Self {
        self.stack_size = stack_size;
        self
    }

    /// Build the allocator
    pub fn build(self) -> Result<VxWorksRtpAllocator, Error> {
        Ok(VxWorksRtpAllocator {
            max_pages: self.max_pages,
            allocated_pages: 0,
            heap_size: self.heap_size,
            stack_size: self.stack_size,
            #[cfg(feature = "alloc")]
            allocations: Vec::new(),
        })
    }
}

impl Default for VxWorksRtpAllocatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// VxWorks LKM (Loadable Kernel Module) memory allocator
///
/// This allocator uses VxWorks kernel-space memory allocation with
/// dedicated memory partitions for deterministic behavior.
pub struct VxWorksLkmAllocator {
    max_pages: usize,
    allocated_pages: usize,
    use_memory_partitions: bool,
    priority_inheritance: bool,
    partition_id: Option<usize>,
    #[cfg(feature = "alloc")]
    partition_memory: Option<Vec<u8>>,
}

impl VxWorksLkmAllocator {
    /// Create a new LKM allocator builder
    pub fn new() -> VxWorksLkmAllocatorBuilder {
        VxWorksLkmAllocatorBuilder::new()
    }

    /// Initialize memory partition if enabled
    fn init_memory_partition(&mut self) -> Result<(), Error> {
        if !self.use_memory_partitions {
            return Ok(());
        }

        #[cfg(target_os = "vxworks")]
        {
            extern "C" {
                fn memPartCreate(pool: *mut u8, pool_size: usize) -> usize;
            }
            
            let partition_size = self.max_pages * WASM_PAGE_SIZE;
            
            #[cfg(feature = "alloc")]
            {
                let mut partition_memory = vec![0u8; partition_size];
                let pool_ptr = partition_memory.as_mut_ptr();
                
                let partition_id = unsafe { memPartCreate(pool_ptr, partition_size) };
                if partition_id == 0 {
                    return Err(Error::new(
                        ErrorKind::Memory,
                        "Failed to create VxWorks memory partition"
                    ));
                }

                self.partition_id = Some(partition_id);
                self.partition_memory = Some(partition_memory);
            }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation for non-VxWorks platforms
            #[cfg(feature = "alloc")]
            {
                let partition_size = self.max_pages * WASM_PAGE_SIZE;
                let partition_memory = vec![0u8; partition_size];
                self.partition_memory = Some(partition_memory);
                self.partition_id = Some(1); // Mock partition ID
            }
        }

        Ok(())
    }

    /// Allocate memory using VxWorks LKM APIs
    fn allocate_memory(&self, size: usize) -> Result<NonNull<u8>, Error> {
        #[cfg(target_os = "vxworks")]
        {
            match self.partition_id {
                Some(partition_id) => {
                    extern "C" {
                        fn memPartAlignedAlloc(mem_part_id: usize, size: usize, alignment: usize) -> *mut u8;
                    }
                    
                    let ptr = unsafe { memPartAlignedAlloc(partition_id, size, WASM_PAGE_SIZE) };
                    if ptr.is_null() {
                        return Err(Error::new(
                            ErrorKind::Memory,
                            "VxWorks LKM partition allocation failed"
                        ));
                    }
                    
                    NonNull::new(ptr).ok_or_else(|| {
                        Error::new(ErrorKind::Memory, "Allocated null pointer")
                    })
                }
                None => {
                    extern "C" {
                        fn kernelAlloc(size: usize) -> *mut u8;
                    }
                    
                    let ptr = unsafe { kernelAlloc(size) };
                    if ptr.is_null() {
                        return Err(Error::new(
                            ErrorKind::Memory,
                            "VxWorks LKM kernel allocation failed"
                        ));
                    }
                    
                    NonNull::new(ptr).ok_or_else(|| {
                        Error::new(ErrorKind::Memory, "Allocated null pointer")
                    })
                }
            }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation for non-VxWorks platforms
            use core::alloc::{alloc, Layout};
            
            let layout = Layout::from_size_align(size, WASM_PAGE_SIZE)
                .map_err(|_| Error::new(ErrorKind::Memory, "Invalid layout"))?;
            
            let ptr = unsafe { alloc(layout) };
            if ptr.is_null() {
                return Err(Error::new(
                    ErrorKind::Memory,
                    "Mock LKM allocation failed"
                ));
            }
            
            // Zero the memory for security
            unsafe { core::ptr::write_bytes(ptr, 0, size) };
            
            NonNull::new(ptr).ok_or_else(|| {
                Error::new(ErrorKind::Memory, "Allocated null pointer")
            })
        }
    }

    /// Free memory using VxWorks LKM APIs
    fn free_memory(&self, ptr: NonNull<u8>) -> Result<(), Error> {
        #[cfg(target_os = "vxworks")]
        {
            match self.partition_id {
                Some(partition_id) => {
                    extern "C" {
                        fn memPartFree(mem_part_id: usize, ptr: *mut u8) -> i32;
                    }
                    
                    let result = unsafe { memPartFree(partition_id, ptr.as_ptr()) };
                    if result != 0 {
                        return Err(Error::new(
                            ErrorKind::Memory,
                            "VxWorks LKM partition free failed"
                        ));
                    }
                }
                None => {
                    extern "C" {
                        fn kernelFree(ptr: *mut u8);
                    }
                    
                    unsafe { kernelFree(ptr.as_ptr()) };
                }
            }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // Mock implementation for non-VxWorks platforms
            use core::alloc::{dealloc, Layout};
            
            let layout = Layout::from_size_align(WASM_PAGE_SIZE, WASM_PAGE_SIZE)
                .map_err(|_| Error::new(ErrorKind::Memory, "Invalid layout"))?;
            
            unsafe { dealloc(ptr.as_ptr(), layout) };
        }
        
        Ok(())
    }
}

impl PageAllocator for VxWorksLkmAllocator {
    fn allocate_pages(&mut self, pages: usize) -> Result<NonNull<u8>, Error> {
        if self.allocated_pages + pages > self.max_pages {
            return Err(Error::new(
                ErrorKind::Memory,
                "VxWorks LKM allocator page limit exceeded"
            ));
        }

        let size = pages * WASM_PAGE_SIZE;
        let ptr = self.allocate_memory(size)?;
        
        self.allocated_pages += pages;
        Ok(ptr)
    }

    fn deallocate_pages(&mut self, ptr: NonNull<u8>, pages: usize) -> Result<(), Error> {
        if pages > self.allocated_pages {
            return Err(Error::new(
                ErrorKind::Memory,
                "Attempting to deallocate more pages than allocated"
            ));
        }

        self.free_memory(ptr)?;
        self.allocated_pages -= pages;
        Ok(())
    }

    fn grow_pages(&mut self, old_ptr: NonNull<u8>, old_pages: usize, new_pages: usize) -> Result<NonNull<u8>, Error> {
        if new_pages <= old_pages {
            return Ok(old_ptr);
        }

        let new_ptr = self.allocate_pages(new_pages)?;
        
        let copy_size = old_pages * WASM_PAGE_SIZE;
        unsafe {
            core::ptr::copy_nonoverlapping(old_ptr.as_ptr(), new_ptr.as_ptr(), copy_size);
        }

        self.deallocate_pages(old_ptr, old_pages)?;
        Ok(new_ptr)
    }

    fn allocated_pages(&self) -> usize {
        self.allocated_pages
    }

    fn max_pages(&self) -> usize {
        self.max_pages
    }
}

/// Builder for VxWorks LKM allocator
pub struct VxWorksLkmAllocatorBuilder {
    max_pages: usize,
    use_memory_partitions: bool,
    priority_inheritance: bool,
}

impl VxWorksLkmAllocatorBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            max_pages: 1024,
            use_memory_partitions: true,
            priority_inheritance: true,
        }
    }

    /// Set maximum pages
    pub fn with_max_pages(mut self, max_pages: usize) -> Self {
        self.max_pages = max_pages;
        self
    }

    /// Enable memory partitions
    pub fn with_memory_partitions(mut self, enable: bool) -> Self {
        self.use_memory_partitions = enable;
        self
    }

    /// Enable priority inheritance
    pub fn with_priority_inheritance(mut self, enable: bool) -> Self {
        self.priority_inheritance = enable;
        self
    }

    /// Build the allocator
    pub fn build(self) -> Result<VxWorksLkmAllocator, Error> {
        let mut allocator = VxWorksLkmAllocator {
            max_pages: self.max_pages,
            allocated_pages: 0,
            use_memory_partitions: self.use_memory_partitions,
            priority_inheritance: self.priority_inheritance,
            partition_id: None,
            #[cfg(feature = "alloc")]
            partition_memory: None,
        };

        allocator.init_memory_partition()?;
        Ok(allocator)
    }
}

impl Default for VxWorksLkmAllocatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtp_allocator_builder() {
        let allocator = VxWorksRtpAllocator::new()
            .with_max_pages(512)
            .with_heap_size(2048 * 1024)
            .with_stack_size(128 * 1024)
            .build()
            .unwrap();

        assert_eq!(allocator.max_pages(), 512);
        assert_eq!(allocator.allocated_pages(), 0);
        assert_eq!(allocator.heap_size, 2048 * 1024);
        assert_eq!(allocator.stack_size, 128 * 1024);
    }

    #[test]
    fn test_lkm_allocator_builder() {
        let allocator = VxWorksLkmAllocator::new()
            .with_max_pages(256)
            .with_memory_partitions(false)
            .with_priority_inheritance(true)
            .build()
            .unwrap();

        assert_eq!(allocator.max_pages(), 256);
        assert_eq!(allocator.allocated_pages(), 0);
        assert!(!allocator.use_memory_partitions);
        assert!(allocator.priority_inheritance);
    }

    #[cfg(not(target_os = "vxworks"))]
    #[test]
    fn test_rtp_allocation() {
        let mut allocator = VxWorksRtpAllocator::new()
            .with_max_pages(10)
            .build()
            .unwrap();

        // Test allocation
        let ptr = allocator.allocate_pages(2).unwrap();
        assert_eq!(allocator.allocated_pages(), 2);

        // Test deallocation
        allocator.deallocate_pages(ptr, 2).unwrap();
        assert_eq!(allocator.allocated_pages(), 0);
    }

    #[cfg(not(target_os = "vxworks"))]
    #[test]
    fn test_lkm_allocation() {
        let mut allocator = VxWorksLkmAllocator::new()
            .with_max_pages(10)
            .with_memory_partitions(false) // Disable for testing
            .build()
            .unwrap();

        // Test allocation
        let ptr = allocator.allocate_pages(3).unwrap();
        assert_eq!(allocator.allocated_pages(), 3);

        // Test deallocation
        allocator.deallocate_pages(ptr, 3).unwrap();
        assert_eq!(allocator.allocated_pages(), 0);
    }
}