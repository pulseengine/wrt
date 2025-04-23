// Canonical ABI Implementation for WebAssembly Component Model
//
// This module provides the implementation of the Canonical ABI used
// in the WebAssembly Component Model to interface between components.

use crate::resources::{BufferPool, MemoryStrategy, ResourceTable, VerificationLevel};
use std::any::Any;
use std::sync::Arc;
use wrt_types::values::Value;
use wrt_error::{kinds, Error, Result};
use wrt_format::component::{ResourceOperation as FormatResourceOperation, ValType};
use wrt_intercept::{LinkInterceptor, LinkInterceptorStrategy};
use wrt_runtime::Memory;
use wrt_types::values::Value;

#[cfg(feature = "std")]
use std::{
    collections::HashMap,
    sync::{Mutex, RwLock},
    vec::Vec,
};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{
    collections::BTreeMap as HashMap,
    sync::{Arc, Mutex, RwLock},
    vec::Vec,
};

// Maximum allowed allocation size for safety
const MAX_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10MB

/// Canonical ABI implementation for the WebAssembly Component Model
#[derive(Debug)]
pub struct CanonicalABI {
    /// Buffer pool for temporary allocations
    buffer_pool: Arc<RwLock<BufferPool>>,
    /// Memory strategy for canonical operations
    memory_strategy: MemoryStrategy,
    /// Verification level for canonical operations
    verification_level: VerificationLevel,
    /// Optional interceptor for canonical operations
    interceptor: Option<Arc<LinkInterceptor>>,
    /// Metrics for canonical operations
    metrics: Arc<Mutex<CanonicalMetrics>>,
}

/// Metrics for canonical operations
#[derive(Debug, Default, Clone)]
pub struct CanonicalMetrics {
    /// Number of lift operations performed
    pub lift_count: u64,
    /// Number of lower operations performed
    pub lower_count: u64,
    /// Total bytes lifted
    pub lift_bytes: u64,
    /// Total bytes lowered
    pub lower_bytes: u64,
    /// Max bytes lifted in a single operation
    pub max_lift_bytes: u64,
    /// Max bytes lowered in a single operation
    pub max_lower_bytes: u64,
}

impl CanonicalABI {
    /// Create a new CanonicalABI instance
    pub fn new(buffer_pool_size: usize) -> Self {
        Self {
            buffer_pool: Arc::new(RwLock::new(BufferPool::new(buffer_pool_size))),
            memory_strategy: MemoryStrategy::BoundedCopy,
            verification_level: VerificationLevel::Critical,
            interceptor: None,
            metrics: Arc::new(Mutex::new(CanonicalMetrics::default())),
        }
    }

    /// Create a new CanonicalABI instance with default settings
    pub fn default() -> Self {
        Self::new(1024 * 1024) // 1MB default buffer pool
    }

    /// Set the memory strategy for canonical operations
    pub fn with_memory_strategy(mut self, strategy: MemoryStrategy) -> Self {
        self.memory_strategy = strategy;
        self
    }

    /// Set the verification level for canonical operations
    pub fn with_verification_level(mut self, level: VerificationLevel) -> Self {
        self.verification_level = level;
        self
    }

    /// Set the interceptor for canonical operations
    pub fn with_interceptor(mut self, interceptor: Arc<LinkInterceptor>) -> Self {
        self.interceptor = Some(interceptor);
        self
    }

    /// Lift a value from the WebAssembly memory into a Value
    pub fn lift(
        &self,
        ty: &ValType,
        addr: u32,
        resource_table: &ResourceTable,
        memory_bytes: &[u8],
    ) -> Result<Value> {
        // Get memory strategy from interceptor or use default
        let memory_strategy = self.get_strategy_from_interceptor();

        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.lift_count += 1;
        }

