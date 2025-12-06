//! Canonical Function Executor for WebAssembly Components
//!
//! This module provides execution of canonical functions (canon.lift and canon.lower)
//! and integrates with the wasip2 host implementation for WASI preview2 support.

extern crate alloc;

use crate::bounded_component_infra::ComponentProvider;
use crate::canonical_abi::ComponentValue;
use wrt_error::{Error, Result};
use wrt_runtime::wasip2_host::Wasip2Host;
use wrt_foundation::float_repr::{FloatBits32, FloatBits64};
use alloc::vec::Vec;
use alloc::string::String;

/// Canonical function types
#[derive(Debug, Clone)]
pub enum CanonicalFunction {
    /// Canon.lift - exports a component function by lifting core WASM function
    Lift {
        func_idx: u32,
        opts: CanonicalOptions,
    },
    /// Canon.lower - imports a host function by lowering to core WASM
    Lower {
        func_idx: u32,
        interface: String,
        function: String,
        opts: CanonicalOptions,
    },
}

/// Canonical ABI options
#[derive(Debug, Clone, Default)]
pub struct CanonicalOptions {
    pub memory: Option<u32>,
    pub realloc: Option<u32>,
    pub post_return: Option<u32>,
    pub string_encoding: StringEncoding,
}

/// String encoding for canonical ABI
#[derive(Debug, Clone)]
pub enum StringEncoding {
    Utf8,
    Utf16,
    Latin1,
}

impl Default for StringEncoding {
    fn default() -> Self {
        StringEncoding::Utf8
    }
}

/// Executor for canonical functions with wasip2 support
pub struct CanonicalExecutor {
    /// WASI Preview2 host implementation
    wasip2: Wasip2Host,
    /// Registered canonical functions
    canonicals: Vec<CanonicalFunction>,
}

impl CanonicalExecutor {
    pub fn new() -> Self {
        Self {
            wasip2: Wasip2Host::new(),
            canonicals: Vec::new(),
        }
    }

    /// Register a canonical function
    pub fn register_canonical(&mut self, func: CanonicalFunction) -> u32 {
        let idx = self.canonicals.len() as u32;
        self.canonicals.push(func);
        idx
    }

    /// Execute a canon.lower import (host function call)
    pub fn execute_canon_lower(
        &mut self,
        interface: &str,
        function: &str,
        args: Vec<ComponentValue>,
        memory: Option<&mut [u8]>,
    ) -> Result<Vec<ComponentValue>> {
        eprintln!("[CANON_EXECUTOR] Executing canon.lower: {}::{}", interface, function);

        // Convert ComponentValue to wasip2 Value format
        let wasip2_args = self.convert_to_wasip2_values(args)?;

        // Dispatch to wasip2 host
        let wasip2_results = self.wasip2.dispatch(interface, function, wasip2_args, memory)?;

        // Convert results back to ComponentValue
        let results = self.convert_from_wasip2_values(wasip2_results)?;

        eprintln!("[CANON_EXECUTOR] Canon.lower completed with {} results", results.len());
        Ok(results)
    }

    /// Execute a canon.lift export (component function export)
    pub fn execute_canon_lift(
        &mut self,
        func_idx: u32,
        args: Vec<ComponentValue>,
        _opts: &CanonicalOptions,
    ) -> Result<Vec<ComponentValue>> {
        eprintln!("[CANON_EXECUTOR] Executing canon.lift: func_idx={}", func_idx);

        // Canon.lift would call the underlying core WASM function
        // This requires integration with the core WASM engine
        // For now, return a placeholder

        Err(Error::runtime_not_implemented("Canon.lift execution not yet implemented"))
    }

    /// Convert ComponentValue to wasip2 Value
    fn convert_to_wasip2_values(&self, values: Vec<ComponentValue>) -> Result<Vec<wrt_foundation::values::Value>> {
        use wrt_foundation::values::Value;

        let mut result = Vec::new();
        for val in values {
            let wasip2_val = match val {
                ComponentValue::Bool(b) => Value::I32(if b { 1 } else { 0 }),
                ComponentValue::S8(i) => Value::I32(i as i32),
                ComponentValue::U8(u) => Value::I32(u as i32),
                ComponentValue::S16(i) => Value::I32(i as i32),
                ComponentValue::U16(u) => Value::I32(u as i32),
                ComponentValue::S32(i) => Value::I32(i),
                ComponentValue::U32(u) => Value::I32(u as i32),
                ComponentValue::S64(i) => Value::I64(i),
                ComponentValue::U64(u) => Value::I64(u as i64),
                ComponentValue::F32(f) => Value::F32(FloatBits32::from_f32(f)),
                ComponentValue::F64(f) => Value::F64(FloatBits64::from_f64(f)),
                _ => {
                    eprintln!("[CANON_EXECUTOR] Unsupported ComponentValue type for conversion");
                    return Err(Error::runtime_error("Unsupported value type"));
                }
            };
            result.push(wasip2_val);
        }
        Ok(result)
    }

    /// Convert wasip2 Value to ComponentValue
    fn convert_from_wasip2_values(&self, values: Vec<wrt_foundation::values::Value>) -> Result<Vec<ComponentValue>> {
        use wrt_foundation::values::Value;

        let mut result = Vec::new();
        for val in values {
            let comp_val = match val {
                Value::I32(i) => ComponentValue::S32(i),
                Value::I64(i) => ComponentValue::S64(i),
                Value::F32(f) => ComponentValue::F32(f.to_f32()),
                Value::F64(f) => ComponentValue::F64(f.to_f64()),
                _ => {
                    eprintln!("[CANON_EXECUTOR] Unsupported Value type for conversion");
                    return Err(Error::runtime_error("Unsupported value type"));
                }
            };
            result.push(comp_val);
        }
        Ok(result)
    }
}

/// Create a global canonical executor instance
#[cfg(feature = "std")]
pub fn create_canonical_executor() -> CanonicalExecutor {
    CanonicalExecutor::new()
}

/// Check if a function name represents a wasip2 canonical import
pub fn is_wasip2_canonical(name: &str) -> bool {
    name.starts_with("wasi:") && name.contains("@0.2")
}