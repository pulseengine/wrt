//! Component management and lifecycle
//!
//! This module handles component instantiation, communication, linking,
//! and registry management for the WebAssembly Component Model.

pub mod component;
pub mod component_communication;
pub mod component_instantiation;
pub mod component_linker;
pub mod component_no_std;
pub mod component_registry;
pub mod component_registry_no_std;
pub mod component_resolver;

// Re-export based on feature flags to avoid ambiguous imports
#[cfg(feature = "std")]
pub use component::*;
#[cfg(not(feature = "std"))]
pub use component_no_std::*;

pub use component_communication::*;
pub use component_instantiation::*;
pub use component_linker::*;

#[cfg(feature = "std")]
pub use component_registry::*;
#[cfg(not(feature = "std"))]
pub use component_registry_no_std::*;

pub use component_resolver::*;
