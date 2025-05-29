// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Built-in function host implementation for WebAssembly components.
//!
//! This module provides the functionality for executing built-in functions
//! as defined in the WebAssembly Component Model.

// Use the prelude for consistent imports
use crate::prelude::*;

// Type aliases for no_std compatibility
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
type HostString = wrt_foundation::bounded::BoundedString<256, wrt_foundation::NoStdProvider<256>>;

#[cfg(any(feature = "std", feature = "alloc"))]
type HostString = String;

// Value vectors for function parameters/returns
#[cfg(any(feature = "std", feature = "alloc"))]
type ValueVec = Vec<Value>;

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
type ValueVec = wrt_foundation::BoundedVec<Value, 16, wrt_foundation::NoStdProvider<512>>;

// Handler function type alias
#[cfg(any(feature = "std", feature = "alloc"))]
type HandlerFn = Box<dyn Fn(&mut dyn Any, ValueVec) -> Result<ValueVec> + Send + Sync>;

// Handler data wrapper for no_std
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HandlerData {
    _phantom: core::marker::PhantomData<()>,
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl wrt_foundation::traits::Checksummable for HandlerData {
    fn update_checksum(&self, _checksum: &mut wrt_foundation::verification::Checksum) {
        // HandlerData has no content to checksum
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl wrt_foundation::traits::ToBytes for HandlerData {
    fn serialized_size(&self) -> usize {
        0
    }

    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        _writer: &mut wrt_foundation::traits::WriteStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<()> {
        Ok(())
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl wrt_foundation::traits::FromBytes for HandlerData {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        _reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        Ok(HandlerData::default())
    }
}

// Handler map type for different configurations
#[cfg(any(feature = "std", feature = "alloc"))]
type HandlerMap = HashMap<String, HandlerFn>;

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
type HandlerMap = HashMap<HostString, HandlerData, 32, wrt_foundation::NoStdProvider<1024>>;

// Critical builtins map type for different configurations
#[cfg(any(feature = "std", feature = "alloc"))]
type CriticalBuiltinsMap = HashMap<BuiltinType, HandlerFn>;

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
type CriticalBuiltinsMap = HashMap<BuiltinType, HandlerData, 32, wrt_foundation::NoStdProvider<1024>>;

/// Converts wrt_foundation::values::Value to
/// wrt_foundation::component_value::ComponentValue
///
/// This function converts WebAssembly core values to Component Model values
/// with support for both std and no_std environments.
#[cfg(any(feature = "std", feature = "alloc"))]
fn convert_to_component_values(
    values: &[Value],
) -> Vec<ComponentValue<wrt_foundation::NoStdProvider<64>>> {
    values
        .iter()
        .map(|v| match v {
            Value::I32(i) => ComponentValue::S32(*i),
            Value::I64(i) => ComponentValue::S64(*i),
            Value::F32(f) => ComponentValue::F32(wrt_foundation::FloatBits32(f.to_bits())),
            Value::F64(f) => ComponentValue::F64(wrt_foundation::FloatBits64(f.to_bits())),
            // Add other conversions as needed
            _ => ComponentValue::S32(0), // Default fallback
        })
        .collect()
}

/// Converts wrt_foundation::component_value::ComponentValue to
/// wrt_foundation::values::Value
///
/// This function converts Component Model values to WebAssembly core values
/// with support for both std and no_std environments.
#[cfg(any(feature = "std", feature = "alloc"))]
fn convert_from_component_values(
    values: &[ComponentValue<wrt_foundation::NoStdProvider<64>>],
) -> ValueVec {
    values
        .iter()
        .map(|v| match v {
            ComponentValue::S8(i) => Value::I32(*i as i32),
            ComponentValue::S16(i) => Value::I32(*i as i32),
            ComponentValue::S32(i) => Value::I32(*i),
            ComponentValue::S64(i) => Value::I64(*i),
            ComponentValue::U8(i) => Value::I32(*i as i32),
            ComponentValue::U16(i) => Value::I32(*i as i32),
            ComponentValue::U32(i) => Value::I32(*i as i32),
            ComponentValue::U64(i) => Value::I64(*i as i64),
            ComponentValue::F32(f) => Value::F32(*f),
            ComponentValue::F64(f) => Value::F64(*f),
            // Add other conversions as needed
            _ => Value::I32(0), // Default fallback
        })
        .collect()
}

/// WebAssembly built-in function host implementation
pub struct BuiltinHost {
    /// Component name
    component_name: HostString,
    /// Host ID
    host_id: HostString,
    /// Interceptor for built-in calls
    #[cfg(any(feature = "std", feature = "alloc"))]
    interceptor: Option<Arc<dyn BuiltinInterceptor>>,
    /// Built-in handlers (builtin_type_name -> handler)
    handlers: HandlerMap,
    /// Critical built-ins that should have fallbacks
    critical_builtins: CriticalBuiltinsMap,
}

impl Default for BuiltinHost {
    fn default() -> Self {
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        {
            Self {
                component_name: HostString::from_str("", wrt_foundation::NoStdProvider::<256>::default())
                    .expect("Failed to create empty string"),
                host_id: HostString::from_str("", wrt_foundation::NoStdProvider::<256>::default())
                    .expect("Failed to create empty string"),
                handlers: HandlerMap::new(wrt_foundation::NoStdProvider::<1024>::default())
                    .expect("Failed to create HandlerMap"),
                critical_builtins: CriticalBuiltinsMap::new(wrt_foundation::NoStdProvider::<1024>::default())
                    .expect("Failed to create CriticalBuiltinsMap"),
            }
        }
        
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            Self {
                component_name: HostString::default(),
                host_id: HostString::default(),
                interceptor: None,
                handlers: HandlerMap::default(),
                critical_builtins: CriticalBuiltinsMap::default(),
            }
        }
    }
}

impl BuiltinHost {
    /// Create a new built-in host
    ///
    /// # Arguments
    ///
    /// * `component_name` - The name of the component
    /// * `host_id` - The host identifier
    ///
    /// # Returns
    ///
    /// A new `BuiltinHost` instance
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn new(component_name: &str, host_id: &str) -> Self {
        Self {
            component_name: component_name.to_string(),
            host_id: host_id.to_string(),
            interceptor: None,
            handlers: HashMap::new(),
            critical_builtins: HashMap::new(),
        }
    }

    /// Create a new built-in host (no_std version)
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    pub fn new(component_name: &str, host_id: &str) -> Self {
        let string_provider = wrt_foundation::NoStdProvider::<256>::default();
        let map_provider = wrt_foundation::NoStdProvider::<1024>::default();
        let comp_name = HostString::from_str(component_name, string_provider.clone())
            .expect("Failed to create component name");
        let host_name = HostString::from_str(host_id, string_provider)
            .expect("Failed to create host id");
        
        Self {
            component_name: comp_name,
            host_id: host_name,
            handlers: HashMap::new(map_provider.clone()).unwrap(),
            critical_builtins: HashMap::new(map_provider).unwrap(),
        }
    }

    /// Set the interceptor for built-in calls
    ///
    /// # Arguments
    ///
    /// * `interceptor` - The interceptor to use
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn set_interceptor(&mut self, interceptor: Arc<dyn BuiltinInterceptor>) {
        self.interceptor = Some(interceptor);
    }

    /// Register a handler for a built-in function
    ///
    /// # Arguments
    ///
    /// * `builtin_type` - The built-in type
    /// * `handler` - The handler function
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn register_handler<F>(&mut self, builtin_type: BuiltinType, handler: F)
    where
        F: Fn(&mut dyn Any, ValueVec) -> Result<ValueVec> + Send + Sync + 'static,
    {
        self.handlers.insert(builtin_type.name().to_string(), Box::new(handler));
    }

    /// Register a handler for a built-in function (no_std version)
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    pub fn register_handler<F>(&mut self, builtin_type: BuiltinType, _handler: F)
    where
        F: Fn(&mut dyn Any, ValueVec) -> Result<ValueVec> + Send + Sync + 'static,
    {
        // In no_std mode, we can't store function handlers dynamically
        let name = HostString::from_str(builtin_type.name(), wrt_foundation::NoStdProvider::<256>::default())
            .expect("Failed to create builtin name");
        let _ = self.handlers.insert(name, HandlerData::default());
    }

    /// Register a fallback for a critical built-in function
    ///
    /// # Arguments
    ///
    /// * `builtin_type` - The built-in type
    /// * `handler` - The fallback handler function
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn register_fallback<F>(&mut self, builtin_type: BuiltinType, handler: F)
    where
        F: Fn(&mut dyn Any, ValueVec) -> Result<ValueVec> + Send + Sync + 'static,
    {
        self.critical_builtins.insert(builtin_type, Box::new(handler));
    }

    /// Register a fallback for a critical built-in function (no_std version)
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    pub fn register_fallback<F>(&mut self, builtin_type: BuiltinType, _handler: F)
    where
        F: Fn(&mut dyn Any, ValueVec) -> Result<ValueVec> + Send + Sync + 'static,
    {
        // In no_std mode, we can't store function handlers dynamically
        let _ = self.critical_builtins.insert(builtin_type, HandlerData::default());
    }

    /// Check if a built-in type is implemented
    ///
    /// # Arguments
    ///
    /// * `builtin_type` - The built-in type to check
    ///
    /// # Returns
    ///
    /// `true` if the built-in is implemented, `false` otherwise
    pub fn is_implemented(&self, builtin_type: BuiltinType) -> bool {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            self.handlers.contains_key(builtin_type.name())
        }
        
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        {
            // In no_std mode, check if we have any handlers registered
            let name = HostString::from_str(builtin_type.name(), wrt_foundation::NoStdProvider::<256>::default())
                .expect("Failed to create builtin name");
            self.handlers.contains_key(&name).unwrap_or(false)
        }
    }

    /// Check if a built-in type has a fallback
    ///
    /// # Arguments
    ///
    /// * `builtin_type` - The built-in type to check
    ///
    /// # Returns
    ///
    /// `true` if the built-in has a fallback, `false` otherwise
    pub fn has_fallback(&self, builtin_type: BuiltinType) -> bool {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            self.critical_builtins.contains_key(&builtin_type)
        }
        
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        {
            self.critical_builtins.contains_key(&builtin_type).unwrap_or(false)
        }
    }

    /// Call a built-in function
    ///
    /// # Arguments
    ///
    /// * `engine` - The engine context
    /// * `builtin_type` - The built-in type to call
    /// * `args` - The arguments to the function
    ///
    /// # Returns
    ///
    /// A `Result` containing the function results or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the built-in is not implemented or fails during
    /// execution
    pub fn call_builtin(
        &self,
        engine: &mut dyn Any,
        builtin_type: BuiltinType,
        args: ValueVec,
    ) -> Result<ValueVec> {
        // Apply interception if available and alloc is enabled
        #[cfg(any(feature = "std", feature = "alloc"))]
        if let Some(interceptor) = &self.interceptor {
            let context = InterceptContext::new(&self.component_name, builtin_type, &self.host_id);
            let component_args = convert_to_component_values(&args);

            // Before interceptor
            match interceptor.before_builtin(&context, &component_args)? {
                BeforeBuiltinResult::Continue(modified_args) => {
                    // Convert the modified args back to regular values
                    let modified_values = convert_from_component_values(&modified_args);

                    // Execute with potentially modified args
                    let result =
                        self.execute_builtin_internal(engine, builtin_type, modified_values);

                    // After interceptor - convert result to component values and back
                    let component_result = match &result {
                        Ok(values) => Ok(convert_to_component_values(values)),
                        Err(_e) => Err(Error::new(
                            ErrorCategory::Runtime,
                            codes::RUNTIME_ERROR,
                            "Runtime error during interception",
                        )),
                    };

                    let modified_result =
                        interceptor.after_builtin(&context, &component_args, component_result)?;
                    Ok(convert_from_component_values(&modified_result))
                }
                BeforeBuiltinResult::Bypass(result) => {
                    // Skip execution and use provided result
                    Ok(convert_from_component_values(&result))
                }
            }
        } else {
            // No interceptor, just execute
            self.execute_builtin_internal(engine, builtin_type, args)
        }
        
        // For no_std without alloc, interception is not available
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            self.execute_builtin_internal(engine, builtin_type, args)
        }
    }

    /// Internal implementation of execute_builtin without interception
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn execute_builtin_internal(
        &self,
        engine: &mut dyn Any,
        builtin_type: BuiltinType,
        args: ValueVec,
    ) -> Result<ValueVec> {
        let builtin_name = builtin_type.name();

        // Try to find the handler
        if let Some(handler) = self.handlers.get(builtin_name) {
            return handler(engine, args);
        }

        // Try to use a fallback for critical built-ins
        if let Some(fallback) = self.critical_builtins.get(&builtin_type) {
            return fallback(engine, args);
        }
        
        // No handler or fallback found
        Err(Error::new(
            ErrorCategory::Runtime,
            codes::RUNTIME_ERROR,
            "Built-in function not implemented",
        ))
    }

    /// Internal implementation of execute_builtin without interception (no_std version)
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    fn execute_builtin_internal(
        &self,
        _engine: &mut dyn Any,
        _builtin_type: BuiltinType,
        _args: ValueVec,
    ) -> Result<ValueVec> {
        // In no_std mode, built-in functions are not supported
        Err(Error::new(
            ErrorCategory::Runtime,
            codes::RUNTIME_ERROR,
            "Built-in functions not supported in no_std mode",
        ))
    }
}

