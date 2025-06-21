//! Memory optimization and layout calculations for WebAssembly parsing
//!
//! This module provides memory-optimized parsing utilities, memory pool management,
//! and canonical ABI memory layout calculations with ASIL-D compliance.

use wrt_error::{Error, ErrorCategory, Result, codes};
use crate::types::{ValueType, MemoryType, Limits};
use crate::bounded_types::{SimpleBoundedVec, SimpleBoundedString};
use crate::simple_module::SimpleModule;

/// Page size for WebAssembly memory (64KB)
pub const WASM_PAGE_SIZE: u32 = 65536;

/// Maximum memory size in bytes for ASIL-D compliance
pub const MAX_MEMORY_SIZE_ASIL_D: u64 = 64 * 1024 * 1024; // 64MB

/// Alignment requirements for different value types
pub const I32_ALIGNMENT: u32 = 4;
pub const I64_ALIGNMENT: u32 = 8;
pub const F32_ALIGNMENT: u32 = 4;
pub const F64_ALIGNMENT: u32 = 8;
pub const POINTER_ALIGNMENT: u32 = 4; // WebAssembly pointers are 32-bit

/// Memory layout information
#[derive(Debug, Clone)]
pub struct MemoryLayout {
    /// Total size in bytes
    pub total_size: u64,
    
    /// Memory segments
    pub segments: SimpleBoundedVec<MemorySegment, 256>,
    
    /// Stack size requirements
    pub stack_size: u32,
    
    /// Heap size requirements  
    pub heap_size: u32,
    
    /// Alignment padding overhead
    pub padding_overhead: u32,
}

/// Memory segment information
#[derive(Debug, Clone)]
pub struct MemorySegment {
    /// Segment offset in memory
    pub offset: u32,
    
    /// Segment size in bytes
    pub size: u32,
    
    /// Segment type
    pub segment_type: SegmentType,
    
    /// Required alignment
    pub alignment: u32,
}

/// Types of memory segments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    /// Static data segment
    Data,
    
    /// Function table
    Table,
    
    /// Global variables
    Globals,
    
    /// Stack space
    Stack,
    
    /// Heap space
    Heap,
    
    /// Reserved/padding
    Reserved,
}

/// Memory pool for reusing allocations during parsing
#[derive(Debug)]
pub struct MemoryPool {
    /// Pre-allocated byte vectors for reuse
    byte_vectors: SimpleBoundedVec<Vec<u8>, 32>,
    
    /// Pre-allocated string buffers
    string_buffers: SimpleBoundedVec<String, 16>,
    
    /// Memory usage statistics
    stats: MemoryStats,
    
    /// Pool capacity limits
    max_vector_size: usize,
    max_vectors: usize,
}

/// Memory usage statistics
#[derive(Debug, Default)]
pub struct MemoryStats {
    /// Total bytes allocated
    pub total_allocated: u64,
    
    /// Peak memory usage
    pub peak_usage: u64,
    
    /// Current memory usage
    pub current_usage: u64,
    
    /// Number of allocations
    pub allocation_count: u64,
    
    /// Number of reused allocations
    pub reuse_count: u64,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new() -> Self {
        Self {
            byte_vectors: SimpleBoundedVec::new(),
            string_buffers: SimpleBoundedVec::new(),
            stats: MemoryStats::default(),
            max_vector_size: 1024 * 1024, // 1MB max per vector
            max_vectors: 32,
        }
    }
    
    /// Create memory pool with ASIL-D limits
    pub fn asil_d() -> Self {
        Self {
            byte_vectors: SimpleBoundedVec::new(),
            string_buffers: SimpleBoundedVec::new(),
            stats: MemoryStats::default(),
            max_vector_size: 64 * 1024, // 64KB max per vector
            max_vectors: 16,
        }
    }
    
