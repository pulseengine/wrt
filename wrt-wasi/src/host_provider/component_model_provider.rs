//! Component model provider for WASI integration
//!
//! This module provides the ComponentModelProvider that integrates WASI host functions
//! with the WRT component model using proven patterns from wrt-host and wrt-component.

use crate::prelude::*;
use wrt_host::{HostFunction, HostFunctionHandler, CloneableFn};
use wrt_format::component::ExternType;

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
    cached_functions: Option<BoundedVec<HostFunction, 64>>,
}

impl ComponentModelProvider {
    /// Create a new component model provider with the given capabilities
    pub fn new(capabilities: WasiCapabilities) -> Result<Self> {
        let resource_manager = WasiResourceManager::new()?;
        
        Ok(Self {
            capabilities,
            resource_manager,
            cached_functions: None,
        })
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
    
    /// Create host function for WASI filesystem operations
    #[cfg(feature = "wasi-filesystem")]
    fn create_filesystem_functions(&self) -> Result<BoundedVec<HostFunction, 16>> {
        use crate::preview2::filesystem;
        
        let provider = safe_managed_alloc!(4096, CrateId::WrtWasi)?;
        let mut functions = BoundedVec::new(provider)?;
        
        if self.capabilities.filesystem.read_access {
            functions.push(HostFunction {
                name: "wasi:filesystem/types.read".to_string(),
                handler: HostFunctionHandler::new(CloneableFn::new(
                    |_target, _args| filesystem::wasi_filesystem_read(_target, _args)
                )),
                extern_type: ExternType::Function {
                    params: vec![("fd".to_string(), wrt_format::component::ValType::S32)],
                    results: vec![wrt_format::component::ValType::List(
                        Box::new(wrt_format::component::ValType::U8)
                    )],
                },
            })?;
        }
        
        if self.capabilities.filesystem.write_access {
            functions.push(HostFunction {
                name: "wasi:filesystem/types.write".to_string(), 
                handler: HostFunctionHandler::new(CloneableFn::new(
                    |_target, _args| filesystem::wasi_filesystem_write(_target, _args)
                )),
                extern_type: ExternType::Function {
                    params: vec![
                        ("fd".to_string(), wrt_format::component::ValType::S32),
                        ("data".to_string(), wrt_format::component::ValType::List(
                            Box::new(wrt_format::component::ValType::U8)
                        )),
                    ],
                    results: vec![wrt_format::component::ValType::S32],
                },
            })?;
        }
        
        Ok(functions)
    }
    
    /// Create host functions for WASI CLI operations
    #[cfg(feature = "wasi-cli")]
    fn create_cli_functions(&self) -> Result<BoundedVec<HostFunction, 8>> {
        use crate::preview2::cli;
        
        let provider = safe_managed_alloc!(2048, CrateId::WrtWasi)?;
        let mut functions = BoundedVec::new(provider)?;
        
        if self.capabilities.environment.args_access {
            functions.push(HostFunction {
                name: "wasi:cli/environment.get-arguments".to_string(),
                handler: HostFunctionHandler::new(CloneableFn::new(
                    |_target, _args| cli::wasi_get_arguments(_target, _args)
                )),
                extern_type: ExternType::Function {
                    params: vec![],
                    results: vec![wrt_format::component::ValType::List(
                        Box::new(wrt_format::component::ValType::String)
                    )],
                },
            })?;
        }
        
        if self.capabilities.environment.environ_access {
            functions.push(HostFunction {
                name: "wasi:cli/environment.get-environment".to_string(),
                handler: HostFunctionHandler::new(CloneableFn::new(
                    |_target, _args| cli::wasi_get_environment(_target, _args)
                )),
                extern_type: ExternType::Function {
                    params: vec![],
                    results: vec![wrt_format::component::ValType::List(
                        Box::new(wrt_format::component::ValType::Tuple(vec![
                            wrt_format::component::ValType::String,
                            wrt_format::component::ValType::String,
                        ]))
                    )],
                },
            })?;
        }
        
        Ok(functions)
    }
    
    /// Create host functions for WASI clock operations
    #[cfg(feature = "wasi-clocks")]
    fn create_clock_functions(&self) -> Result<BoundedVec<HostFunction, 8>> {
        use crate::preview2::clocks;
        
        let provider = safe_managed_alloc!(2048, CrateId::WrtWasi)?;
        let mut functions = BoundedVec::new(provider)?;
        
        if self.capabilities.clocks.monotonic_access {
            functions.push(HostFunction {
                name: "wasi:clocks/monotonic-clock.now".to_string(),
                handler: HostFunctionHandler::new(CloneableFn::new(
                    |_target, _args| clocks::wasi_monotonic_clock_now(_target, _args)
                )),
                extern_type: ExternType::Function {
                    params: vec![],
                    results: vec![wrt_format::component::ValType::U64],
                },
            })?;
        }
        
        if self.capabilities.clocks.realtime_access {
            functions.push(HostFunction {
                name: "wasi:clocks/wall-clock.now".to_string(),
                handler: HostFunctionHandler::new(CloneableFn::new(
                    |_target, _args| clocks::wasi_wall_clock_now(_target, _args)
                )),
                extern_type: ExternType::Function {
                    params: vec![],
                    results: vec![wrt_format::component::ValType::Tuple(vec![
                        wrt_format::component::ValType::U64, // seconds
                        wrt_format::component::ValType::U32, // nanoseconds
                    ])],
                },
            })?;
        }
        
        Ok(functions)
    }
    
    /// Create host functions for WASI I/O operations
    #[cfg(feature = "wasi-io")]
    fn create_io_functions(&self) -> Result<BoundedVec<HostFunction, 16>> {
        use crate::preview2::io;
        
        let provider = safe_managed_alloc!(4096, CrateId::WrtWasi)?;
        let mut functions = BoundedVec::new(provider)?;
        
        if self.capabilities.io.stdin_access {
            functions.push(HostFunction {
                name: "wasi:io/streams.read".to_string(),
                handler: HostFunctionHandler::new(CloneableFn::new(
                    |_target, _args| io::wasi_stream_read(_target, _args)
                )),
                extern_type: ExternType::Function {
                    params: vec![
                        ("stream".to_string(), wrt_format::component::ValType::S32),
                        ("len".to_string(), wrt_format::component::ValType::U64),
                    ],
                    results: vec![wrt_format::component::ValType::List(
                        Box::new(wrt_format::component::ValType::U8)
                    )],
                },
            })?;
        }
        
        if self.capabilities.io.stdout_access {
            functions.push(HostFunction {
                name: "wasi:io/streams.write".to_string(),
                handler: HostFunctionHandler::new(CloneableFn::new(
                    |_target, _args| io::wasi_stream_write(_target, _args)
                )),
                extern_type: ExternType::Function {
                    params: vec![
                        ("stream".to_string(), wrt_format::component::ValType::S32),
                        ("data".to_string(), wrt_format::component::ValType::List(
                            Box::new(wrt_format::component::ValType::U8)
                        )),
                    ],
                    results: vec![wrt_format::component::ValType::U64],
                },
            })?;
        }
        
        Ok(functions)
    }
    
    /// Create host functions for WASI random operations
    #[cfg(feature = "wasi-random")]
    fn create_random_functions(&self) -> Result<BoundedVec<HostFunction, 4>> {
        use crate::preview2::random;
        
        let provider = safe_managed_alloc!(1024, CrateId::WrtWasi)?;
        let mut functions = BoundedVec::new(provider)?;
        
        if self.capabilities.random.secure_random {
            functions.push(HostFunction {
                name: "wasi:random/random.get-random-bytes".to_string(),
                handler: HostFunctionHandler::new(CloneableFn::new(
                    |_target, _args| random::wasi_get_random_bytes(_target, _args)
                )),
                extern_type: ExternType::Function {
                    params: vec![("len".to_string(), wrt_format::component::ValType::U64)],
                    results: vec![wrt_format::component::ValType::List(
                        Box::new(wrt_format::component::ValType::U8)
                    )],
                },
            })?;
        }
        
        if self.capabilities.random.pseudo_random {
            functions.push(HostFunction {
                name: "wasi:random/insecure.get-insecure-random-bytes".to_string(),
                handler: HostFunctionHandler::new(CloneableFn::new(
                    |_target, _args| random::wasi_get_insecure_random_bytes(_target, _args)
                )),
                extern_type: ExternType::Function {
                    params: vec![("len".to_string(), wrt_format::component::ValType::U64)],
                    results: vec![wrt_format::component::ValType::List(
                        Box::new(wrt_format::component::ValType::U8)
                    )],
                },
            })?;
        }
        
        Ok(functions)
    }
    
    /// Extract module name from full WASI function name
    fn extract_module_name(function_name: &str) -> &str {
        if let Some(pos) = function_name.find('/') {
            &function_name[..pos]
        } else {
            "wasi"
        }
    }
    
    /// Build all host functions based on enabled capabilities
    fn build_all_functions(&self) -> Result<BoundedVec<HostFunction, 64>> {
        let provider = safe_managed_alloc!(16384, CrateId::WrtWasi)?;
        let mut all_functions = BoundedVec::new(provider)?;
        
        // Add filesystem functions if enabled
        #[cfg(feature = "wasi-filesystem")]
        {
            let fs_functions = self.create_filesystem_functions()?;
            for function in fs_functions.into_iter() {
                all_functions.push(function)?;
            }
        }
        
        // Add CLI functions if enabled
        #[cfg(feature = "wasi-cli")]
        {
            let cli_functions = self.create_cli_functions()?;
            for function in cli_functions.into_iter() {
                all_functions.push(function)?;
            }
        }
        
        // Add clock functions if enabled
        #[cfg(feature = "wasi-clocks")]
        {
            let clock_functions = self.create_clock_functions()?;
            for function in clock_functions.into_iter() {
                all_functions.push(function)?;
            }
        }
        
        // Add I/O functions if enabled
        #[cfg(feature = "wasi-io")]
        {
            let io_functions = self.create_io_functions()?;
            for function in io_functions.into_iter() {
                all_functions.push(function)?;
            }
        }
        
        // Add random functions if enabled
        #[cfg(feature = "wasi-random")]
        {
            let random_functions = self.create_random_functions()?;
            for function in random_functions.into_iter() {
                all_functions.push(function)?;
            }
        }
        
        Ok(all_functions)
    }
}

impl WasiHostProvider for ComponentModelProvider {
    fn get_host_functions(&self) -> Result<Vec<HostFunction>> {
        // Build functions if not cached
        if self.cached_functions.is_none() {
            // Note: In a real implementation, we'd need mutable access to cache
            // For now, rebuild each time (could be optimized later)
        }
        
        let functions = self.build_all_functions()?;
        
        // Convert BoundedVec to Vec for the trait interface
        let mut result = Vec::new();
        for function in functions.into_iter() {
            result.push(function);
        }
        
        Ok(result)
    }
    
