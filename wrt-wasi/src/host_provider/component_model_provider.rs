//! Component model provider for WASI integration
//!
//! This module provides the ComponentModelProvider that integrates WASI host functions
//! with the WRT component model using proven patterns from wrt-host and wrt-component.
//! Uses safety-aware allocation and respects configured safety levels.

use crate::{prelude::*, WASI_CRATE_ID, wasi_safety_level};
use crate::capabilities::WasiCapabilities;
use wrt_host::{HostFunctionHandler, CloneableFn};
use crate::HostFunction;
use wrt_format::component::ExternType;
use wrt_foundation::safe_managed_alloc;

/// Component model provider for WASI Preview2
///
/// This provider integrates WASI host functions with the WRT component model,
/// using the same patterns as other WRT host providers.
#[derive(Debug)]
pub struct ComponentModelProvider {
    /// WASI capabilities for this provider
    capabilities: WasiCapabilities,
    /// Resource manager for WASI handles
    resource_manager: WasiResourceManager,
    /// Cached host functions
    #[cfg(feature = "std")]
    cached_functions: Option<Vec<HostFunction>>,
    #[cfg(not(feature = "std"))]
    cached_functions: Option<u8>, // Simplified for no_std mode
}

impl ComponentModelProvider {
    /// Create a new component model provider with the given capabilities
    /// Allocates function cache using safety-aware allocation
    pub fn new(capabilities: WasiCapabilities) -> Result<Self> {
        let resource_manager = WasiResourceManager::new()?;
        
        // Initialize function cache
        #[cfg(feature = "std")]
        let cached_functions = Some(Vec::new());
        #[cfg(not(feature = "std"))]
        let cached_functions = Some(0);
        
        Ok(Self {
            capabilities,
            resource_manager,
            cached_functions,
        })
    }
    
