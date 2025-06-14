//! CrateId Definitions
//!
//! This module provides the CrateId enum used throughout the WRT memory system.

use crate::memory_coordinator::CrateIdentifier;

/// Crate identifiers for budget tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrateId {
    Foundation,
    Decoder,
    Runtime,
    Component,
    Host,
    Debug,
    Platform,
    Instructions,
    Format,
    Intercept,
    Sync,
    Math,
    Logging,
    Panic,
    TestRegistry,
    VerificationTool,
    Unknown,
    Wasi,
    WasiComponents,
}

impl CrateIdentifier for CrateId {
    fn as_index(&self) -> usize {
        match self {
            CrateId::Foundation => 0,
            CrateId::Decoder => 1,
            CrateId::Runtime => 2,
            CrateId::Component => 3,
            CrateId::Host => 4,
            CrateId::Debug => 5,
            CrateId::Platform => 6,
            CrateId::Instructions => 7,
            CrateId::Format => 8,
            CrateId::Intercept => 9,
            CrateId::Sync => 10,
            CrateId::Math => 11,
            CrateId::Logging => 12,
            CrateId::Panic => 13,
            CrateId::TestRegistry => 14,
            CrateId::VerificationTool => 15,
            CrateId::Unknown => 16,
            CrateId::Wasi => 17,
            CrateId::WasiComponents => 18,
        }
    }
    
    fn name(&self) -> &'static str {
        match self {
            CrateId::Foundation => "foundation",
            CrateId::Decoder => "decoder",
            CrateId::Runtime => "runtime",
            CrateId::Component => "component",
            CrateId::Host => "host",
            CrateId::Debug => "debug",
            CrateId::Platform => "platform",
            CrateId::Instructions => "instructions",
            CrateId::Format => "format",
            CrateId::Intercept => "intercept",
            CrateId::Sync => "sync",
            CrateId::Math => "math",
            CrateId::Logging => "logging",
            CrateId::Panic => "panic",
            CrateId::TestRegistry => "test_registry",
            CrateId::VerificationTool => "verification_tool",
            CrateId::Unknown => "unknown",
            CrateId::Wasi => "wasi",
            CrateId::WasiComponents => "wasi_components",
        }
    }
    
    fn count() -> usize {
        19
    }
}