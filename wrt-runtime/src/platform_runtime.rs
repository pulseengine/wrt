// WRT - wrt-runtime
// Module: Platform-Aware Runtime Implementation
// SW-REQ-ID: REQ_RUNTIME_PLATFORM_001
//
// Copyright (c) 2025 The WRT Project Developers
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Platform-Aware WebAssembly Runtime
//!
//! This module provides a runtime that adapts to platform-specific capabilities
//! and resource limits, integrating with the CFI engine and memory management
//! to provide optimal performance within platform constraints.

#![allow(clippy::module_name_repetitions)]

use crate::{
    foundation_stubs::{SafetyContext, UnifiedMemoryProvider, AsilLevel, MediumProvider},
    component_stubs::ComponentId,
    cfi_engine::{CfiExecutionEngine, CfiViolationPolicy},
    execution::ExecutionContext,
    func::Function as RuntimeFunction,
    unified_types::UnifiedMemoryAdapter as UnifiedMemoryAdapterTrait,
    prelude::*,
};

// Import Box, Vec, and other types for allocating memory adapters
#[cfg(feature = "std")]
use std::{boxed::Box, vec::Vec};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{boxed::Box, vec::Vec};

// For no_std without alloc, use BoundedVec instead of Vec
#[cfg(not(any(feature = "std", feature = "alloc")))]
use wrt_foundation::bounded::BoundedVec;
#[cfg(not(any(feature = "std", feature = "alloc")))]
type Vec<T> = BoundedVec<T, 16, wrt_foundation::NoStdProvider<1024>>;

// Import Value type
use wrt_foundation::Value;
// CFI imports temporarily disabled since CFI module is disabled
// use wrt_instructions::CfiControlFlowProtection;
use crate::cfi_engine::CfiControlFlowProtection;
use wrt_error::{Error, ErrorCategory, Result};

// Import from wrt-platform for comprehensive platform limits
#[cfg(feature = "std")]
use wrt_platform::{ComprehensivePlatformLimits, PlatformId};

// Stub definitions for when wrt-platform is not available
#[cfg(not(feature = "std"))]
mod platform_stubs {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PlatformId {
        Linux,
        QNX,
        MacOS,
        Embedded,
        Zephyr,
    }
    
    #[derive(Debug, Clone)]
    pub struct ComprehensivePlatformLimits {
        pub platform_id: PlatformId,
        pub max_total_memory: usize,
        pub max_stack_bytes: usize,
        pub max_components: usize,
        pub asil_level: crate::foundation_stubs::AsilLevel,
    }
    
    impl Default for ComprehensivePlatformLimits {
        fn default() -> Self {
            Self {
                platform_id: PlatformId::Linux,
                max_total_memory: 64 * 1024 * 1024,
                max_stack_bytes: 1024 * 1024,
                max_components: 64,
                asil_level: crate::foundation_stubs::AsilLevel::QM,
            }
        }
    }
}

#[cfg(not(feature = "std"))]
use platform_stubs::{ComprehensivePlatformLimits, PlatformId};

// Helper function to convert between ASIL level types
#[cfg(feature = "std")]
fn convert_asil_level(platform_asil: wrt_platform::AsilLevel) -> AsilLevel {
    match platform_asil {
        wrt_platform::AsilLevel::QM => AsilLevel::QM,
        wrt_platform::AsilLevel::AsilA => AsilLevel::A,
        wrt_platform::AsilLevel::AsilB => AsilLevel::B,
        wrt_platform::AsilLevel::AsilC => AsilLevel::C,
        wrt_platform::AsilLevel::AsilD => AsilLevel::D,
    }
}

// No conversion needed for no_std since it already uses foundation_stubs::AsilLevel
#[cfg(not(feature = "std"))]
fn convert_asil_level(asil: AsilLevel) -> AsilLevel {
    asil
}

/// Simple platform memory adapter trait for platform_runtime.rs
pub trait PlatformMemoryAdapter: Send + Sync + Debug {
    fn allocate(&mut self, size: usize) -> Result<&mut [u8]>;
    fn deallocate(&mut self, ptr: &mut [u8]) -> Result<()>;
    fn available_memory(&self) -> usize;
    fn total_memory(&self) -> usize;
    fn platform_id(&self) -> PlatformId;
}

