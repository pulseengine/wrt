// WRT - wrt-foundation
// Module: Platform Abstraction Interface (PAI)
// SW-REQ-ID: REQ_PLATFORM_ABSTRACTION_001
//
// Copyright (c) 2025 The WRT Project Developers
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Platform Abstraction Interface (PAI) for ASIL-compliant runtime
//! 
//! This module provides a clean abstraction layer between the runtime
//! and platform-specific implementations. It ensures deterministic
//! compilation and proper feature gating for safety-critical systems.
//!
//! # Design Principles
//! - Zero-cost abstractions through compile-time dispatch
//! - Clear separation between platform-dependent and independent code
//! - Deterministic feature combinations for ASIL compliance
//! - No implicit dependencies on platform features
//! - Integration with universal safety contexts
//! - Proper no_std compatibility

#![forbid(unsafe_code)] // Platform abstractions must be safe

use core::marker::PhantomData;
use crate::{Error, ErrorCategory, WrtResult, codes};
use crate::safety_system::{AsilLevel, SafetyStandard, UniversalSafetyContext, SafetyStandardType};

// Box for trait objects
#[cfg(feature = "std")]
use std::boxed::Box;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

// Platform-agnostic atomic types
#[cfg(not(feature = "platform-sync"))]
mod atomic_fallback {
    use core::cell::UnsafeCell;
    use core::sync::atomic::{AtomicBool, Ordering};
    
    /// Fallback atomic u32 for platforms without native atomics
    pub struct AtomicU32 {
        lock: AtomicBool,
        value: UnsafeCell<u32>,
    }
    
    // SAFETY: AtomicU32 uses proper synchronization through AtomicBool
    // The UnsafeCell is protected by the lock and accessed atomically
    #[allow(unsafe_code)]
    unsafe impl Sync for AtomicU32 {}
    #[allow(unsafe_code)]
    unsafe impl Send for AtomicU32 {}
    
    impl AtomicU32 {
        pub const fn new(value: u32) -> Self {
            Self {
                lock: AtomicBool::new(false),
                value: UnsafeCell::new(value),
            }
        }
        
        pub fn load(&self, _ordering: Ordering) -> u32 {
            // Spin lock for safety
            while self.lock.compare_exchange_weak(
                false, true, Ordering::Acquire, Ordering::Relaxed
            ).is_err() {
                core::hint::spin_loop();
            }
            let value = unsafe { *self.value.get() };
            self.lock.store(false, Ordering::Release);
            value
        }
        
        pub fn store(&self, value: u32, _ordering: Ordering) {
            while self.lock.compare_exchange_weak(
                false, true, Ordering::Acquire, Ordering::Relaxed
            ).is_err() {
                core::hint::spin_loop();
            }
            unsafe { *self.value.get() = value; }
            self.lock.store(false, Ordering::Release);
        }
        
        pub fn compare_exchange(
            &self,
            current: u32,
            new: u32,
            _success: Ordering,
            _failure: Ordering,
        ) -> core::result::Result<u32, u32> {
            while self.lock.compare_exchange_weak(
                false, true, Ordering::Acquire, Ordering::Relaxed
            ).is_err() {
                core::hint::spin_loop();
            }
            let old_value = unsafe { *self.value.get() };
            if old_value == current {
                unsafe { *self.value.get() = new; }
                self.lock.store(false, Ordering::Release);
                Ok(old_value)
            } else {
                self.lock.store(false, Ordering::Release);
                Err(old_value)
            }
        }
        
        pub fn fetch_add(&self, value: u32, _ordering: Ordering) -> u32 {
            while self.lock.compare_exchange_weak(
                false, true, Ordering::Acquire, Ordering::Relaxed
            ).is_err() {
                core::hint::spin_loop();
            }
            let old_value = unsafe { *self.value.get() };
            unsafe { *self.value.get() = old_value.wrapping_add(value); }
            self.lock.store(false, Ordering::Release);
            old_value
        }
    }
    
