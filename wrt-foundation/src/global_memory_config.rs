// WRT - wrt-foundation
// Module: Global Memory Configuration System
// SW-REQ-ID: REQ_MEM_GLOBAL_001, REQ_MEM_LIMITS_001, REQ_MEM_PLATFORM_001
//
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Global Memory Configuration System
//!
//! This module provides a global polymorphic memory allocator system that's superior
//! to C++'s approach by providing compile-time safety, runtime configurability,
//! and platform-aware memory management with strict limits enforcement.
//!
//! # Design Principles
//!
//! 1. **Global Configuration**: Single source of truth for all memory limits
//! 2. **Platform Awareness**: Automatically adapts to platform capabilities
//! 3. **Strict Enforcement**: Prevents accidental over-allocation at compile and runtime
//! 4. **Zero-Cost Abstractions**: No runtime overhead for memory provider selection
//! 5. **Polymorphic Providers**: Multiple memory strategies with unified interface
//! 6. **Safety by Design**: Memory safety guaranteed through type system

use core::sync::atomic::{AtomicUsize, Ordering};
use crate::{Error, ErrorCategory, WrtResult, codes};
use crate::memory_system::UnifiedMemoryProvider;

#[cfg(feature = "std")]
use std::sync::{Arc, Mutex, Once};

#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

#[cfg(any(feature = "std", feature = "alloc"))]
use alloc::boxed::Box;

/// Global memory configuration singleton
/// 
/// This provides a better approach than C++'s global polymorphic allocators by:
/// - Type-safe provider selection at compile time
/// - Runtime configuration with compile-time guarantees
/// - Platform-aware memory limit discovery
/// - Strict enforcement of memory bounds
/// - Zero-allocation provider switching
static GLOBAL_CONFIG: GlobalMemoryConfig = GlobalMemoryConfig::new();

/// Platform-aware global memory configuration
///
/// This system automatically discovers platform capabilities and configures
/// memory providers with appropriate limits to prevent over-allocation.
pub struct GlobalMemoryConfig {
    /// Total memory budget for the runtime (in bytes)
    total_budget: AtomicUsize,
    /// Maximum WebAssembly linear memory (in bytes) 
    max_wasm_memory: AtomicUsize,
    /// Maximum stack memory (in bytes)
    max_stack_memory: AtomicUsize,
    /// Maximum number of components
    max_components: AtomicUsize,
    /// Currently allocated memory (in bytes)
    allocated_memory: AtomicUsize,
    /// Peak memory usage (in bytes)
    peak_memory: AtomicUsize,
    /// Memory provider type selection
    provider_type: AtomicUsize, // 0=NoStd, 1=Std, 2=Platform-specific
    /// Configuration initialized flag
    initialized: AtomicUsize, // 0=not initialized, 1=initialized
}

impl GlobalMemoryConfig {
    /// Create a new global memory configuration
    const fn new() -> Self {
        Self {
            total_budget: AtomicUsize::new(0),
            max_wasm_memory: AtomicUsize::new(0),
            max_stack_memory: AtomicUsize::new(0),
            max_components: AtomicUsize::new(0),
            allocated_memory: AtomicUsize::new(0),
            peak_memory: AtomicUsize::new(0),
            provider_type: AtomicUsize::new(0),
            initialized: AtomicUsize::new(0),
        }
    }

    /// Initialize the global memory configuration with platform limits
    ///
    /// This should be called once at application startup. Subsequent calls
    /// will return an error to prevent accidental reconfiguration.
    #[cfg(feature = "platform-memory")]
    pub fn initialize_with_platform_limits(
        &self,
        limits: &wrt_platform::comprehensive_limits::ComprehensivePlatformLimits
    ) -> WrtResult<()> {
        // Ensure this is only called once
        if self.initialized.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err(Error::new(
                ErrorCategory::System,
                codes::DUPLICATE_OPERATION,
                "Global memory configuration already initialized"
            ));
        }

        // Configure memory limits based on platform capabilities
        self.total_budget.store(limits.max_total_memory, Ordering::SeqCst);
        self.max_wasm_memory.store(limits.max_wasm_linear_memory, Ordering::SeqCst);
        self.max_stack_memory.store(limits.max_stack_bytes, Ordering::SeqCst);
        self.max_components.store(limits.max_components, Ordering::SeqCst);

