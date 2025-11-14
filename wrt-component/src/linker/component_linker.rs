//! Component linker for resolving imports to providers

use wrt_error::{Error, Result};
use wrt_format::component::Import;

#[cfg(feature = "std")]
use std::{format, vec::Vec};
#[cfg(not(feature = "std"))]
use alloc::{format, vec::Vec};

use super::wasi_provider::WasiInstanceProvider;

// Import the types we need
use crate::instantiation::{
    InstanceImport,
    ResolvedImport,
};

/// Component linker that resolves imports to providers
///
/// The linker matches component imports against available providers
/// and creates runtime instances that satisfy the imports.
///
/// # Example
/// ```ignore
/// let mut linker = ComponentLinker::new()?;
/// let linked = linker.link_imports(&component.imports)?;
/// ```
pub struct ComponentLinker {
    /// WASI instance provider for WASI Preview 2 interfaces
    wasi_provider: WasiInstanceProvider,
}

impl ComponentLinker {
    /// Create a new component linker
    pub fn new() -> Result<Self> {
        Ok(Self {
            wasi_provider: WasiInstanceProvider::new()?,
        })
    }

    /// Link component imports to providers
    ///
    /// This performs a simple name-based matching for MVP.
    /// Future versions will add type validation.
    ///
    /// Returns ResolvedImport structures that can be stored in ComponentInstance.imports
    pub fn link_imports(&mut self, imports: &[Import]) -> Result<Vec<ResolvedImport>> {
        let mut resolved = Vec::with_capacity(imports.len());

        for import in imports {
            // Build full import name
            let name = if import.name.namespace.is_empty() {
                import.name.name.clone()
            } else {
                format!("{}:{}", import.name.namespace, import.name.name)
            };

            // Check if this is a WASI import
            if name.starts_with("wasi:") || name.contains("wasi:") {
                // Create WASI instance with actual function exports
                let instance_import = self.wasi_provider.create_instance(&name)?;

                // Create ResolvedImport enum variant
                let resolved_import = ResolvedImport::Instance(instance_import);

                resolved.push(resolved_import);
            } else {
                // Unknown import - for now, just skip with a warning
                #[cfg(feature = "std")]
                println!("[LINKER] Warning: Unknown import '{}' - skipping", name);
            }
        }

        #[cfg(feature = "std")]
        println!("[LINKER] Successfully resolved {} / {} imports", resolved.len(), imports.len());

        Ok(resolved)
    }
}

impl Default for ComponentLinker {
    fn default() -> Self {
        Self::new().expect("Failed to create ComponentLinker")
    }
}
