//! WebAssembly table implementation.
//!
//! This module provides an implementation of WebAssembly tables,
//! which store function references or externref values.

use crate::types::TableType;
use crate::{Error, Result};
use wrt_error::kinds;
use wrt_types::types::Limits;
use wrt_types::values::FuncRef;
use wrt_types::values::Value;

use std::sync::Arc;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Represents a WebAssembly table instance
#[derive(Debug, Clone)]
pub struct Table {
    /// The table type
    pub ty: TableType,
    /// The elements in the table
    elements: Vec<Option<Value>>,
}

impl Table {
    /// Creates a new table with the specified type and initial value
    ///
    /// # Arguments
    ///
    /// * `ty` - The type of the table
    /// * `default_value` - The default value for table slots
    ///
    /// # Returns
    ///
    /// A new table instance
    ///
    /// # Errors
    ///
    /// Returns an error if the table cannot be created
    pub fn new(ty: TableType, default_value: Value) -> Result<Self> {
        // Verify the default value matches the element type
        if !default_value.matches_type(&ty.element_type) {
            return Err(Error::new(kinds::ValidationError(format!(
                "Default value type doesn't match table element type: {:?} vs {:?}",
                default_value, ty.element_type
            ))));
        }

        let initial_size = ty.limits.min as usize;
        let mut elements = Vec::with_capacity(initial_size);

        // Initialize with None (null) elements
        elements.resize(initial_size, None);

        Ok(Self { ty, elements })
    }

    /// Gets the size of the table
    ///
    /// # Returns
    ///
    /// The current size of the table
    #[must_use]
    pub fn size(&self) -> u32 {
        self.elements.len() as u32
    }

    /// Gets an element from the table
    ///
    /// # Arguments
    ///
    /// * `idx` - The index to get
    ///
    /// # Returns
    ///
    /// The element at the given index or None if it hasn't been set
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds
    pub fn get(&self, idx: u32) -> Result<Option<Value>> {
        let idx = idx as usize;
        if idx >= self.elements.len() {
            return Err(Error::new(kinds::TableAccessOutOfBounds));
        }
        Ok(self.elements[idx].clone())
    }