    /// Fallback atomic u64 for platforms without native atomics
    pub struct AtomicU64 {
        lock: AtomicBool,
        value: UnsafeCell<u64>,
    }
    
    // SAFETY: AtomicU64 uses proper synchronization through AtomicBool
    // The UnsafeCell is protected by the lock and accessed atomically
    #[allow(unsafe_code)]
    unsafe impl Sync for AtomicU64 {}
    #[allow(unsafe_code)]
    unsafe impl Send for AtomicU64 {}
    
    impl AtomicU64 {
        pub const fn new(value: u64) -> Self {
            Self {
                lock: AtomicBool::new(false),
                value: UnsafeCell::new(value),
            }
        }
        
        pub fn load(&self, _ordering: Ordering) -> u64 {
            while self.lock.compare_exchange_weak(
                false, true, Ordering::Acquire, Ordering::Relaxed
            ).is_err() {
                core::hint::spin_loop();
            }
            let value = unsafe { *self.value.get() };
            self.lock.store(false, Ordering::Release);
            value
        }
        
        pub fn store(&self, value: u64, _ordering: Ordering) {
            while self.lock.compare_exchange_weak(
                false, true, Ordering::Acquire, Ordering::Relaxed
            ).is_err() {
                core::hint::spin_loop();
            }
            unsafe { *self.value.get() = value; }
            self.lock.store(false, Ordering::Release);
        }
    }
    
    /// Fallback atomic usize
    pub type AtomicUsize = core::sync::atomic::AtomicUsize;
}

// Platform abstraction layer
#[cfg(feature = "platform-sync")]
pub use wrt_platform::sync::{AtomicU32, AtomicU64, AtomicUsize, Ordering as PlatformOrdering};

#[cfg(not(feature = "platform-sync"))]
pub use self::atomic_fallback::{AtomicU32, AtomicU64, AtomicUsize};
#[cfg(not(feature = "platform-sync"))]
pub use core::sync::atomic::Ordering as PlatformOrdering;

// Duration abstraction
#[cfg(feature = "std")]
pub use std::time::Duration;

#[cfg(all(not(feature = "std"), feature = "platform-sync"))]
pub use wrt_platform::sync::Duration;

#[cfg(all(not(feature = "std"), not(feature = "platform-sync")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    secs: u64,
    nanos: u32,
}

#[cfg(all(not(feature = "std"), not(feature = "platform-sync")))]
impl Duration {
    pub const fn from_millis(millis: u64) -> Self {
        Self {
            secs: millis / 1000,
            nanos: ((millis % 1000) * 1_000_000) as u32,
        }
    }
    
    pub const fn as_millis(&self) -> u128 {
        self.secs as u128 * 1000 + self.nanos as u128 / 1_000_000
    }
}

/// Platform identification enum with ASIL mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformId {
    /// Linux - General purpose OS (QM)
    Linux,
    /// QNX - Safety-critical RTOS (ASIL-B)
    QNX,
    /// Embedded - Bare metal or RTOS (ASIL-D)
    Embedded,
    /// macOS - Development platform (QM)
    MacOS,
    /// Windows - Development platform (QM)
    Windows,
    /// Unknown platform (QM with restrictions)
    Unknown,
}

impl PlatformId {
    /// Get the default ASIL level for this platform
    pub const fn default_asil_level(&self) -> AsilLevel {
        match self {
            PlatformId::Linux => AsilLevel::QM,
            PlatformId::QNX => AsilLevel::AsilB,
            PlatformId::Embedded => AsilLevel::AsilD,
            PlatformId::MacOS => AsilLevel::QM,
            PlatformId::Windows => AsilLevel::QM,
            PlatformId::Unknown => AsilLevel::QM,
        }
    }
    
    /// Get the recommended safety standard for this platform
    pub const fn recommended_safety_standard(&self) -> SafetyStandard {
        match self {
            PlatformId::Linux | PlatformId::MacOS | PlatformId::Windows => {
                SafetyStandard::Iso26262(AsilLevel::QM)
            }
            PlatformId::QNX => SafetyStandard::Iso26262(AsilLevel::AsilB),
            PlatformId::Embedded => SafetyStandard::Iso26262(AsilLevel::AsilD),
            PlatformId::Unknown => SafetyStandard::Iso26262(AsilLevel::QM),
        }
    }
    
