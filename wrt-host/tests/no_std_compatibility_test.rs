// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Test no_std compatibility for wrt-host
//!
//! This file validates that the wrt-host crate works correctly in no_std
//! environments.

// For testing in a no_std environment
#![cfg_attr(not(feature = "std"), no_std)]

// External crate imports
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(test)]
mod tests {
    // Import necessary types for no_std environment
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
    #[cfg(feature = "std")]
    use std::{boxed::Box, string::String, vec, vec::Vec};

    // Import from wrt-foundation and wrt-error
    use wrt_error::{codes, Error, ErrorCategory, Result};
    use wrt_foundation::values::Value;
    // Import from wrt-host
    use wrt_host::{
        builder::HostBuilder,
        callback::{CallbackRegistry, CallbackType},
        function::{CloneableFn, HostFunctionHandler},
        host::BuiltinHost,
    };

    // Test host function
    fn test_host_function(params: &[Value]) -> Result<Value> {
        // Simple function that adds two i32 parameters
        if params.len() != 2 {
            return Err(Error::new(
                ErrorCategory::Core,
                codes::INVALID_ARGUMENT_COUNT,
                format!("Expected 2 arguments, got {}", params.len()),
            ));
        }

        if let (Value::I32(a), Value::I32(b)) = (&params[0], &params[1]) {
            Ok(Value::I32(a + b))
        } else {
            Err(Error::new(
                ErrorCategory::Core,
                codes::INVALID_ARGUMENT_TYPE,
                "Expected two i32 arguments".to_string(),
            ))
        }
    }

    #[test]
    fn test_host_builder() {
        // Create a host builder
        let mut builder = HostBuilder::new();

        // Add a host function
        builder.add_function("test_add", test_host_function);

        // Build the host
        let host = builder.build();

        // Verify the host has the function
        assert!(host.has_function("test_add"));
    }

    #[test]
    fn test_function_call() {
        // Create a host with a function
        let mut builder = HostBuilder::new();
        builder.add_function("test_add", test_host_function);
        let host = builder.build();

        // Call the function
        let params = vec![Value::I32(5), Value::I32(3)];
        let result = host.call_function("test_add", &params).unwrap();

        // Verify the result
        assert_eq!(result, Value::I32(8));
    }

    #[test]
    fn test_callback_registry() {
        // Create a callback registry
        let mut registry = CallbackRegistry::new();

        // Create a callback function
        let callback: Box<dyn FnMut() -> Result<()>> = Box::new(|| Ok(()));

        // Register the callback
        registry.register(CallbackType::BeforeInit, callback);

        // Verify the registry has the callback
        assert!(registry.has_callback(CallbackType::BeforeInit));
        assert!(!registry.has_callback(CallbackType::AfterInit));
    }

    #[test]
    fn test_cloneable_fn() {
        // Create a cloneable function
        let func: CloneableFn<dyn Fn(&[Value]) -> Result<Value>> =
            CloneableFn::new(|params: &[Value]| {
                if params.len() == 1 {
                    if let Value::I32(val) = params[0] {
                        return Ok(Value::I32(val * 2));
                    }
                }

                Err(Error::new(
                    ErrorCategory::Core,
                    codes::INVALID_ARGUMENT_TYPE,
                    "Expected one i32 argument".to_string(),
                ))
            });

        // Call the function
        let params = vec![Value::I32(5)];
        let result = func.call(&params).unwrap();

        // Verify the result
        assert_eq!(result, Value::I32(10));
    }
}