    /// Get a reusable byte vector
    pub fn get_byte_vector(&mut self, min_capacity: usize) -> Result<Vec<u8>> {
        if min_capacity > self.max_vector_size {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_LIMIT_EXCEEDED,
                "Requested vector size exceeds pool limits"
            ));
        }
        
        // Try to reuse an existing vector
        for i in 0..self.byte_vectors.len() {
            if self.byte_vectors[i].capacity() >= min_capacity {
                let mut vec = self.byte_vectors.swap_remove(i);
                vec.clear();
                self.stats.reuse_count += 1;
                return Ok(vec);
            }
        }
        
        // Allocate new vector
        let mut vec = Vec::with_capacity(min_capacity);
        self.stats.allocation_count += 1;
        self.stats.total_allocated += min_capacity as u64;
        self.stats.current_usage += min_capacity as u64;
        
        if self.stats.current_usage > self.stats.peak_usage {
            self.stats.peak_usage = self.stats.current_usage;
        }
        
        Ok(vec)
    }
    
    /// Return a byte vector to the pool
    pub fn return_byte_vector(&mut self, mut vec: Vec<u8>) -> Result<()> {
        if vec.capacity() <= self.max_vector_size && self.byte_vectors.len() < self.max_vectors {
            vec.clear();
            self.byte_vectors.push(vec)?;
        }
        Ok(())
    }
    
    /// Get a reusable string buffer
    pub fn get_string_buffer(&mut self, min_capacity: usize) -> Result<String> {
        // Try to reuse an existing string
        for i in 0..self.string_buffers.len() {
            if self.string_buffers[i].capacity() >= min_capacity {
                let mut string = self.string_buffers.swap_remove(i);
                string.clear();
                self.stats.reuse_count += 1;
                return Ok(string);
            }
        }
        
        // Allocate new string
        let string = String::with_capacity(min_capacity);
        self.stats.allocation_count += 1;
        self.stats.total_allocated += min_capacity as u64;
        
        Ok(string)
    }
    
    /// Return a string buffer to the pool
    pub fn return_string_buffer(&mut self, mut string: String) -> Result<()> {
        if string.capacity() <= 1024 && self.string_buffers.len() < 16 {
            string.clear();
            self.string_buffers.push(string)?;
        }
        Ok(())
    }
    
    /// Get memory usage statistics
    pub fn stats(&self) -> &MemoryStats {
        &self.stats
    }
    
    /// Clear the pool and reset statistics
    pub fn clear(&mut self) {
        self.byte_vectors.clear();
        self.string_buffers.clear();
        self.stats = MemoryStats::default();
    }
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory layout calculator
#[derive(Debug)]
pub struct MemoryLayoutCalculator {
    /// Memory pool for temporary allocations
    pool: MemoryPool,
    
    /// ASIL-D compliance mode
    asil_d_mode: bool,
}

impl MemoryLayoutCalculator {
    /// Create a new memory layout calculator
    pub fn new() -> Self {
        Self {
            pool: MemoryPool::new(),
            asil_d_mode: false,
        }
    }
    
    /// Create calculator with ASIL-D compliance
    pub fn asil_d() -> Self {
        Self {
            pool: MemoryPool::asil_d(),
            asil_d_mode: true,
        }
    }
    