        // Intercept if necessary
        if let Some(interceptor) = &self.interceptor {
            for strategy in &interceptor.strategies {
                if strategy.should_intercept_canonical() {
                    if let Some(value) = strategy.intercept_lift(ty, addr, memory_bytes)? {
                        // Convert the strategy's result into a Value
                        // This is a placeholder - actual implementation would depend on the return format
                        return Ok(value); // Placeholder
                    }
                }
            }
        }

        // Perform the lift operation
        match ty {
            ValType::Bool => self.lift_bool(addr, memory_bytes),
            ValType::S8 => self.lift_s8(addr, memory_bytes),
            ValType::U8 => self.lift_u8(addr, memory_bytes),
            ValType::S16 => self.lift_s16(addr, memory_bytes),
            ValType::U16 => self.lift_u16(addr, memory_bytes),
            ValType::S32 => self.lift_s32(addr, memory_bytes),
            ValType::U32 => self.lift_u32(addr, memory_bytes),
            ValType::S64 => self.lift_s64(addr, memory_bytes),
            ValType::U64 => self.lift_u64(addr, memory_bytes),
            ValType::F32 => self.lift_f32(addr, memory_bytes),
            ValType::F64 => self.lift_f64(addr, memory_bytes),
            ValType::Char => self.lift_char(addr, memory_bytes),
            ValType::String => self.lift_string(addr, memory_bytes),
            ValType::List(inner_ty) => self.lift_list(inner_ty, addr, resource_table, memory_bytes),
            ValType::Record(fields) => self.lift_record(fields, addr, resource_table, memory_bytes),
            ValType::Variant(cases) => self.lift_variant(cases, addr, resource_table, memory_bytes),
            ValType::Enum(cases) => self.lift_enum(cases, addr, memory_bytes),
            ValType::Option(inner_ty) => {
                self.lift_option(inner_ty, addr, resource_table, memory_bytes)
            }
            ValType::Result(ok_ty) => {
                // Handle single-value result (ok only)
                self.lift_result(Some(ok_ty), None, addr, resource_table, memory_bytes)
            }
            ValType::ResultErr(err_ty) => {
                // Handle single-value result (err only)
                self.lift_result(None, Some(err_ty), addr, resource_table, memory_bytes)
            }
            ValType::ResultBoth(ok_ty, err_ty) => {
                // Handle dual-value result (ok and err)
                self.lift_result(
                    Some(ok_ty),
                    Some(err_ty),
                    addr,
                    resource_table,
                    memory_bytes,
                )
            }
            ValType::Tuple(types) => self.lift_tuple(types, addr, resource_table, memory_bytes),
            ValType::Flags(names) => self.lift_flags(names, addr, memory_bytes),
            ValType::Own(type_idx) => {
                self.lift_resource(*type_idx, addr, resource_table, memory_bytes)
            }
            ValType::Borrow(type_idx) => {
                self.lift_resource(*type_idx, addr, resource_table, memory_bytes)
            }
            _ => Err(Error::new(kinds::NotImplementedError(format!(
                "Lifting value of type {:?} not implemented",
                ty
            )))),
        }
    }

    /// Lower a Value into the WebAssembly memory
    pub fn lower(
        &self,
        value: &Value,
        addr: u32,
        resource_table: &ResourceTable,
        memory_bytes: &mut [u8],
    ) -> Result<()> {
        // Get memory strategy from interceptor or use default
        let memory_strategy = self.get_strategy_from_interceptor();

        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.lower_count += 1;
        }

        // Intercept if necessary
        if let Some(interceptor) = &self.interceptor {
            for strategy in &interceptor.strategies {
                if strategy.should_intercept_canonical() {
                    // In a real implementation, we'd serialize the value and pass it to the interceptor
                    // For now, this is a placeholder
                    let value_type = value.get_type();
                    let value_data: &[u8] = &[]; // Placeholder - actual implementation would serialize the value

                    // Convert value_type to FormatValType directly
                    if let Ok(format_val_type) = crate::type_conversion::value_type_to_format_val_type(&value_type) {
                        if strategy.intercept_lower(&format_val_type, value_data, addr, memory_bytes)? {
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Perform the lower operation
        match value {
            Value::Bool(b) => self.lower_bool(*b, addr, memory_bytes),
            Value::S8(v) => self.lower_s8(*v, addr, memory_bytes),
            Value::U8(v) => self.lower_u8(*v, addr, memory_bytes),
            Value::S16(v) => self.lower_s16(*v, addr, memory_bytes),
            Value::U16(v) => self.lower_u16(*v, addr, memory_bytes),
            Value::S32(v) => self.lower_s32(*v, addr, memory_bytes),
            Value::U32(v) => self.lower_u32(*v, addr, memory_bytes),
            Value::S64(v) => self.lower_s64(*v, addr, memory_bytes),
            Value::U64(v) => self.lower_u64(*v, addr, memory_bytes),
            Value::F32(v) => self.lower_f32(*v, addr, memory_bytes),
            Value::F64(v) => self.lower_f64(*v, addr, memory_bytes),
            Value::Char(c) => self.lower_char(*c, addr, memory_bytes),
            Value::String(s) => self.lower_string(s, addr, memory_bytes),
            Value::List(values) => self.lower_list(values, addr, resource_table, memory_bytes),
            Value::Record(fields) => self.lower_record(fields, addr, resource_table, memory_bytes),
            Value::Variant { case, value } => {
                self.lower_variant(*case, value, addr, resource_table, memory_bytes)
            }
            Value::Enum(idx) => self.lower_enum(*idx, addr, memory_bytes),
            Value::Option(value) => {
                self.lower_option(value.as_ref().map(|v| v.as_ref()), addr, resource_table, memory_bytes)
            }
            Value::Result(result) => self.lower_result(result, addr, resource_table, memory_bytes),
            Value::Tuple(values) => self.lower_tuple(values, addr, resource_table, memory_bytes),
            Value::Flags(flags) => self.lower_flags(flags, addr, memory_bytes),
            Value::Own(handle) => {
                self.lower_resource(*handle, addr, resource_table, memory_bytes)
            }
            Value::Borrow(handle) => {
                self.lower_resource(*handle, addr, resource_table, memory_bytes)
            }
            _ => Err(Error::new(kinds::NotImplementedError(format!(
                "Lowering value {:?} not implemented",
                value
            )))),
        }
    }

    // Primitive lifting operations
    fn lift_bool(&self, addr: u32, memory_bytes: &[u8]) -> Result<Value> {
        if (addr as usize) < memory_bytes.len() {
            let v = memory_bytes[addr as usize] != 0;
            Ok(Value::Bool(v))
        } else {
            Err(Error::new(kinds::OutOfBoundsAccess(format!(
                "Address {} out of bounds for memory of size {}",
                addr,
                memory_bytes.len()
            ))))
        }
    }

    fn lift_s8(&self, addr: u32, memory_bytes: &[u8]) -> Result<Value> {
        if (addr as usize) < memory_bytes.len() {
            let v = memory_bytes[addr as usize] as i8;
            Ok(Value::S8(v))
        } else {
            Err(Error::new(kinds::OutOfBoundsAccess(format!(
                "Address {} out of bounds for memory of size {}",
                addr,
                memory_bytes.len()
            ))))
        }
    }

    fn lift_u8(&self, addr: u32, memory_bytes: &[u8]) -> Result<Value> {
        if (addr as usize) < memory_bytes.len() {
            let v = memory_bytes[addr as usize];
            Ok(Value::U8(v))
        } else {
            Err(Error::new(kinds::OutOfBoundsAccess(format!(
                "Address {} out of bounds for memory of size {}",
                addr,
                memory_bytes.len()
            ))))
        }
    }

    fn lift_s16(&self, addr: u32, memory_bytes: &[u8]) -> Result<Value> {
        self.check_bounds(addr, 2, memory_bytes)?;
        let v = i16::from_le_bytes([memory_bytes[addr as usize], memory_bytes[addr as usize + 1]]);
        Ok(Value::S16(v))
    }

    fn lift_u16(&self, addr: u32, memory_bytes: &[u8]) -> Result<Value> {
        self.check_bounds(addr, 2, memory_bytes)?;
        let v = u16::from_le_bytes([memory_bytes[addr as usize], memory_bytes[addr as usize + 1]]);
        Ok(Value::U16(v))
    }

    fn lift_s32(&self, addr: u32, memory_bytes: &[u8]) -> Result<Value> {
        self.check_bounds(addr, 4, memory_bytes)?;
        let v = i32::from_le_bytes([
            memory_bytes[addr as usize],
            memory_bytes[addr as usize + 1],
            memory_bytes[addr as usize + 2],
            memory_bytes[addr as usize + 3],
        ]);
        Ok(Value::S32(v))
    }

    fn lift_u32(&self, addr: u32, memory_bytes: &[u8]) -> Result<Value> {
        self.check_bounds(addr, 4, memory_bytes)?;
        let v = u32::from_le_bytes([
            memory_bytes[addr as usize],
            memory_bytes[addr as usize + 1],
            memory_bytes[addr as usize + 2],
            memory_bytes[addr as usize + 3],
        ]);
        Ok(Value::U32(v))
    }

    fn lift_s64(&self, addr: u32, memory_bytes: &[u8]) -> Result<Value> {
        self.check_bounds(addr, 8, memory_bytes)?;
        let v = i64::from_le_bytes([
            memory_bytes[addr as usize],
            memory_bytes[addr as usize + 1],
            memory_bytes[addr as usize + 2],
            memory_bytes[addr as usize + 3],
            memory_bytes[addr as usize + 4],
            memory_bytes[addr as usize + 5],
            memory_bytes[addr as usize + 6],
            memory_bytes[addr as usize + 7],
        ]);
        Ok(Value::S64(v))
    }

    fn lift_u64(&self, addr: u32, memory_bytes: &[u8]) -> Result<Value> {
        self.check_bounds(addr, 8, memory_bytes)?;
        let v = u64::from_le_bytes([
            memory_bytes[addr as usize],
            memory_bytes[addr as usize + 1],
            memory_bytes[addr as usize + 2],
            memory_bytes[addr as usize + 3],
            memory_bytes[addr as usize + 4],
            memory_bytes[addr as usize + 5],
            memory_bytes[addr as usize + 6],
            memory_bytes[addr as usize + 7],
        ]);
        Ok(Value::U64(v))
    }

    fn lift_f32(&self, addr: u32, memory_bytes: &[u8]) -> Result<Value> {
        self.check_bounds(addr, 4, memory_bytes)?;
        let bytes = [
            memory_bytes[addr as usize],
            memory_bytes[addr as usize + 1],
            memory_bytes[addr as usize + 2],
            memory_bytes[addr as usize + 3],
        ];
        let v = f32::from_le_bytes(bytes);
        Ok(Value::F32(v))
    }

    fn lift_f64(&self, addr: u32, memory_bytes: &[u8]) -> Result<Value> {
        self.check_bounds(addr, 8, memory_bytes)?;
        let bytes = [
            memory_bytes[addr as usize],
            memory_bytes[addr as usize + 1],
            memory_bytes[addr as usize + 2],
            memory_bytes[addr as usize + 3],
            memory_bytes[addr as usize + 4],
            memory_bytes[addr as usize + 5],
            memory_bytes[addr as usize + 6],
            memory_bytes[addr as usize + 7],
        ];
        let v = f64::from_le_bytes(bytes);
        Ok(Value::F64(v))
    }

    fn lift_char(&self, addr: u32, memory_bytes: &[u8]) -> Result<Value> {
        // Chars are 4 bytes in canonical ABI
        self.check_bounds(addr, 4, memory_bytes)?;
        let code_point = u32::from_le_bytes([
            memory_bytes[addr as usize],
            memory_bytes[addr as usize + 1],
            memory_bytes[addr as usize + 2],
            memory_bytes[addr as usize + 3],
        ]);

        match char::from_u32(code_point) {
            Some(c) => Ok(Value::Char(c)),
            None => Err(Error::new(kinds::InvalidValue(format!(
                "Invalid UTF-8 code point: {}",
                code_point
            )))),
        }
    }

    fn lift_string(&self, addr: u32, memory_bytes: &[u8]) -> Result<Value> {
        // String format in canonical ABI:
        // - 4 bytes length prefix (u32)
        // - UTF-8 encoded string data
        self.check_bounds(addr, 4, memory_bytes)?;

        let len = u32::from_le_bytes([
            memory_bytes[addr as usize],
            memory_bytes[addr as usize + 1],
            memory_bytes[addr as usize + 2],
            memory_bytes[addr as usize + 3],
        ]) as usize;

        // Check bounds for the string content
        self.check_bounds(addr + 4, len as u32, memory_bytes)?;

        // Extract the string bytes
        let string_bytes = &memory_bytes[(addr as usize + 4)..(addr as usize + 4 + len)];

        // Convert to a Rust string
        match std::str::from_utf8(string_bytes) {
            Ok(s) => Ok(Value::String(s.to_string())),
            Err(e) => Err(Error::new(kinds::InvalidValue(format!(
                "Invalid UTF-8 string: {}",
                e
            )))),
        }
    }

    // Complex type lifting operations
    fn lift_list(
        &self,
        _inner_ty: &Box<ValType>,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &[u8],
    ) -> Result<Value> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "List lifting not yet implemented".to_string(),
        )))
    }

    fn lift_record(
        &self,
        _fields: &Vec<(String, ValType)>,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &[u8],
    ) -> Result<Value> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Record lifting not yet implemented".to_string(),
        )))
    }

    fn lift_variant(
        &self,
        _cases: &Vec<(String, Option<ValType>)>,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &[u8],
    ) -> Result<Value> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Variant lifting not yet implemented".to_string(),
        )))
    }

    fn lift_enum(
        &self,
        _cases: &Vec<String>,
        _addr: u32,
        _memory_bytes: &[u8],
    ) -> Result<Value> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Enum lifting not yet implemented".to_string(),
        )))
    }

    fn lift_option(
        &self,
        _inner_ty: &Box<ValType>,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &[u8],
    ) -> Result<Value> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Option lifting not yet implemented".to_string(),
        )))
    }

    fn lift_result(
        &self,
        _ok_ty: Option<&Box<ValType>>,
        _err_ty: Option<&Box<ValType>>,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &[u8],
    ) -> Result<Value> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Result lifting not yet implemented".to_string(),
        )))
    }

    fn lift_tuple(
        &self,
        _types: &Vec<ValType>,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &[u8],
    ) -> Result<Value> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Tuple lifting not yet implemented".to_string(),
        )))
    }

    fn lift_flags(
        &self,
        _names: &Vec<String>,
        _addr: u32,
        _memory_bytes: &[u8],
    ) -> Result<Value> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Flags lifting not yet implemented".to_string(),
        )))
    }

    fn lift_resource(
        &self,
        _type_idx: u32,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &[u8],
    ) -> Result<Value> {
        // TODO: Implement resource lifting
        Err(Error::new(kinds::NotImplementedError(
            "Resource lifting not yet implemented".to_string(),
        )))
    }

    // Primitive lowering operations
    fn lower_bool(&self, value: bool, addr: u32, memory_bytes: &mut [u8]) -> Result<()> {
        if (addr as usize) < memory_bytes.len() {
            memory_bytes[addr as usize] = if value { 1 } else { 0 };
            Ok(())
        } else {
            Err(Error::new(kinds::OutOfBoundsAccess(format!(
                "Address {} out of bounds for memory of size {}",
                addr,
                memory_bytes.len()
            ))))
        }
    }

    fn lower_s8(&self, value: i8, addr: u32, memory_bytes: &mut [u8]) -> Result<()> {
        if (addr as usize) < memory_bytes.len() {
            memory_bytes[addr as usize] = value as u8;
            Ok(())
        } else {
            Err(Error::new(kinds::OutOfBoundsAccess(format!(
                "Address {} out of bounds for memory of size {}",
                addr,
                memory_bytes.len()
            ))))
        }
    }

    fn lower_u8(&self, value: u8, addr: u32, memory_bytes: &mut [u8]) -> Result<()> {
        if (addr as usize) < memory_bytes.len() {
            memory_bytes[addr as usize] = value;
            Ok(())
        } else {
            Err(Error::new(kinds::OutOfBoundsAccess(format!(
                "Address {} out of bounds for memory of size {}",
                addr,
                memory_bytes.len()
            ))))
        }
    }

    fn lower_s16(&self, value: i16, addr: u32, memory_bytes: &mut [u8]) -> Result<()> {
        self.check_bounds(addr, 2, memory_bytes)?;
        let bytes = value.to_le_bytes();
        memory_bytes[addr as usize] = bytes[0];
        memory_bytes[addr as usize + 1] = bytes[1];
        Ok(())
    }

    fn lower_u16(&self, value: u16, addr: u32, memory_bytes: &mut [u8]) -> Result<()> {
        self.check_bounds(addr, 2, memory_bytes)?;
        let bytes = value.to_le_bytes();
        memory_bytes[addr as usize] = bytes[0];
        memory_bytes[addr as usize + 1] = bytes[1];
        Ok(())
    }

    fn lower_s32(&self, value: i32, addr: u32, memory_bytes: &mut [u8]) -> Result<()> {
        self.check_bounds(addr, 4, memory_bytes)?;
        let bytes = value.to_le_bytes();
        memory_bytes[addr as usize] = bytes[0];
        memory_bytes[addr as usize + 1] = bytes[1];
        memory_bytes[addr as usize + 2] = bytes[2];
        memory_bytes[addr as usize + 3] = bytes[3];
        Ok(())
    }

    fn lower_u32(&self, value: u32, addr: u32, memory_bytes: &mut [u8]) -> Result<()> {
        self.check_bounds(addr, 4, memory_bytes)?;
        let bytes = value.to_le_bytes();
        memory_bytes[addr as usize] = bytes[0];
        memory_bytes[addr as usize + 1] = bytes[1];
        memory_bytes[addr as usize + 2] = bytes[2];
        memory_bytes[addr as usize + 3] = bytes[3];
        Ok(())
    }

    fn lower_s64(&self, value: i64, addr: u32, memory_bytes: &mut [u8]) -> Result<()> {
        self.check_bounds(addr, 8, memory_bytes)?;
        let bytes = value.to_le_bytes();
        for i in 0..8 {
            memory_bytes[addr as usize + i] = bytes[i];
        }
        Ok(())
    }

    fn lower_u64(&self, value: u64, addr: u32, memory_bytes: &mut [u8]) -> Result<()> {
        self.check_bounds(addr, 8, memory_bytes)?;
        let bytes = value.to_le_bytes();
        for i in 0..8 {
            memory_bytes[addr as usize + i] = bytes[i];
        }
        Ok(())
    }

    fn lower_f32(&self, value: f32, addr: u32, memory_bytes: &mut [u8]) -> Result<()> {
        self.check_bounds(addr, 4, memory_bytes)?;
        let bytes = value.to_le_bytes();
        for i in 0..4 {
            memory_bytes[addr as usize + i] = bytes[i];
        }
        Ok(())
    }

    fn lower_f64(&self, value: f64, addr: u32, memory_bytes: &mut [u8]) -> Result<()> {
        self.check_bounds(addr, 8, memory_bytes)?;
        let bytes = value.to_le_bytes();
        for i in 0..8 {
            memory_bytes[addr as usize + i] = bytes[i];
        }
        Ok(())
    }

    fn lower_char(&self, value: char, addr: u32, memory_bytes: &mut [u8]) -> Result<()> {
        self.check_bounds(addr, 4, memory_bytes)?;
        let bytes = (value as u32).to_le_bytes();
        for i in 0..4 {
            memory_bytes[addr as usize + i] = bytes[i];
        }
        Ok(())
    }

    fn lower_string(&self, value: &str, addr: u32, memory_bytes: &mut [u8]) -> Result<()> {
        let len = value.len();
        self.check_bounds(addr, 4 + len as u32, memory_bytes)?;

        // Write length prefix
        let len_bytes = (len as u32).to_le_bytes();
        for i in 0..4 {
            memory_bytes[addr as usize + i] = len_bytes[i];
        }

        // Write string content
        for (i, byte) in value.as_bytes().iter().enumerate() {
            memory_bytes[addr as usize + 4 + i] = *byte;
        }

        Ok(())
    }

    fn lower_list(
        &self,
        _values: &Vec<Value>,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &mut [u8],
    ) -> Result<()> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "List lowering not yet implemented".to_string(),
        )))
    }

    fn lower_record(
        &self,
        _fields: &HashMap<String, Value>,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &mut [u8],
    ) -> Result<()> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Record lowering not yet implemented".to_string(),
        )))
    }

    fn lower_variant(
        &self,
        _case: u32,
        _value: &Option<Box<Value>>,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &mut [u8],
    ) -> Result<()> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Variant lowering not yet implemented".to_string(),
        )))
    }

    fn lower_enum(&self, _idx: u32, _addr: u32, _memory_bytes: &mut [u8]) -> Result<()> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Enum lowering not yet implemented".to_string(),
        )))
    }

    fn lower_option(
        &self,
        _value: Option<&Value>,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &mut [u8],
    ) -> Result<()> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Option lowering not yet implemented".to_string(),
        )))
    }

    fn lower_result(
        &self,
        _result: &Result<Option<Box<Value>>, Option<Box<Value>>>,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &mut [u8],
    ) -> Result<()> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Result lowering not yet implemented".to_string(),
        )))
    }

    fn lower_tuple(
        &self,
        _values: &Vec<Value>,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &mut [u8],
    ) -> Result<()> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Tuple lowering not yet implemented".to_string(),
        )))
    }

    fn lower_flags(
        &self,
        _flags: &HashMap<String, bool>,
        _addr: u32,
        _memory_bytes: &mut [u8],
    ) -> Result<()> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Flags lowering not yet implemented".to_string(),
        )))
    }

    fn lower_resource(
        &self,
        _handle: u32,
        _addr: u32,
        _resource_table: &ResourceTable,
        _memory_bytes: &mut [u8],
    ) -> Result<()> {
        // Placeholder implementation
        Err(Error::new(kinds::NotImplementedError(
            "Resource lowering not yet implemented".to_string(),
        )))
    }

    // Utility functions
    fn check_bounds(&self, addr: u32, size: u32, memory_bytes: &[u8]) -> Result<()> {
        let end_addr = addr.checked_add(size).ok_or_else(|| {
            Error::out_of_bounds_access(format!(
                "Memory address overflow: addr={}, size={}",
                addr, size
            ))
        })?;

        let memory_size = memory_bytes.len() as u32;
        if end_addr > memory_size {
            return Err(Error::out_of_bounds_access(format!(
                "Memory access out of bounds: addr={}, size={}, memory_size={}",
                addr, size, memory_size
            )));
        }

        Ok(())
    }

    /// Get the current metrics
    pub fn get_metrics(&self) -> CanonicalMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Reset the metrics to zero
    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        *metrics = CanonicalMetrics::default();
    }

    /// Get memory strategy from interceptor for canonical operations
    fn get_strategy_from_interceptor(&self) -> MemoryStrategy {
        if let Some(interceptor) = &self.interceptor {
            for strategy in &interceptor.strategies {
                if let Some(strategy_val) = strategy.get_memory_strategy(0) {
                    if let Some(memory_strategy) =
                        crate::resources::MemoryStrategy::from_u8(strategy_val)
                    {
                        return memory_strategy;
                    }
                }
            }
        }
        self.memory_strategy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lift_primitive_values() {
        let abi = CanonicalABI::default();
        let memory = vec![1, 0, 0, 0, 2, 0, 0, 0];

        let value = abi.lift_bool(0, &memory).unwrap();
        assert_eq!(value, Value::Bool(true));

        let value = abi.lift_u32(4, &memory).unwrap();
        assert_eq!(value, Value::U32(2));
    }

    #[test]
    fn test_lower_primitive_values() {
        let abi = CanonicalABI::default();
        let mut memory = vec![0; 8];

        abi.lower_bool(true, 0, &mut memory).unwrap();
        assert_eq!(memory[0], 1);

        abi.lower_u32(2, 4, &mut memory).unwrap();
        assert_eq!(&memory[4..8], &[2, 0, 0, 0]);
    }

    #[test]
    fn test_bounds_checking() {
        let abi = CanonicalABI::default();
        let memory = vec![0; 8];

        // This should work - just at the boundary
        assert!(abi.check_bounds(4, 4, &memory).is_ok());

        // This should fail - out of bounds
        assert!(abi.check_bounds(6, 4, &memory).is_err());
    }

    #[test]
    fn test_interceptor_strategy() {
        // Create a mock interceptor for testing
        struct TestStrategy {
            memory_strategy: Option<u8>,
        }

        impl wrt_intercept::LinkInterceptorStrategy for TestStrategy {
            fn before_call(
                &self,
                _source: &str,
                _target: &str,
                _function: &str,
                args: &[wrt_intercept::Value],
            ) -> wrt_error::Result<Vec<wrt_intercept::Value>> {
                Ok(args.to_vec())
            }

            fn after_call(
                &self,
                _source: &str,
                _target: &str,
                _function: &str,
                _args: &[wrt_intercept::Value],
                result: wrt_error::Result<Vec<wrt_intercept::Value>>,
            ) -> wrt_error::Result<Vec<wrt_intercept::Value>> {
                result
            }

            fn get_memory_strategy(&self, _handle: u32) -> Option<u8> {
                self.memory_strategy
            }

            fn clone_strategy(&self) -> Arc<dyn wrt_intercept::LinkInterceptorStrategy> {
                Arc::new(Self {
                    memory_strategy: self.memory_strategy,
                })
            }
        }

        // Test with no interceptor
        let abi = CanonicalABI::default().with_memory_strategy(MemoryStrategy::ZeroCopy);
        assert_eq!(
            abi.get_strategy_from_interceptor(),
            MemoryStrategy::ZeroCopy
        );

        // Test with interceptor that returns None
        let interceptor = Arc::new(wrt_intercept::LinkInterceptor::new("test"));
        let abi = CanonicalABI::default()
            .with_memory_strategy(MemoryStrategy::ZeroCopy)
            .with_interceptor(interceptor);
        assert_eq!(
            abi.get_strategy_from_interceptor(),
            MemoryStrategy::ZeroCopy
        );

        // Test with interceptor that returns Some strategy
        let strategy = Arc::new(TestStrategy {
            memory_strategy: Some(1),
        });
        let mut interceptor = wrt_intercept::LinkInterceptor::new("test");
        interceptor.add_strategy(strategy);

        let abi = CanonicalABI::default()
            .with_memory_strategy(MemoryStrategy::ZeroCopy)
            .with_interceptor(Arc::new(interceptor));
        assert_eq!(
            abi.get_strategy_from_interceptor(),
            MemoryStrategy::BoundedCopy
        );
    }
}
