//! Callback registry for WebAssembly host functions.
//!
//! This module provides a registry for host functions that can be called
//! from WebAssembly components.

use core::any::Any;

#[cfg(feature = "std")]
use std::{collections::HashMap, fmt, string::String, sync::Arc, vec::Vec};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{collections::BTreeMap as HashMap, string::String, sync::Arc, vec::Vec};

use wrt_error::{kinds, Error, Result};
use wrt_intercept::LinkInterceptor;
use wrt_types::values::Value;

use crate::function::HostFunctionHandler;

/// A type for representing different callback types in the registry
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallbackType {
    /// Logging callback
    Logging,
    // Other callback types can be added here as needed
}

/// A callback registry for handling WebAssembly component operations
#[derive(Default)]
pub struct CallbackRegistry {
    /// Generic callback storage for different types of callbacks
    callbacks: HashMap<CallbackType, Box<dyn Any + Send + Sync>>,

    /// Host functions registry (module name -> function name -> handler)
    host_functions: HashMap<String, HashMap<String, HostFunctionHandler>>,

    /// Optional interceptor for monitoring and modifying function calls
    interceptor: Option<Arc<LinkInterceptor>>,
}

#[cfg(feature = "std")]
impl fmt::Debug for CallbackRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CallbackRegistry")
            .field("registered_callbacks", &self.callbacks.keys())
            .field("registered_modules", &self.host_functions.keys())
            .finish()
    }
}

#[cfg(not(feature = "std"))]
impl core::fmt::Debug for CallbackRegistry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CallbackRegistry")
            .field("registered_callbacks", &self.callbacks.keys())
            .field("registered_modules", &self.host_functions.keys())
            .finish()
    }
}

impl CallbackRegistry {
    /// Create a new callback registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::new(),
            host_functions: HashMap::new(),
            interceptor: None,
        }
    }

    /// Sets an interceptor for this registry
    pub fn with_interceptor(mut self, interceptor: Arc<LinkInterceptor>) -> Self {
        self.interceptor = Some(interceptor);
        self
    }

    /// Get the interceptor if one is set
    pub fn get_interceptor(&self) -> Option<&LinkInterceptor> {
        self.interceptor.as_ref().map(|arc| arc.as_ref())
    }

    /// Register a callback
    pub fn register_callback<T: 'static + Send + Sync>(
        &mut self,
        callback_type: CallbackType,
        callback: T,
    ) {
        self.callbacks.insert(callback_type, Box::new(callback));
    }

    /// Get a callback
    pub fn get_callback<T: 'static + Send + Sync>(
        &self,
        callback_type: &CallbackType,
    ) -> Option<&T> {
        self.callbacks
            .get(callback_type)
            .and_then(|cb| cb.downcast_ref())
    }

    /// Get a mutable callback
    pub fn get_callback_mut<T: 'static + Send + Sync>(
        &mut self,
        callback_type: &CallbackType,
    ) -> Option<&mut T> {
        self.callbacks
            .get_mut(callback_type)
            .and_then(|cb| cb.downcast_mut())
    }

    /// Register a host function
    pub fn register_host_function(
        &mut self,
        module_name: &str,
        function_name: &str,
        handler: HostFunctionHandler,
    ) {
        let module_name = module_name.to_string();
        let function_name = function_name.to_string();

        let module_functions = self.host_functions.entry(module_name).or_default();
        module_functions.insert(function_name, handler);
    }

    /// Check if a host function is registered
    #[must_use]
    pub fn has_host_function(&self, module_name: &str, function_name: &str) -> bool {
        self.host_functions
            .get(module_name)
            .and_then(|funcs| funcs.get(function_name))
            .is_some()
    }

    /// Call a host function
    ///
    /// # Errors
    ///
    /// Returns an error if the host function is not found or fails during execution
    pub fn call_host_function(
        &self,
        engine: &mut dyn Any,
        module_name: &str,
        function_name: &str,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        // If we have an interceptor, use it to intercept the call
        if let Some(interceptor) = self.get_interceptor() {
            interceptor.intercept_call(
                "host",
                &format!("{}::{}", module_name, function_name),
                args,
                |modified_args| {
                    self.call_host_function_internal(
                        engine,
                        module_name,
                        function_name,
                        modified_args,
                    )
                },
            )
        } else {
            self.call_host_function_internal(engine, module_name, function_name, args)
        }
    }

    /// Internal implementation of call_host_function without interception
    fn call_host_function_internal(
        &self,
        engine: &mut dyn Any,
        module_name: &str,
        function_name: &str,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        if let Some(module_functions) = self.host_functions.get(module_name) {
            if let Some(handler) = module_functions.get(function_name) {
                return handler.call(engine, args);
            }
        }

        Err(Error::new(kinds::ExecutionError(format!(
            "Host function {module_name}.{function_name} not found"
        ))))
    }

    /// Get all registered module names
    #[must_use]
    pub fn get_registered_modules(&self) -> Vec<&String> {
        self.host_functions.keys().collect()
    }

    /// Get all registered function names for a module
    #[must_use]
    pub fn get_registered_functions(&self, module_name: &str) -> Vec<&String> {
        if let Some(module_functions) = self.host_functions.get(module_name) {
            module_functions.keys().collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::CloneableFn;

    #[test]
    fn test_callback_registry() {
        let mut registry = CallbackRegistry::new();

        // Register a host function
        registry.register_host_function(
            "test_module",
            "test_function",
            CloneableFn::new(|_| Ok(vec![Value::I32(42)])),
        );

        // Check if it was registered
        assert!(registry.has_host_function("test_module", "test_function"));
        assert!(!registry.has_host_function("test_module", "nonexistent"));
        assert!(!registry.has_host_function("nonexistent", "test_function"));

        // Test calling the function
        let mut target = ();
        let result =
            registry.call_host_function(&mut target, "test_module", "test_function", vec![]);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![Value::I32(42)]);

        // Test calling a nonexistent function
        let result = registry.call_host_function(&mut target, "test_module", "nonexistent", vec![]);

        assert!(result.is_err());
    }

    #[test]
    fn test_callback_registry_callback() {
        let mut registry = CallbackRegistry::new();

        // Register a callback
        registry.register_callback(CallbackType::Logging, 42);

        // Retrieve the callback
        let callback = registry.get_callback::<i32>(&CallbackType::Logging);
        assert!(callback.is_some());
        assert_eq!(*callback.unwrap(), 42);

        // Modify the callback
        if let Some(callback) = registry.get_callback_mut::<i32>(&CallbackType::Logging) {
            *callback = 84;
        }

        // Check that it was modified
        let callback = registry.get_callback::<i32>(&CallbackType::Logging);
        assert!(callback.is_some());
        assert_eq!(*callback.unwrap(), 84);
    }
}
