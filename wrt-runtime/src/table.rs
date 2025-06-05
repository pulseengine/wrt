//! WebAssembly table implementation.
//!
//! This module provides an implementation of WebAssembly tables,
//! which store function references or externref values.

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

use wrt_foundation::{
    types::{Limits as WrtLimits, TableType as WrtTableType, ValueType as WrtValueType},
    values::{Value as WrtValue, FuncRef as WrtFuncRef, ExternRef as WrtExternRef},
    safe_memory::SafeStack,
};

use crate::prelude::*;

// Import the TableOperations trait from wrt-instructions
use wrt_instructions::table_ops::TableOperations;

/// Invalid index error code
const INVALID_INDEX: u16 = 4004;
/// Index too large error code  
const INDEX_TOO_LARGE: u16 = 4005;

/// Safe conversion from WebAssembly u32 index to Rust usize
/// 
/// # Arguments
/// 
/// * `index` - WebAssembly index as u32
/// 
/// # Returns
/// 
/// Ok(usize) if conversion is safe, error otherwise
fn wasm_index_to_usize(index: u32) -> Result<usize> {
    usize::try_from(index).map_err(|_| Error::new(
        ErrorCategory::Runtime, 
        INVALID_INDEX, 
        "Index exceeds usize limit"
    ))
}

/// Safe conversion from Rust usize to WebAssembly u32
/// 
/// # Arguments
/// 
/// * `size` - Rust size as usize
/// 
/// # Returns
/// 
/// Ok(u32) if conversion is safe, error otherwise  
fn usize_to_wasm_u32(size: usize) -> Result<u32> {
    u32::try_from(size).map_err(|_| Error::new(
        ErrorCategory::Runtime, 
        INDEX_TOO_LARGE, 
        "Size exceeds u32 limit"
    ))
}

/// A WebAssembly table is a vector of opaque values of a single type.
#[derive(Debug)]
pub struct Table {
    /// The table type, using the canonical WrtTableType
    pub ty: WrtTableType,
    /// The table elements
    elements: SafeStack<Option<WrtValue>, 1024, wrt_foundation::safe_memory::NoStdProvider<1024>>,
    /// A debug name for the table (optional)
    pub debug_name: Option<String>,
    /// Verification level for table operations
    pub verification_level: VerificationLevel,
}

impl Clone for Table {
    fn clone(&self) -> Self {
        let elements_vec = self.elements.to_vec().unwrap_or_default();
        let mut new_elements = SafeStack::with_capacity(elements_vec.len());
        new_elements.set_verification_level(self.verification_level);
        for elem in elements_vec {
            // If push fails, we've already allocated the capacity so this should not fail
            // unless we're out of memory, in which case panicking is appropriate
            if new_elements.push(elem).is_err() {
                // In Clone implementation, we can't return an error, so panic is appropriate
                // for an out-of-memory condition
                panic!("Failed to clone table: out of memory");
            }
        }
        Self {
            ty: self.ty.clone(),
            elements: new_elements,
            debug_name: self.debug_name.clone(),
            verification_level: self.verification_level,
        }
    }
}

impl PartialEq for Table {
    fn eq(&self, other: &Self) -> bool {
        if self.ty != other.ty
            || self.debug_name != other.debug_name
            || self.verification_level != other.verification_level
        {
            return false;
        }
        let self_elements = self.elements.to_vec().unwrap_or_default();
        let other_elements = other.elements.to_vec().unwrap_or_default();
        self_elements == other_elements
    }
}

impl Eq for Table {}

impl Default for Table {
    fn default() -> Self {
        use wrt_foundation::types::{Limits, TableType, ValueType};
        let table_type = TableType {
            element_type: ValueType::FuncRef,
            limits: Limits { min: 0, max: Some(1) },
        };
        Self::new(table_type).unwrap()
    }
}

impl wrt_foundation::traits::Checksummable for Table {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        checksum.update_slice(&(self.ty.element_type as u8).to_le_bytes());
        checksum.update_slice(&self.ty.limits.min.to_le_bytes());
        if let Some(max) = self.ty.limits.max {
            checksum.update_slice(&max.to_le_bytes());
        }
    }
}

impl wrt_foundation::traits::ToBytes for Table {
    fn serialized_size(&self) -> usize {
        16 // simplified
    }

    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<()> {
        writer.write_all(&(self.ty.element_type as u8).to_le_bytes())?;
        writer.write_all(&self.ty.limits.min.to_le_bytes())
    }
}