    /// Get the current safety level for this provider
    pub fn safety_level(&self) -> &'static str {
        wasi_safety_level()
    }
    
    /// Register all WASI functions with a callback registry
    ///
    /// This follows the same pattern as other WRT host providers
    pub fn register_with_registry(&self, registry: &mut CallbackRegistry) -> Result<()> {
        let functions = self.get_host_functions()?;
        
        for function in functions {
            let module_name = Self::extract_module_name(&function.name);
            registry.register_host_function(module_name, &function.name, function.handler)?;
        }
        
        Ok(())
    }
    
    /// Build all WASI host functions based on enabled capabilities
    #[cfg(feature = "std")]
    fn build_host_functions(&mut self) -> Result<&Vec<HostFunction>> {
        if self.cached_functions.is_some() {
            return Ok(self.cached_functions.as_ref().unwrap());
        }
        
        let mut functions = Vec::new();
        
        // Add filesystem functions if capabilities allow
        if self.capabilities.filesystem.read_access {
            functions.push(self.create_filesystem_read_function()?);
        }
        if self.capabilities.filesystem.write_access {
            functions.push(self.create_filesystem_write_function()?);
        }
        if self.capabilities.filesystem.directory_access {
            functions.push(self.create_filesystem_open_function()?);
        }
        
        // Add CLI functions if capabilities allow
        if self.capabilities.environment.args_access {
            functions.push(self.create_cli_args_function()?);
        }
        if self.capabilities.environment.environ_access {
            functions.push(self.create_cli_environ_function()?);
        }
        
        // Add clock functions if capabilities allow
        if self.capabilities.clocks.monotonic_access {
            functions.push(self.create_clock_now_function()?);
        }
        
        // Add I/O functions if capabilities allow
        if self.capabilities.io.stdout_access {
            functions.push(self.create_io_write_function()?);
        }
        
        // Add random functions if capabilities allow
        if self.capabilities.random.secure_random {
            functions.push(self.create_random_get_function()?);
        }
        
        self.cached_functions = Some(functions);
        Ok(self.cached_functions.as_ref().unwrap())
    }
    
    /// Build all WASI host functions based on enabled capabilities (no_std version)
    #[cfg(not(feature = "std"))]
    fn build_host_functions(&mut self) -> Result<u8> {
        // In no_std mode, return a simple count of available functions
        let mut count = 0u8;
        
        if self.capabilities.filesystem.read_access { count += 1; }
        if self.capabilities.filesystem.write_access { count += 1; }
        if self.capabilities.filesystem.directory_access { count += 1; }
        if self.capabilities.environment.args_access { count += 1; }
        if self.capabilities.environment.environ_access { count += 1; }
        if self.capabilities.clocks.monotonic_access { count += 1; }
        if self.capabilities.io.stdout_access { count += 1; }
        if self.capabilities.random.secure_random { count += 1; }
        
        self.cached_functions = Some(count);
        Ok(count)
    }
    
    /// Create host function for WASI filesystem read operations
    fn create_filesystem_read_function(&self) -> Result<HostFunction> {
        use crate::preview2::filesystem::wasi_filesystem_read;
        
        Ok(HostFunction {
            name: "wasi:filesystem/types.read".to_string(),
            handler: HostFunctionHandler::new(CloneableFn::new(wasi_filesystem_read)),
            extern_type: ExternType::Func,
        })
    }
    
    /// Create host function for WASI filesystem write operations  
    fn create_filesystem_write_function(&self) -> Result<HostFunction> {
        use crate::preview2::filesystem::wasi_filesystem_write;
        
        Ok(HostFunction {
            name: "wasi:filesystem/types.write".to_string(),
            handler: HostFunctionHandler::new(CloneableFn::new(wasi_filesystem_write)),
            extern_type: ExternType::Func,
        })
    }
    
    /// Create host function for WASI filesystem open operations
    fn create_filesystem_open_function(&self) -> Result<HostFunction> {
        use crate::preview2::filesystem::wasi_filesystem_open_at;
        
        Ok(HostFunction {
            name: "wasi:filesystem/types.open-at".to_string(),
            handler: HostFunctionHandler::new(CloneableFn::new(wasi_filesystem_open_at)),
            extern_type: ExternType::Func,
        })
    }
    
    /// Create host function for WASI CLI arguments
    fn create_cli_args_function(&self) -> Result<HostFunction> {
        use crate::preview2::cli::wasi_cli_get_arguments;
        
        Ok(HostFunction {
            name: "wasi:cli/environment.get-arguments".to_string(),
            handler: HostFunctionHandler::new(CloneableFn::new(wasi_cli_get_arguments)),
            extern_type: ExternType::Func,
        })
    }
    
    /// Create host function for WASI CLI environment variables
    fn create_cli_environ_function(&self) -> Result<HostFunction> {
        use crate::preview2::cli::wasi_cli_get_environment;
        
        Ok(HostFunction {
            name: "wasi:cli/environment.get-environment".to_string(),
            handler: HostFunctionHandler::new(CloneableFn::new(wasi_cli_get_environment)),
            extern_type: ExternType::Func,
        })
    }
    
    /// Create host function for WASI monotonic clock
    fn create_clock_now_function(&self) -> Result<HostFunction> {
        use crate::preview2::clocks::wasi_monotonic_clock_now;
        
        Ok(HostFunction {
            name: "wasi:clocks/monotonic-clock.now".to_string(),
            handler: HostFunctionHandler::new(CloneableFn::new(wasi_monotonic_clock_now)),
            extern_type: ExternType::Func,
        })
    }
    
    /// Create host function for WASI I/O write
    fn create_io_write_function(&self) -> Result<HostFunction> {
        use crate::preview2::io::wasi_stream_write;
        
        Ok(HostFunction {
            name: "wasi:io/streams.write".to_string(),
            handler: HostFunctionHandler::new(CloneableFn::new(wasi_stream_write)),
            extern_type: ExternType::Func,
        })
    }
    
    /// Create host function for WASI random number generation
    fn create_random_get_function(&self) -> Result<HostFunction> {
        use crate::preview2::random::wasi_get_random_bytes;
        
        Ok(HostFunction {
            name: "wasi:random/random.get-random-bytes".to_string(),
            handler: HostFunctionHandler::new(CloneableFn::new(wasi_get_random_bytes)),
            extern_type: ExternType::Func,
        })
    }
    
    /// Extract module name from full WASI function name
    fn extract_module_name(function_name: &str) -> &str {
        if let Some(colon_pos) = function_name.find(':') {
            if let Some(slash_pos) = function_name[colon_pos..].find('/') {
                return &function_name[..colon_pos + slash_pos];
            }
        }
        function_name
    }
}