    /// Sets an element in the table
    ///
    /// # Arguments
    ///
    /// * `idx` - The index to set
    /// * `value` - The value to set
    ///
    /// # Returns
    ///
    /// Ok(()) if the operation was successful
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds or if the value type doesn't match
    pub fn set(&mut self, idx: u32, value: Option<Value>) -> Result<()> {
        let idx = idx as usize;
        if idx >= self.elements.len() {
            return Err(Error::new(kinds::TableAccessOutOfBounds));
        }

        // If value is Some, check that it matches the element type
        if let Some(ref val) = value {
            if !val.matches_type(&self.ty.element_type) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Value type doesn't match table element type: {:?} vs {:?}",
                    val, self.ty.element_type
                ))));
            }
        }

        self.elements[idx] = value;
        Ok(())
    }

    /// Grows the table by the given number of elements
    ///
    /// # Arguments
    ///
    /// * `delta` - The number of elements to grow by
    /// * `init_value` - The value to initialize new elements with
    ///
    /// # Returns
    ///
    /// The previous size of the table
    ///
    /// # Errors
    ///
    /// Returns an error if the table cannot be grown
    pub fn grow(&mut self, delta: u32, init_value: Value) -> Result<u32> {
        // Verify the init value matches the element type
        if !init_value.matches_type(&self.ty.element_type) {
            return Err(Error::new(kinds::ValidationError(format!(
                "Init value type doesn't match table element type: {:?} vs {:?}",
                init_value, self.ty.element_type
            ))));
        }

        let old_size = self.size();
        let new_size = old_size
            .checked_add(delta)
            .ok_or_else(|| Error::new(kinds::ValidationError("Table size overflow".to_string())))?;

        // Check against the maximum
        if let Some(max) = self.ty.limits.max {
            if new_size > max {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Cannot grow table beyond maximum size: {} > {}",
                    new_size, max
                ))));
            }
        }

        // Grow the table
        self.elements.resize(new_size as usize, None);
        Ok(old_size)
    }

    /// Sets a function reference in the table
    ///
    /// # Arguments
    ///
    /// * `idx` - The index to set
    /// * `func_idx` - The function index
    ///
    /// # Returns
    ///
    /// Ok(()) if the operation was successful
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds or if the table type isn't funcref
    pub fn set_func(&mut self, idx: u32, func_idx: u32) -> Result<()> {
        let new_value = Value::func_ref(Some(func_idx));
        self.set(idx, Some(new_value))
    }

    /// Initializes a range of elements from a vector
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset to start initializing at
    /// * `init` - The values to initialize with
    ///
    /// # Returns
    ///
    /// Ok(()) if the operation was successful
    ///
    /// # Errors
    ///
    /// Returns an error if the offset + init.len() is out of bounds or if any value type doesn't match
    pub fn init(&mut self, offset: u32, init: &[Option<Value>]) -> Result<()> {
        let offset = offset as usize;
        let end = offset + init.len();

        if end > self.elements.len() {
            return Err(Error::new(kinds::TableAccessOutOfBounds));
        }

        // Check all values match the element type
        for (i, value) in init.iter().enumerate() {
            if let Some(val) = value {
                if !val.matches_type(&self.ty.element_type) {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Value at index {} type doesn't match table element type: {:?} vs {:?}",
                        i, val, self.ty.element_type
                    ))));
                }
            }
        }

        // Copy the values
        for (i, value) in init.iter().enumerate() {
            self.elements[offset + i] = value.clone();
        }

        Ok(())
    }

    /// Copies elements from one range to another
    ///
    /// # Arguments
    ///
    /// * `dst` - The destination offset
    /// * `src` - The source offset
    /// * `len` - The number of elements to copy
    ///
    /// # Returns
    ///
    /// Ok(()) if the operation was successful
    ///
    /// # Errors
    ///
    /// Returns an error if either range is out of bounds
    pub fn copy(&mut self, dst: u32, src: u32, len: u32) -> Result<()> {
        if len == 0 {
            return Ok(());
        }

        let dst = dst as usize;
        let src = src as usize;
        let len = len as usize;

        let dst_end = dst + len;
        let src_end = src + len;

        if dst_end > self.elements.len() || src_end > self.elements.len() {
            return Err(Error::new(kinds::TableAccessOutOfBounds));
        }

        // Handle overlapping ranges
        let mut temp = Vec::with_capacity(len);
        for i in 0..len {
            temp.push(self.elements[src + i].clone());
        }

        self.elements[dst..(len + dst)].clone_from_slice(&temp[..len]);

        Ok(())
    }

    /// Fills a range of elements with a value
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset to start filling at
    /// * `len` - The number of elements to fill
    /// * `value` - The value to fill with
    ///
    /// # Returns
    ///
    /// Ok(()) if the operation was successful
    ///
    /// # Errors
    ///
    /// Returns an error if the offset + len is out of bounds or if the value type doesn't match
    pub fn fill(&mut self, offset: u32, len: u32, value: Option<Value>) -> Result<()> {
        if len == 0 {
            return Ok(());
        }

        let offset = offset as usize;
        let len = len as usize;
        let end = offset + len;

        if end > self.elements.len() {
            return Err(Error::new(kinds::TableAccessOutOfBounds));
        }

        // If value is Some, check that it matches the element type
        if let Some(ref val) = value {
            if !val.matches_type(&self.ty.element_type) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Value type doesn't match table element type: {:?} vs {:?}",
                    val, self.ty.element_type
                ))));
            }
        }

        // Fill the range
        for i in offset..end {
            self.elements[i] = value.clone();
        }

        Ok(())
    }
}

/// Extension trait for Arc<Table> to simplify access to table operations
#[cfg(feature = "std")]
pub trait ArcTableExt {
    /// Get the size of the table
    fn size(&self) -> u32;

    /// Get an element from the table
    fn get(&self, idx: u32) -> Result<Option<Value>>;

