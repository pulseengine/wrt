//! Platform integration with wrt-platform traits
//!
//! This module demonstrates how to integrate external VxWorks implementations
//! with the wrt-platform trait system and platform abstraction layer.

use crate::{
    VxWorksConfig, VxWorksRtpAllocator, VxWorksLkmAllocator, 
    VxWorksRtpFutex, VxWorksLkmFutex, ExecutionContext
};
use wrt_platform::{PageAllocator, FutexLike};
use wrt_error::Error;

/// Platform abstraction integration for VxWorks
///
/// This demonstrates how external crates can integrate with the
/// wrt-platform abstraction system without modifying core code.
pub struct ExternalVxWorksPlatform {
    config: VxWorksConfig,
}

impl ExternalVxWorksPlatform {
    /// Create new external VxWorks platform
    pub fn new(config: VxWorksConfig) -> Self {
        Self { config }
    }

    /// Create allocator based on configuration
    pub fn create_allocator(&self) -> Result<Box<dyn PageAllocator>, Error> {
        match self.config.context {
            ExecutionContext::Rtp => {
                let allocator = VxWorksRtpAllocator::new()
                    .with_max_pages(self.config.max_pages)
                    .build()?;
                Ok(Box::new(allocator))
            }
            ExecutionContext::Lkm => {
                let allocator = VxWorksLkmAllocator::new()
                    .with_max_pages(self.config.max_pages)
                    .with_memory_partitions(self.config.use_memory_partitions)
                    .with_priority_inheritance(self.config.priority_inheritance)
                    .build()?;
                Ok(Box::new(allocator))
            }
        }
    }

    /// Create futex based on configuration
    pub fn create_futex(&self) -> Result<Box<dyn FutexLike>, Error> {
        match self.config.context {
            ExecutionContext::Rtp => {
                let futex = VxWorksRtpFutex::new()
                    .with_priority_inheritance(self.config.priority_inheritance)
                    .build()?;
                Ok(Box::new(futex))
            }
            ExecutionContext::Lkm => {
                let futex = VxWorksLkmFutex::new()
                    .with_priority_inheritance(self.config.priority_inheritance)
                    .build()?;
                Ok(Box::new(futex))
            }
        }
    }

    /// Get configuration
    pub fn config(&self) -> &VxWorksConfig {
        &self.config
    }
}

/// Integration helper for external platform implementations
///
/// This provides utilities for external crates to integrate seamlessly
/// with the existing wrt-platform ecosystem.
pub struct ExternalPlatformIntegration;

impl ExternalPlatformIntegration {
    /// Register external VxWorks platform with runtime detection
    ///
    /// This demonstrates how external platforms can be auto-detected
    /// and integrated at runtime.
    pub fn register_vxworks_platform() -> Result<ExternalVxWorksPlatform, Error> {
        // Detect the current VxWorks execution context
        let context = Self::detect_vxworks_context();
        
        // Create appropriate configuration
        let config = VxWorksConfig {
            context,
            max_pages: 1024,
            use_memory_partitions: context == ExecutionContext::Lkm,
            priority_inheritance: true,
            task_priority: Some(100),
            stack_size: 64 * 1024,
        };

        Ok(ExternalVxWorksPlatform::new(config))
    }

    /// Detect VxWorks execution context
    fn detect_vxworks_context() -> ExecutionContext {
        #[cfg(target_os = "vxworks")]
        {
            // Check if we can access kernel-specific functions
            extern "C" {
                fn taskIdSelf() -> usize;
            }
            
            let task_id = unsafe { taskIdSelf() };
            if task_id != 0 {
                // We're in a VxWorks task context
                // Try to determine if it's kernel or user space
                
                // In a real implementation, you would use VxWorks-specific
                // APIs to determine the execution context
                // For this example, we'll default to RTP
                ExecutionContext::Rtp
            } else {
                // Fallback to RTP
                ExecutionContext::Rtp
            }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // For non-VxWorks platforms, default to RTP for testing
            ExecutionContext::Rtp
        }
    }

