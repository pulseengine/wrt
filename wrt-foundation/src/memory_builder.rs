// WRT - wrt-foundation
// Module: Memory Builder patterns
// SW-REQ-ID: REQ_MEMORY_001 REQ_PLATFORM_001
//
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Provides builder patterns for platform-backed memory types.
//!
//! These builders allow for fluent configuration of memory providers and
//! linear memory instances with strongly-typed safety guarantees.

#[cfg(feature = "platform-memory")]
use core::fmt::Debug;

#[cfg(feature = "platform-memory")]
use wrt_platform::memory::PageAllocator;

#[cfg(feature = "platform-memory")]
use crate::{
    linear_memory::PalMemoryProvider, runtime_memory::LinearMemory, verification::VerificationLevel,
    WrtResult,
};

/// Builder for `PalMemoryProvider` instances.
///
/// This builder provides a fluent API for configuring a platform-backed
/// memory provider, including memory size limits and verification settings.
#[cfg(feature = "platform-memory")]
#[derive(Debug)]
pub struct PalMemoryProviderBuilder<A: PageAllocator + Send + Sync + Clone + 'static> {
    allocator: A,
    initial_pages: u32,
    maximum_pages: Option<u32>,
    verification_level: VerificationLevel,
}

#[cfg(feature = "platform-memory")]
impl<A: PageAllocator + Send + Sync + Clone + 'static> PalMemoryProviderBuilder<A> {
    /// Creates a new builder with the given page allocator.
    pub fn new(allocator: A) -> Self {
        Self {
            allocator,
            initial_pages: 1, // Default to 1 page (64KiB)
            maximum_pages: None,
            verification_level: VerificationLevel::Standard,
        }
    }

    /// Sets the initial size in WebAssembly pages (64KiB each).
    pub fn with_initial_pages(mut self, pages: u32) -> Self {
        self.initial_pages = pages;
        self
    }

    /// Sets the maximum size in WebAssembly pages (64KiB each).
    pub fn with_maximum_pages(mut self, pages: u32) -> Self {
        self.maximum_pages = Some(pages);
        self
    }

    /// Sets the verification level for memory operations.
    pub fn with_verification_level(mut self, level: VerificationLevel) -> Self {
        self.verification_level = level;
        self
    }

    /// Builds and returns a configured `PalMemoryProvider`.
    pub fn build(self) -> WrtResult<PalMemoryProvider<A>> {
        PalMemoryProvider::new(
            self.allocator,
            self.initial_pages,
            self.maximum_pages,
            self.verification_level,
        )
    }
}

/// Builder for `LinearMemory` instances.
///
/// This builder provides a fluent API for configuring WebAssembly linear memory
/// instances backed by a platform allocator.
#[cfg(feature = "platform-memory")]
#[derive(Debug)]
pub struct LinearMemoryBuilder<A: PageAllocator + Send + Sync + Clone + 'static> {
    allocator: A,
    initial_pages: u32,
    maximum_pages: Option<u32>,
    verification_level: VerificationLevel,
}

#[cfg(feature = "platform-memory")]
impl<A: PageAllocator + Send + Sync + Clone + 'static> LinearMemoryBuilder<A> {
    /// Creates a new builder with the given page allocator.
    pub fn new(allocator: A) -> Self {
        Self {
            allocator,
            initial_pages: 1, // Default to 1 page (64KiB)
            maximum_pages: None,
            verification_level: VerificationLevel::Standard,
        }
    }

    /// Sets the initial size in WebAssembly pages (64KiB each).
    pub fn with_initial_pages(mut self, pages: u32) -> Self {
        self.initial_pages = pages;
        self
    }

    /// Sets the maximum size in WebAssembly pages (64KiB each).
    pub fn with_maximum_pages(mut self, pages: u32) -> Self {
        self.maximum_pages = Some(pages);
        self
    }

    /// Sets the verification level for memory operations.
    pub fn with_verification_level(mut self, level: VerificationLevel) -> Self {
        self.verification_level = level;
        self
    }

    /// Builds and returns a configured `LinearMemory`.
    pub fn build(self) -> WrtResult<LinearMemory<A>> {
        LinearMemory::new(
            self.allocator,
            self.initial_pages,
            self.maximum_pages,
            self.verification_level,
        )
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "platform-memory")]
    use super::*;

    #[cfg(all(feature = "platform-memory", feature = "platform-macos", target_os = "macos"))]
    #[test]
    fn test_pal_memory_provider_builder() {
        use wrt_platform::macos_memory::MacOsAllocator;

        let allocator = MacOsAllocator::new(None);
        let builder = PalMemoryProviderBuilder::new(allocator)
            .with_initial_pages(2)
            .with_maximum_pages(10)
            .with_verification_level(VerificationLevel::Critical);

        let provider = builder.build().unwrap();
        assert_eq!(provider.pages(), 2);
        assert_eq!(provider.verification_level(), VerificationLevel::Critical);
    }

    #[cfg(all(feature = "platform-memory", feature = "platform-macos", target_os = "macos"))]
    #[test]
    fn test_linear_memory_builder() {
        use wrt_platform::macos_memory::MacOsAllocator;

        let allocator = MacOsAllocator::new(None);
        let builder = LinearMemoryBuilder::new(allocator)
            .with_initial_pages(2)
            .with_maximum_pages(10)
            .with_verification_level(VerificationLevel::Critical);

        let memory = builder.build().unwrap();
        // LinearMemory delegates to the provider, so we know these will match
        assert_eq!(memory.size(), 2 * 65_536);
    }
}
