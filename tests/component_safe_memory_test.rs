//! Tests for component safe memory integration
//!
//! These tests verify that the Component implementation correctly
//! uses safe memory structures and verification levels.

use wrt_component::component::{Component, WrtComponentType};
use wrt_foundation::verification::VerificationLevel;
use wrt_error::Result;
use std::sync::Arc;

#[test]
fn test_component_with_verification_levels() -> Result<()> {
    // Create a new component type
    let mut component_type = WrtComponentType::new();
    
    // Set verification level
    component_type.set_verification_level(VerificationLevel::Full);
    assert_eq!(component_type.verification_level(), VerificationLevel::Full);
    
    // Create a component from the type
    let mut component = Component::new(component_type);
    
    // Verify the verification level was passed correctly
    assert_eq!(component.verification_level(), VerificationLevel::Full);
    
    // Change the verification level
    component.set_verification_level(VerificationLevel::Standard);
    assert_eq!(component.verification_level(), VerificationLevel::Standard);
    
    // The resource table should also have the verification level updated
    // We can't directly test this without exposing more methods, but at least
    // we ensure the code doesn't crash
    
    Ok(())
}

#[test]
fn test_arc_component_verification() -> Result<()> {
    // Create a component with Sampling verification
    let mut component_type = WrtComponentType::new();
    component_type.set_verification_level(VerificationLevel::Sampling);
    
    let component = Component::new(component_type);
    let arc_component = Arc::new(component);
    
    // Even with Arc, we should still have the verification level preserved
    assert_eq!(arc_component.verification_level(), VerificationLevel::Sampling);
    
    Ok(())
} 