/// Platform-aware WebAssembly runtime
pub struct PlatformAwareRuntime {
    /// Execution engine with CFI protection
    execution_engine: CfiExecutionEngine,
    /// Unified memory adapter for the platform
    #[cfg(any(feature = "std", feature = "alloc"))]
    memory_adapter: Box<dyn PlatformMemoryAdapter>,
    /// Memory adapter for no_std environments
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    memory_adapter: GenericMemoryAdapter,
    /// Platform-specific limits and capabilities
    platform_limits: ComprehensivePlatformLimits,
    /// Safety context for ASIL compliance
    safety_context: SafetyContext,
    /// Runtime statistics and metrics
    metrics: RuntimeMetrics,
}


/// Runtime performance and resource metrics
#[derive(Debug, Clone, Default)]
pub struct RuntimeMetrics {
    /// Total instructions executed
    pub instructions_executed: u64,
    /// Total memory allocated
    pub memory_allocated: usize,
    /// Peak memory usage
    pub peak_memory_usage: usize,
    /// Number of components instantiated
    pub components_instantiated: u32,
    /// CFI violations detected
    pub cfi_violations: u64,
    /// Execution time in nanoseconds
    pub execution_time_ns: u64,
}

impl PlatformAwareRuntime {
    /// Create new platform-aware runtime
    pub fn new(limits: ComprehensivePlatformLimits) -> Result<Self> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        let memory_adapter = Self::create_memory_adapter(&limits)?;
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        let memory_adapter = Self::create_memory_adapter_nostd(&limits)?;
        
        let cfi_protection = Self::create_cfi_protection(&limits);
        let execution_engine = CfiExecutionEngine::new(cfi_protection);
        let safety_context = SafetyContext::new(convert_asil_level(limits.asil_level));
        
