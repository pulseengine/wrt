use crate::error::{Error, Result};
use crate::types::*;
use crate::values::Value;
use crate::Vec;

/// Represents a WebAssembly table instance
#[derive(Debug)]
pub struct Table {
    /// Table type
    table_type: TableType,
    /// Table elements
    elements: Vec<Option<Value>>,
}

impl Table {
    /// Creates a new table instance
    pub fn new(table_type: TableType) -> Self {
        let initial_size = table_type.min;
        Self {
            table_type,
            elements: {
                let mut v = Vec::with_capacity(initial_size as usize);
                v.resize(initial_size as usize, None);
                v
            },
        }
    }

    /// Returns the table type
    pub fn type_(&self) -> &TableType {
        &self.table_type
    }

    /// Returns the current size
    pub fn size(&self) -> u32 {
        self.elements.len() as u32
    }

    /// Grows the table by the specified number of elements
    pub fn grow(&mut self, delta: u32) -> Result<u32> {
        let old_size = self.size();
        let new_size = old_size
            .checked_add(delta)
            .ok_or_else(|| Error::Execution("Table size overflow".into()))?;

        if new_size > self.table_type.max.unwrap_or(u32::MAX) {
            return Err(Error::Execution("Table size exceeds maximum".into()));
        }

        self.elements.resize(new_size as usize, None);
        Ok(old_size)
    }

    /// Gets an element from the table
    pub fn get(&self, idx: u32) -> Result<Option<Value>> {
        self.check_bounds(idx)?;
        Ok(self.elements[idx as usize].clone())
    }

    /// Sets an element in the table
    pub fn set(&mut self, idx: u32, value: Option<Value>) -> Result<()> {
        self.check_bounds(idx)?;
        self.elements[idx as usize] = value;
        Ok(())
    }

    /// Initializes a range of elements from a vector
    pub fn init(&mut self, offset: u32, init: &[Option<Value>]) -> Result<()> {
        let end = offset
            .checked_add(init.len() as u32)
            .ok_or_else(|| Error::Execution("Table initialization overflow".into()))?;
        self.check_bounds(end - 1)?;

        // Clone each element individually since Option<Value> doesn't implement Copy
        for (i, value) in init.iter().enumerate() {
            self.elements[offset as usize + i] = value.clone();
        }

        Ok(())
    }

    /// Copies elements from one range to another
    pub fn copy(&mut self, dst: u32, src: u32, len: u32) -> Result<()> {
        let dst_end = dst
            .checked_add(len)
            .ok_or_else(|| Error::Execution("Table copy destination overflow".into()))?;
        let src_end = src
            .checked_add(len)
            .ok_or_else(|| Error::Execution("Table copy source overflow".into()))?;
        self.check_bounds(dst_end - 1)?;
        self.check_bounds(src_end - 1)?;

        if dst <= src {
            // Forward copy
            for i in 0..len {
                self.elements[(dst + i) as usize] = self.elements[(src + i) as usize].clone();
            }
        } else {
            // Backward copy
            for i in (0..len).rev() {
                self.elements[(dst + i) as usize] = self.elements[(src + i) as usize].clone();
            }
        }
        Ok(())
    }

    /// Fills a range of elements with a value
    pub fn fill(&mut self, offset: u32, len: u32, value: Option<Value>) -> Result<()> {
        let end = offset
            .checked_add(len)
            .ok_or_else(|| Error::Execution("Table fill overflow".into()))?;
        self.check_bounds(end - 1)?;
        for i in offset..end {
            self.elements[i as usize] = value.clone();
        }
        Ok(())
    }

    /// Checks if a table access is within bounds
    fn check_bounds(&self, idx: u32) -> Result<()> {
        if idx >= self.elements.len() as u32 {
            return Err(Error::Execution("Table access out of bounds".into()));
        }
        Ok(())
    }
}
