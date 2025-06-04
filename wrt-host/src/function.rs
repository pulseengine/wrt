// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Host function infrastructure.
//!
//! This module provides the core infrastructure for host functions
//! that can be called from WebAssembly components.

// Use the prelude for consistent imports
use crate::prelude::*;

// Value vectors for function parameters/returns
#[cfg(any(feature = "std", feature = "alloc"))]
type ValueVec = Vec<Value>;

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
type ValueVec = wrt_foundation::BoundedVec<Value, 16, wrt_foundation::NoStdProvider<512>>;

/// A trait for functions that can be cloned and operate on value vectors.
/// This is used for storing host functions that can be called by the Wasm
/// engine.
#[cfg(any(feature = "std", feature = "alloc"))]
pub trait FnWithVecValue: Send + Sync {
    /// Calls the function with the given target and arguments.
    fn call(&self, target: &mut dyn Any, args: ValueVec) -> Result<ValueVec>;

    /// Clones the function into a `Box`.
    fn clone_box(&self) -> Box<dyn FnWithVecValue>;
}

/// Simplified trait for no_std environments without dynamic dispatch
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub trait FnWithVecValue: Send + Sync {
    /// Calls the function with the given target and arguments.
    fn call(&self, target: &mut dyn Any, args: ValueVec) -> Result<ValueVec>;
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<F> FnWithVecValue for F
where
    F: Fn(&mut dyn Any) -> Result<ValueVec> + Send + Sync + Clone + 'static,
{
    fn call(&self, target: &mut dyn Any, _args: ValueVec) -> Result<ValueVec> {
        // Using target but ignoring args since the function only takes target
        // This could be extended in the future to support functions that take args
        self(target)
    }

    fn clone_box(&self) -> Box<dyn FnWithVecValue> {
        Box::new(self.clone())
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<F> FnWithVecValue for F
where
    F: Fn(&mut dyn Any) -> Result<ValueVec> + Send + Sync + Clone + 'static,
{
    fn call(&self, target: &mut dyn Any, _args: ValueVec) -> Result<ValueVec> {
        // Using target but ignoring args since the function only takes target
        // This could be extended in the future to support functions that take args
        self(target)
    }
}

/// A wrapper struct that makes a closure implementing `Fn` cloneable
/// by boxing it and handling the cloning via the `FnWithVecValue` trait.
#[cfg(any(feature = "std", feature = "alloc"))]
pub struct CloneableFn(Box<dyn FnWithVecValue>);

/// Simplified function wrapper for no_std environments
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub struct CloneableFn;

#[cfg(any(feature = "std", feature = "alloc"))]
impl CloneableFn {
    /// Creates a new `CloneableFn` from a closure.
    ///
    /// The closure must be `Send`, `Sync`, `Clone`, and `'static`.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut dyn Any) -> Result<ValueVec> + Send + Sync + Clone + 'static,
    {
        Self(Box::new(f))
    }

    /// Calls the wrapped function.
    pub fn call(&self, target: &mut dyn Any, args: ValueVec) -> Result<ValueVec> {
        self.0.call(target, args)
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl CloneableFn {
    /// Creates a new `CloneableFn` from a closure.
    ///
    /// In no_std mode, this is a no-op since we can't store dynamic functions.
    pub fn new<F>(_f: F) -> Self
    where
        F: Fn(&mut dyn Any) -> Result<ValueVec> + Send + Sync + Clone + 'static,
    {
        Self
    }

    /// Calls the wrapped function.
    ///
    /// In no_std mode, this always returns an error since we can't store dynamic functions.
    pub fn call(&self, _target: &mut dyn Any, _args: ValueVec) -> Result<ValueVec> {
        Err(Error::new(
            ErrorCategory::Runtime,
            wrt_error::codes::NOT_IMPLEMENTED,
            "Dynamic function calls not supported in pure no_std mode"
        ))
    }
}

impl Clone for CloneableFn {
    fn clone(&self) -> Self {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            Self(self.0.clone_box())
        }
        
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        {
            // In no_std mode, create a default function
            CloneableFn::default()
        }
    }
}

impl PartialEq for CloneableFn {
    fn eq(&self, _other: &Self) -> bool {
        // Function pointers can't be meaningfully compared
        false
    }
}

impl Eq for CloneableFn {}

/// Host function handler type for implementing WebAssembly imports
pub type HostFunctionHandler = CloneableFn;

// Implement required traits for CloneableFn to work with BoundedMap in no_std mode
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl wrt_foundation::traits::Checksummable for CloneableFn {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        // Function pointers can't be meaningfully checksummed, use a placeholder
        checksum.update_slice(b"cloneable_fn");
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl wrt_foundation::traits::ToBytes for CloneableFn {
    fn serialized_size(&self) -> usize {
        // Function pointers can't be serialized, return 0
        0
    }

    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        _writer: &mut wrt_foundation::traits::WriteStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<()> {
        // Function pointers can't be serialized
        Ok(())
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl wrt_foundation::traits::FromBytes for CloneableFn {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        _reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        // Function pointers can't be deserialized, return a dummy function
        Ok(CloneableFn::new(|_| Err(wrt_foundation::Error::new(
            wrt_error::ErrorCategory::Runtime,
            wrt_error::codes::RUNTIME_ERROR,
            "Deserialized function not implemented",
        ))))
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl Default for CloneableFn {
    fn default() -> Self {
        CloneableFn::new(|_| Err(wrt_foundation::Error::new(
            wrt_error::ErrorCategory::Runtime,
            wrt_error::codes::RUNTIME_ERROR,
            "Default function not implemented",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloneable_fn() {
        let f = CloneableFn::new(|_| {
            #[cfg(any(feature = "std", feature = "alloc"))]
            return Ok(vec![Value::I32(42)]);
            
            #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
            {
                let provider = wrt_foundation::NoStdProvider::default();
                let mut vec = ValueVec::new(provider).unwrap();
                vec.push(Value::I32(42)).unwrap();
                Ok(vec)
            }
        });
        let f2 = f.clone();

        let mut target = ();
        
        #[cfg(any(feature = "std", feature = "alloc"))]
        let empty_args = vec![];
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        let empty_args = {
            let provider = wrt_foundation::NoStdProvider::default();
            ValueVec::new(provider).unwrap()
        };
        
        let result = f.call(&mut target, empty_args.clone());
        let result2 = f2.call(&mut target, empty_args);

        assert!(result.is_ok());
        let result_vec = result.unwrap();
        assert_eq!(result_vec.len(), 1);
        assert!(matches!(result_vec[0], Value::I32(42)));

        assert!(result2.is_ok());
        let result2_vec = result2.unwrap();
        assert_eq!(result2_vec.len(), 1);
        assert!(matches!(result2_vec[0], Value::I32(42)));
    }

    #[test]
    fn test_host_function_handler() {
        let handler = HostFunctionHandler::new(|_| {
            #[cfg(any(feature = "std", feature = "alloc"))]
            return Ok(vec![Value::I32(42)]);
            
            #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
            {
                let provider = wrt_foundation::NoStdProvider::default();
                let mut vec = ValueVec::new(provider).unwrap();
                vec.push(Value::I32(42)).unwrap();
                Ok(vec)
            }
        });

        let mut target = ();
        
        #[cfg(any(feature = "std", feature = "alloc"))]
        let empty_args = vec![];
        #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
        let empty_args = {
            let provider = wrt_foundation::NoStdProvider::default();
            ValueVec::new(provider).unwrap()
        };
        
        let result = handler.call(&mut target, empty_args);

        assert!(result.is_ok());
        let result_vec = result.unwrap();
        assert_eq!(result_vec.len(), 1);
        assert!(matches!(result_vec[0], Value::I32(42)));
    }
}
