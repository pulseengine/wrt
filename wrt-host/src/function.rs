// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Host function infrastructure.
//!
//! This module provides the core infrastructure for host functions
//! that can be called from WebAssembly components.

// Use the prelude for consistent imports
use crate::prelude::*;

/// A trait for functions that can be cloned and operate on `Vec<Value>`.
/// This is used for storing host functions that can be called by the Wasm
/// engine.
pub trait FnWithVecValue: Send + Sync {
    /// Calls the function with the given target and arguments.
    fn call(&self, target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>>;

    /// Clones the function into a `Box`.
    fn clone_box(&self) -> Box<dyn FnWithVecValue>;
}

impl<F> FnWithVecValue for F
where
    F: Fn(&mut dyn Any) -> Result<Vec<Value>> + Send + Sync + Clone + 'static,
{
    fn call(&self, target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
        // Using target but ignoring args since the function only takes target
        // This could be extended in the future to support functions that take args
        self(target)
    }

    fn clone_box(&self) -> Box<dyn FnWithVecValue> {
        Box::new(self.clone())
    }
}

/// A wrapper struct that makes a closure implementing `Fn` cloneable
/// by boxing it and handling the cloning via the `FnWithVecValue` trait.
pub struct CloneableFn(Box<dyn FnWithVecValue>);

impl CloneableFn {
    /// Creates a new `CloneableFn` from a closure.
    ///
    /// The closure must be `Send`, `Sync`, `Clone`, and `'static`.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut dyn Any) -> Result<Vec<Value>> + Send + Sync + Clone + 'static,
    {
        Self(Box::new(f))
    }

    /// Calls the wrapped function.
    pub fn call(&self, target: &mut dyn Any, args: Vec<Value>) -> Result<Vec<Value>> {
        self.0.call(target, args)
    }
}

impl Clone for CloneableFn {
    fn clone(&self) -> Self {
        Self(self.0.clone_box())
    }
}

/// Host function handler type for implementing WebAssembly imports
pub type HostFunctionHandler = CloneableFn;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloneable_fn() {
        let f = CloneableFn::new(|_| Ok(vec![Value::I32(42)]));
        let f2 = f.clone();

        let mut target = ();
        let result = f.call(&mut target, vec![]);
        let result2 = f2.call(&mut target, vec![]);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![Value::I32(42)]);

        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), vec![Value::I32(42)]);
    }

    #[test]
    fn test_host_function_handler() {
        let handler = HostFunctionHandler::new(|_| Ok(vec![Value::I32(42)]));

        let mut target = ();
        let result = handler.call(&mut target, vec![]);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![Value::I32(42)]);
    }
}