    /// Check if platform supports dynamic allocation
    pub const fn supports_dynamic_allocation(&self) -> bool {
        match self {
            PlatformId::Linux | PlatformId::MacOS | PlatformId::Windows => true,
            PlatformId::QNX => true, // Limited support
            PlatformId::Embedded | PlatformId::Unknown => false,
        }
    }
    
    /// Get platform-specific memory alignment requirements
    pub const fn memory_alignment(&self) -> usize {
        match self {
            PlatformId::Linux | PlatformId::MacOS | PlatformId::Windows => 8,
            PlatformId::QNX => 16, // More strict for real-time
            PlatformId::Embedded => 4, // Conservative for embedded
            PlatformId::Unknown => 8,
        }
    }
}

impl Default for PlatformId {
    fn default() -> Self {
        #[cfg(target_os = "linux")]
        return PlatformId::Linux;
        #[cfg(target_os = "macos")]
        return PlatformId::MacOS;
        #[cfg(target_os = "windows")]
        return PlatformId::Windows;
        #[cfg(target_os = "nto")]
        return PlatformId::QNX;
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows", target_os = "nto")))]
        return PlatformId::Embedded;
    }
}

/// Comprehensive platform limits with safety context integration
#[derive(Debug, Clone)]
pub struct ComprehensivePlatformLimits {
    pub platform_id: PlatformId,
    pub max_total_memory: usize,
    pub max_wasm_linear_memory: usize,
    pub max_stack_bytes: usize,
    pub max_components: usize,
    pub max_debug_overhead: usize,
    pub asil_level: AsilLevel,
    pub safety_context: UniversalSafetyContext,
    pub memory_alignment: usize,
    pub supports_virtual_memory: bool,
    pub supports_memory_protection: bool,
}

impl Default for ComprehensivePlatformLimits {
    fn default() -> Self {
        let platform_id = PlatformId::default();
        let safety_standard = platform_id.recommended_safety_standard();
        
        Self {
            platform_id,
            max_total_memory: 1024 * 1024 * 1024, // 1GB
            max_wasm_linear_memory: 256 * 1024 * 1024, // 256MB
            max_stack_bytes: 8 * 1024 * 1024, // 8MB
            max_components: 256,
            max_debug_overhead: 64 * 1024 * 1024, // 64MB
            asil_level: platform_id.default_asil_level(),
            safety_context: UniversalSafetyContext::new(safety_standard),
            memory_alignment: platform_id.memory_alignment(),
            supports_virtual_memory: platform_id.supports_dynamic_allocation(),
            supports_memory_protection: platform_id.supports_dynamic_allocation(),
        }
    }
}

impl ComprehensivePlatformLimits {
    /// Create new limits with specified memory
    pub fn with_memory(max_memory: usize) -> Self {
        Self {
            max_total_memory: max_memory,
            max_wasm_linear_memory: max_memory / 4,
            max_stack_bytes: max_memory / 128,
            ..Default::default()
        }
    }
    
    /// Create minimal limits for embedded systems
    pub fn minimal() -> Self {
        let platform_id = PlatformId::Embedded;
        let safety_standard = SafetyStandard::Iso26262(AsilLevel::AsilD);
        
        Self {
            platform_id,
            max_total_memory: 256 * 1024, // 256KB
            max_wasm_linear_memory: 64 * 1024, // 64KB
            max_stack_bytes: 16 * 1024, // 16KB
            max_components: 8,
            max_debug_overhead: 0,
            asil_level: AsilLevel::AsilD,
            safety_context: UniversalSafetyContext::new(safety_standard),
            memory_alignment: 4,
            supports_virtual_memory: false,
            supports_memory_protection: false,
        }
    }
    