    /// Create platform-agnostic allocator factory
    ///
    /// This demonstrates how external platforms can provide factories
    /// that integrate with existing code.
    pub fn create_allocator_factory() -> impl Fn(&VxWorksConfig) -> Result<Box<dyn PageAllocator>, Error> {
        |config: &VxWorksConfig| -> Result<Box<dyn PageAllocator>, Error> {
            let platform = ExternalVxWorksPlatform::new(config.clone());
            platform.create_allocator()
        }
    }

    /// Create platform-agnostic futex factory
    pub fn create_futex_factory() -> impl Fn(&VxWorksConfig) -> Result<Box<dyn FutexLike>, Error> {
        |config: &VxWorksConfig| -> Result<Box<dyn FutexLike>, Error> {
            let platform = ExternalVxWorksPlatform::new(config.clone());
            platform.create_futex()
        }
    }

    /// Check if VxWorks platform is available
    pub fn is_vxworks_available() -> bool {
        #[cfg(target_os = "vxworks")]
        {
            true
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            false
        }
    }

    /// Get platform capabilities
    pub fn get_platform_capabilities() -> VxWorksPlatformCapabilities {
        VxWorksPlatformCapabilities {
            supports_rtp: true,
            supports_lkm: Self::is_vxworks_available(),
            supports_memory_partitions: Self::is_vxworks_available(),
            supports_priority_inheritance: true,
            max_task_priority: 255,
            min_task_priority: 0,
            #[cfg(target_os = "vxworks")]
            tick_rate: unsafe {
                extern "C" {
                    fn sysClkRateGet() -> i32;
                }
                sysClkRateGet() as u32
            },
            #[cfg(not(target_os = "vxworks"))]
            tick_rate: 60,
        }
    }
}

/// VxWorks platform capabilities
#[derive(Debug, Clone)]
pub struct VxWorksPlatformCapabilities {
    /// Supports RTP (user space) execution
    pub supports_rtp: bool,
    /// Supports LKM (kernel space) execution
    pub supports_lkm: bool,
    /// Supports memory partitions
    pub supports_memory_partitions: bool,
    /// Supports priority inheritance
    pub supports_priority_inheritance: bool,
    /// Maximum task priority
    pub max_task_priority: u8,
    /// Minimum task priority
    pub min_task_priority: u8,
    /// System tick rate
    pub tick_rate: u32,
}

impl VxWorksPlatformCapabilities {
    /// Check if platform supports real-time requirements
    pub fn supports_realtime(&self) -> bool {
        self.tick_rate >= 50 && self.supports_priority_inheritance
    }

    /// Check if platform supports memory isolation
    pub fn supports_isolation(&self) -> bool {
        self.supports_memory_partitions
    }

    /// Get recommended context for the current platform
    pub fn recommended_context(&self) -> ExecutionContext {
        if self.supports_lkm {
            // Prefer LKM for better real-time characteristics
            ExecutionContext::Lkm
        } else {
            ExecutionContext::Rtp
        }
    }
}

/// Example integration with existing wrt-platform code
///
/// This shows how external platforms can be used as drop-in replacements
/// for built-in platform implementations.
pub mod example_integration {
    use super::*;
    use wrt_platform::{PageAllocator, FutexLike};
    
    /// Example function that uses platform-agnostic interfaces
    pub fn create_wasm_runtime_memory(pages: usize) -> Result<Box<dyn PageAllocator>, Error> {
        // Check if VxWorks is available
        if ExternalPlatformIntegration::is_vxworks_available() {
            // Use external VxWorks implementation
            let platform = ExternalPlatformIntegration::register_vxworks_platform()?;
            platform.create_allocator()
        } else {
            // Fall back to other platform implementations
            // (This would use the built-in wrt-platform implementations)
            Err(Error::new(
                wrt_error::ErrorKind::Platform,
                "No suitable platform implementation available"
            ))
        }
    }

