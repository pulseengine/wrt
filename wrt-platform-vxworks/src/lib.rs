//! External VxWorks Platform Implementation for WRT
//!
//! This crate demonstrates how external developers can implement platform-specific
//! support for WRT without modifying the core `wrt-platform` crate.
//!
//! ## Architecture
//!
//! This external implementation provides:
//! - Custom VxWorks memory allocators for both RTP and LKM contexts
//! - VxWorks-specific synchronization primitives
//! - Integration with the `wrt-platform` trait system
//! - Example usage patterns and best practices
//!
//! ## Usage Patterns
//!
//! ### RTP (Real-Time Process) Applications
//! ```rust,no_run
//! use wrt_platform_vxworks::{VxWorksRtpAllocator, VxWorksRtpFutex};
//! use wrt_platform::PageAllocator;
//!
//! // Create RTP-specific allocator
//! let allocator = VxWorksRtpAllocator::new()
//!     .with_heap_size(1024 * 1024) // 1MB heap
//!     .with_stack_size(64 * 1024)  // 64KB stack
//!     .build()?;
//!
//! // Allocate WASM pages
//! let pages = allocator.allocate_pages(10)?;
//! # Ok::<(), wrt_error::Error>(())
//! ```
//!
//! ### LKM (Loadable Kernel Module) Applications
//! ```rust,no_run
//! use wrt_platform_vxworks::{VxWorksLkmAllocator, VxWorksLkmFutex};
//! use wrt_platform::PageAllocator;
//!
//! // Create LKM-specific allocator with memory partitions
//! let allocator = VxWorksLkmAllocator::new()
//!     .with_memory_partition(2 * 1024 * 1024) // 2MB partition
//!     .with_priority_inheritance(true)
//!     .build()?;
//! # Ok::<(), wrt_error::Error>(())
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// Re-export core traits from wrt-platform
pub use wrt_platform::{PageAllocator, FutexLike, WASM_PAGE_SIZE};
pub use wrt_error::Error;

// VxWorks-specific modules
pub mod allocator;
pub mod sync;
pub mod threading;
pub mod integration;

// Platform abstraction integration
pub mod platform_integration;

// Re-export main types
pub use allocator::{VxWorksRtpAllocator, VxWorksLkmAllocator};
pub use sync::{VxWorksRtpFutex, VxWorksLkmFutex};
pub use threading::{VxWorksTask, VxWorksThread, TaskPriority};
pub use integration::{
    VxWorksRuntimeAdapter, VxWorksMemoryManager, VxWorksExecutionContext
};

/// VxWorks execution context selector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionContext {
    /// Real-Time Process (user space)
    Rtp,
    /// Loadable Kernel Module (kernel space)
    Lkm,
}

/// VxWorks-specific configuration
#[derive(Debug, Clone)]
pub struct VxWorksConfig {
    /// Execution context (RTP or LKM)
    pub context: ExecutionContext,
    /// Maximum pages to allocate
    pub max_pages: usize,
    /// Use dedicated memory partitions
    pub use_memory_partitions: bool,
    /// Enable priority inheritance
    pub priority_inheritance: bool,
    /// Task priority (0-255, lower is higher priority)
    pub task_priority: Option<u8>,
    /// Stack size for tasks/threads
    pub stack_size: usize,
}

impl Default for VxWorksConfig {
    fn default() -> Self {
        Self {
            context: ExecutionContext::Rtp,
            max_pages: 1024,
            use_memory_partitions: false,
            priority_inheritance: true,
            task_priority: None,
            stack_size: 64 * 1024, // 64KB
        }
    }
}

/// Builder for VxWorks configuration
pub struct VxWorksConfigBuilder {
    config: VxWorksConfig,
}

impl VxWorksConfigBuilder {
    /// Create new configuration builder
    pub fn new() -> Self {
        Self {
            config: VxWorksConfig::default(),
        }
    }

    /// Set execution context
    pub fn context(mut self, context: ExecutionContext) -> Self {
        self.config.context = context;
        self
    }

    /// Set maximum pages
    pub fn max_pages(mut self, max_pages: usize) -> Self {
        self.config.max_pages = max_pages;
        self
    }

    /// Enable memory partitions
    pub fn use_memory_partitions(mut self, enable: bool) -> Self {
        self.config.use_memory_partitions = enable;
        self
    }

    /// Enable priority inheritance
    pub fn priority_inheritance(mut self, enable: bool) -> Self {
        self.config.priority_inheritance = enable;
        self
    }

    /// Set task priority
    pub fn task_priority(mut self, priority: u8) -> Self {
        self.config.task_priority = Some(priority);
        self
    }

    /// Set stack size
    pub fn stack_size(mut self, size: usize) -> Self {
        self.config.stack_size = size;
        self
    }

    /// Build the configuration
    pub fn build(self) -> VxWorksConfig {
        self.config
    }
}

impl Default for VxWorksConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Main VxWorks platform adapter that integrates with wrt-platform
pub struct VxWorksPlatform {
    config: VxWorksConfig,
}