    /// Create limits for safety-critical systems with specific ASIL level
    pub fn safety_critical(asil_level: AsilLevel) -> Self {
        let safety_standard = SafetyStandard::Iso26262(asil_level);
        let memory_multiplier = match asil_level {
            AsilLevel::QM => 1024,
            AsilLevel::AsilA => 512,
            AsilLevel::AsilB => 256,
            AsilLevel::AsilC => 128,
            AsilLevel::AsilD => 64,
        };
        
        Self {
            platform_id: PlatformId::Embedded,
            max_total_memory: memory_multiplier * 1024, // Smaller for higher ASIL
            max_wasm_linear_memory: (memory_multiplier * 1024) / 4,
            max_stack_bytes: (memory_multiplier * 1024) / 16,
            max_components: match asil_level {
                AsilLevel::QM => 256,
                AsilLevel::AsilA => 64,
                AsilLevel::AsilB => 32,
                AsilLevel::AsilC => 16,
                AsilLevel::AsilD => 8,
            },
            max_debug_overhead: 0, // No debug overhead in safety-critical mode
            asil_level,
            safety_context: UniversalSafetyContext::new(safety_standard),
            memory_alignment: 8, // Stricter alignment for safety
            supports_virtual_memory: false,
            supports_memory_protection: false,
        }
    }
    
    /// Validate that the limits are appropriate for the safety context
    pub fn validate_safety_compliance(&self) -> WrtResult<()> {
        // Check if memory limits are appropriate for the safety level
        match self.asil_level {
            AsilLevel::AsilD => {
                if self.max_total_memory > 128 * 1024 {
                    return Err(Error::new(
                        ErrorCategory::SafetyViolation,
                        codes::SAFETY_VIOLATION,
                        "ASIL-D systems should use minimal memory limits"
                    ));
                }
            }
            AsilLevel::AsilC => {
                if self.max_total_memory > 256 * 1024 {
                    return Err(Error::new(
                        ErrorCategory::SafetyViolation,
                        codes::SAFETY_VIOLATION,
                        "ASIL-C systems should use constrained memory limits"
                    ));
                }
            }
            _ => {} // More lenient for lower ASIL levels
        }
        
        // Validate debug overhead restrictions
        if self.asil_level >= AsilLevel::AsilB && self.max_debug_overhead > 0 {
            return Err(Error::new(
                ErrorCategory::SafetyViolation,
                codes::SAFETY_VIOLATION,
                "ASIL-B+ systems should not allow debug overhead"
            ));
        }
        
        Ok(())
    }
}

/// Platform limit provider trait with safety context integration
pub trait ComprehensiveLimitProvider: Send + Sync {
    fn discover_limits(&self) -> WrtResult<ComprehensivePlatformLimits>;
    fn platform_id(&self) -> PlatformId;
    fn safety_context(&self) -> &UniversalSafetyContext;
    fn supports_safety_standard(&self, standard_type: SafetyStandardType) -> bool;
}

/// Default platform limit provider with safety context
#[derive(Default)]
pub struct DefaultLimitProvider {
    safety_context: UniversalSafetyContext,
}

impl DefaultLimitProvider {
    pub fn new() -> Self {
        let platform_id = PlatformId::default();
        let safety_standard = platform_id.recommended_safety_standard();
        
        Self {
            safety_context: UniversalSafetyContext::new(safety_standard),
        }
    }
    
    pub fn with_safety_context(safety_context: UniversalSafetyContext) -> Self {
        Self { safety_context }
    }
}

impl ComprehensiveLimitProvider for DefaultLimitProvider {
    fn discover_limits(&self) -> WrtResult<ComprehensivePlatformLimits> {
        let mut limits = ComprehensivePlatformLimits::default();
        limits.safety_context = self.safety_context.clone();
        limits.validate_safety_compliance()?;
        Ok(limits)
    }
    
    fn platform_id(&self) -> PlatformId {
        PlatformId::default()
    }
    
    fn safety_context(&self) -> &UniversalSafetyContext {
        &self.safety_context
    }
    