    /// Calculate memory layout for a module
    pub fn calculate_layout(&mut self, module: &SimpleModule) -> Result<MemoryLayout> {
        let mut layout = MemoryLayout {
            total_size: 0,
            segments: SimpleBoundedVec::new(),
            stack_size: 0,
            heap_size: 0,
            padding_overhead: 0,
        };
        
        let mut current_offset = 0u32;
        
        // Calculate data segments
        current_offset = self.layout_data_segments(module, &mut layout, current_offset)?;
        
        // Calculate global variables
        current_offset = self.layout_globals(module, &mut layout, current_offset)?;
        
        // Calculate table segments
        current_offset = self.layout_tables(module, &mut layout, current_offset)?;
        
        // Calculate stack requirements
        current_offset = self.layout_stack(module, &mut layout, current_offset)?;
        
        // Calculate heap requirements
        current_offset = self.layout_heap(module, &mut layout, current_offset)?;
        
        layout.total_size = current_offset as u64;
        
        // Validate against ASIL-D limits
        if self.asil_d_mode && layout.total_size > MAX_MEMORY_SIZE_ASIL_D {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_LIMIT_EXCEEDED,
                "Memory layout exceeds ASIL-D limits"
            ));
        }
        
        Ok(layout)
    }
    
    /// Layout data segments
    fn layout_data_segments(&mut self, module: &SimpleModule, layout: &mut MemoryLayout, mut offset: u32) -> Result<u32> {
        for data_segment in &module.data {
            // Align to pointer boundary
            offset = self.align_offset(offset, POINTER_ALIGNMENT);
            
            let segment = MemorySegment {
                offset,
                size: data_segment.data.len() as u32,
                segment_type: SegmentType::Data,
                alignment: POINTER_ALIGNMENT,
            };
            
            layout.segments.push(segment)?;
            offset += data_segment.data.len() as u32;
        }
        
        Ok(offset)
    }
    
    /// Layout global variables
    fn layout_globals(&mut self, module: &SimpleModule, layout: &mut MemoryLayout, mut offset: u32) -> Result<u32> {
        if module.globals.is_empty() {
            return Ok(offset);
        }
        
        // Calculate total size needed for globals
        let mut globals_size = 0u32;
        let mut max_alignment = 1u32;
        
        for global in &module.globals {
            let (size, alignment) = self.get_type_size_alignment(global.value_type);
            globals_size += size;
            max_alignment = max_alignment.max(alignment);
        }
        
        // Align globals section
        offset = self.align_offset(offset, max_alignment);
        
        let segment = MemorySegment {
            offset,
            size: globals_size,
            segment_type: SegmentType::Globals,
            alignment: max_alignment,
        };
        
        layout.segments.push(segment)?;
        Ok(offset + globals_size)
    }
    
    /// Layout table segments
    fn layout_tables(&mut self, module: &SimpleModule, layout: &mut MemoryLayout, mut offset: u32) -> Result<u32> {
        for table in &module.tables {
            // Tables store function pointers (4 bytes each in WebAssembly)
            let table_size = table.limits.min * 4; // 4 bytes per function reference
            
            // Align to pointer boundary
            offset = self.align_offset(offset, POINTER_ALIGNMENT);
            
            let segment = MemorySegment {
                offset,
                size: table_size,
                segment_type: SegmentType::Table,
                alignment: POINTER_ALIGNMENT,
            };
            
            layout.segments.push(segment)?;
            offset += table_size;
        }
        
        Ok(offset)
    }
    
    /// Layout stack space
    fn layout_stack(&mut self, module: &SimpleModule, layout: &mut MemoryLayout, mut offset: u32) -> Result<u32> {
        // Estimate stack size based on functions and call depth
        let estimated_stack_size = self.estimate_stack_size(module)?;
        
        // Align to 16-byte boundary for stack
        offset = self.align_offset(offset, 16);
        
        let segment = MemorySegment {
            offset,
            size: estimated_stack_size,
            segment_type: SegmentType::Stack,
            alignment: 16,
        };
        
        layout.segments.push(segment)?;
        layout.stack_size = estimated_stack_size;
        
        Ok(offset + estimated_stack_size)
    }
    
    /// Layout heap space
    fn layout_heap(&mut self, module: &SimpleModule, layout: &mut MemoryLayout, mut offset: u32) -> Result<u32> {
        // Calculate minimum heap size needed
        let min_heap_size = if self.asil_d_mode {
            16 * 1024 // 16KB minimum for ASIL-D
        } else {
            64 * 1024 // 64KB minimum for general use
        };
        
        // Check if module has memory declarations
        let heap_size = if let Some(memory) = module.memories.first() {
            let memory_size = memory.limits.min * WASM_PAGE_SIZE;
            memory_size.saturating_sub(offset).max(min_heap_size)
        } else {
            min_heap_size
        };
        
        // Align to page boundary
        offset = self.align_offset(offset, WASM_PAGE_SIZE);
        
        let segment = MemorySegment {
            offset,
            size: heap_size,
            segment_type: SegmentType::Heap,
            alignment: WASM_PAGE_SIZE,
        };
        
        layout.segments.push(segment)?;
        layout.heap_size = heap_size;
        
        Ok(offset + heap_size)
    }
    
    /// Estimate stack size requirements
    fn estimate_stack_size(&self, module: &SimpleModule) -> Result<u32> {
        let mut max_stack_depth = 0u32;
        let mut max_locals_size = 0u32;
        
        // Analyze each function
        for (func_idx, function_body) in module.code.iter().enumerate() {
            // Calculate locals size
            let mut locals_size = 0u32;
            for local_entry in &function_body.locals {
                let (type_size, _) = self.get_type_size_alignment(local_entry.value_type);
                locals_size += type_size * local_entry.count;
            }
            max_locals_size = max_locals_size.max(locals_size);
            
            // Estimate call depth (simplified analysis)
            let call_depth = self.estimate_function_call_depth(function_body.code.as_slice())?;
            max_stack_depth = max_stack_depth.max(call_depth);
        }
        
        // Calculate total stack requirement
        let frame_size = 64; // Estimated frame overhead
        let total_stack = (max_stack_depth * frame_size) + max_locals_size;
        
        // Apply ASIL-D limits
        if self.asil_d_mode {
            Ok(total_stack.min(32 * 1024)) // 32KB max for ASIL-D
        } else {
            Ok(total_stack.min(1024 * 1024)) // 1MB max for general use
        }
    }
    
    /// Estimate function call depth (simplified)
    fn estimate_function_call_depth(&self, _bytecode: &[u8]) -> Result<u32> {
        // Simplified estimation - would need full instruction parsing for accuracy
        // For now, assume moderate nesting
        if self.asil_d_mode {
            Ok(8) // Conservative depth for ASIL-D
        } else {
            Ok(32) // Higher depth for general use
        }
    }
    
    /// Get size and alignment for a value type
    fn get_type_size_alignment(&self, value_type: ValueType) -> (u32, u32) {
        match value_type {
            ValueType::I32 => (4, I32_ALIGNMENT),
            ValueType::I64 => (8, I64_ALIGNMENT),
            ValueType::F32 => (4, F32_ALIGNMENT),
            ValueType::F64 => (8, F64_ALIGNMENT),
            ValueType::V128 => (16, 16), // 128-bit SIMD
            ValueType::FuncRef => (4, POINTER_ALIGNMENT), // Function pointer
            ValueType::ExternRef => (4, POINTER_ALIGNMENT), // External reference
        }
    }
    
    /// Align offset to required boundary
    fn align_offset(&self, offset: u32, alignment: u32) -> u32 {
        let padding = (alignment - (offset % alignment)) % alignment;
        offset + padding
    }
    
    /// Get memory pool statistics
    pub fn memory_stats(&self) -> &MemoryStats {
        self.pool.stats()
    }
}