    fn function_count(&self) -> usize {
        // Calculate based on enabled capabilities
        let mut count = 0;
        
        #[cfg(feature = "wasi-filesystem")]
        {
            if self.capabilities.filesystem.read_access { count += 1; }
            if self.capabilities.filesystem.write_access { count += 1; }
        }
        
        #[cfg(feature = "wasi-cli")]
        {
            if self.capabilities.environment.args_access { count += 1; }
            if self.capabilities.environment.environ_access { count += 1; }
        }
        
        #[cfg(feature = "wasi-clocks")]
        {
            if self.capabilities.clocks.monotonic_access { count += 1; }
            if self.capabilities.clocks.realtime_access { count += 1; }
        }
        
        #[cfg(feature = "wasi-io")]
        {
            if self.capabilities.io.stdin_access { count += 1; }
            if self.capabilities.io.stdout_access { count += 1; }
        }
        
        #[cfg(feature = "wasi-random")]
        {
            if self.capabilities.random.secure_random { count += 1; }
            if self.capabilities.random.pseudo_random { count += 1; }
        }
        
        count
    }
    
    fn version(&self) -> WasiVersion {
        WasiVersion::Preview2
    }
    
    fn capabilities(&self) -> &WasiCapabilities {
        &self.capabilities
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_component_model_provider_creation() -> Result<()> {
        let capabilities = WasiCapabilities::minimal()?;
        let provider = ComponentModelProvider::new(capabilities)?;
        
        assert_eq!(provider.version(), WasiVersion::Preview2);
        assert!(!provider.capabilities().filesystem.read_access);
        
        Ok(())
    }
    
    #[test]
    fn test_function_count_calculation() -> Result<()> {
        let mut capabilities = WasiCapabilities::minimal()?;
        capabilities.filesystem.read_access = true;
        capabilities.environment.args_access = true;
        
        let provider = ComponentModelProvider::new(capabilities)?;
        
        // Should have at least 2 functions enabled
        assert!(provider.function_count() >= 2);
        
        Ok(())
    }
    
    #[test]
    fn test_module_name_extraction() {
        assert_eq!(
            ComponentModelProvider::extract_module_name("wasi:filesystem/types.read"),
            "wasi:filesystem"
        );
        assert_eq!(
            ComponentModelProvider::extract_module_name("simple_name"),
            "wasi"
        );
    }
}