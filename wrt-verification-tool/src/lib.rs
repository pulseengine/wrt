//! WRT Verification Tool
//!
//! A comprehensive verification framework for WebAssembly Runtime (WRT) that provides
//! safety-critical verification capabilities inspired by SCORE methodology.
//!
//! # Features
//!
//! - **Requirements Traceability**: Track requirements through implementation, testing, and documentation
//! - **ASIL-Tagged Testing**: Automotive Safety Integrity Level aware test categorization
//! - **Safety Verification**: Comprehensive compliance checking for safety standards
//! - **Documentation Verification**: Automated documentation completeness and quality checking
//! - **Platform Verification**: Hardware and platform-specific verification capabilities
//!
//! # Usage
//!
//! ```rust
//! use wrt_verification_tool::{
//!     requirements::{RequirementRegistry, SafetyRequirement, RequirementId, RequirementType},
//!     safety_verification::SafetyVerificationFramework,
//!     documentation_verification::DocumentationVerificationFramework,
//! };
//! 
//! // Create requirement registry
//! let mut registry = RequirementRegistry::new();
//!
//! // Add safety requirements
//! let req = SafetyRequirement::new(
//!     RequirementId::new("REQ_SAFETY_001"),
//!     "Memory Safety".to_string(),
//!     "All memory operations must be bounds-checked".to_string(),
//!     RequirementType::Safety,
//!     AsilLevel::AsilC,
//! );
//! registry.add_requirement(req);
//!
//! // Create verification framework
//! let mut framework = SafetyVerificationFramework::new();
//! framework.add_requirement_registry(registry);
//!
//! // Verify compliance
//! let compliance = framework.verify_asil_compliance(AsilLevel::AsilC);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

// Re-export foundation types
pub use wrt_foundation::safety_system::AsilLevel;

// Public modules
pub mod requirements;
pub mod safety_verification;
pub mod documentation_verification;
pub mod platform_verification;
pub mod requirements_file;

// Internal modules
mod tests;

// Re-export key types for convenience
pub use requirements::{
    RequirementRegistry, SafetyRequirement, RequirementId, RequirementType,
    VerificationMethod, VerificationStatus, CoverageLevel
};

pub use safety_verification::{
    SafetyVerificationFramework, ComplianceVerificationResult, TestResult,
    TestCoverageType, CoverageData, PlatformVerification, SafetyReport
};

pub use documentation_verification::{
    DocumentationVerificationFramework, DocumentationVerificationResult,
    DocumentationAnalysis, DocumentationViolation, DocumentationReport
};

pub use platform_verification::{
    PlatformVerificationEngine, PlatformVerificationConfig,
    PlatformVerificationConfigBuilder
};