        Ok(Self {
            execution_engine,
            memory_adapter,
            platform_limits: limits,
            safety_context,
            metrics: RuntimeMetrics::default(),
        })
    }
    
    /// Create runtime with custom CFI violation policy
    pub fn new_with_cfi_policy(
        limits: ComprehensivePlatformLimits,
        cfi_policy: CfiViolationPolicy,
    ) -> Result<Self> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        let memory_adapter = Self::create_memory_adapter(&limits)?;
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        let memory_adapter = Self::create_memory_adapter_nostd(&limits)?;
        
        let cfi_protection = Self::create_cfi_protection(&limits);
        let execution_engine = CfiExecutionEngine::new_with_policy(cfi_protection, cfi_policy);
        let safety_context = SafetyContext::new(convert_asil_level(limits.asil_level));
        
        Ok(Self {
            execution_engine,
            memory_adapter,
            platform_limits: limits,
            safety_context,
            metrics: RuntimeMetrics::default(),
        })
    }
    
    /// Execute WebAssembly function with platform-aware resource management
    pub fn execute_function(
        &mut self,
        function: &RuntimeFunction,
        args: &[Value],
    ) -> Result<Vec<Value>> {
        let start_time = self.get_timestamp();
        
        // Validate execution against platform limits
        self.validate_execution_limits(function, args)?;
        
        // Create execution context with platform limits
        let mut execution_context = ExecutionContext::new_with_limits(
            self.platform_limits.max_stack_bytes / 8, // Approximate stack depth
        );
        
        // Execute with CFI protection
        let instruction = self.create_call_instruction(function);
        let cfi_result = self.execution_engine.execute_instruction_with_cfi(
            &instruction,
            &mut execution_context,
        )?;
        
        // Update metrics
        let end_time = self.get_timestamp();
        self.metrics.instructions_executed += 1;
        self.metrics.execution_time_ns += end_time.saturating_sub(start_time);
        self.update_memory_metrics();
        
        // Extract return values from CFI result
        self.extract_return_values(cfi_result, args.len())
    }
    
    /// Instantiate component with resource budget validation
    pub fn instantiate_component(&mut self, component_bytes: &[u8]) -> Result<ComponentId> {
        // Validate component against platform limits
        let requirements = self.analyze_component_requirements(component_bytes)?;
        
        if requirements.memory_usage > self.memory_adapter.available_memory() {
            return Err(Error::new(
                ErrorCategory::Resource,
                wrt_error::codes::MEMORY_ALLOCATION_ERROR,
                "Insufficient memory for component instantiation",
            ));
        }
        
        if self.metrics.components_instantiated >= self.platform_limits.max_components as u32 {
            return Err(Error::new(
                ErrorCategory::Resource,
                wrt_error::codes::RESOURCE_LIMIT_EXCEEDED,
                "Maximum component count exceeded",
            ));
        }
        
        // Create component instance with bounded resources
        let component_id = ComponentId::new(self.metrics.components_instantiated);
        self.metrics.components_instantiated += 1;
        
        Ok(component_id)
    }
    
    /// Get current runtime metrics
    pub fn metrics(&self) -> &RuntimeMetrics {
        &self.metrics
    }
    
    /// Get platform limits
    pub fn platform_limits(&self) -> &ComprehensivePlatformLimits {
        &self.platform_limits
    }
    
    /// Get safety context
    pub fn safety_context(&self) -> &SafetyContext {
        &self.safety_context
    }
    
    /// Get memory adapter
    pub fn memory_adapter(&self) -> &dyn PlatformMemoryAdapter {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            self.memory_adapter.as_ref()
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            &self.memory_adapter
        }
    }
    
    /// Create platform-specific memory adapter
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn create_memory_adapter(limits: &ComprehensivePlatformLimits) -> Result<Box<dyn PlatformMemoryAdapter>> {
        match limits.platform_id {
            PlatformId::Linux => Ok(Box::new(LinuxMemoryAdapter::new(limits.max_total_memory)?)),
            PlatformId::QNX => Ok(Box::new(QnxMemoryAdapter::new(limits.max_total_memory)?)),
            PlatformId::Embedded => Ok(Box::new(EmbeddedMemoryAdapter::new(limits.max_total_memory)?)),
            PlatformId::MacOS => Ok(Box::new(MacOSMemoryAdapter::new(limits.max_total_memory)?)),
            _ => Ok(Box::new(GenericMemoryAdapter::new(limits.max_total_memory)?)),
        }
    }
    
    /// Create memory adapter for no_std environments
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn create_memory_adapter_nostd(limits: &ComprehensivePlatformLimits) -> Result<GenericMemoryAdapter> {
        // In no_std, always use GenericMemoryAdapter with fixed size
        GenericMemoryAdapter::new(limits.max_total_memory)
    }
    
    /// Create CFI protection configuration based on platform capabilities
    fn create_cfi_protection(limits: &ComprehensivePlatformLimits) -> CfiControlFlowProtection {
        let protection_level = match convert_asil_level(limits.asil_level) {
            AsilLevel::QM => 0, // Basic protection level
            AsilLevel::A | AsilLevel::B => 1, // Enhanced protection level
            AsilLevel::C | AsilLevel::D => 2, // Maximum protection level
        };
        
        CfiControlFlowProtection::new_with_level(protection_level)
    }
    
    /// Validate execution against platform limits
    fn validate_execution_limits(&self, function: &RuntimeFunction, args: &[Value]) -> Result<()> {
        // Check stack depth estimate
        let estimated_stack = (args.len() + 32) * 8; // Rough estimate
        if estimated_stack > self.platform_limits.max_stack_bytes {
            return Err(Error::new(
                ErrorCategory::Resource,
                wrt_error::codes::STACK_OVERFLOW,
                "Function call would exceed stack limits",
            ));
        }
        
        // Check memory availability
        if self.memory_adapter.available_memory() < 4096 {
            return Err(Error::new(
                ErrorCategory::Resource,
                wrt_error::codes::MEMORY_ALLOCATION_ERROR,
                "Insufficient memory for function execution",
            ));
        }
        
        Ok(())
    }
    
    /// Create call instruction for function execution
    fn create_call_instruction(&self, function: &RuntimeFunction) -> Instruction {
        // Create a call instruction for the function
        Instruction::Call(function.index().unwrap_or(0))
    }
    
    /// Analyze component resource requirements
    fn analyze_component_requirements(&self, component_bytes: &[u8]) -> Result<crate::component_stubs::ComponentRequirements> {
        // Simple analysis - in real implementation this would parse the component
        let memory_usage = component_bytes.len() * 2; // Estimate 2x size for runtime overhead
        
        Ok(crate::component_stubs::ComponentRequirements {
            component_count: 1,
            resource_count: 0,
            memory_usage,
        })
    }
    
    /// Update memory usage metrics
    fn update_memory_metrics(&mut self) {
        let current_usage = self.memory_adapter.total_memory() - self.memory_adapter.available_memory();
        self.metrics.memory_allocated = current_usage;
        if current_usage > self.metrics.peak_memory_usage {
            self.metrics.peak_memory_usage = current_usage;
        }
    }
    
    /// Extract return values from CFI execution result
    fn extract_return_values(
        &self,
        _cfi_result: crate::cfi_engine::CfiExecutionResult,
        _arg_count: usize,
    ) -> Result<Vec<Value>> {
        // Simplified implementation - in real scenario this would extract actual values
        #[cfg(feature = "std")]
        {
            use std::vec;
            Ok(vec![Value::I32(0)])
        }
        #[cfg(all(not(feature = "std"), feature = "alloc"))]
        {
            use alloc::vec;
            Ok(vec![Value::I32(0)])
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            // For no_std no_alloc, return a fixed array wrapped as Vec-like
            use wrt_foundation::bounded::BoundedVec;
            use wrt_foundation::safe_memory::NoStdProvider;
            let provider = NoStdProvider::<1024>::new();
            let mut result: BoundedVec<Value, 16, _> = BoundedVec::new(provider).map_err(|_| Error::new(
                ErrorCategory::Memory,
                wrt_error::codes::MEMORY_ALLOCATION_ERROR,
                "Failed to create result vector",
            ))?;
            result.push(Value::I32(0)).map_err(|_| Error::new(
                ErrorCategory::Memory,
                wrt_error::codes::MEMORY_ALLOCATION_ERROR,
                "Failed to push result value",
            ))?;
            // Convert to Vec for compatibility - this is a temporary workaround
            Err(Error::new(
                ErrorCategory::Runtime,
                wrt_error::codes::UNSUPPORTED_OPERATION,
                "Function execution not supported in no_std no_alloc mode",
            ))
        }
    }
    
    /// Get current timestamp for performance tracking
    fn get_timestamp(&self) -> u64 {
        // Platform-specific timestamp implementation
        #[cfg(feature = "std")]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
        }
        #[cfg(not(feature = "std"))]
        {
            // Simple counter for no_std environments
            use core::sync::atomic::{AtomicU64, Ordering};
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            COUNTER.fetch_add(1, Ordering::Relaxed)
        }
    }
}