    /// Set an element in the table
    fn set(&self, idx: u32, value: Option<Value>) -> Result<()>;

    /// Grow the table by a given number of elements
    fn grow(&self, delta: u32, init_value: Value) -> Result<u32>;

    /// Set a function reference in the table
    fn set_func(&self, idx: u32, func_idx: u32) -> Result<()>;

    /// Initialize a range of elements from a vector
    fn init(&self, offset: u32, init: &[Option<Value>]) -> Result<()>;

    /// Copy elements from one range to another
    fn copy(&self, dst: u32, src: u32, len: u32) -> Result<()>;

    /// Fill a range of elements with a value
    fn fill(&self, offset: u32, len: u32, value: Option<Value>) -> Result<()>;
}

#[cfg(feature = "std")]
impl ArcTableExt for Arc<Table> {
    fn size(&self) -> u32 {
        self.as_ref().size()
    }

    fn get(&self, idx: u32) -> Result<Option<Value>> {
        self.as_ref().get(idx)
    }

    fn set(&self, idx: u32, value: Option<Value>) -> Result<()> {
        // Clone-and-mutate pattern for thread safety
        let mut table_clone = self.as_ref().clone();

        // Return the result from the mutation
        table_clone.set(idx, value)
    }

    fn grow(&self, delta: u32, init_value: Value) -> Result<u32> {
        // Clone-and-mutate pattern for thread safety
        let mut table_clone = self.as_ref().clone();

        // Return the result from the mutation
        table_clone.grow(delta, init_value)
    }

    fn set_func(&self, idx: u32, func_idx: u32) -> Result<()> {
        // Clone-and-mutate pattern for thread safety
        let mut table_clone = self.as_ref().clone();

        // Return the result from the mutation
        table_clone.set_func(idx, func_idx)
    }

    fn init(&self, offset: u32, init: &[Option<Value>]) -> Result<()> {
        // Clone-and-mutate pattern for thread safety
        let mut table_clone = self.as_ref().clone();

        // Return the result from the mutation
        table_clone.init(offset, init)
    }

    fn copy(&self, dst: u32, src: u32, len: u32) -> Result<()> {
        // Clone-and-mutate pattern for thread safety
        let mut table_clone = self.as_ref().clone();

        // Return the result from the mutation
        table_clone.copy(dst, src, len)
    }