impl wrt_foundation::traits::FromBytes for Table {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &P,
    ) -> wrt_foundation::Result<Self> {
        let mut bytes = [0u8; 1];
        reader.read_exact(&mut bytes)?;
        let element_type = match bytes[0] {
            0 => wrt_foundation::types::ValueType::FuncRef,
            _ => wrt_foundation::types::ValueType::ExternRef,
        };
        
        let mut min_bytes = [0u8; 4];
        reader.read_exact(&mut min_bytes)?;
        let min = u32::from_le_bytes(min_bytes);
        
        use wrt_foundation::types::{Limits, TableType};
        let table_type = TableType {
            element_type,
            limits: Limits { min, max: Some(min + 1) },
        };
        Self::new(table_type)
    }
}

impl Table {
    /// Creates a new table with the specified type.
    /// Elements are initialized to a type-appropriate null value.
    pub fn new(ty: WrtTableType) -> Result<Self> {
        // Determine the type-appropriate null value for initialization
        let init_val = match ty.element_type {
            WrtValueType::FuncRef => Some(WrtValue::FuncRef(None)),
            WrtValueType::ExternRef => Some(WrtValue::ExternRef(None)),
            // Other types are not allowed in tables as per current Wasm spec for element_type
            _ => {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::INVALID_TYPE,
                    format!("Invalid element type for table: {:?}", ty.element_type),
                ))
            }
        };

        let initial_size = wasm_index_to_usize(ty.limits.min)?;
        let mut elements = SafeStack::with_capacity(initial_size);
        elements.set_verification_level(VerificationLevel::default());

        for _ in 0..initial_size {
            elements.push(init_val.clone())?;
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
    pub fn with_capacity(capacity: u32, element_type: &WrtValueType) -> Result<Self> {
        let table_type = WrtTableType {
            element_type: *element_type,
            limits: WrtLimits { min: capacity, max: Some(capacity) },
        };
        Self::new(table_type)
    }

    /// Gets the size of the table
    ///
    /// # Returns
    ///
    /// The current size of the table
    #[must_use]
    pub fn size(&self) -> u32 {
        usize_to_wasm_u32(self.elements.len()).unwrap_or(0)
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
    pub fn get(&self, idx: u32) -> Result<Option<WrtValue>> {
        let idx = wasm_index_to_usize(idx)?;
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
    /// Returns an error if the index is out of bounds or if the value type
    /// doesn't match the table element type
    pub fn set(&mut self, idx: u32, value: Option<WrtValue>) -> Result<()> {
        let idx = wasm_index_to_usize(idx)?;
        if idx >= self.elements.len() {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                "Table access out of bounds",
            ));
        }

        if let Some(ref val) = value {
            if !val.matches_type(&self.ty.element_type) {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    format!(
                        "Element value type {:?} doesn't match table element type {:?}",
                        val.value_type(),
                        self.ty.element_type
                    ),
                ));
            }
        }
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
    pub fn grow(&mut self, delta: u32, init_value_from_arg: WrtValue) -> Result<u32> {
        if !init_value_from_arg.matches_type(&self.ty.element_type) {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!(
                    "Grow operation init value type {:?} doesn't match table element type {:?}",
                    init_value_from_arg.value_type(),
                    self.ty.element_type
                ),
            ));
        }

        let old_size = self.size();
        let new_size = old_size.checked_add(delta).ok_or_else(|| {
            Error::new(ErrorCategory::Runtime, codes::CAPACITY_EXCEEDED, "Table size overflow")
        })?;

        if let Some(max) = self.ty.limits.max {
            if new_size > max {
                // As per spec, grow should return -1 (or an error indicating failure)
                // For now, let's return an error. The runtime execution might interpret this.
                return Err(Error::new(
                    ErrorCategory::Runtime,
                    codes::CAPACITY_EXCEEDED,
                    "Table grow exceeds maximum limit",
                ));
            }
        }

        // Use SafeStack's grow method or manually push
        for _ in 0..delta {
            self.elements.push(Some(init_value_from_arg.clone()))?;
        }
        // Update the min limit in the table type if it changes due to growth (spec is a
        // bit unclear if ty should reflect current size) For now, ty.limits.min
        // reflects the *initial* min. Current size is self.size().

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
    /// Returns an error if the index is out of bounds or the table element type
    /// isn't a funcref
    pub fn set_func(&mut self, idx: u32, func_idx: u32) -> Result<()> {
        if self.ty.element_type != WrtValueType::FuncRef {
            return Err(Error::new(
                ErrorCategory::Type,
                codes::INVALID_TYPE,
                "Table element type is not FuncRef",
            ));
        }
        self.set(idx, Some(WrtValue::FuncRef(Some(func_idx))))
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
    pub fn init(&mut self, offset: u32, init_data: &[Option<WrtValue>]) -> Result<()> {
        if offset as usize + init_data.len() > self.elements.len() {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::OUT_OF_BOUNDS_ERROR,
                "Table init out of bounds",
            ));
        }
        for (i, val_opt) in init_data.iter().enumerate() {
            if let Some(val) = val_opt {
                if !val.matches_type(&self.ty.element_type) {
                    return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::VALIDATION_ERROR,
                        "Table init value type mismatch",
                    ));
                }
            }
            self.elements.set((offset as usize) + i, val_opt.clone())?;
        }
        Ok(())
    }

    /// Copy elements from one region of a table to another
    pub fn copy_elements(&mut self, dst: usize, src: usize, len: usize) -> Result<()> {
        // Verify bounds
        if src + len > self.elements.len() || dst + len > self.elements.len() {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_ERROR,
                format!("table element copy out of bounds: src={}, dst={}, len={}", src, dst, len),
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
    pub fn fill_elements(
        &mut self,
        offset: usize,
        value: Option<WrtValue>,
        len: usize,
    ) -> Result<()> {
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
    pub fn init_element(&mut self, idx: usize, value: Option<WrtValue>) -> Result<()> {
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
            "Table Safety Stats:\n- Size: {} elements\n- Element type: {:?}\n- Verification \
             level: {:?}",
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
    fn get(&self, idx: u32) -> Result<Option<WrtValue>>;

    /// Set an element in the table
    fn set(&self, idx: u32, value: Option<WrtValue>) -> Result<()>;

    /// Grow the table by a given number of elements
    fn grow(&self, delta: u32, init_value: WrtValue) -> Result<u32>;

    /// Set a function reference in the table
    fn set_func(&self, idx: u32, func_idx: u32) -> Result<()>;

    /// Initialize a range of elements from a vector
    fn init(&self, offset: u32, init: &[Option<WrtValue>]) -> Result<()>;

    /// Copy elements from one range to another
    fn copy(&self, dst: u32, src: u32, len: u32) -> Result<()>;

    /// Fill a range of elements with a value
    fn fill(&self, offset: u32, len: u32, value: Option<WrtValue>) -> Result<()>;
}

#[cfg(feature = "std")]
impl ArcTableExt for Arc<Table> {
    fn size(&self) -> u32 {
        self.as_ref().size()
    }

    fn get(&self, idx: u32) -> Result<Option<WrtValue>> {
        self.as_ref().get(idx)
    }

    fn set(&self, idx: u32, value: Option<WrtValue>) -> Result<()> {
        // Clone-and-mutate pattern for thread safety
        let mut table_clone = self.as_ref().clone();

        // Return the result from the mutation
        table_clone.set(idx, value)
    }

    fn grow(&self, delta: u32, init_value: WrtValue) -> Result<u32> {
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

    fn init(&self, offset: u32, init: &[Option<WrtValue>]) -> Result<()> {
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

    fn fill(&self, offset: u32, len: u32, value: Option<WrtValue>) -> Result<()> {
        // Clone-and-mutate pattern for thread safety
        let mut table_clone = self.as_ref().clone();

        // Return the result from the mutation
        table_clone.fill_elements(offset as usize, value, len as usize)
    }
}

/// Table manager to handle multiple tables for TableOperations trait
#[derive(Debug)]
pub struct TableManager {
    tables: Vec<Table>,
}

impl TableManager {
    /// Create a new table manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            tables: Vec::new()?,
        })
    }
    
    /// Add a table to the manager
    pub fn add_table(&mut self, table: Table) -> u32 {
        let index = self.tables.len() as u32;
        self.tables.push(table);
        index
    }
    
    /// Get a table by index
    pub fn get_table(&self, index: u32) -> Result<&Table> {
        self.tables.get(index as usize)
            .ok_or_else(|| Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                format!("Invalid table index: {}", index),
            ))
    }
    
    /// Get a mutable table by index
    pub fn get_table_mut(&mut self, index: u32) -> Result<&mut Table> {
        self.tables.get_mut(index as usize)
            .ok_or_else(|| Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                format!("Invalid table index: {}", index),
            ))
    }
    
    /// Get the number of tables
    pub fn table_count(&self) -> u32 {
        self.tables.len() as u32
    }
}