/// Linux-specific memory adapter (requires alloc feature)
#[cfg(any(feature = "std", feature = "alloc"))]
#[derive(Debug)]
struct LinuxMemoryAdapter {
    memory: Vec<u8>,
    allocated: usize,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl LinuxMemoryAdapter {
    fn new(size: usize) -> Result<Self> {
        Ok(Self {
            memory: vec![0; size],
            allocated: 0,
        })
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl PlatformMemoryAdapter for LinuxMemoryAdapter {
    fn allocate(&mut self, size: usize) -> Result<&mut [u8]> {
        if self.allocated + size > self.memory.len() {
            return Err(Error::new(
                ErrorCategory::Resource,
                wrt_error::codes::MEMORY_ALLOCATION_ERROR,
                "Linux memory allocation failed",
            ));
        }
        
        let start = self.allocated;
        self.allocated += size;
        let slice = self.memory.as_mut_slice(); Ok(&mut slice[start..self.allocated])
    }
    
    fn deallocate(&mut self, _ptr: &mut [u8]) -> Result<()> {
        // Simple implementation - reset allocation
        self.allocated = 0;
        Ok(())
    }
    
    fn available_memory(&self) -> usize {
        self.memory.len() - self.allocated
    }
    
    fn total_memory(&self) -> usize {
        self.memory.len()
    }
    
    fn platform_id(&self) -> PlatformId {
        PlatformId::Linux
    }
}

/// QNX-specific memory adapter
#[derive(Debug)]
struct QnxMemoryAdapter {
    memory: Vec<u8>,
    allocated: usize,
}

impl QnxMemoryAdapter {
    fn new(size: usize) -> Result<Self> {
        Ok(Self {
            memory: vec![0; size],
            allocated: 0,
        })
    }
}

impl PlatformMemoryAdapter for QnxMemoryAdapter {
    
    fn allocate(&mut self, size: usize) -> Result<&mut [u8]> {
        if self.allocated + size > self.memory.len() {
            return Err(Error::new(
                ErrorCategory::Resource,
                wrt_error::codes::MEMORY_ALLOCATION_ERROR,
                "QNX memory allocation failed",
            ));
        }
        
        let start = self.allocated;
        self.allocated += size;
        let slice = self.memory.as_mut_slice(); Ok(&mut slice[start..self.allocated])
    }
    
    fn deallocate(&mut self, _ptr: &mut [u8]) -> Result<()> {
        self.allocated = 0;
        Ok(())
    }
    
    fn available_memory(&self) -> usize {
        self.memory.len() - self.allocated
    }
    
    fn total_memory(&self) -> usize {
        self.memory.len()
    }
    
    fn platform_id(&self) -> PlatformId {
        PlatformId::QNX
    }
}

// LinuxMemoryAdapter platform_id is now part of the trait implementation

/// Embedded system memory adapter
#[derive(Debug)]
struct EmbeddedMemoryAdapter {
    buffer: [u8; 65536], // Fixed 64KB buffer for embedded
    allocated: usize,
}

impl EmbeddedMemoryAdapter {
    fn new(_size: usize) -> Result<Self> {
        Ok(Self {
            buffer: [0; 65536],
            allocated: 0,
        })
    }
}

impl PlatformMemoryAdapter for EmbeddedMemoryAdapter {
    
    fn allocate(&mut self, size: usize) -> Result<&mut [u8]> {
        if self.allocated + size > self.buffer.len() {
            return Err(Error::new(
                ErrorCategory::Resource,
                wrt_error::codes::MEMORY_ALLOCATION_ERROR,
                "Embedded memory allocation failed",
            ));
        }
        
        let start = self.allocated;
        self.allocated += size;
        Ok(&mut self.buffer[start..self.allocated])
    }
    
    fn deallocate(&mut self, _ptr: &mut [u8]) -> Result<()> {
        self.allocated = 0;
        Ok(())
    }
    
    fn available_memory(&self) -> usize {
        self.buffer.len() - self.allocated
    }
    
    fn total_memory(&self) -> usize {
        self.buffer.len()
    }
    
    fn platform_id(&self) -> PlatformId {
        PlatformId::Zephyr  // Use Zephyr as the embedded platform ID
    }
}

// EmbeddedMemoryAdapter platform_id is now part of the trait implementation

/// macOS-specific memory adapter
#[derive(Debug)]
struct MacOSMemoryAdapter {
    memory: Vec<u8>,
    allocated: usize,
}

impl MacOSMemoryAdapter {
    fn new(size: usize) -> Result<Self> {
        Ok(Self {
            memory: vec![0; size],
            allocated: 0,
        })
    }
}

impl PlatformMemoryAdapter for MacOSMemoryAdapter {
    
    fn allocate(&mut self, size: usize) -> Result<&mut [u8]> {
        if self.allocated + size > self.memory.len() {
            return Err(Error::new(
                ErrorCategory::Resource,
                wrt_error::codes::MEMORY_ALLOCATION_ERROR,
                "macOS memory allocation failed",
            ));
        }
        
        let start = self.allocated;
        self.allocated += size;
        let slice = self.memory.as_mut_slice(); Ok(&mut slice[start..self.allocated])
    }
    
    fn deallocate(&mut self, _ptr: &mut [u8]) -> Result<()> {
        self.allocated = 0;
        Ok(())
    }
    
    fn available_memory(&self) -> usize {
        self.memory.len() - self.allocated
    }
    
    fn total_memory(&self) -> usize {
        self.memory.len()
    }
    
    fn platform_id(&self) -> PlatformId {
        PlatformId::MacOS
    }
}

// MacOSMemoryAdapter platform_id is now part of the trait implementation

/// Generic memory adapter for unknown platforms
#[derive(Debug, Clone)]
pub struct GenericMemoryAdapter {
    #[cfg(any(feature = "std", feature = "alloc"))]
    memory: Vec<u8>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    memory: [u8; 65536], // Fixed size for no_std
    allocated: usize,
}

impl GenericMemoryAdapter {
    fn new(size: usize) -> Result<Self> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            Ok(Self {
                memory: vec![0; size],
                allocated: 0,
            })
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            let _ = size; // Ignore size parameter in no_std
            Ok(Self {
                memory: [0; 65536],
                allocated: 0,
            })
        }
    }
}

impl PlatformMemoryAdapter for GenericMemoryAdapter {
    