impl Default for MemoryLayoutCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory-optimized parsing utilities
pub struct MemoryOptimizedParser {
    /// Memory pool for allocations
    pool: MemoryPool,
    
    /// Layout calculator
    layout_calc: MemoryLayoutCalculator,
    
    /// Zero-copy parsing mode
    zero_copy_mode: bool,
}

impl MemoryOptimizedParser {
    /// Create memory-optimized parser
    pub fn new() -> Self {
        Self {
            pool: MemoryPool::new(),
            layout_calc: MemoryLayoutCalculator::new(),
            zero_copy_mode: false,
        }
    }
    
    /// Enable zero-copy parsing mode
    pub fn enable_zero_copy(&mut self) {
        self.zero_copy_mode = true;
    }
    
    /// Parse string with memory reuse
    pub fn parse_string(&mut self, data: &[u8], offset: usize, length: usize) -> Result<String> {
        if offset + length > data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "String extends beyond data bounds"
            ));
        }
        
        if self.zero_copy_mode {
            // In zero-copy mode, we still need to validate UTF-8
            let str_slice = core::str::from_utf8(&data[offset..offset + length])
                .map_err(|_| Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid UTF-8 in string"
                ))?;
            Ok(str_slice.to_string())
        } else {
            // Use pool for memory management
            let mut string = self.pool.get_string_buffer(length)?;
            let str_slice = core::str::from_utf8(&data[offset..offset + length])
                .map_err(|_| Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid UTF-8 in string"
                ))?;
            string.push_str(str_slice);
            Ok(string)
        }
    }
    
    /// Parse byte vector with memory reuse
    pub fn parse_bytes(&mut self, data: &[u8], offset: usize, length: usize) -> Result<Vec<u8>> {
        if offset + length > data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Byte vector extends beyond data bounds"
            ));
        }
        
        let mut vec = self.pool.get_byte_vector(length)?;
        vec.extend_from_slice(&data[offset..offset + length]);
        Ok(vec)
    }
    
    /// Calculate module memory layout
    pub fn calculate_memory_layout(&mut self, module: &SimpleModule) -> Result<MemoryLayout> {
        self.layout_calc.calculate_layout(module)
    }
    
    /// Get memory statistics
    pub fn get_memory_stats(&self) -> &MemoryStats {
        self.pool.stats()
    }
}

