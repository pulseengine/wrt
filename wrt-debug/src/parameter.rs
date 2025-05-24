/// Parameter and type information support
/// Provides the missing 2% for parameter information

use crate::strings::DebugString;
use wrt_foundation::{
    bounded::{BoundedVec, MAX_DWARF_ABBREV_CACHE},
    NoStdProvider,
};

/// Basic type information
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BasicType {
    /// Void type
    Void,
    /// Boolean type
    Bool,
    /// Signed integer (size in bytes)
    SignedInt(u8),
    /// Unsigned integer (size in bytes)
    UnsignedInt(u8),
    /// Floating point (size in bytes)
    Float(u8),
    /// Pointer to another type
    Pointer,
    /// Reference to another type
    Reference,
    /// Array type
    Array,
    /// Structure type
    Struct,
    /// Unknown/complex type
    Unknown,
}

impl BasicType {
    /// Parse from DWARF encoding and size
    pub fn from_encoding(encoding: u8, byte_size: u8) -> Self {
        match encoding {
            0x00 => Self::Void,           // DW_ATE_address
            0x01 => Self::Pointer,         // DW_ATE_address
            0x02 => Self::Bool,            // DW_ATE_boolean
            0x04 => Self::Float(byte_size), // DW_ATE_float
            0x05 => Self::SignedInt(byte_size), // DW_ATE_signed
            0x07 => Self::UnsignedInt(byte_size), // DW_ATE_unsigned
            0x08 => Self::UnsignedInt(byte_size), // DW_ATE_unsigned_char
            0x10 => Self::Reference,       // DW_ATE_UTF (reused for references)
            _ => Self::Unknown,
        }
    }

    /// Get a string representation of the type
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Void => "void",
            Self::Bool => "bool",
            Self::SignedInt(1) => "i8",
            Self::SignedInt(2) => "i16",
            Self::SignedInt(4) => "i32",
            Self::SignedInt(8) => "i64",
            Self::UnsignedInt(1) => "u8",
            Self::UnsignedInt(2) => "u16",
            Self::UnsignedInt(4) => "u32",
            Self::UnsignedInt(8) => "u64",
            Self::Float(4) => "f32",
            Self::Float(8) => "f64",
            Self::Pointer => "ptr",
            Self::Reference => "ref",
            Self::Array => "array",
            Self::Struct => "struct",
            _ => "unknown",
        }
    }
}

/// Function parameter information
#[derive(Debug, Clone)]
pub struct Parameter<'a> {
    /// Parameter name
    pub name: Option<DebugString<'a>>,
    /// Parameter type
    pub param_type: BasicType,
    /// Source file index where declared
    pub file_index: u16,
    /// Source line where declared
    pub line: u32,
    /// Parameter position (0-based)
    pub position: u16,
    /// Is this a variadic parameter?
    pub is_variadic: bool,
}

/// Collection of parameters for a function
#[derive(Debug)]
pub struct ParameterList<'a> {
    /// Parameters in order
    parameters: BoundedVec<Parameter<'a>, MAX_DWARF_ABBREV_CACHE, NoStdProvider>,
}

impl<'a> ParameterList<'a> {
    /// Create a new empty parameter list
    pub fn new() -> Self {
        Self {
            parameters: BoundedVec::new(NoStdProvider),
        }
    }

    /// Add a parameter to the list
    pub fn add_parameter(&mut self, param: Parameter<'a>) -> Result<(), ()> {
        self.parameters.push(param).map_err(|_| ())
    }

    /// Get all parameters
    pub fn parameters(&self) -> &[Parameter<'a>] {
        self.parameters.as_slice()
    }

    /// Get parameter count
    pub fn count(&self) -> usize {
        self.parameters.len()
    }

    /// Check if function has variadic parameters
    pub fn is_variadic(&self) -> bool {
        self.parameters.iter().any(|p| p.is_variadic)
    }

    /// Get parameter by position
    pub fn get_by_position(&self, position: u16) -> Option<&Parameter<'a>> {
        self.parameters.iter().find(|p| p.position == position)
    }

    /// Format parameter list for display
    pub fn display<F>(&self, mut writer: F) -> Result<(), core::fmt::Error>
    where
        F: FnMut(&str) -> Result<(), core::fmt::Error>,
    {
        writer("(")?;
        
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                writer(", ")?;
            }
            
            // Parameter name
            if let Some(ref name) = param.name {
                writer(name.as_str())?;
                writer(": ")?;
            }
            
            // Parameter type
            writer(param.param_type.type_name())?;
            
            if param.is_variadic {
                writer("...")?;
            }
        }
        
        writer(")")?;
        Ok(())
    }
}