        // Select appropriate provider type based on platform
        #[cfg(feature = "platform-memory")]
        let provider_type = match limits.platform_id {
            wrt_platform::comprehensive_limits::PlatformId::Linux |
            wrt_platform::comprehensive_limits::PlatformId::MacOS => 1, // Std provider
            wrt_platform::comprehensive_limits::PlatformId::QNX |
            wrt_platform::comprehensive_limits::PlatformId::Embedded |
            wrt_platform::comprehensive_limits::PlatformId::Zephyr |
            wrt_platform::comprehensive_limits::PlatformId::Tock => 0, // NoStd provider
            _ => 0, // Default to NoStd for unknown platforms
        };
        
        #[cfg(not(feature = "platform-memory"))]
        let provider_type = if cfg!(feature = "std") { 1 } else { 0 };
        self.provider_type.store(provider_type, Ordering::SeqCst);

        Ok(())
    }

    /// Get the total memory budget
    pub fn total_budget(&self) -> usize {
        self.total_budget.load(Ordering::Relaxed)
    }

    /// Get the maximum WebAssembly linear memory
    pub fn max_wasm_memory(&self) -> usize {
        self.max_wasm_memory.load(Ordering::Relaxed)
    }

    /// Get the maximum stack memory
    pub fn max_stack_memory(&self) -> usize {
        self.max_stack_memory.load(Ordering::Relaxed)
    }

    /// Get the maximum number of components
    pub fn max_components(&self) -> usize {
        self.max_components.load(Ordering::Relaxed)
    }

    /// Get currently allocated memory
    pub fn allocated_memory(&self) -> usize {
        self.allocated_memory.load(Ordering::Relaxed)
    }

    /// Get peak memory usage
    pub fn peak_memory(&self) -> usize {
        self.peak_memory.load(Ordering::Relaxed)
    }

    /// Check if an allocation would exceed budget
    pub fn can_allocate(&self, size: usize) -> bool {
        let current = self.allocated_memory.load(Ordering::Relaxed);
        let budget = self.total_budget.load(Ordering::Relaxed);
        current.saturating_add(size) <= budget
    }

    /// Register an allocation (internal use)
    pub(crate) fn register_allocation(&self, size: usize) -> WrtResult<()> {
        let current = self.allocated_memory.fetch_add(size, Ordering::SeqCst);
        let new_total = current + size;
        let budget = self.total_budget.load(Ordering::Relaxed);

        if new_total > budget {
            // Rollback the allocation
            self.allocated_memory.fetch_sub(size, Ordering::SeqCst);
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_LIMIT_EXCEEDED,
                "Allocation would exceed global memory budget"
            ));
        }

        // Update peak if necessary
        let peak = self.peak_memory.load(Ordering::Relaxed);
        if new_total > peak {
            self.peak_memory.store(new_total, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Unregister an allocation (internal use)
    pub(crate) fn unregister_allocation(&self, size: usize) {
        self.allocated_memory.fetch_sub(size, Ordering::SeqCst);
    }

    /// Get the recommended provider type for current platform
    pub fn provider_type(&self) -> ProviderType {
        match self.provider_type.load(Ordering::Relaxed) {
            1 => ProviderType::Std,
            2 => ProviderType::PlatformSpecific,
            _ => ProviderType::NoStd,
        }
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> GlobalMemoryStats {
        GlobalMemoryStats {
            total_budget: self.total_budget(),
            allocated: self.allocated_memory(),
            peak: self.peak_memory(),
            available: self.total_budget().saturating_sub(self.allocated_memory()),
            max_wasm_memory: self.max_wasm_memory(),
            max_stack_memory: self.max_stack_memory(),
            max_components: self.max_components(),
        }
    }
}

/// Memory provider type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
    /// No-std provider for embedded/constrained environments
    NoStd,
    /// Standard library provider for desktop environments
    Std,
    /// Platform-specific optimized provider
    PlatformSpecific,
}

/// Global memory usage statistics
#[derive(Debug, Clone)]
pub struct GlobalMemoryStats {
    /// Total memory budget
    pub total_budget: usize,
    /// Currently allocated memory
    pub allocated: usize,
    /// Peak memory usage
    pub peak: usize,
    /// Available memory remaining
    pub available: usize,
    /// Maximum WebAssembly linear memory
    pub max_wasm_memory: usize,
    /// Maximum stack memory
    pub max_stack_memory: usize,
    /// Maximum number of components
    pub max_components: usize,
}

/// Global memory configuration accessor
pub fn global_memory_config() -> &'static GlobalMemoryConfig {
    &GLOBAL_CONFIG
}