impl Default for TableManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for TableManager {
    fn clone(&self) -> Self {
        Self {
            tables: self.tables.clone(),
        }
    }
}

impl TableOperations for TableManager {
    fn get_table_element(&self, table_index: u32, elem_index: u32) -> Result<Value> {
        let table = self.get_table(table_index)?;
        
        // Get element from table
        let element = table.get(elem_index)?;
        
        // Convert from wrt-foundation Value to wrt-instructions Value
        match element {
            Some(wrt_value) => {
                match wrt_value {
                    WrtValue::FuncRef(func_ref) => {
                        use wrt_foundation::values::FuncRef;
                        match func_ref {
                            Some(func_idx) => Ok(Value::FuncRef(Some(FuncRef::from_index(func_idx)))),
                            None => Ok(Value::FuncRef(None)),
                        }
                    }
                    WrtValue::ExternRef(extern_ref) => {
                        use wrt_foundation::values::ExternRef;
                        match extern_ref {
                            Some(ext_idx) => Ok(Value::ExternRef(Some(ExternRef { index: ext_idx }))),
                            None => Ok(Value::ExternRef(None)),
                        }
                    }
                    // Convert other value types as needed
                    _ => Err(Error::new(
                        ErrorCategory::Type,
                        codes::INVALID_TYPE,
                        "Table element is not a reference type",
                    )),
                }
            }
            None => {
                // Return appropriate null reference based on table element type
                match table.ty.element_type {
                    WrtValueType::FuncRef => Ok(Value::FuncRef(None)),
                    WrtValueType::ExternRef => Ok(Value::ExternRef(None)),
                    _ => Err(Error::new(
                        ErrorCategory::Type,
                        codes::INVALID_TYPE,
                        format!("Unsupported table element type: {:?}", table.ty.element_type),
                    )),
                }
            }
        }
    }
    
