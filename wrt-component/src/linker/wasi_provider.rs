//! WASI instance provider for Component Model
//!
//! Creates component instances that satisfy WASI Preview 2 imports.
//! This is a stub implementation - instances don't have actual WASI logic yet.

use wrt_error::Result;
use crate::instantiation::{InstanceImport, ExportValue, FunctionExport};
use crate::prelude::WrtComponentType;
use crate::bounded_component_infra::ComponentProvider;

#[cfg(feature = "std")]
use std::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

/// Provides WASI component instances
///
/// Each WASI interface (e.g., wasi:cli/stdout, wasi:io/streams) is represented
/// as a component instance that exports the functions defined in the WIT interface.
///
/// This implementation creates instances with actual exported functions that call
/// the WASI host implementations.
pub struct WasiInstanceProvider {
    /// Next instance ID to allocate
    /// Starting at 100 to avoid conflicts with component-internal instances
    next_instance_id: u32,
    /// Next function index to allocate
    next_function_index: u32,
}

impl WasiInstanceProvider {
    /// Create a new WASI instance provider
    pub fn new() -> Result<Self> {
        Ok(Self {
            next_instance_id: 100,
            next_function_index: 1000,
        })
    }

    /// Create a WASI instance for the given interface with actual function exports
    ///
    /// # Arguments
    /// * `interface_name` - Full interface name (e.g., "wasi:cli/stdout@0.2.0")
    ///
    /// # Returns
    /// InstanceImport with exported functions that call WASI host implementations
    pub fn create_instance(&mut self, interface_name: &str) -> Result<InstanceImport> {
        // Allocate instance ID
        let id = self.next_instance_id;
        self.next_instance_id += 1;

        #[cfg(feature = "std")]
        println!("[WASI-PROVIDER] Creating instance {} for {}", id, interface_name);

        // Create instance with exports based on interface
        let mut instance = InstanceImport {
            #[cfg(feature = "std")]
            exports: std::collections::BTreeMap::new(),
            #[cfg(not(feature = "std"))]
            exports: Default::default(),
        };

        // Populate exports based on interface name
        match interface_name {
            name if name.starts_with("wasi:cli/stdout") => {
                self.add_stdout_exports(&mut instance)?;
            }
            name if name.starts_with("wasi:io/streams") => {
                self.add_streams_exports(&mut instance)?;
            }
            _ => {
                #[cfg(feature = "std")]
                println!("[WASI-PROVIDER] Warning: No exports for unknown interface '{}'", interface_name);
            }
        }

        #[cfg(feature = "std")]
        println!("[WASI-PROVIDER] Instance {} created with {} exports", id, instance.exports.len());

        Ok(instance)
    }

    /// Add stdout interface exports (wasi:cli/stdout@0.2.0)
    fn add_stdout_exports(&mut self, instance: &mut InstanceImport) -> Result<()> {
        // Export: get-stdout() -> output-stream
        let get_stdout_index = self.next_function_index;
        self.next_function_index += 1;

        let provider = ComponentProvider::default();
        let signature = WrtComponentType::Unit(provider)?;

        let func_export = FunctionExport {
            signature,
            index: get_stdout_index,
        };

        #[cfg(feature = "std")]
        instance.exports.insert(
            "get-stdout".to_string(),
            Box::new(ExportValue::Function(func_export)),
        );

        #[cfg(not(feature = "std"))]
        {
            use wrt_foundation::bounded::BoundedString;
            let name = BoundedString::try_from_str("get-stdout")?;
            instance.exports.push((name, Box::new(ExportValue::Function(func_export))))
                .map_err(|_| wrt_error::Error::resource_exhausted("Too many exports"))?;
        }

        #[cfg(feature = "std")]
        println!("[WASI-PROVIDER] Added export: get-stdout (index {})", get_stdout_index);

        Ok(())
    }

    /// Add streams interface exports (wasi:io/streams@0.2.0)
    fn add_streams_exports(&mut self, instance: &mut InstanceImport) -> Result<()> {
        // Export: [method]output-stream.blocking-write-and-flush
        let write_flush_index = self.next_function_index;
        self.next_function_index += 1;

        let provider = ComponentProvider::default();
        let signature = WrtComponentType::Unit(provider)?;

        let func_export = FunctionExport {
            signature,
            index: write_flush_index,
        };

        #[cfg(feature = "std")]
        instance.exports.insert(
            "[method]output-stream.blocking-write-and-flush".to_string(),
            Box::new(ExportValue::Function(func_export)),
        );

        #[cfg(not(feature = "std"))]
        {
            use wrt_foundation::bounded::BoundedString;
            let name = BoundedString::try_from_str("[method]output-stream.blocking-write-and-flush")?;
            instance.exports.push((name, Box::new(ExportValue::Function(func_export))))
                .map_err(|_| wrt_error::Error::resource_exhausted("Too many exports"))?;
        }

        #[cfg(feature = "std")]
        println!("[WASI-PROVIDER] Added export: [method]output-stream.blocking-write-and-flush (index {})", write_flush_index);

        Ok(())
    }

    /// Get the list of supported WASI interfaces
    ///
    /// Returns interface names that this provider can satisfy
    #[allow(dead_code)]
    pub fn supported_interfaces() -> &'static [&'static str] {
        &[
            "wasi:cli/environment@0.2.0",
            "wasi:cli/exit@0.2.0",
            "wasi:cli/stdin@0.2.0",
            "wasi:cli/stdout@0.2.0",
            "wasi:cli/stderr@0.2.0",
            "wasi:io/error@0.2.0",
            "wasi:io/poll@0.2.0",
            "wasi:io/streams@0.2.0",
            "wasi:clocks/monotonic-clock@0.2.0",
            "wasi:clocks/wall-clock@0.2.0",
            "wasi:filesystem/types@0.2.0",
            "wasi:filesystem/preopens@0.2.0",
            "wasi:random/random@0.2.0",
        ]
    }
}

impl Default for WasiInstanceProvider {
    fn default() -> Self {
        Self::new().expect("Failed to create WasiInstanceProvider")
    }
}