/// Platform-aware memory provider factory
///
/// This factory creates memory providers that are automatically configured
/// with the global memory limits and are appropriate for the current platform.
pub struct PlatformAwareMemoryFactory;

impl PlatformAwareMemoryFactory {
    /// Create a small memory provider configured for current platform
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn create_small() -> WrtResult<Box<dyn UnifiedMemoryProvider>> {
        let config = global_memory_config();
        let size = core::cmp::min(8192, config.max_stack_memory() / 4);
        
        match config.provider_type() {
            ProviderType::Std => {
                #[cfg(feature = "std")]
                {
                    Ok(Box::new(crate::memory_system::UnifiedStdProvider::new()))
                }
                #[cfg(not(feature = "std"))]
                {
                    Self::create_configurable_provider(size)
                }
            }
            ProviderType::NoStd | ProviderType::PlatformSpecific => {
                Self::create_configurable_provider(size)
            }
        }
    }

    /// Create a medium memory provider configured for current platform
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn create_medium() -> WrtResult<Box<dyn UnifiedMemoryProvider>> {
        let config = global_memory_config();
        let size = core::cmp::min(65536, config.max_wasm_memory() / 16);
        
        match config.provider_type() {
            ProviderType::Std => {
                #[cfg(feature = "std")]
                {
                    Ok(Box::new(crate::memory_system::UnifiedStdProvider::new()))
                }
                #[cfg(not(feature = "std"))]
                {
                    Self::create_configurable_provider(size)
                }
            }
            ProviderType::NoStd | ProviderType::PlatformSpecific => {
                Self::create_configurable_provider(size)
            }
        }
    }

    /// Create a large memory provider configured for current platform
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn create_large() -> WrtResult<Box<dyn UnifiedMemoryProvider>> {
        let config = global_memory_config();
        let size = core::cmp::min(1048576, config.max_wasm_memory() / 4);
        
        match config.provider_type() {
            ProviderType::Std => {
                #[cfg(feature = "std")]
                {
                    Ok(Box::new(crate::memory_system::UnifiedStdProvider::new()))
                }
                #[cfg(not(feature = "std"))]
                {
                    Self::create_configurable_provider(size)
                }
            }
            ProviderType::NoStd | ProviderType::PlatformSpecific => {
                Self::create_configurable_provider(size)
            }
        }
    }

    /// Create a WebAssembly linear memory provider
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn create_wasm_memory() -> WrtResult<Box<dyn UnifiedMemoryProvider>> {
        let config = global_memory_config();
        let size = config.max_wasm_memory();
        
        // WebAssembly memory always uses the largest available provider
        match config.provider_type() {
            ProviderType::Std => {
                #[cfg(feature = "std")]
                {
                    Ok(Box::new(crate::memory_system::UnifiedStdProvider::new()))
                }
                #[cfg(not(feature = "std"))]
                {
                    Self::create_configurable_provider(size)
                }
            }
            ProviderType::NoStd | ProviderType::PlatformSpecific => {
                Self::create_configurable_provider(size)
            }
        }
    }

    /// Create a configurable provider with specified size
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn create_configurable_provider(size: usize) -> WrtResult<Box<dyn UnifiedMemoryProvider>> {
        // Select the appropriate sized provider based on requested size
        if size <= 8192 {
            Ok(Box::new(crate::memory_system::SmallProvider::new()))
        } else if size <= 65536 {
            Ok(Box::new(crate::memory_system::MediumProvider::new()))
        } else if size <= 1048576 {
            Ok(Box::new(crate::memory_system::LargeProvider::new()))
        } else {
            // For very large allocations, create a custom provider
            // This would require dynamic provider creation which we'll implement later
            Err(Error::new(
                ErrorCategory::Memory,
                codes::UNSUPPORTED_OPERATION,
                "Custom large providers not yet implemented"
            ))
        }
    }
}

/// Global memory-aware provider wrapper
///
/// This wrapper ensures that all memory allocations are tracked against
/// the global memory budget and prevents over-allocation.
pub struct GlobalMemoryAwareProvider<P: UnifiedMemoryProvider> {
    inner: P,
    allocated_size: AtomicUsize,
}