    fn supports_safety_standard(&self, _standard_type: SafetyStandardType) -> bool {
        // Default provider supports basic automotive standard only
        matches!(_standard_type, SafetyStandardType::Iso26262)
    }
}

/// Linux limit provider with safety context
pub struct LinuxLimitProvider {
    safety_context: UniversalSafetyContext,
}

impl LinuxLimitProvider {
    pub fn new() -> Self {
        Self {
            safety_context: UniversalSafetyContext::new(SafetyStandard::Iso26262(AsilLevel::QM)),
        }
    }
}

impl Default for LinuxLimitProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ComprehensiveLimitProvider for LinuxLimitProvider {
    fn discover_limits(&self) -> WrtResult<ComprehensivePlatformLimits> {
        let mut limits = ComprehensivePlatformLimits {
            platform_id: PlatformId::Linux,
            max_total_memory: 8 * 1024 * 1024 * 1024, // 8GB
            max_wasm_linear_memory: 2 * 1024 * 1024 * 1024, // 2GB
            max_stack_bytes: 32 * 1024 * 1024, // 32MB
            max_components: 1024,
            max_debug_overhead: 512 * 1024 * 1024, // 512MB
            asil_level: AsilLevel::QM,
            safety_context: self.safety_context.clone(),
            memory_alignment: 8,
            supports_virtual_memory: true,
            supports_memory_protection: true,
        };
        limits.validate_safety_compliance()?;
        Ok(limits)
    }
    
    fn platform_id(&self) -> PlatformId {
        PlatformId::Linux
    }
    
    fn safety_context(&self) -> &UniversalSafetyContext {
        &self.safety_context
    }
    
    fn supports_safety_standard(&self, standard_type: SafetyStandardType) -> bool {
        // Linux can support multiple standards for development/testing
        matches!(standard_type, SafetyStandardType::Iso26262 | SafetyStandardType::Iec61508)
    }
}

/// QNX limit provider with safety context
pub struct QnxLimitProvider {
    safety_context: UniversalSafetyContext,
}

impl QnxLimitProvider {
    pub fn new() -> Self {
        Self {
            safety_context: UniversalSafetyContext::new(SafetyStandard::Iso26262(AsilLevel::AsilB)),
        }
    }
}

impl Default for QnxLimitProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ComprehensiveLimitProvider for QnxLimitProvider {
    fn discover_limits(&self) -> WrtResult<ComprehensivePlatformLimits> {
        let mut limits = ComprehensivePlatformLimits {
            platform_id: PlatformId::QNX,
            max_total_memory: 512 * 1024 * 1024, // 512MB
            max_wasm_linear_memory: 128 * 1024 * 1024, // 128MB
            max_stack_bytes: 4 * 1024 * 1024, // 4MB
            max_components: 64,
            max_debug_overhead: 16 * 1024 * 1024, // 16MB
            asil_level: AsilLevel::AsilB,
            safety_context: self.safety_context.clone(),
            memory_alignment: 16,
            supports_virtual_memory: true,
            supports_memory_protection: true,
        };
        limits.validate_safety_compliance()?;
        Ok(limits)
    }
    
    fn platform_id(&self) -> PlatformId {
        PlatformId::QNX
    }
    
    fn safety_context(&self) -> &UniversalSafetyContext {
        &self.safety_context
    }
    
    fn supports_safety_standard(&self, standard_type: SafetyStandardType) -> bool {
        // QNX supports automotive and industrial standards
        matches!(standard_type, SafetyStandardType::Iso26262 | SafetyStandardType::Iec61508)
    }
}

/// Platform abstraction interface for runtime with enhanced safety integration
pub trait PlatformInterface: Send + Sync {
    /// Get platform limits
    fn limits(&self) -> &ComprehensivePlatformLimits;
    
    /// Check if a feature is supported
    fn supports_feature(&self, feature: PlatformFeature) -> bool;
    