impl WasiHostProvider for ComponentModelProvider {
    /// Get all host functions provided by this WASI implementation
    fn get_host_functions(&self) -> Result<Vec<HostFunction>> {
        #[cfg(feature = "std")]
        {
            if let Some(ref functions) = self.cached_functions {
                Ok(functions.clone())
            } else {
                Ok(Vec::new())
            }
        }
        #[cfg(not(feature = "std"))]
        {
            // In no_std mode, return empty vector since we can't store dynamic functions
            Ok(Vec::new())
        }
    }
    
    /// Get the number of functions provided
    fn function_count(&self) -> usize {
        #[cfg(feature = "std")]
        {
            if let Some(ref functions) = self.cached_functions {
                functions.len()
            } else {
                0
            }
        }
        #[cfg(not(feature = "std"))]
        {
            if let Some(count) = self.cached_functions {
                count as usize
            } else {
                0
            }
        }
    }
    
    /// Get the WASI version supported by this provider
    fn version(&self) -> WasiVersion {
        WasiVersion::Preview2
    }
    
    /// Get the capabilities enabled for this provider
    fn capabilities(&self) -> &WasiCapabilities {
        &self.capabilities
    }
}

/// Safety-aware WASI provider factory
pub struct WasiProviderBuilder {
    capabilities: Option<WasiCapabilities>,
    safety_level: Option<&'static str>,
}

impl WasiProviderBuilder {
    /// Create a new WASI provider builder
    pub fn new() -> Self {
        Self {
            capabilities: None,
            safety_level: None,
        }
    }
    
    /// Set capabilities for the provider
    pub fn with_capabilities(mut self, capabilities: WasiCapabilities) -> Self {
        self.capabilities = Some(capabilities);
        self
    }
    
    /// Set safety level for the provider (overrides capability-based detection)
    pub fn with_safety_level(mut self, level: &'static str) -> Self {
        self.safety_level = Some(level);
        self
    }
    
    /// Build the WASI provider with safety-aware defaults
    pub fn build(self) -> Result<ComponentModelProvider> {
        let capabilities = match self.capabilities {
            Some(caps) => caps,
            None => {
                // Choose default capabilities based on safety level
                match self.safety_level.unwrap_or(wasi_safety_level()) {
                    "maximum-safety" => WasiCapabilities::minimal()?,
                    "static-memory-safety" => WasiCapabilities::sandboxed()?,
                    "bounded-collections" => WasiCapabilities::sandboxed()?,
                    _ => WasiCapabilities::system_utility()?,
                }
            }
        };
        
        ComponentModelProvider::new(capabilities)
    }
}

impl Default for WasiProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_provider_creation() -> Result<()> {
        let capabilities = WasiCapabilities::minimal()?;
        let provider = ComponentModelProvider::new(capabilities)?;
        
        assert_eq!(provider.version(), WasiVersion::Preview2);
        assert!(!provider.capabilities().filesystem.read_access);
        
        Ok(())
    }
    
    #[test]
    fn test_extract_module_name() {
        assert_eq!(
            ComponentModelProvider::extract_module_name("wasi:filesystem/types.read"),
            "wasi:filesystem"
        );
        assert_eq!(
            ComponentModelProvider::extract_module_name("wasi:cli/environment.get-arguments"),
            "wasi:cli"
        );
    }
    
    #[test]
    fn test_safety_level_integration() -> Result<()> {
        let capabilities = WasiCapabilities::minimal()?;
        let provider = ComponentModelProvider::new(capabilities)?;
        
        // Should return current compile-time safety level
        let safety_level = provider.safety_level();
        assert!(!safety_level.is_empty());
        
        Ok(())
    }
    
    #[test]
    fn test_provider_builder() -> Result<()> {
        let provider = WasiProviderBuilder::new()
            .with_safety_level("bounded-collections")
            .build()?;
            
        assert_eq!(provider.version(), WasiVersion::Preview2);
        
        Ok(())
    }
}