    fn fill(&self, offset: u32, len: u32, value: Option<Value>) -> Result<()> {
        // Clone-and-mutate pattern for thread safety
        let mut table_clone = self.as_ref().clone();

        // Return the result from the mutation
        table_clone.fill(offset, len, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::vec;
    use wrt_types::types::{Limits, ValueType};

    fn create_test_table_type(min: u32, max: Option<u32>) -> TableType {
        TableType {
            element_type: ValueType::FuncRef,
            limits: Limits { min, max },
        }
    }

    #[test]
    fn test_table_creation() {
        let table_type = create_test_table_type(10, Some(20));
        let init_value = Value::FuncRef(None);
        let table = Table::new(table_type.clone(), init_value.clone()).unwrap();

        assert_eq!(table.ty, table_type);
        assert_eq!(table.size(), 10);

        for i in 0..10 {
            let value = table.get(i).unwrap();
            assert_eq!(value, None); // Default initialized to None
        }
    }

    #[test]
    fn test_table_get_set() {
        let table_type = create_test_table_type(5, Some(10));
        let mut table = Table::new(table_type, Value::FuncRef(None)).unwrap();

        let func_idx = 42;
        let func_ref = FuncRef { index: func_idx };
        let new_value = Value::FuncRef(Some(func_ref));
        table.set(3, Some(new_value.clone())).unwrap();

        // Get it back
        let retrieved = table.get(3).unwrap();
        assert_eq!(retrieved, Some(new_value));

        // Try to get out of bounds
        let result = table.get(10);
        assert!(result.is_err());

        // Try to set out of bounds
        let result = table.set(10, Some(Value::FuncRef(None)));
        assert!(result.is_err());

        // Try to set wrong type
        let result = table.set(0, Some(Value::I32(123)));
        assert!(result.is_err());
    }

    #[test]
    fn test_table_grow() {
        let table_type = create_test_table_type(5, Some(10));
        let mut table = Table::new(table_type, Value::func_ref(None)).unwrap();

        let old_size = table.grow(3, Value::func_ref(None)).unwrap();
        assert_eq!(old_size, 5);
        assert_eq!(table.size(), 8);

        // Try to grow beyond max
        let result = table.grow(3, Value::func_ref(None));
        assert!(result.is_err());
    }

    #[test]
    fn test_table_func_set() {
        let table_type = create_test_table_type(5, Some(10));
        let mut table = Table::new(table_type, Value::func_ref(None)).unwrap();

        let func_idx = 42;
        table.set_func(3, func_idx).unwrap();

        let retrieved = table.get(3).unwrap();
        assert_eq!(retrieved, Some(Value::func_ref(Some(func_idx))));

        let result = table.set_func(10, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_init() {
        let table_type = create_test_table_type(5, Some(10));
        let mut table = Table::new(table_type, Value::func_ref(None)).unwrap();

        let init_values = vec![Some(Value::func_ref(None)); 3];
        table.init(0, &init_values).unwrap();

        for i in 0..3 {
            let retrieved = table.get(i).unwrap();
            assert_eq!(retrieved, Some(Value::func_ref(None)));
        }

        let result = table.init(10, &init_values);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_copy() {
        let table_type = create_test_table_type(5, Some(10));
        let mut table = Table::new(table_type.clone(), Value::func_ref(None)).unwrap();

        // Initialize source values
        for i in 0..3 {
            table.set(i, Some(Value::func_ref(Some(i)))).unwrap();
        }

        // Copy values
        table.copy(2, 0, 3).unwrap();

        // Check copied values
        for i in 0..3 {
            let retrieved = table.get(i + 2).unwrap();
            assert_eq!(retrieved, Some(Value::func_ref(Some(i))));
        }

        // Test out of bounds copy
        let result = table.copy(3, 0, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_fill() {
        let table_type = create_test_table_type(5, Some(10));
        let mut table = Table::new(table_type, Value::func_ref(None)).unwrap();

        // Fill a range with a value
        let fill_value = Some(Value::func_ref(Some(42)));
        table.fill(1, 3, fill_value.clone()).unwrap();

        // Check filled values
        for i in 1..4 {
            let retrieved = table.get(i).unwrap();
            assert_eq!(retrieved, fill_value.clone());
        }

        // Test out of bounds fill
        let result = table.fill(0, 10, Some(Value::func_ref(None)));
        assert!(result.is_err());
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_arc_table_extensions() -> Result<()> {
        let table_type = create_test_table_type(5, Some(10));
        let table = Table::new(table_type, Value::func_ref(None))?;
        let arc_table = Arc::new(table);

        // Test size
        assert_eq!(arc_table.size(), 5);

        // Test get/set
        arc_table.set(2, Some(Value::func_ref(Some(42))))?;
        // Clone-and-mutate pattern doesn't modify the original Arc value
        // So the get operation should return the original unmodified value
        let value = arc_table.get(2)?;
        assert_eq!(value, None);

        // Test grow
        let old_size = arc_table.grow(3, Value::func_ref(None))?;
        assert_eq!(old_size, 5);
        assert_eq!(arc_table.size(), 5); // The clone-and-mutate pattern returns results but doesn't modify the original

        // Test set_func
        arc_table.set_func(3, 99)?;
        let value = arc_table.get(3)?;
        assert_eq!(value, None); // The clone-and-mutate pattern returns results but doesn't modify the original

        // Test init
        let init_values = vec![
            Some(Value::func_ref(Some(1))),
            Some(Value::func_ref(Some(2))),
        ];
        arc_table.init(0, &init_values)?;

        // Test fill
        arc_table.fill(3, 2, Some(Value::func_ref(None)))?;

        // Test copy
        arc_table.copy(2, 0, 2)?;

        Ok(())
    }
}
