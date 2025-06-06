//! WIT (WebAssembly Interface Types) parser with simplified types

#[cfg(feature = "std")]
use std::collections::BTreeMap;
#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std")))]
use std::{boxed::Box, collections::BTreeMap, vec::Vec};

use core::fmt;

use wrt_foundation::{
    BoundedVec, BoundedString,
    bounded::MAX_GENERATIVE_TYPES,
    NoStdProvider,
};

use wrt_error::Error;

// Include trait implementations  
#[path = "wit_parser_traits.rs"]
mod wit_parser_traits;

/// Type aliases for WIT parser using a fixed memory provider

/// Bounded string for WIT identifiers and names (64 bytes max)
pub type WitBoundedString = BoundedString<64, NoStdProvider<1024>>;
/// Small bounded string for WIT parameters and short names (32 bytes max)
pub type WitBoundedStringSmall = BoundedString<32, NoStdProvider<1024>>;
/// Large bounded string for WIT error messages and long strings (128 bytes max)
pub type WitBoundedStringLarge = BoundedString<128, NoStdProvider<1024>>;

/// A WIT world definition containing imports, exports, and type definitions
#[derive(Debug, Clone, PartialEq)]
pub struct WitWorld {
    /// World name
    pub name: WitBoundedString,
    /// Imported items
    pub imports: BoundedVec<WitImport, MAX_GENERATIVE_TYPES, NoStdProvider<1024>>,
    /// Exported items
    pub exports: BoundedVec<WitExport, MAX_GENERATIVE_TYPES, NoStdProvider<1024>>,
    /// Type definitions
    pub types: BoundedVec<WitTypeDef, MAX_GENERATIVE_TYPES, NoStdProvider<1024>>,
}

/// A WIT interface definition containing functions and types
#[derive(Debug, Clone, PartialEq)]
pub struct WitInterface {
    /// Interface name
    pub name: WitBoundedString,
    /// Functions in this interface
    pub functions: BoundedVec<WitFunction, MAX_GENERATIVE_TYPES, NoStdProvider<1024>>,
    /// Type definitions in this interface
    pub types: BoundedVec<WitTypeDef, MAX_GENERATIVE_TYPES, NoStdProvider<1024>>,
}

/// A WIT import statement
#[derive(Debug, Clone, PartialEq)]
pub struct WitImport {
    /// Import name
    pub name: WitBoundedString,
    /// Imported item
    pub item: WitItem,
}

/// A WIT export statement
#[derive(Debug, Clone, PartialEq)]
pub struct WitExport {
    /// Export name
    pub name: WitBoundedString,
    /// Exported item
    pub item: WitItem,
}