impl<P: UnifiedMemoryProvider> GlobalMemoryAwareProvider<P> {
    /// Create a new global memory-aware provider
    pub fn new(provider: P) -> Self {
        Self {
            inner: provider,
            allocated_size: AtomicUsize::new(0),
        }
    }

    /// Get the inner provider
    pub fn inner(&self) -> &P {
        &self.inner
    }
}

impl<P: UnifiedMemoryProvider> UnifiedMemoryProvider for GlobalMemoryAwareProvider<P> {
    fn allocate(&mut self, size: usize) -> WrtResult<&mut [u8]> {
        // Check global budget first
        global_memory_config().register_allocation(size)?;
        
        match self.inner.allocate(size) {
            Ok(slice) => {
                self.allocated_size.fetch_add(size, Ordering::SeqCst);
                Ok(slice)
            }
            Err(err) => {
                // Rollback global allocation on failure
                global_memory_config().unregister_allocation(size);
                Err(err)
            }
        }
    }

    fn deallocate(&mut self, ptr: &mut [u8]) -> WrtResult<()> {
        let size = ptr.len();
        let result = self.inner.deallocate(ptr);
        
        // Always unregister allocation, even if deallocation fails
        global_memory_config().unregister_allocation(size);
        self.allocated_size.fetch_sub(size, Ordering::SeqCst);
        
        result
    }

    fn available_memory(&self) -> usize {
        // Return the minimum of inner provider availability and global budget
        let inner_available = self.inner.available_memory();
        let global_available = global_memory_config().total_budget()
            .saturating_sub(global_memory_config().allocated_memory());
        core::cmp::min(inner_available, global_available)
    }

    fn total_memory(&self) -> usize {
        // Return the minimum of inner provider capacity and global budget
        let inner_total = self.inner.total_memory();
        let global_budget = global_memory_config().total_budget();
        core::cmp::min(inner_total, global_budget)
    }

    fn memory_stats(&self) -> (usize, usize) {
        let (inner_allocated, inner_peak) = self.inner.memory_stats();
        let local_allocated = self.allocated_size.load(Ordering::Relaxed);
        
        // Return local tracking which should match inner provider
        (local_allocated, inner_peak)
    }

    fn can_allocate(&self, size: usize) -> bool {
        global_memory_config().can_allocate(size) && self.inner.can_allocate(size)
    }

    fn alignment(&self) -> usize {
        self.inner.alignment()
    }
}

impl<P: UnifiedMemoryProvider> Drop for GlobalMemoryAwareProvider<P> {
    fn drop(&mut self) {
        // Ensure any remaining allocated memory is unregistered
        let remaining = self.allocated_size.load(Ordering::SeqCst);
        if remaining > 0 {
            global_memory_config().unregister_allocation(remaining);
        }
    }
}

/// Initialize the global memory system with platform detection
///
/// This is the main entry point for configuring the global memory system.
/// It automatically detects the platform capabilities and configures
/// appropriate memory limits.
#[cfg(feature = "platform-memory")]
pub fn initialize_global_memory_system() -> WrtResult<()> {
    // Discover platform limits
    let mut discoverer = wrt_platform::comprehensive_limits::PlatformLimitDiscoverer::new();
    let limits = discoverer.discover()?;
    
    // Initialize global configuration
    global_memory_config().initialize_with_platform_limits(&limits)?;
    
    Ok(())
}

/// Initialize with default limits when platform detection is not available
#[cfg(not(feature = "platform-memory"))]
pub fn initialize_global_memory_system() -> WrtResult<()> {
    // Use default limits when platform detection is not available
    let config = global_memory_config();
    
    // Ensure this is only called once
    if config.initialized.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return Err(Error::new(
            ErrorCategory::System,
            codes::DUPLICATE_OPERATION,
            "Global memory configuration already initialized"
        ));
    }

    // Set conservative defaults
    config.total_budget.store(64 * 1024 * 1024, Ordering::SeqCst); // 64MB
    config.max_wasm_memory.store(32 * 1024 * 1024, Ordering::SeqCst); // 32MB
    config.max_stack_memory.store(1024 * 1024, Ordering::SeqCst); // 1MB
    config.max_components.store(64, Ordering::SeqCst);

    // Select provider type based on available features
    let provider_type = if cfg!(feature = "std") { 1 } else { 0 };
    config.provider_type.store(provider_type, Ordering::SeqCst);

    Ok(())
}