    /// Get platform-specific configuration
    fn get_config<T: PlatformConfig>(&self) -> Option<&T>;
    
    /// Get the safety context for this platform
    fn safety_context(&self) -> &UniversalSafetyContext;
    
    /// Validate that an operation is safe for the current platform
    fn validate_operation(&self, operation: PlatformOperation) -> WrtResult<()>;
    
    /// Get platform-specific time information
    fn current_time_ms(&self) -> u64;
}

/// Platform features that may or may not be available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformFeature {
    /// Native atomic operations
    NativeAtomics,
    /// Memory-mapped files
    MemoryMappedFiles,
    /// Guard pages for memory protection
    GuardPages,
    /// Hardware memory tagging
    MemoryTagging,
    /// Real-time scheduling
    RealtimeScheduling,
    /// Formal verification support
    FormalVerification,
    /// Dynamic memory allocation
    DynamicAllocation,
    /// Virtual memory support
    VirtualMemory,
    /// Hardware-assisted memory protection
    MemoryProtection,
    /// High-resolution timers
    HighResolutionTimers,
}

/// Platform operations that need safety validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformOperation {
    /// Memory allocation operation
    MemoryAllocation { size: usize },
    /// Component instantiation
    ComponentInstantiation,
    /// Debug operation
    DebugOperation,
    /// Real-time operation
    RealtimeOperation,
    /// File system access
    FileSystemAccess,
    /// Network operation
    NetworkOperation,
}

/// Marker trait for platform-specific configurations
pub trait PlatformConfig: Send + Sync + 'static {}

/// Create platform abstraction based on compile-time features
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn create_platform_interface() -> WrtResult<Box<dyn PlatformInterface>> {
    #[cfg(feature = "platform-sync")]
    {
        // Use full platform abstraction from wrt-platform
        create_full_platform_interface()
    }
    
    #[cfg(not(feature = "platform-sync"))]
    {
        // Use minimal fallback implementation
        Ok(Box::new(MinimalPlatform::new()))
    }
}

/// Create platform interface for pure no_std environments
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub fn create_platform_interface() -> WrtResult<&'static dyn PlatformInterface> {
    // In pure no_std, return a static reference
    static MINIMAL_PLATFORM: MinimalPlatform = MinimalPlatform {
        limits: ComprehensivePlatformLimits {
            platform_id: PlatformId::Embedded,
            max_total_memory: 256 * 1024,
            max_wasm_linear_memory: 64 * 1024,
            max_stack_bytes: 16 * 1024,
            max_components: 8,
            max_debug_overhead: 0,
            asil_level: AsilLevel::AsilD,
            safety_context: UniversalSafetyContext::new(SafetyStandard::Iso26262(AsilLevel::AsilD)),
            memory_alignment: 4,
            supports_virtual_memory: false,
            supports_memory_protection: false,
        },
    };
    Ok(&MINIMAL_PLATFORM)
}

/// Minimal platform implementation for no_std without platform features
struct MinimalPlatform {
    limits: ComprehensivePlatformLimits,
}

impl MinimalPlatform {
    fn new() -> Self {
        Self {
            limits: ComprehensivePlatformLimits::minimal(),
        }
    }
}

impl PlatformInterface for MinimalPlatform {
    fn limits(&self) -> &ComprehensivePlatformLimits {
        &self.limits
    }
    
    fn supports_feature(&self, feature: PlatformFeature) -> bool {
        match feature {
            PlatformFeature::NativeAtomics => false, // Using fallback
            PlatformFeature::FormalVerification => true, // Always support verification
            PlatformFeature::DynamicAllocation => false, // No dynamic allocation in minimal
            PlatformFeature::VirtualMemory => false,
            PlatformFeature::MemoryProtection => false,
            PlatformFeature::HighResolutionTimers => false,
            _ => false,
        }
    }
    
    fn get_config<T: PlatformConfig>(&self) -> Option<&T> {
        None // No platform-specific config in minimal mode
    }
    
    fn safety_context(&self) -> &UniversalSafetyContext {
        &self.limits.safety_context
    }
    