impl VxWorksPlatform {
    /// Create new VxWorks platform with configuration
    pub fn new(config: VxWorksConfig) -> Self {
        Self { config }
    }

    /// Create RTP-specific platform
    pub fn new_rtp() -> Result<Self, Error> {
        let config = VxWorksConfigBuilder::new()
            .context(ExecutionContext::Rtp)
            .use_memory_partitions(false) // RTP uses system heap
            .priority_inheritance(true)
            .build();
        Ok(Self::new(config))
    }

    /// Create LKM-specific platform  
    pub fn new_lkm() -> Result<Self, Error> {
        let config = VxWorksConfigBuilder::new()
            .context(ExecutionContext::Lkm)
            .use_memory_partitions(true) // LKM uses dedicated partitions
            .priority_inheritance(true)
            .build();
        Ok(Self::new(config))
    }

    /// Get the execution context
    pub fn context(&self) -> ExecutionContext {
        self.config.context
    }

    /// Get the configuration
    pub fn config(&self) -> &VxWorksConfig {
        &self.config
    }

    /// Create allocator for this platform
    pub fn create_allocator(&self) -> Result<Box<dyn PageAllocator>, Error> {
        match self.config.context {
            ExecutionContext::Rtp => {
                let allocator = VxWorksRtpAllocator::new()
                    .with_max_pages(self.config.max_pages)
                    .with_stack_size(self.config.stack_size)
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

    /// Create synchronization primitive for this platform
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

    /// Detect VxWorks capabilities at runtime
    pub fn detect_capabilities(&self) -> VxWorksCapabilities {
        VxWorksCapabilities::detect()
    }
}

/// VxWorks-specific capabilities
#[derive(Debug, Clone)]
pub struct VxWorksCapabilities {
    /// VxWorks kernel version
    pub kernel_version: String,
    /// Available memory in bytes
    pub available_memory: usize,
    /// Supports memory partitions
    pub memory_partitions: bool,
    /// Supports priority inheritance
    pub priority_inheritance: bool,
    /// Maximum task priority
    pub max_priority: u8,
    /// Minimum task priority  
    pub min_priority: u8,
    /// Tick rate (ticks per second)
    pub tick_rate: u32,
}

impl VxWorksCapabilities {
    /// Detect VxWorks capabilities at runtime
    pub fn detect() -> Self {
        // In a real implementation, this would query VxWorks system calls
        Self {
            kernel_version: "7.0.0".to_string(),
            available_memory: 64 * 1024 * 1024, // 64MB
            memory_partitions: true,
            priority_inheritance: true,
            max_priority: 255,
            min_priority: 0,
            tick_rate: 60, // 60 Hz
        }
    }

    /// Check if the system supports real-time requirements
    pub fn supports_realtime(&self) -> bool {
        self.tick_rate >= 50 && self.priority_inheritance
    }

    /// Check if the system supports memory isolation
    pub fn supports_isolation(&self) -> bool {
        self.memory_partitions
    }

    /// Get recommended configuration for this system
    pub fn recommended_config(&self) -> VxWorksConfig {
        VxWorksConfigBuilder::new()
            .max_pages((self.available_memory / WASM_PAGE_SIZE) / 4) // Use 25% of available memory
            .use_memory_partitions(self.memory_partitions)
            .priority_inheritance(self.priority_inheritance)
            .stack_size(64 * 1024) // 64KB stack
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = VxWorksConfigBuilder::new()
            .context(ExecutionContext::Lkm)
            .max_pages(512)
            .use_memory_partitions(true)
            .priority_inheritance(false)
            .task_priority(100)
            .stack_size(128 * 1024)
            .build();

        assert_eq!(config.context, ExecutionContext::Lkm);
        assert_eq!(config.max_pages, 512);
        assert!(config.use_memory_partitions);
        assert!(!config.priority_inheritance);
        assert_eq!(config.task_priority, Some(100));
        assert_eq!(config.stack_size, 128 * 1024);
    }

    #[test]
    fn test_platform_creation() {
        let rtp_platform = VxWorksPlatform::new_rtp().unwrap();
        assert_eq!(rtp_platform.context(), ExecutionContext::Rtp);
        assert!(!rtp_platform.config().use_memory_partitions);

        let lkm_platform = VxWorksPlatform::new_lkm().unwrap();
        assert_eq!(lkm_platform.context(), ExecutionContext::Lkm);
        assert!(lkm_platform.config().use_memory_partitions);
    }

    #[test]
    fn test_capabilities_detection() {
        let capabilities = VxWorksCapabilities::detect();
        assert!(!capabilities.kernel_version.is_empty());
        assert!(capabilities.available_memory > 0);
        assert!(capabilities.tick_rate > 0);
        assert!(capabilities.supports_realtime());
    }

    #[test]
    fn test_recommended_config() {
        let capabilities = VxWorksCapabilities::detect();
        let config = capabilities.recommended_config();
        assert!(config.max_pages > 0);
        assert_eq!(config.stack_size, 64 * 1024);
    }
}