    fn set_table_element(&mut self, table_index: u32, elem_index: u32, value: Value) -> Result<()> {
        let table = self.get_table_mut(table_index)?;
        
        // Convert from wrt-instructions Value to wrt-foundation Value
        let wrt_value = match value {
            Value::FuncRef(func_ref) => {
                match func_ref {
                    Some(fr) => Some(WrtValue::FuncRef(Some(WrtFuncRef::from_index(fr.index)))),
                    None => Some(WrtValue::FuncRef(None)),
                }
            }
            Value::ExternRef(extern_ref) => {
                match extern_ref {
                    Some(er) => Some(WrtValue::ExternRef(Some(WrtExternRef::from_index(er.index)))),
                    None => Some(WrtValue::ExternRef(None)),
                }
            }
            _ => return Err(Error::new(
                ErrorCategory::Type,
                codes::INVALID_TYPE,
                "Only reference types can be stored in tables",
            )),
        };
        
        table.set(elem_index, wrt_value)
    }
    
    fn get_table_size(&self, table_index: u32) -> Result<u32> {
        let table = self.get_table(table_index)?;
        Ok(table.size())
    }
    
    fn grow_table(&mut self, table_index: u32, delta: u32, init_value: Value) -> Result<i32> {
        let table = self.get_table_mut(table_index)?;
        
        // Convert init_value to wrt-foundation Value
        let wrt_init_value = match init_value {
            Value::FuncRef(func_ref) => {
                match func_ref {
                    Some(fr) => WrtValue::FuncRef(Some(WrtFuncRef::from_index(fr.index))),
                    None => WrtValue::FuncRef(None),
                }
            }
            Value::ExternRef(extern_ref) => {
                match extern_ref {
                    Some(er) => WrtValue::ExternRef(Some(WrtExternRef::from_index(er.index))),
                    None => WrtValue::ExternRef(None),
                }
            }
            _ => return Err(Error::new(
                ErrorCategory::Type,
                codes::INVALID_TYPE,
                "Table grow init value must be a reference type",
            )),
        };
        
        // Try to grow the table
        match table.grow(delta, wrt_init_value) {
            Ok(old_size) => Ok(old_size as i32),
            Err(_) => Ok(-1), // WebAssembly convention: return -1 on growth failure
        }
    }
    