    fn allocate(&mut self, size: usize) -> Result<&mut [u8]> {
        let memory_len = self.memory.len();
        if self.allocated + size > memory_len {
            return Err(Error::new(
                ErrorCategory::Resource,
                wrt_error::codes::MEMORY_ALLOCATION_ERROR,
                "Generic memory allocation failed",
            ));
        }
        
        let start = self.allocated;
        self.allocated += size;
        let slice = self.memory.as_mut_slice(); Ok(&mut slice[start..self.allocated])
    }
    
    fn deallocate(&mut self, _ptr: &mut [u8]) -> Result<()> {
        self.allocated = 0;
        Ok(())
    }
    
    fn available_memory(&self) -> usize {
        self.memory.len() - self.allocated
    }
    
    fn total_memory(&self) -> usize {
        self.memory.len()
    }
    
    fn platform_id(&self) -> PlatformId {
        PlatformId::Linux  // Use Linux as default for generic adapter
    }
}

// GenericMemoryAdapter platform_id is now part of the trait implementation

/// Create a generic memory adapter for use by other modules
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn create_generic_memory_adapter(size: usize) -> Result<Box<dyn PlatformMemoryAdapter>> {
    let adapter = GenericMemoryAdapter::new(size)?;
    Ok(Box::new(adapter))
}

/// Create a memory adapter suitable for the current platform
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn create_platform_memory_adapter(size: usize) -> Result<Box<dyn PlatformMemoryAdapter>> {
    // For now, use the generic adapter. In the future, this could detect
    // the platform and return the appropriate adapter type.
    create_generic_memory_adapter(size)
}

/// Create a memory adapter suitable for the current platform (no_std version)
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub fn create_platform_memory_adapter(size: usize) -> Result<GenericMemoryAdapter> {
    GenericMemoryAdapter::new(size)
}

/// For no_std environments, create a static generic adapter
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub fn create_generic_memory_adapter_static(size: usize) -> Result<GenericMemoryAdapter> {
    GenericMemoryAdapter::new(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrt_foundation::platform_abstraction::ComprehensivePlatformLimits;

    #[test]
    fn test_platform_runtime_creation() {
        let limits = ComprehensivePlatformLimits::default();
        let runtime = PlatformAwareRuntime::new(limits.clone());
        
        assert!(runtime.is_ok());
        let runtime = runtime.unwrap();
        assert_eq!(runtime.platform_limits.platform_id, limits.platform_id);
    }
    
    #[test]
    fn test_memory_adapter_allocation() {
        let limits = ComprehensivePlatformLimits::default();
        let mut runtime = PlatformAwareRuntime::new(limits).unwrap();
        
        let initial_available = runtime.memory_adapter.available_memory();
        assert!(initial_available > 0);
    }
    
    #[test]
    fn test_component_instantiation_limits() {
        let mut limits = ComprehensivePlatformLimits::default();
        limits.max_components = 1;
        
        let mut runtime = PlatformAwareRuntime::new(limits).unwrap();
        
        // First component should succeed
        let component_bytes = b"dummy component";
        let result1 = runtime.instantiate_component(component_bytes);
        assert!(result1.is_ok());
        
        // Second component should fail due to limit
        let result2 = runtime.instantiate_component(component_bytes);
        assert!(result2.is_err());
    }
    
    #[test]
    fn test_embedded_memory_adapter() {
        let adapter = EmbeddedMemoryAdapter::new(0).unwrap();
        assert_eq!(adapter.total_memory(), 65536);
        assert_eq!(adapter.platform_id(), PlatformId::Embedded);
    }
}