impl Default for MemoryOptimizedParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_pool_creation() {
        let pool = MemoryPool::new();
        assert_eq!(pool.stats.allocation_count, 0);
        assert_eq!(pool.stats.current_usage, 0);
    }
    
    #[test]
    fn test_memory_pool_asil_d() {
        let pool = MemoryPool::asil_d();
        assert_eq!(pool.max_vector_size, 64 * 1024);
        assert_eq!(pool.max_vectors, 16);
    }
    
    #[test]
    fn test_type_size_alignment() {
        let calc = MemoryLayoutCalculator::new();
        
        assert_eq!(calc.get_type_size_alignment(ValueType::I32), (4, 4));
        assert_eq!(calc.get_type_size_alignment(ValueType::I64), (8, 8));
        assert_eq!(calc.get_type_size_alignment(ValueType::F32), (4, 4));
        assert_eq!(calc.get_type_size_alignment(ValueType::F64), (8, 8));
    }
    
    #[test]
    fn test_offset_alignment() {
        let calc = MemoryLayoutCalculator::new();
        
        assert_eq!(calc.align_offset(0, 4), 0);
        assert_eq!(calc.align_offset(1, 4), 4);
        assert_eq!(calc.align_offset(3, 4), 4);
        assert_eq!(calc.align_offset(4, 4), 4);
        assert_eq!(calc.align_offset(5, 8), 8);
    }
    
    #[test]
    fn test_empty_module_layout() {
        let mut calc = MemoryLayoutCalculator::new();
        let module = SimpleModule::new();
        
        let layout = calc.calculate_layout(&module).unwrap();
        assert!(layout.total_size > 0); // Should have at least stack and heap
        assert!(layout.segments.len() >= 2); // At least stack and heap segments
    }
    
    #[test]
    fn test_memory_optimized_parser() {
        let parser = MemoryOptimizedParser::new();
        assert!(!parser.zero_copy_mode);
        assert_eq!(parser.get_memory_stats().allocation_count, 0);
    }
    
    #[test]
    fn test_parse_string_bounds_check() {
        let mut parser = MemoryOptimizedParser::new();
        let data = b"hello";
        
        // Should fail when accessing beyond bounds
        let result = parser.parse_string(data, 0, 10);
        assert!(result.is_err());
        
        // Should succeed within bounds
        let result = parser.parse_string(data, 0, 5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }
}