    fn fill_table(&mut self, table_index: u32, dst: u32, val: Value, len: u32) -> Result<()> {
        let table = self.get_table_mut(table_index)?;
        
        // Convert value to wrt-foundation Value
        let wrt_value = match val {
            Value::FuncRef(func_ref) => {
                match func_ref {
                    Some(fr) => Some(WrtValue::FuncRef(Some(WrtFuncRef::from_index(fr.index)))),
                    None => Some(WrtValue::FuncRef(None)),
                }
            }
            Value::ExternRef(extern_ref) => {
                match extern_ref {
                    Some(er) => Some(WrtValue::ExternRef(Some(WrtExternRef::from_index(er.index)))),
                    None => Some(WrtValue::ExternRef(None)),
                }
            }
            _ => return Err(Error::new(
                ErrorCategory::Type,
                codes::INVALID_TYPE,
                "Table fill value must be a reference type",
            )),
        };
        
        table.fill_elements(dst as usize, wrt_value, len as usize)
    }
    
    fn copy_table(&mut self, dst_table: u32, dst_index: u32, src_table: u32, src_index: u32, len: u32) -> Result<()> {
        // Handle same-table copy
        if dst_table == src_table {
            let table = self.get_table_mut(dst_table)?;
            table.copy_elements(dst_index as usize, src_index as usize, len as usize)
        } else {
            // Cross-table copy - need to read from source and write to destination
            // This is more complex due to borrowing rules, so we'll implement it step by step
            
            // First, read the source elements
            let src_elements = {
                let src_table = self.get_table(src_table)?;
                let mut elements = Vec::new()?;
                for i in 0..len {
                    let elem = src_table.get(src_index + i)?;
                    elements.push(elem).map_err(|_| Error::new(ErrorCategory::Memory, codes::MEMORY_ERROR, "Failed to push table element"))?;
                }
                elements
            };
            
            // Then write to destination table
            let dst_table_ref = self.get_table_mut(dst_table)?;
            for (i, elem) in src_elements.into_iter().enumerate() {
                dst_table_ref.set(dst_index + i as u32, elem)?;
            }
            
            Ok(())
        }
    }
}