    fn validate_operation(&self, operation: PlatformOperation) -> WrtResult<()> {
        match operation {
            PlatformOperation::MemoryAllocation { size } => {
                if size > self.limits.max_total_memory {
                    return Err(Error::new(
                        ErrorCategory::ResourceExhausted,
                        codes::RESOURCE_EXHAUSTED,
                        "Memory allocation exceeds platform limits"
                    ));
                }
            }
            PlatformOperation::DebugOperation => {
                if self.limits.asil_level >= AsilLevel::AsilB {
                    return Err(Error::new(
                        ErrorCategory::SafetyViolation,
                        codes::SAFETY_VIOLATION,
                        "Debug operations not allowed in safety-critical mode"
                    ));
                }
            }
            PlatformOperation::FileSystemAccess | PlatformOperation::NetworkOperation => {
                return Err(Error::new(
                    ErrorCategory::NotSupported,
                    codes::NOT_IMPLEMENTED,
                    "File system and network operations not supported in minimal platform"
                ));
            }
            _ => {} // Other operations are allowed
        }
        Ok(())
    }
    
    fn current_time_ms(&self) -> u64 {
        // Simple counter for minimal platform
        // In a real implementation, this would use a platform-specific timer
        0
    }
}

#[cfg(feature = "platform-sync")]
fn create_full_platform_interface() -> WrtResult<Box<dyn PlatformInterface>> {
    // This would integrate with wrt-platform's full abstraction
    // For now, return error to indicate implementation needed
    Err(Error::new(
        ErrorCategory::NotImplemented,
        codes::NOT_IMPLEMENTED,
        "Full platform interface integration pending"
    ))
}

/// Create platform-appropriate limit provider (legacy compatibility)
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn create_limit_provider() -> Box<dyn ComprehensiveLimitProvider> {
    match PlatformId::default() {
        PlatformId::Linux => Box::new(LinuxLimitProvider::new()),
        PlatformId::QNX => Box::new(QnxLimitProvider::new()),
        _ => Box::new(DefaultLimitProvider::new()),
    }
}

/// Create platform limit provider with specific safety context
#[cfg(any(feature = "std", feature = "alloc"))]
pub fn create_limit_provider_with_safety(safety_context: UniversalSafetyContext) -> Box<dyn ComprehensiveLimitProvider> {
    match PlatformId::default() {
        PlatformId::Linux => {
            let mut provider = LinuxLimitProvider::new();
            provider.safety_context = safety_context;
            Box::new(provider)
        }
        PlatformId::QNX => {
            let mut provider = QnxLimitProvider::new();
            provider.safety_context = safety_context;
            Box::new(provider)
        }
        _ => Box::new(DefaultLimitProvider::with_safety_context(safety_context)),
    }
}

/// Create platform limit provider for pure no_std
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub fn create_limit_provider() -> &'static dyn ComprehensiveLimitProvider {
    static DEFAULT_PROVIDER: DefaultLimitProvider = DefaultLimitProvider {
        safety_context: UniversalSafetyContext::new(SafetyStandard::Iso26262(AsilLevel::AsilD)),
    };
    &DEFAULT_PROVIDER
}

/// Time abstraction for cross-platform compatibility
pub struct PlatformTime;

impl PlatformTime {
    /// Get current time in milliseconds since some epoch
    pub fn current_time_ms() -> u64 {
        #[cfg(feature = "std")]
        {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64
        }
        
        #[cfg(not(feature = "std"))]
        {
            // In no_std environments, return a simple counter
            // Real implementations would use platform-specific timers
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            COUNTER.fetch_add(1, PlatformOrdering::Relaxed)
        }
    }
    
    /// Sleep for the specified duration (no-op in no_std)
    pub fn sleep(duration: Duration) {
        #[cfg(feature = "std")]
        {
            std::thread::sleep(duration);
        }
        
        #[cfg(not(feature = "std"))]
        {
            // No-op in no_std environments
            let _ = duration;
        }
    }
}