/// Inline function information
#[derive(Debug, Clone)]
pub struct InlinedFunction<'a> {
    /// Name of the inlined function
    pub name: Option<DebugString<'a>>,
    /// Abstract origin (reference to original function)
    pub abstract_origin: u32,
    /// Low PC (start address in parent)
    pub low_pc: u32,
    /// High PC (end address in parent)
    pub high_pc: u32,
    /// Call site file
    pub call_file: u16,
    /// Call site line
    pub call_line: u32,
    /// Call site column
    pub call_column: u16,
    /// Depth of inlining (0 = directly inlined into parent)
    pub depth: u8,
}

/// Collection of inlined functions
#[derive(Debug)]
pub struct InlinedFunctions<'a> {
    /// Inlined function entries
    entries: BoundedVec<InlinedFunction<'a>, MAX_DWARF_ABBREV_CACHE, NoStdProvider>,
}

impl<'a> InlinedFunctions<'a> {
    /// Create new inlined functions collection
    pub fn new() -> Self {
        Self {
            entries: BoundedVec::new(NoStdProvider),
        }
    }

    /// Add an inlined function
    pub fn add(&mut self, func: InlinedFunction<'a>) -> Result<(), ()> {
        self.entries.push(func).map_err(|_| ())
    }

    /// Find all inlined functions containing the given PC
    pub fn find_at_pc(&self, pc: u32) -> impl Iterator<Item = &InlinedFunction<'a>> {
        self.entries.iter()
            .filter(move |f| pc >= f.low_pc && pc < f.high_pc)
    }

    /// Get all inlined functions
    pub fn all(&self) -> &[InlinedFunction<'a>] {
        self.entries.as_slice()
    }

    /// Check if any functions are inlined at this PC
    pub fn has_inlined_at(&self, pc: u32) -> bool {
        self.find_at_pc(pc).next().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_type_parsing() {
        assert_eq!(BasicType::from_encoding(0x02, 1), BasicType::Bool);
        assert_eq!(BasicType::from_encoding(0x05, 4), BasicType::SignedInt(4));
        assert_eq!(BasicType::from_encoding(0x07, 8), BasicType::UnsignedInt(8));
        assert_eq!(BasicType::from_encoding(0x04, 4), BasicType::Float(4));
    }

    #[test]
    fn test_type_names() {
        assert_eq!(BasicType::SignedInt(4).type_name(), "i32");
        assert_eq!(BasicType::UnsignedInt(8).type_name(), "u64");
        assert_eq!(BasicType::Float(4).type_name(), "f32");
        assert_eq!(BasicType::Bool.type_name(), "bool");
    }

    #[test]
    fn test_parameter_list_display() {
        let mut params = ParameterList::new();
        
        // Add some test parameters
        let param1 = Parameter {
            name: None,
            param_type: BasicType::SignedInt(4),
            file_index: 0,
            line: 0,
            position: 0,
            is_variadic: false,
        };
        
        let param2 = Parameter {
            name: None,
            param_type: BasicType::Pointer,
            file_index: 0,
            line: 0,
            position: 1,
            is_variadic: false,
        };
        
        params.add_parameter(param1).unwrap();
        params.add_parameter(param2).unwrap();
        
        let mut output = String::new();
        params.display(|s| {
            output.push_str(s);
            Ok(())
        }).unwrap();
        
        assert_eq!(output, "(i32, ptr)");
    }

    #[test]
    fn test_inlined_functions() {
        let mut inlined = InlinedFunctions::new();
        
        let func = InlinedFunction {
            name: None,
            abstract_origin: 0x100,
            low_pc: 0x1000,
            high_pc: 0x1100,
            call_file: 1,
            call_line: 42,
            call_column: 8,
            depth: 0,
        };
        
        inlined.add(func).unwrap();
        
        // Test PC lookup
        assert!(inlined.has_inlined_at(0x1050));
        assert!(!inlined.has_inlined_at(0x2000));
        
        let found: Vec<_> = inlined.find_at_pc(0x1050).collect();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].call_line, 42);
    }
}