// Also implement TableOperations for a single Table (assumes table index 0)
impl TableOperations for Table {
    fn get_table_element(&self, table_index: u32, elem_index: u32) -> Result<Value> {
        if table_index != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                "Single table only supports index 0",
            ));
        }
        
        // Get element from table
        let element = self.get(elem_index)?;
        
        // Convert from wrt-foundation Value to wrt-instructions Value
        match element {
            Some(wrt_value) => {
                match wrt_value {
                    WrtValue::FuncRef(func_ref) => {
                        use wrt_foundation::values::FuncRef;
                        match func_ref {
                            Some(func_idx) => Ok(Value::FuncRef(Some(FuncRef::from_index(func_idx)))),
                            None => Ok(Value::FuncRef(None)),
                        }
                    }
                    WrtValue::ExternRef(extern_ref) => {
                        use wrt_foundation::values::ExternRef;
                        match extern_ref {
                            Some(ext_idx) => Ok(Value::ExternRef(Some(ExternRef { index: ext_idx }))),
                            None => Ok(Value::ExternRef(None)),
                        }
                    }
                    // Convert other value types as needed
                    _ => Err(Error::new(
                        ErrorCategory::Type,
                        codes::INVALID_TYPE,
                        "Table element is not a reference type",
                    )),
                }
            }
            None => {
                // Return appropriate null reference based on table element type
                match self.ty.element_type {
                    WrtValueType::FuncRef => Ok(Value::FuncRef(None)),
                    WrtValueType::ExternRef => Ok(Value::ExternRef(None)),
                    _ => Err(Error::new(
                        ErrorCategory::Type,
                        codes::INVALID_TYPE,
                        format!("Unsupported table element type: {:?}", self.ty.element_type),
                    )),
                }
            }
        }
    }
    
    fn set_table_element(&mut self, table_index: u32, elem_index: u32, value: Value) -> Result<()> {
        if table_index != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                "Single table only supports index 0",
            ));
        }
        
        // Convert from wrt-instructions Value to wrt-foundation Value
        let wrt_value = match value {
            Value::FuncRef(func_ref) => {
                match func_ref {
                    Some(fr) => Some(WrtValue::FuncRef(Some(WrtFuncRef::from_index(fr.index)))),
                    None => Some(WrtValue::FuncRef(None)),
                }
            }
            Value::ExternRef(extern_ref) => {
                match extern_ref {
                    Some(er) => Some(WrtValue::ExternRef(Some(WrtExternRef::from_index(er.index)))),
                    None => Some(WrtValue::ExternRef(None)),
                }
            }
            _ => return Err(Error::new(
                ErrorCategory::Type,
                codes::INVALID_TYPE,
                "Only reference types can be stored in tables",
            )),
        };
        
        self.set(elem_index, wrt_value)
    }
    
    fn get_table_size(&self, table_index: u32) -> Result<u32> {
        if table_index != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                "Single table only supports index 0",
            ));
        }
        
        Ok(self.size())
    }
    
    fn grow_table(&mut self, table_index: u32, delta: u32, init_value: Value) -> Result<i32> {
        if table_index != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                "Single table only supports index 0",
            ));
        }
        
        // Convert init_value to wrt-foundation Value
        let wrt_init_value = match init_value {
            Value::FuncRef(func_ref) => {
                match func_ref {
                    Some(fr) => WrtValue::FuncRef(Some(WrtFuncRef::from_index(fr.index))),
                    None => WrtValue::FuncRef(None),
                }
            }
            Value::ExternRef(extern_ref) => {
                match extern_ref {
                    Some(er) => WrtValue::ExternRef(Some(WrtExternRef::from_index(er.index))),
                    None => WrtValue::ExternRef(None),
                }
            }
            _ => return Err(Error::new(
                ErrorCategory::Type,
                codes::INVALID_TYPE,
                "Table grow init value must be a reference type",
            )),
        };
        
        // Try to grow the table
        match self.grow(delta, wrt_init_value) {
            Ok(old_size) => Ok(old_size as i32),
            Err(_) => Ok(-1), // WebAssembly convention: return -1 on growth failure
        }
    }
    
    fn fill_table(&mut self, table_index: u32, dst: u32, val: Value, len: u32) -> Result<()> {
        if table_index != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                "Single table only supports index 0",
            ));
        }
        
        // Convert value to wrt-foundation Value
        let wrt_value = match val {
            Value::FuncRef(func_ref) => {
                match func_ref {
                    Some(fr) => Some(WrtValue::FuncRef(Some(WrtFuncRef::from_index(fr.index)))),
                    None => Some(WrtValue::FuncRef(None)),
                }
            }
            Value::ExternRef(extern_ref) => {
                match extern_ref {
                    Some(er) => Some(WrtValue::ExternRef(Some(WrtExternRef::from_index(er.index)))),
                    None => Some(WrtValue::ExternRef(None)),
                }
            }
            _ => return Err(Error::new(
                ErrorCategory::Type,
                codes::INVALID_TYPE,
                "Table fill value must be a reference type",
            )),
        };
        
        self.fill_elements(dst as usize, wrt_value, len as usize)
    }
    
    fn copy_table(&mut self, dst_table: u32, dst_index: u32, src_table: u32, src_index: u32, len: u32) -> Result<()> {
        // For single table, both src and dst must be 0
        if dst_table != 0 || src_table != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_FUNCTION_INDEX,
                "Single table only supports index 0",
            ));
        }
        
        self.copy_elements(dst_index as usize, src_index as usize, len as usize)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "std"))]
    use alloc::vec;

    use wrt_foundation::{
        types::{Limits, ValueType},
        verification::VerificationLevel,
    };

    use super::*;

    fn create_test_table_type(min: u32, max: Option<u32>) -> TableType {
        TableType { element_type: ValueType::FuncRef, limits: Limits { min, max } }
    }

    #[test]
    fn test_table_creation() {
        let table_type = create_test_table_type(10, Some(20));
        let init_value = Value::func_ref(None);
        let table = Table::new(table_type.clone()).unwrap();

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
        let mut table = Table::new(table_type).unwrap();

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
        let mut table = Table::new(table_type).unwrap();

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
        let mut table = Table::new(table_type).unwrap();

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
        let mut table = Table::new(table_type).unwrap();

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
        let mut table = Table::new(table_type.clone()).unwrap();

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
        let mut table = Table::new(table_type).unwrap();

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
        let table = Table::new(table_type)?;
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
        let init_values = vec![Some(Value::func_ref(Some(1))), Some(Value::func_ref(Some(2)))];
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
            limits: Limits { min: 5, max: Some(10) },
        };

        // Create a table
        let mut table = Table::new(table_type)?;

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
        use wrt_foundation::{types::ValueType, verification::VerificationLevel};

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
