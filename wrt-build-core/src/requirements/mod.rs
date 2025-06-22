//! Requirements verification system with SCORE-inspired methodology
//!
//! This module provides comprehensive requirements traceability and verification
//! capabilities for safety-critical systems, supporting both simple file-based
//! verification and advanced SCORE-inspired safety requirement tracking.

pub mod model;

pub use model::{
    RequirementId, RequirementType, VerificationMethod, VerificationStatus,
    CoverageLevel, SafetyRequirement, RequirementRegistry, ComplianceReport,
};

// Re-export simple requirements types for backward compatibility
pub use super::requirements::{
    Requirements, RequirementsMetadata, Requirement,
    RequirementsVerificationResult,
};