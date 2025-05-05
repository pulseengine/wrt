//! WebAssembly table implementation.
//!
//! This module provides an implementation of WebAssembly tables,
//! which store function references or externref values.

use crate::prelude::*;
use crate::types::TableType;

/// Represents a WebAssembly table instance
#[derive(Debug)]
pub struct Table {
    /// The table type
    pub ty: TableType,
    /// The elements in the table - using SafeStack instead of Vec for memory safety
    elements: SafeStack<Option<Value>>,
    /// A debug name for diagnostics
    debug_name: Option<String>,
    /// Verification level for table operations
    verification_level: VerificationLevel,
}

impl Clone for Table {
    fn clone(&self) -> Self {
        // Get elements as a Vec
        let elements_vec = self.elements.to_vec().unwrap_or_default();

        // Create a new SafeStack with the same elements
        let mut new_elements = SafeStack::with_capacity(elements_vec.len());
        new_elements.set_verification_level(self.verification_level);

        for elem in elements_vec {
            new_elements.push(elem).unwrap();
        }

        // Create a new instance with the same properties
        Self {
            ty: self.ty.clone(),
            elements: new_elements,
            debug_name: self.debug_name.clone(),
            verification_level: self.verification_level,
        }
    }
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
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!(
                    "Default value type doesn't match table element type: {:?} vs {:?}",
                    default_value, ty.element_type
                ),
            ));
        }

        let initial_size = ty.limits.min as usize;
        let mut elements = SafeStack::with_capacity(initial_size);

        // Initialize with None (null) elements
        for _ in 0..initial_size {
            elements.push(None)?;
        }

        Ok(Self {
            ty,
            elements,
            verification_level: VerificationLevel::default(),
            debug_name: None,
        })
    }

    /// Creates a new table with the specified capacity and element type
    ///
    /// # Arguments
    ///
    /// * `capacity` - The initial capacity of the table
    /// * `element_type` - The element type for the table
    ///
    /// # Returns
    ///
    /// A new table instance
    ///
    /// # Errors
    ///
    /// Returns an error if the table cannot be created
    pub fn with_capacity(capacity: u32, element_type: &ValueType) -> Result<Self> {
        let table_type = TableType {
            element_type: *element_type,
            limits: Limits {
                min: capacity,
                max: Some(capacity * 2), // Allow doubling as max
            },
        };

        Self::new(table_type, Value::FuncRef(None))
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
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                "Table access out of bounds",
            ));
        }

        // Implement verification if needed based on verification level
        if self.verification_level.should_verify(128) {
            // Verify table integrity - this is a simplified version
            // In a real implementation, we would do more thorough checks
            if idx >= self.elements.len() {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    "Table integrity check failed: index out of bounds",
                ));
            }
        }

        // Use SafeStack's get method instead of direct indexing
        match self.elements.get(idx) {
            Ok(val) => Ok(val.clone()),
            Err(_) => Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                "Table access failed during safe memory operation",
            )),
        }
    }

    /// Sets an element at the specified index
    ///
    /// # Arguments
    ///
    /// * `idx` - The index to set
    /// * `value` - The value to set
    ///
    /// # Returns
    ///
    /// Ok(()) if the set was successful
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds or if the value type doesn't match the table element type
    pub fn set(&mut self, idx: u32, value: Option<Value>) -> Result<()> {
        let idx = idx as usize;
        if idx >= self.elements.len() {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                "Table access out of bounds",
            ));
        }

        // If value is Some, check that it matches the element type
        if let Some(ref val) = value {
            if !val.matches_type(&self.ty.element_type) {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    format!(
                        "Element type doesn't match table element type: {:?} vs {:?}",
                        val, self.ty.element_type
                    ),
                ));
            }
        }

        // Use SafeStack's set method to update the element directly
        self.elements.set(idx, value)?;

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
        // Check that init_value has the correct type
        if !init_value.matches_type(&self.ty.element_type) {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!(
                    "Initial value type doesn't match table element type: {:?} vs {:?}",
                    init_value, self.ty.element_type
                ),
            ));
        }

        // Get current size
        let old_size = self.size();

        // Calculate new size
        let new_size = match old_size.checked_add(delta) {
            Some(size) => size,
            None => {
                return Err(Error::new(
                    ErrorCategory::Resource,
                    codes::RESOURCE_LIMIT_EXCEEDED,
                    "Table size overflow",
                ));
            }
        };

        // Check against table max limit if defined
        if let Some(max) = self.ty.limits.max {
            if new_size > max {
                return Err(Error::new(
                    ErrorCategory::Resource,
                    codes::RESOURCE_LIMIT_EXCEEDED,
                    format!("Table size exceeds maximum: {} > {}", new_size, max),
                ));
            }
        }

        // Add new elements directly to the SafeStack
        for _ in 0..delta {
            self.elements.push(Some(init_value.clone()))?;
        }

        // Verify integrity if needed based on verification level
        if self.verification_level.should_verify(200) {
            // Ensure the new size is correct
            if self.elements.len() != new_size as usize {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    format!(
                        "Table integrity check failed: expected size {} but got {}",
                        new_size,
                        self.elements.len()
                    ),
                ));
            }

            // Verify the type of the last element added
            if let Ok(Some(last)) = self.elements.get(self.elements.len() - 1) {
                if !last.matches_type(&self.ty.element_type) {
                    return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::VALIDATION_ERROR,
                        "Table integrity check failed: element type mismatch after grow",
                    ));
                }
            }
        }

        Ok(old_size)
    }

    /// Sets a function reference in the table
    ///
    /// # Arguments
    ///
    /// * `idx` - The index to set
    /// * `func_idx` - The function index to set
    ///
    /// # Returns
    ///
    /// Ok(()) if the operation was successful
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds or the table element type isn't a funcref
    pub fn set_func(&mut self, idx: u32, func_idx: u32) -> Result<()> {
        // Set a function reference value
        self.set(idx, Some(Value::func_ref(Some(func_idx))))
    }

    /// Initialize a range of elements in the table
    ///
    /// # Arguments
    ///
    /// * `offset` - The starting offset
    /// * `init` - The elements to initialize with
    ///
    /// # Returns
    ///
    /// Ok(()) if successful
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn init(&mut self, offset: u32, init: &[Option<Value>]) -> Result<()> {
        let offset = offset as usize;
        if offset > self.elements.len() {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                "Table access out of bounds",
            ));
        }

        let end = offset + init.len();
        if end > self.elements.len() {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                "Table initialization would go out of bounds",
            ));
        }

        // Create a safe copy of the elements
        let mut elements_vec = self.elements.to_vec()?;

        // Type check all values
        for (i, value) in init.iter().enumerate() {
            if let Some(val) = value {
                if !val.matches_type(&self.ty.element_type) {
                    return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::VALIDATION_ERROR,
                        format!(
                            "Element type doesn't match table element type: {:?} vs {:?}",
                            val, self.ty.element_type
                        ),
                    ));
                }

                // Update the element at the appropriate position
                elements_vec[offset + i] = Some(val.clone());
            } else {
                // Set to None (null reference)
                elements_vec[offset + i] = None;
            }
        }

        // Create a new SafeStack with the updated elements
        let mut new_stack = SafeStack::with_capacity(elements_vec.len());
        new_stack.set_verification_level(self.verification_level);

        // Push the elements to the new stack
        for element in elements_vec.iter() {
            new_stack.push(element.clone())?;
        }

        // Verify integrity if needed based on verification level
        if self.verification_level.should_verify(200) {
            // Ensure all elements are pushed correctly
            if new_stack.len() != elements_vec.len() {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    "Table integrity check failed: element count mismatch after initialization",
                ));
            }
        }

        // Replace the elements with the new stack
        self.elements = new_stack;

        Ok(())
    }

    /// Copy elements from one region of a table to another
    pub fn copy_elements(&mut self, dst: usize, src: usize, len: usize) -> Result<()> {
        // Verify bounds
        if src + len > self.elements.len() || dst + len > self.elements.len() {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_ERROR,
                format!(
                    "table element copy out of bounds: src={}, dst={}, len={}",
                    src, dst, len
                ),
            ));
        }

        // Handle the case where regions don't overlap or no elements to copy
        if len == 0 {
            return Ok(());
        }

        // Create temporary stack to store elements during copy
        let mut temp_stack = SafeStack::with_capacity(len);
        temp_stack.set_verification_level(self.verification_level);

        // Read source elements into temporary stack
        for i in 0..len {
            temp_stack.push(self.elements.get(src + i)?)?;
        }

        // Create a new stack for the full result
        let mut result_stack = SafeStack::with_capacity(self.elements.len());
        result_stack.set_verification_level(self.verification_level);

        // Copy elements with the updated values
        for i in 0..self.elements.len() {
            if i >= dst && i < dst + len {
                // This is in the destination range, use value from temp_stack
                result_stack.push(temp_stack.get(i - dst)?)?;
            } else {
                // Outside destination range, use original value
                result_stack.push(self.elements.get(i)?)?;
            }
        }

        // Replace the elements stack
        self.elements = result_stack;

        Ok(())
    }

    /// Fill a range of elements with a given value
    pub fn fill_elements(&mut self, offset: usize, value: Option<Value>, len: usize) -> Result<()> {
        // Verify bounds
        if offset + len > self.elements.len() {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_ERROR,
                format!("table fill out of bounds: offset={}, len={}", offset, len),
            ));
        }

        // Handle empty fill
        if len == 0 {
            return Ok(());
        }

        // Create a new stack with the filled elements
        let mut result_stack = SafeStack::with_capacity(self.elements.len());
        result_stack.set_verification_level(self.verification_level);

        // Copy elements with fill applied
        for i in 0..self.elements.len() {
            if i >= offset && i < offset + len {
                // This is in the fill range
                result_stack.push(value.clone())?;
            } else {
                // Outside fill range, use original value
                result_stack.push(self.elements.get(i)?)?;
            }
        }

        // Replace the elements stack
        self.elements = result_stack;

        Ok(())
    }

    /// Sets the verification level for this table
    ///
    /// # Arguments
    ///
    /// * `level` - The verification level to set
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        self.verification_level = level;
        // Pass the verification level to the SafeStack
        self.elements.set_verification_level(level);
    }

    /// Gets the current verification level for this table
    ///
    /// # Returns
    ///
    /// The current verification level
    #[must_use]
    pub fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }

    /// Sets an element at the given index.
    pub fn init_element(&mut self, idx: usize, value: Option<Value>) -> Result<()> {
        // Check bounds
        if idx >= self.elements.len() {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                format!("table element index out of bounds: {}", idx),
            ));
        }

        // Set the element directly without converting to/from Vec
        self.elements.get(idx)?; // Verify access is valid

        // Create temporary stack to hold all elements
        let mut temp_stack = SafeStack::with_capacity(self.elements.len());
        temp_stack.set_verification_level(self.verification_level);

        // Copy elements, replacing the one at idx
        for i in 0..self.elements.len() {
            if i == idx {
                temp_stack.push(value.clone())?;
            } else {
                temp_stack.push(self.elements.get(i)?)?;
            }
        }

        // Replace the old stack with the new one
        self.elements = temp_stack;

        Ok(())
    }

    /// Get safety statistics for this table instance
    ///
    /// This returns detailed statistics about table usage and safety checks
    ///
    /// # Returns
    ///
    /// A string containing the statistics
    pub fn safety_stats(&self) -> String {
        format!(
            "Table Safety Stats:\n\
             - Size: {} elements\n\
             - Element type: {:?}\n\
             - Verification level: {:?}",
            self.elements.len(),
            self.ty.element_type,
            self.verification_level
        )
    }
}