/// A WIT item that can be imported or exported
#[derive(Debug, Clone, PartialEq)]
pub enum WitItem {
    /// Function item
    Function(WitFunction),
    /// Interface item
    Interface(WitInterface),
    /// Type item
    Type(WitType),
    /// Instance item
    Instance(WitInstance),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitFunction {
    pub name: WitBoundedString,
    pub params: BoundedVec<WitParam, 32, NoStdProvider<1024>>,
    pub results: BoundedVec<WitResult, 16, NoStdProvider<1024>>,
    pub is_async: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitParam {
    pub name: WitBoundedStringSmall,
    pub ty: WitType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitResult {
    pub name: Option<WitBoundedStringSmall>,
    pub ty: WitType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitInstance {
    pub interface_name: WitBoundedString,
    pub args: BoundedVec<WitInstanceArg, 32, NoStdProvider<1024>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitInstanceArg {
    pub name: WitBoundedStringSmall,
    pub value: WitValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WitValue {
    Type(WitType),
    Instance(WitBoundedString),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitTypeDef {
    pub name: WitBoundedString,
    pub ty: WitType,
    pub is_resource: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WitType {
    /// Basic primitive types
    Bool,
    U8,
    U16,
    U32,
    U64,
    S8,
    S16,
    S32,
    S64,
    F32,
    F64,
    Char,
    String,
    
    /// Compound types
    List(Box<WitType>),
    Option(Box<WitType>),
    Result {
        ok: Option<Box<WitType>>,
        err: Option<Box<WitType>>,
    },
    Tuple(BoundedVec<WitType, 16, NoStdProvider<1024>>),
    Record(WitRecord),
    Variant(WitVariant),
    Enum(WitEnum),
    Flags(WitFlags),
    
    /// Resource types
    Own(WitBoundedString),
    Borrow(WitBoundedString),
    
    /// Named type reference
    Named(WitBoundedString),
    
    /// Stream and Future for async support
    Stream(Box<WitType>),
    Future(Box<WitType>),
}

/// A WIT record type with named fields
#[derive(Debug, Clone, PartialEq)]
pub struct WitRecord {
    /// The fields of the record
    pub fields: BoundedVec<WitRecordField, 32, NoStdProvider<1024>>,
}

/// A field in a WIT record
#[derive(Debug, Clone, PartialEq)]
pub struct WitRecordField {
    /// The name of the field
    pub name: WitBoundedStringSmall,
    /// The type of the field
    pub ty: WitType,
}

/// A WIT variant type with multiple cases
#[derive(Debug, Clone, PartialEq)]
pub struct WitVariant {
    /// The cases of the variant
    pub cases: BoundedVec<WitVariantCase, 32, NoStdProvider<1024>>,
}

/// A case in a WIT variant
#[derive(Debug, Clone, PartialEq)]
pub struct WitVariantCase {
    /// The name of the case
    pub name: WitBoundedStringSmall,
    /// The optional type of the case
    pub ty: Option<WitType>,
}

/// A WIT enumeration type
#[derive(Debug, Clone, PartialEq)]
pub struct WitEnum {
    /// The enumeration cases
    pub cases: BoundedVec<WitBoundedStringSmall, 64, NoStdProvider<1024>>,
}

/// A WIT flags type for bitwise operations
#[derive(Debug, Clone, PartialEq)]
pub struct WitFlags {
    /// The individual flags
    pub flags: BoundedVec<WitBoundedStringSmall, 64, NoStdProvider<1024>>,
}

/// A parser for WIT (WebAssembly Interface Types) source code
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are part of future parser state implementation
pub struct WitParser {
    current_position: usize,
    type_definitions: BTreeMap<WitBoundedString, WitType>,
    provider: NoStdProvider<1024>,
}

/// Errors that can occur during WIT parsing
#[derive(Debug, Clone, PartialEq)]
pub enum WitParseError {
    /// Unexpected end of input
    UnexpectedEnd,
    /// Invalid syntax encountered
    InvalidSyntax(WitBoundedStringLarge),
    /// Unknown type referenced
    UnknownType(WitBoundedString),
    /// Too many items for bounded collections
    TooManyItems,
    /// Invalid identifier format
    InvalidIdentifier(WitBoundedString),
    /// Duplicate definition found
    DuplicateDefinition(WitBoundedString),
}

impl From<WitParseError> for Error {
    fn from(err: WitParseError) -> Self {
        match err {
            WitParseError::UnexpectedEnd => Error::parse_error("Unexpected end of WIT input"),
            WitParseError::InvalidSyntax(_) => Error::parse_error("Invalid WIT syntax"),
            WitParseError::UnknownType(_) => Error::parse_error("Unknown WIT type"),
            WitParseError::TooManyItems => Error::parse_error("Too many WIT items"),
            WitParseError::InvalidIdentifier(_) => Error::parse_error("Invalid WIT identifier"),
            WitParseError::DuplicateDefinition(_) => Error::parse_error("Duplicate WIT definition"),
        }
    }
}

impl WitParser {
    /// Create a new WIT parser
    pub fn new() -> Self {
        Self {
            current_position: 0,
            type_definitions: BTreeMap::new(),
            provider: NoStdProvider::default(),
        }
    }

    /// Parse a WIT world definition from source code
    pub fn parse_world(&mut self, source: &str) -> Result<WitWorld, WitParseError> {
        let mut world = WitWorld {
            name: BoundedString::from_str("", self.provider.clone()).unwrap_or_default(),
            imports: BoundedVec::new(self.provider.clone()).unwrap_or_default(),
            exports: BoundedVec::new(self.provider.clone()).unwrap_or_default(),
            types: BoundedVec::new(self.provider.clone()).unwrap_or_default(),
        };

        #[cfg(feature = "std")]
        {
            let lines: Vec<&str> = source.lines().collect();
            let mut i = 0;

            while i < lines.len() {
                let line = lines[i].trim();
                
                if line.is_empty() || line.starts_with("//") {
                    i += 1;
                    continue;
                }

                if line.starts_with("world ") {
                    let name = self.extract_identifier(line, "world ")?;
                    world.name = name;
                } else if line.starts_with("import ") {
                    let import = self.parse_import(line)?;
                    world.imports.push(import)
                        .map_err(|_| WitParseError::TooManyItems)?;
                } else if line.starts_with("export ") {
                    let export = self.parse_export(line)?;
                    world.exports.push(export)
                        .map_err(|_| WitParseError::TooManyItems)?;
                } else if line.starts_with("type ") {
                    let type_def = self.parse_type_def(line)?;
                    world.types.push(type_def)
                        .map_err(|_| WitParseError::TooManyItems)?;
                }

                i += 1;
            }
        }

        Ok(world)
    }

    /// Parse a WIT interface definition from source code
    pub fn parse_interface(&mut self, source: &str) -> Result<WitInterface, WitParseError> {
        let mut interface = WitInterface {
            name: BoundedString::from_str("", self.provider.clone()).unwrap_or_default(),
            functions: BoundedVec::new(self.provider.clone()).unwrap_or_default(),
            types: BoundedVec::new(self.provider.clone()).unwrap_or_default(),
        };

        #[cfg(feature = "std")]
        {
            let lines: Vec<&str> = source.lines().collect();
            let mut i = 0;

            while i < lines.len() {
                let line = lines[i].trim();
                
                if line.is_empty() || line.starts_with("//") {
                    i += 1;
                    continue;
                }

                if line.starts_with("interface ") {
                    let name = self.extract_identifier(line, "interface ")?;
                    interface.name = name;
                } else if line.contains(":") && (line.contains("func") || line.contains("->")) {
                    let function = self.parse_function(line)?;
                    interface.functions.push(function)
                        .map_err(|_| WitParseError::TooManyItems)?;
                } else if line.starts_with("type ") {
                    let type_def = self.parse_type_def(line)?;
                    interface.types.push(type_def)
                        .map_err(|_| WitParseError::TooManyItems)?;
                }

                i += 1;
            }
        }

        Ok(interface)
    }

    fn parse_import(&mut self, line: &str) -> Result<WitImport, WitParseError> {
        #[cfg(feature = "std")]
        {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                return Err(WitParseError::InvalidSyntax(
                    BoundedString::from_str("Invalid import syntax", self.provider.clone()).unwrap()
                ));
            }

            let name = BoundedString::from_str(parts[1], self.provider.clone())
                .map_err(|_| WitParseError::InvalidIdentifier(
                    BoundedString::from_str(parts[1], self.provider.clone()).unwrap_or_default()
                ))?;

            let item_type = parts[2];
            let item = match item_type {
                "func" => {
                    let func = self.parse_function(line)?;
                    WitItem::Function(func)
                }
                _ => {
                    return Err(WitParseError::InvalidSyntax(
                        BoundedString::from_str("Unsupported import type", self.provider.clone()).unwrap()
                    ));
                }
            };

            Ok(WitImport { name, item })
        }
        
        #[cfg(not(any(feature = "std", )))]
        Err(WitParseError::InvalidSyntax(
            BoundedString::from_str("Parsing not supported in no_std", self.provider.clone()).unwrap()
        ))
    }

    fn parse_export(&mut self, line: &str) -> Result<WitExport, WitParseError> {
        #[cfg(feature = "std")]
        {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                return Err(WitParseError::InvalidSyntax(
                    BoundedString::from_str("Invalid export syntax", self.provider.clone()).unwrap()
                ));
            }

            let name = BoundedString::from_str(parts[1], self.provider.clone())
                .map_err(|_| WitParseError::InvalidIdentifier(
                    BoundedString::from_str(parts[1], self.provider.clone()).unwrap_or_default()
                ))?;

            let item_type = parts[2];
            let item = match item_type {
                "func" => {
                    let func = self.parse_function(line)?;
                    WitItem::Function(func)
                }
                _ => {
                    return Err(WitParseError::InvalidSyntax(
                        BoundedString::from_str("Unsupported export type", self.provider.clone()).unwrap()
                    ));
                }
            };

            Ok(WitExport { name, item })
        }
        
        #[cfg(not(any(feature = "std", )))]
        Err(WitParseError::InvalidSyntax(
            BoundedString::from_str("Parsing not supported in no_std", self.provider.clone()).unwrap()
        ))
    }

    fn parse_function(&mut self, line: &str) -> Result<WitFunction, WitParseError> {
        let mut function = WitFunction {
            name: BoundedString::from_str("", self.provider.clone()).unwrap_or_default(),
            params: BoundedVec::new(self.provider.clone()).unwrap_or_default(),
            results: BoundedVec::new(self.provider.clone()).unwrap_or_default(),
            is_async: line.contains("async"),
        };

        #[cfg(feature = "std")]
        if let Some(colon_pos) = line.find(':') {
            let name_part = &line[..colon_pos].trim();
            let parts: Vec<&str> = name_part.split_whitespace().collect();
            
            if let Some(name) = parts.last() {
                function.name = BoundedString::from_str(name, self.provider.clone())
                    .map_err(|_| WitParseError::InvalidIdentifier(
                        BoundedString::from_str(name, self.provider.clone()).unwrap_or_default()
                    ))?;
            }
        }

        Ok(function)
    }

    fn parse_type_def(&mut self, line: &str) -> Result<WitTypeDef, WitParseError> {
        #[cfg(feature = "std")]
        {
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() < 3 {
                return Err(WitParseError::InvalidSyntax(
                    BoundedString::from_str("Invalid type definition", self.provider.clone()).unwrap()
                ));
            }

            let name = BoundedString::from_str(parts[1], self.provider.clone())
                .map_err(|_| WitParseError::InvalidIdentifier(
                    BoundedString::from_str(parts[1], self.provider.clone()).unwrap_or_default()
                ))?;

            let type_str = parts[2];
            let is_resource = type_str.starts_with("resource");
            
            let ty = self.parse_type(type_str)?;

            Ok(WitTypeDef {
                name: name.clone(),
                ty: ty.clone(),
                is_resource,
            })
        }
        
        #[cfg(not(any(feature = "std", )))]
        Err(WitParseError::InvalidSyntax(
            BoundedString::from_str("Parsing not supported in no_std", self.provider.clone()).unwrap()
        ))
    }

    fn parse_type(&mut self, type_str: &str) -> Result<WitType, WitParseError> {
        let type_str = type_str.trim();
        
        match type_str {
            "bool" => Ok(WitType::Bool),
            "u8" => Ok(WitType::U8),
            "u16" => Ok(WitType::U16),
            "u32" => Ok(WitType::U32),
            "u64" => Ok(WitType::U64),
            "s8" => Ok(WitType::S8),
            "s16" => Ok(WitType::S16),
            "s32" => Ok(WitType::S32),
            "s64" => Ok(WitType::S64),
            "f32" => Ok(WitType::F32),
            "f64" => Ok(WitType::F64),
            "char" => Ok(WitType::Char),
            "string" => Ok(WitType::String),
            _ => {
                #[cfg(feature = "std")]
                {
                    if type_str.starts_with("list<") && type_str.ends_with(">") {
                        let inner = &type_str[5..type_str.len()-1];
                        let inner_type = self.parse_type(inner)?;
                        Ok(WitType::List(Box::new(inner_type)))
                    } else if type_str.starts_with("option<") && type_str.ends_with(">") {
                        let inner = &type_str[7..type_str.len()-1];
                        let inner_type = self.parse_type(inner)?;
                        Ok(WitType::Option(Box::new(inner_type)))
                    } else if type_str.starts_with("stream<") && type_str.ends_with(">") {
                        let inner = &type_str[7..type_str.len()-1];
                        let inner_type = self.parse_type(inner)?;
                        Ok(WitType::Stream(Box::new(inner_type)))
                    } else if type_str.starts_with("future<") && type_str.ends_with(">") {
                        let inner = &type_str[7..type_str.len()-1];
                        let inner_type = self.parse_type(inner)?;
                        Ok(WitType::Future(Box::new(inner_type)))
                    } else {
                        let name = BoundedString::from_str(type_str, self.provider.clone())
                            .map_err(|_| WitParseError::InvalidIdentifier(
                                BoundedString::from_str(type_str, self.provider.clone()).unwrap_or_default()
                            ))?;
                        Ok(WitType::Named(name))
                    }
                }
                
                #[cfg(not(any(feature = "std", )))]
                {
                    let name = BoundedString::from_str(type_str, self.provider.clone())
                        .map_err(|_| WitParseError::InvalidIdentifier(
                            BoundedString::from_str(type_str, self.provider.clone()).unwrap_or_default()
                        ))?;
                    Ok(WitType::Named(name))
                }
            }
        }
    }

    fn extract_identifier(&self, line: &str, prefix: &str) -> Result<WitBoundedString, WitParseError> {
        let remaining = line.strip_prefix(prefix)
            .ok_or_else(|| WitParseError::InvalidSyntax(
                BoundedString::from_str("Missing prefix", self.provider.clone()).unwrap()
            ))?;
        
        let identifier = remaining.split_whitespace().next()
            .ok_or_else(|| WitParseError::InvalidSyntax(
                BoundedString::from_str("Missing identifier", self.provider.clone()).unwrap()
            ))?;

        BoundedString::from_str(identifier, self.provider.clone())
            .map_err(|_| WitParseError::InvalidIdentifier(
                BoundedString::from_str(identifier, self.provider.clone()).unwrap_or_default()
            ))
    }

    #[cfg(feature = "std")]
    /// Convert a WIT type to a WebAssembly value type
    pub fn convert_to_valtype(&self, wit_type: &WitType) -> Result<crate::types::ValueType, Error> {
        match wit_type {
            WitType::Bool | WitType::U8 | WitType::U16 | WitType::U32 | WitType::U64 |
            WitType::S8 | WitType::S16 | WitType::S32 | WitType::S64 => {
                Ok(crate::types::ValueType::I32) // WIT integers map to i32 in core wasm
            },
            WitType::F32 => Ok(crate::types::ValueType::F32),
            WitType::F64 => Ok(crate::types::ValueType::F64),
            WitType::Char | WitType::String => {
                Ok(crate::types::ValueType::I32) // Strings/chars are represented as pointers
            },
            WitType::List(_) | WitType::Option(_) => {
                // Complex types are represented as references in core wasm
                Ok(crate::types::ValueType::I32)
            },
            _ => Err(Error::parse_error("Unsupported WIT type conversion")),
        }
    }
}

impl Default for WitParser {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for WitParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WitParseError::UnexpectedEnd => write!(f, "Unexpected end of input"),
            WitParseError::InvalidSyntax(msg) => {
                write!(f, "Invalid syntax: {}", msg.as_str().unwrap_or("<invalid>"))
            }
            WitParseError::UnknownType(name) => {
                write!(f, "Unknown type: {}", name.as_str().unwrap_or("<invalid>"))
            }
            WitParseError::TooManyItems => write!(f, "Too many items"),
            WitParseError::InvalidIdentifier(name) => {
                write!(f, "Invalid identifier: {}", name.as_str().unwrap_or("<invalid>"))
            }
            WitParseError::DuplicateDefinition(name) => {
                write!(f, "Duplicate definition: {}", name.as_str().unwrap_or("<invalid>"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wit_parser_creation() {
        let parser = WitParser::new();
        assert_eq!(parser.current_position, 0);
        assert_eq!(parser.type_definitions.len(), 0);
    }

    #[test]
    fn test_parse_basic_types() {
        let mut parser = WitParser::new();
        
        assert_eq!(parser.parse_type("bool").unwrap(), WitType::Bool);
        assert_eq!(parser.parse_type("u32").unwrap(), WitType::U32);
        assert_eq!(parser.parse_type("string").unwrap(), WitType::String);
        assert_eq!(parser.parse_type("f64").unwrap(), WitType::F64);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_parse_compound_types() {
        let mut parser = WitParser::new();
        
        let list_type = parser.parse_type("list<u32>").unwrap();
        match list_type {
            WitType::List(inner) => assert_eq!(*inner, WitType::U32),
            _ => panic!("Expected list type"),
        }

        let option_type = parser.parse_type("option<string>").unwrap();
        match option_type {
            WitType::Option(inner) => assert_eq!(*inner, WitType::String),
            _ => panic!("Expected option type"),
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_parse_async_types() {
        let mut parser = WitParser::new();
        
        let stream_type = parser.parse_type("stream<u8>").unwrap();
        match stream_type {
            WitType::Stream(inner) => assert_eq!(*inner, WitType::U8),
            _ => panic!("Expected stream type"),
        }

        let future_type = parser.parse_type("future<string>").unwrap();
        match future_type {
            WitType::Future(inner) => assert_eq!(*inner, WitType::String),
            _ => panic!("Expected future type"),
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_parse_simple_world() {
        let mut parser = WitParser::new();
        let source = r#"
            world test-world {
                import test-func: func()
                export result-func: func() -> u32
            }
        "#;

        let world = parser.parse_world(source);
        assert!(world.is_ok());
        
        let world = world.unwrap();
        assert_eq!(world.name.as_str().unwrap(), "test-world");
        
        // Import BoundedCapacity trait for len() method
        use wrt_foundation::traits::BoundedCapacity;
        assert_eq!(world.imports.len(), 1);
        assert_eq!(world.exports.len(), 1);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_convert_to_valtype() {
        let parser = WitParser::new();
        
        assert_eq!(parser.convert_to_valtype(&WitType::Bool).unwrap(), crate::types::ValueType::I32);
        assert_eq!(parser.convert_to_valtype(&WitType::U32).unwrap(), crate::types::ValueType::I32);
        assert_eq!(parser.convert_to_valtype(&WitType::String).unwrap(), crate::types::ValueType::I32);
        
        let list_wit = WitType::List(Box::new(WitType::U32));
        let list_val = parser.convert_to_valtype(&list_wit).unwrap();
        assert_eq!(list_val, crate::types::ValueType::I32); // Lists are represented as pointers
    }
}