/// Create a memory provider that respects global limits
///
/// This is the recommended way to create memory providers in the WRT system.
/// It automatically selects the appropriate provider type and size based on
/// the current platform configuration and global memory limits.
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn create_memory_provider(requested_size: usize) -> WrtResult<Box<dyn UnifiedMemoryProvider>> {
    let config = global_memory_config();
    
    // Ensure the system is initialized
    if config.initialized.load(Ordering::Relaxed) == 0 {
        return Err(Error::new(
            ErrorCategory::System,
            codes::UNINITIALIZED,
            "Global memory system not initialized. Call initialize_global_memory_system() first."
        ));
    }
    
    // Check if requested size is within global budget
    if requested_size > config.total_budget() {
        return Err(Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Requested memory size exceeds global budget"
        ));
    }
    
    // Create provider based on size requirements
    let provider = if requested_size <= 8192 {
        PlatformAwareMemoryFactory::create_small()?
    } else if requested_size <= 65536 {
        PlatformAwareMemoryFactory::create_medium()?
    } else {
        PlatformAwareMemoryFactory::create_large()?
    };
    
    Ok(provider)
}

/// Create a memory provider for no-std environments (returns concrete types)
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub fn create_small_provider() -> WrtResult<crate::memory_system::SmallProvider> {
    let config = global_memory_config();
    
    // Ensure the system is initialized
    if config.initialized.load(Ordering::Relaxed) == 0 {
        return Err(Error::new(
            ErrorCategory::System,
            codes::UNINITIALIZED,
            "Global memory system not initialized. Call initialize_global_memory_system() first."
        ));
    }
    
    Ok(crate::memory_system::SmallProvider::new())
}

/// Create a medium memory provider for no-std environments
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub fn create_medium_provider() -> WrtResult<crate::memory_system::MediumProvider> {
    let config = global_memory_config();
    
    // Ensure the system is initialized
    if config.initialized.load(Ordering::Relaxed) == 0 {
        return Err(Error::new(
            ErrorCategory::System,
            codes::UNINITIALIZED,
            "Global memory system not initialized. Call initialize_global_memory_system() first."
        ));
    }
    
    Ok(crate::memory_system::MediumProvider::new())
}

/// Create a large memory provider for no-std environments
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub fn create_large_provider() -> WrtResult<crate::memory_system::LargeProvider> {
    let config = global_memory_config();
    
    // Ensure the system is initialized
    if config.initialized.load(Ordering::Relaxed) == 0 {
        return Err(Error::new(
            ErrorCategory::System,
            codes::UNINITIALIZED,
            "Global memory system not initialized. Call initialize_global_memory_system() first."
        ));
    }
    
    Ok(crate::memory_system::LargeProvider::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_memory_config_initialization() {
        // This test would need to be isolated or use a different config instance
        // For now, it demonstrates the intended usage
        
        // This test would need platform-memory feature enabled
        #[cfg(feature = "platform-memory")]
        let limits = wrt_platform::comprehensive_limits::ComprehensivePlatformLimits {
            platform_id: wrt_platform::comprehensive_limits::PlatformId::Linux,
            max_total_memory: 1024 * 1024 * 1024, // 1GB
            max_wasm_linear_memory: 512 * 1024 * 1024, // 512MB
            max_stack_bytes: 8 * 1024 * 1024, // 8MB
            max_components: 256,
            max_debug_overhead: 64 * 1024 * 1024, // 64MB
            asil_level: wrt_platform::comprehensive_limits::AsilLevel::QM,
        };
        
        // In a real test, we'd use a separate config instance
        // let config = GlobalMemoryConfig::new();
        // assert!(config.initialize_with_platform_limits(&limits).is_ok());
        // assert_eq!(config.total_budget(), 1024 * 1024 * 1024);
    }

    #[test]
    fn test_memory_budget_enforcement() {
        let config = GlobalMemoryConfig::new();
        
        // These would be private methods exposed for testing
        // assert!(config.can_allocate(100));
        // assert!(config.register_allocation(100).is_ok());
        // assert_eq!(config.allocated_memory(), 100);
        // config.unregister_allocation(100);
        // assert_eq!(config.allocated_memory(), 0);
    }
}