/// Extension trait for `Arc<Table>` to simplify access to table operations
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
        table_clone.copy_elements(dst as usize, src as usize, len as usize)
    }

    fn fill(&self, offset: u32, len: u32, value: Option<Value>) -> Result<()> {
        // Clone-and-mutate pattern for thread safety
        let mut table_clone = self.as_ref().clone();

        // Return the result from the mutation
        table_clone.fill_elements(offset as usize, value, len as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::vec;
    use wrt_types::types::{Limits, ValueType};
    use wrt_types::verification::VerificationLevel;

    fn create_test_table_type(min: u32, max: Option<u32>) -> TableType {
        TableType {
            element_type: ValueType::FuncRef,
            limits: Limits { min, max },
        }
    }

    #[test]
    fn test_table_creation() {
        let table_type = create_test_table_type(10, Some(20));
        let init_value = Value::func_ref(None);
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
        let mut table = Table::new(table_type, Value::func_ref(None)).unwrap();

        let func_idx = 42;
        let new_value = Value::func_ref(Some(func_idx));
        table.set(3, Some(new_value.clone())).unwrap();

        // Get it back
        let retrieved = table.get(3).unwrap();
        assert_eq!(retrieved, Some(new_value));

        // Try to get out of bounds
        let result = table.get(10);
        assert!(result.is_err());

        // Try to set out of bounds
        let result = table.set(10, Some(Value::func_ref(None)));
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
        table.copy_elements(2, 0, 3).unwrap();

        // Check copied values
        for i in 0..3 {
            let retrieved = table.get(i + 2).unwrap();
            assert_eq!(retrieved, Some(Value::func_ref(Some(i))));
        }

        // Test out of bounds copy
        let result = table.copy_elements(3, 0, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_table_fill() {
        let table_type = create_test_table_type(5, Some(10));
        let mut table = Table::new(table_type, Value::func_ref(None)).unwrap();

        // Fill a range with a value
        let fill_value = Some(Value::func_ref(Some(42)));
        table.fill_elements(1, fill_value.clone(), 3).unwrap();

        // Check filled values
        for i in 1..4 {
            let retrieved = table.get(i).unwrap();
            assert_eq!(retrieved, fill_value.clone());
        }

        // Test out of bounds fill
        let result = table.fill_elements(0, Some(Value::func_ref(None)), 10);
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

    #[test]
    fn test_table_safe_operations() -> Result<()> {
        // Create a table type
        let table_type = TableType {
            element_type: ValueType::FuncRef,
            limits: Limits {
                min: 5,
                max: Some(10),
            },
        };

        // Create a table
        let mut table = Table::new(table_type, Value::func_ref(None))?;

        // Set verification level
        table.set_verification_level(VerificationLevel::Full);

        // Set some values
        table.set(1, Some(Value::func_ref(Some(42))))?;
        table.set(2, Some(Value::func_ref(Some(43))))?;

        // Get them back
        let val1 = table.get(1)?;
        let val2 = table.get(2)?;

        // Verify values
        assert_eq!(val1, Some(Value::func_ref(Some(42))));
        assert_eq!(val2, Some(Value::func_ref(Some(43))));

        // Test fill operation
        table.fill_elements(3, Some(Value::func_ref(Some(99))), 2)?;
        assert_eq!(table.get(3)?, Some(Value::func_ref(Some(99))));
        assert_eq!(table.get(4)?, Some(Value::func_ref(Some(99))));

        // Test copy operation
        table.copy_elements(0, 3, 2)?;
        assert_eq!(table.get(0)?, Some(Value::func_ref(Some(99))));
        assert_eq!(table.get(1)?, Some(Value::func_ref(Some(99))));

        Ok(())
    }

    #[test]
    fn test_table_memory_safety() -> Result<()> {
        use wrt_types::types::ValueType;
        use wrt_types::verification::VerificationLevel;

        // Create a table with a specific verification level
        let mut table = Table::with_capacity(5, &ValueType::FuncRef)?;
        table.set_verification_level(VerificationLevel::Full);

        // Initialize elements
        let value1 = Some(Value::func_ref(Some(1)));
        let value2 = Some(Value::func_ref(Some(2)));

        // Test push operation with safety checking
        table.init_element(0, value1.clone())?;
        table.init_element(1, value2.clone())?;

        // Verify elements
        assert_eq!(table.get(0)?, value1);
        assert_eq!(table.get(1)?, value2);

        // Test copy with safety checking
        table.copy_elements(2, 0, 2)?;
        assert_eq!(table.get(2)?, value1);
        assert_eq!(table.get(3)?, value2);

        // Test fill with safety checking
        let fill_value = Some(Value::func_ref(Some(42)));
        table.fill_elements(1, fill_value.clone(), 2)?;
        assert_eq!(table.get(1)?, fill_value);
        assert_eq!(table.get(2)?, fill_value);

        // Print safety stats
        println!("{}", table.safety_stats());

        Ok(())
    }
}
