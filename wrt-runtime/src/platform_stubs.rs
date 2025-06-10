// WRT - wrt-runtime
// Module: Platform Type Stubs (Agent D)
// TEMPORARY - These stubs will be replaced by Agent B's work
//
// Copyright (c) 2025 The WRT Project Developers
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Temporary stubs for Agent B's platform types
//! 
//! These types allow Agent D to work independently while Agent B
//! implements comprehensive platform detection and limits.
//! They will be removed during the final integration phase.

#![allow(dead_code)] // Allow during stub phase

#[cfg(feature = "std")]
use std::boxed::Box;
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

use super::foundation_stubs::AsilLevel;

/// Platform identification enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformId {
    Linux,
    QNX,
    Embedded,
    MacOS,
    Windows,
    Unknown,
}

impl Default for PlatformId {
    fn default() -> Self {
        #[cfg(target_os = "linux")]
        return PlatformId::Linux;
        #[cfg(target_os = "macos")]
        return PlatformId::MacOS;
        #[cfg(target_os = "windows")]
        return PlatformId::Windows;
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        return PlatformId::Unknown;
    }
}

/// Comprehensive platform limits stub
#[derive(Debug, Clone)]
pub struct ComprehensivePlatformLimits {
    pub platform_id: PlatformId,
    pub max_total_memory: usize,
    pub max_wasm_linear_memory: usize,
    pub max_stack_bytes: usize,
    pub max_components: usize,
    pub max_debug_overhead: usize,
    pub asil_level: AsilLevel,
}

impl Default for ComprehensivePlatformLimits {
    fn default() -> Self {
        Self {
            platform_id: PlatformId::default(),
            max_total_memory: 1024 * 1024 * 1024, // 1GB
            max_wasm_linear_memory: 256 * 1024 * 1024, // 256MB
            max_stack_bytes: 8 * 1024 * 1024, // 8MB
            max_components: 256,
            max_debug_overhead: 64 * 1024 * 1024, // 64MB
            asil_level: AsilLevel::QM,
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
        Self {
            platform_id: PlatformId::Embedded,
            max_total_memory: 256 * 1024, // 256KB
            max_wasm_linear_memory: 64 * 1024, // 64KB
            max_stack_bytes: 16 * 1024, // 16KB
            max_components: 8,
            max_debug_overhead: 0,
            asil_level: AsilLevel::AsilD,
        }
    }
}

/// Platform limit provider trait stub
pub trait ComprehensiveLimitProvider: Send + Sync {
    fn discover_limits(&self) -> Result<ComprehensivePlatformLimits, wrt_error::Error>;
    fn platform_id(&self) -> PlatformId;
}

/// Default platform limit provider
#[derive(Default)]
pub struct DefaultLimitProvider;

impl ComprehensiveLimitProvider for DefaultLimitProvider {
    fn discover_limits(&self) -> Result<ComprehensivePlatformLimits, wrt_error::Error> {
        Ok(ComprehensivePlatformLimits::default())
    }
    
    fn platform_id(&self) -> PlatformId {
        PlatformId::default()
    }
}

/// Linux limit provider stub
pub struct LinuxLimitProvider;

impl ComprehensiveLimitProvider for LinuxLimitProvider {
    fn discover_limits(&self) -> Result<ComprehensivePlatformLimits, wrt_error::Error> {
        Ok(ComprehensivePlatformLimits {
            platform_id: PlatformId::Linux,
            max_total_memory: 8 * 1024 * 1024 * 1024, // 8GB
            max_wasm_linear_memory: 2 * 1024 * 1024 * 1024, // 2GB
            max_stack_bytes: 32 * 1024 * 1024, // 32MB
            max_components: 1024,
            max_debug_overhead: 512 * 1024 * 1024, // 512MB
            asil_level: AsilLevel::QM,
        })
    }
    
    fn platform_id(&self) -> PlatformId {
        PlatformId::Linux
    }
}

/// QNX limit provider stub
pub struct QnxLimitProvider;

impl ComprehensiveLimitProvider for QnxLimitProvider {
    fn discover_limits(&self) -> Result<ComprehensivePlatformLimits, wrt_error::Error> {
        Ok(ComprehensivePlatformLimits {
            platform_id: PlatformId::QNX,
            max_total_memory: 512 * 1024 * 1024, // 512MB
            max_wasm_linear_memory: 128 * 1024 * 1024, // 128MB
            max_stack_bytes: 4 * 1024 * 1024, // 4MB
            max_components: 64,
            max_debug_overhead: 16 * 1024 * 1024, // 16MB
            asil_level: AsilLevel::AsilB,
        })
    }
    
    fn platform_id(&self) -> PlatformId {
        PlatformId::QNX
    }
}

/// Create platform-appropriate limit provider
pub fn create_limit_provider() -> Box<dyn ComprehensiveLimitProvider> {
    match PlatformId::default() {
        PlatformId::Linux => Box::new(LinuxLimitProvider),
        PlatformId::QNX => Box::new(QnxLimitProvider),
        _ => Box::new(DefaultLimitProvider),
    }
}