impl Clone for BuiltinHost {
    fn clone(&self) -> Self {
        // This is a simplified clone that doesn't actually clone the handlers
        // In a real implementation, you would need to properly clone all handlers
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            Self {
                component_name: self.component_name.clone(),
                host_id: self.host_id.clone(),
                interceptor: self.interceptor.clone(),
                handlers: HashMap::new(),
                critical_builtins: HashMap::new(),
            }
        }
        
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        {
            let provider = wrt_foundation::NoStdProvider::default();
            Self {
                component_name: self.component_name.clone(),
                host_id: self.host_id.clone(),
                handlers: HashMap::new(provider.clone()).unwrap(),
                critical_builtins: HashMap::new(provider).unwrap(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use wrt_foundation::values::Value;

    use super::*;

    #[test]
    fn test_builtin_host_basics() {
        let host = BuiltinHost::new("test-component", "test-host");

        assert!(!host.is_implemented(BuiltinType::ResourceCreate));
        assert!(!host.has_fallback(BuiltinType::ResourceCreate));
    }

    #[test]
    fn test_register_handler() {
        let mut host = BuiltinHost::new("test-component", "test-host");

        host.register_handler(BuiltinType::ResourceCreate, |_, _| Ok(vec![Value::I32(42)]));

        assert!(host.is_implemented(BuiltinType::ResourceCreate));

        let mut engine = ();
        let result = host.call_builtin(&mut engine, BuiltinType::ResourceCreate, vec![]);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![Value::I32(42)]);
    }

    #[test]
    fn test_fallback_mechanism() {
        let mut host = BuiltinHost::new("test-component", "test-host");

        // Register a fallback for ResourceCreate
        host.register_fallback(BuiltinType::ResourceCreate, |_, _| Ok(vec![Value::I32(99)]));

        assert!(!host.is_implemented(BuiltinType::ResourceCreate));
        assert!(host.has_fallback(BuiltinType::ResourceCreate));

        let mut engine = ();
        let result = host.call_builtin(&mut engine, BuiltinType::ResourceCreate, vec![]);

        // Should use the fallback
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![Value::I32(99)]);

        // Now register a regular handler
        host.register_handler(BuiltinType::ResourceCreate, |_, _| Ok(vec![Value::I32(42)]));

        let result = host.call_builtin(&mut engine, BuiltinType::ResourceCreate, vec![]);

        // Should use the regular handler, not the fallback
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![Value::I32(42)]);
    }

    #[test]
    fn test_nonexistent_builtin() {
        let host = BuiltinHost::new("test-component", "test-host");

        let mut engine = ();
        let result = host.call_builtin(&mut engine, BuiltinType::ResourceCreate, vec![]);

        // Should fail because the built-in is not implemented
        assert!(result.is_err());
    }
}
