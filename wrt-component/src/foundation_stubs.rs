// Foundation stubs for Agent C independent development
// These will be replaced with real implementations during integration

use alloc::vec::Vec;

// Temporary stubs for bounded collections from Agent A's work
pub type SmallVec<T> = Vec<T>;
pub type MediumVec<T> = Vec<T>;
pub type LargeVec<T> = Vec<T>;

// Safety context stub
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsilLevel {
    QM = 0,
    ASIL_A = 1,
    ASIL_B = 2,
    ASIL_C = 3,
    ASIL_D = 4,
}

#[derive(Debug, Clone)]
pub struct SafetyContext {
    pub compile_time_asil: AsilLevel,
    pub runtime_asil: Option<AsilLevel>,
}

impl SafetyContext {
    pub const fn new(compile_time: AsilLevel) -> Self {
        Self { 
            compile_time_asil: compile_time, 
            runtime_asil: None 
        }
    }
    
    pub fn effective_asil(&self) -> AsilLevel {
        self.runtime_asil.unwrap_or(self.compile_time_asil)
    }
}

// Memory provider stubs
pub trait UnifiedMemoryProvider: Send + Sync {
    fn allocate(&mut self, size: usize) -> Result<&mut [u8], wrt_error::Error>;
    fn deallocate(&mut self, ptr: &mut [u8]) -> Result<(), wrt_error::Error>;
    fn available_memory(&self) -> usize;
    fn total_memory(&self) -> usize;
}

pub struct NoStdProvider<const SIZE: usize> {
    buffer: [u8; SIZE],
    allocated: usize,
}

impl<const SIZE: usize> NoStdProvider<SIZE> {
    pub fn new() -> Self {
        Self {
            buffer: [0; SIZE],
            allocated: 0,
        }
    }
}

impl<const SIZE: usize> Default for NoStdProvider<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize> UnifiedMemoryProvider for NoStdProvider<SIZE> {
    fn allocate(&mut self, size: usize) -> Result<&mut [u8], wrt_error::Error> {
        if self.allocated + size > SIZE {
            return Err(wrt_error::Error::OUT_OF_MEMORY);
        }
        let start = self.allocated;
        self.allocated += size;
        Ok(&mut self.buffer[start..self.allocated])
    }
    
    fn deallocate(&mut self, _ptr: &mut [u8]) -> Result<(), wrt_error::Error> {
        // Simple implementation - could reset if ptr is at end
        Ok(())
    }
    
    fn available_memory(&self) -> usize {
        SIZE - self.allocated
    }
    
    fn total_memory(&self) -> usize {
        SIZE
    }
}

// Error types from Agent A
pub use wrt_error::Error;