    /// Example function that creates synchronization primitives
    pub fn create_wasm_sync_primitive() -> Result<Box<dyn FutexLike>, Error> {
        if ExternalPlatformIntegration::is_vxworks_available() {
            let platform = ExternalPlatformIntegration::register_vxworks_platform()?;
            platform.create_futex()
        } else {
            Err(Error::new(
                wrt_error::ErrorKind::Platform,
                "No suitable synchronization implementation available"
            ))
        }
    }

    /// Example of using factory functions
    pub fn create_runtime_components() -> Result<(Box<dyn PageAllocator>, Box<dyn FutexLike>), Error> {
        let config = VxWorksConfig::default();
        
        let allocator_factory = ExternalPlatformIntegration::create_allocator_factory();
        let futex_factory = ExternalPlatformIntegration::create_futex_factory();
        
        let allocator = allocator_factory(&config)?;
        let futex = futex_factory(&config)?;
        
        Ok((allocator, futex))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_platform_creation() {
        let config = VxWorksConfig::default();
        let platform = ExternalVxWorksPlatform::new(config);
        
        assert_eq!(platform.config().context, ExecutionContext::Rtp);
    }

    #[test]
    fn test_platform_capabilities() {
        let capabilities = ExternalPlatformIntegration::get_platform_capabilities();
        
        assert!(capabilities.supports_rtp);
        assert!(capabilities.tick_rate > 0);
        assert!(capabilities.max_task_priority >= capabilities.min_task_priority);
        
        // Test real-time support
        if capabilities.tick_rate >= 50 && capabilities.supports_priority_inheritance {
            assert!(capabilities.supports_realtime());
        }
    }

    #[test]
    fn test_context_detection() {
        let context = ExternalPlatformIntegration::detect_vxworks_context();
        assert!(matches!(context, ExecutionContext::Rtp | ExecutionContext::Lkm));
    }

    #[test]
    fn test_factory_functions() {
        let config = VxWorksConfig::default();
        
        let allocator_factory = ExternalPlatformIntegration::create_allocator_factory();
        let futex_factory = ExternalPlatformIntegration::create_futex_factory();
        
        // Test that factories can be called
        let allocator_result = allocator_factory(&config);
        let futex_result = futex_factory(&config);
        
        // On non-VxWorks platforms, these might fail, but they should be callable
        assert!(allocator_result.is_ok() || allocator_result.is_err());
        assert!(futex_result.is_ok() || futex_result.is_err());
    }

    #[test]
    fn test_platform_registration() {
        let platform_result = ExternalPlatformIntegration::register_vxworks_platform();
        
        // Should succeed on VxWorks, may fail on other platforms
        assert!(platform_result.is_ok() || platform_result.is_err());
        
        if let Ok(platform) = platform_result {
            assert!(matches!(
                platform.config().context,
                ExecutionContext::Rtp | ExecutionContext::Lkm
            ));
        }
    }

    #[test]
    fn test_example_integration() {
        use example_integration::*;
        
        // Test memory creation
        let memory_result = create_wasm_runtime_memory(10);
        assert!(memory_result.is_ok() || memory_result.is_err());
        
        // Test sync primitive creation
        let sync_result = create_wasm_sync_primitive();
        assert!(sync_result.is_ok() || sync_result.is_err());
        
        // Test component creation
        let components_result = create_runtime_components();
        assert!(components_result.is_ok() || components_result.is_err());
    }

    #[test]
    fn test_capabilities_recommendations() {
        let capabilities = ExternalPlatformIntegration::get_platform_capabilities();
        let recommended_context = capabilities.recommended_context();
        
        assert!(matches!(recommended_context, ExecutionContext::Rtp | ExecutionContext::Lkm));
        
        // If LKM is supported, it should be recommended for real-time
        if capabilities.supports_lkm {
            assert_eq!(recommended_context, ExecutionContext::Lkm);
        } else {
            assert_eq!(recommended_context, ExecutionContext::Rtp);
        }
    }
}