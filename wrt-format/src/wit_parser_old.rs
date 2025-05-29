#[cfg(feature = "std")]
use std::collections::BTreeMap;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{boxed::Box, collections::BTreeMap, vec::Vec, string::String};
#[cfg(not(any(feature = "alloc", feature = "std")))]
use wrt_foundation::{BoundedVec as Vec, no_std_hashmap::SimpleHashMap as BTreeMap};

// Box alternative for no_std environments - use a simple wrapper
#[cfg(not(any(feature = "alloc", feature = "std")))]
type Box<T> = T;

use core::fmt;

use wrt_foundation::{
    BoundedVec, BoundedString,
    bounded::MAX_GENERATIVE_TYPES,
    prelude::*,
    MemoryProvider, NoStdProvider,
};

use crate::component::ValType;

use wrt_error::{Error, ErrorCategory};

#[derive(Debug, Clone, PartialEq)]
pub struct WitWorld<P: MemoryProvider + Default + Clone + PartialEq + Eq = NoStdProvider<1024>> {
    pub name: BoundedString<64, P>,
    pub imports: BoundedVec<WitImport<P>, MAX_GENERATIVE_TYPES, P>,
    pub exports: BoundedVec<WitExport<P>, MAX_GENERATIVE_TYPES, P>,
    pub types: BoundedVec<WitTypeDef, MAX_GENERATIVE_TYPES>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitInterface {
    pub name: BoundedString<64>,
    pub functions: BoundedVec<WitFunction, MAX_GENERATIVE_TYPES>,
    pub types: BoundedVec<WitTypeDef, MAX_GENERATIVE_TYPES>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitImport {
    pub name: BoundedString<64>,
    pub item: WitItem,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitExport {
    pub name: BoundedString<64>,
    pub item: WitItem,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WitItem {
    Function(WitFunction),
    Interface(WitInterface),
    Type(WitType),
    Instance(WitInstance),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitFunction {
    pub name: BoundedString<64>,
    pub params: BoundedVec<WitParam, 32>,
    pub results: BoundedVec<WitResult, 16>,
    pub is_async: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitParam {
    pub name: BoundedString<32>,
    pub ty: WitType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitResult {
    pub name: Option<BoundedString<32>>,
    pub ty: WitType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitInstance {
    pub interface_name: BoundedString<64>,
    pub args: BoundedVec<WitInstanceArg, 32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitInstanceArg {
    pub name: BoundedString<32>,
    pub value: WitValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WitValue {
    Type(WitType),
    Instance(BoundedString<64>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitTypeDef {
    pub name: BoundedString<64>,
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
    Tuple(BoundedVec<WitType, 16>),
    Record(WitRecord),
    Variant(WitVariant),
    Enum(WitEnum),
    Flags(WitFlags),
    
    /// Resource types
    Own(BoundedString<64>),
    Borrow(BoundedString<64>),
    
    /// Named type reference
    Named(BoundedString<64>),
    
    /// Stream and Future for async support
    Stream(Box<WitType>),
    Future(Box<WitType>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitRecord {
    pub fields: BoundedVec<WitRecordField, 32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitRecordField {
    pub name: BoundedString<32>,
    pub ty: WitType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitVariant {
    pub cases: BoundedVec<WitVariantCase, 32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitVariantCase {
    pub name: BoundedString<32>,
    pub ty: Option<WitType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitEnum {
    pub cases: BoundedVec<BoundedString<32>, 64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WitFlags {
    pub flags: BoundedVec<BoundedString<32>, 64>,
}

#[derive(Debug, Clone)]
pub struct WitParser {
    current_position: usize,
    type_definitions: BTreeMap<BoundedString<64>, WitType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WitParseError {
    UnexpectedEnd,
    InvalidSyntax(BoundedString<128>),
    UnknownType(BoundedString<64>),
    TooManyItems,
    InvalidIdentifier(BoundedString<64>),
    DuplicateDefinition(BoundedString<64>),
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
    pub fn new() -> Self {
        Self {
            current_position: 0,
            type_definitions: BTreeMap::new(),
        }
    }

    pub fn parse_world(&mut self, source: &str) -> Result<WitWorld, WitParseError> {
        let mut world = WitWorld {
            name: BoundedString::new(),
            imports: BoundedVec::new(),
            exports: BoundedVec::new(),
            types: BoundedVec::new(),
        };

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

        Ok(world)
    }

    pub fn parse_interface(&mut self, source: &str) -> Result<WitInterface, WitParseError> {
        let mut interface = WitInterface {
            name: BoundedString::new(),
            functions: BoundedVec::new(),
            types: BoundedVec::new(),
        };

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

        Ok(interface)
    }

    fn parse_import(&mut self, line: &str) -> Result<WitImport, WitParseError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(WitParseError::InvalidSyntax(
                BoundedString::from_str("Invalid import syntax").unwrap()
            ));
        }

        let name = BoundedString::from_str(parts[1])
            .map_err(|_| WitParseError::InvalidIdentifier(
                BoundedString::from_str(parts[1]).unwrap_or_default()
            ))?;

        let item_type = parts[2];
        let item = match item_type {
            "func" => {
                let func = self.parse_function(line)?;
                WitItem::Function(func)
            }
            _ => {
                return Err(WitParseError::InvalidSyntax(
                    BoundedString::from_str("Unsupported import type").unwrap()
                ));
            }
        };

        Ok(WitImport { name, item })
    }

    fn parse_export(&mut self, line: &str) -> Result<WitExport, WitParseError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(WitParseError::InvalidSyntax(
                BoundedString::from_str("Invalid export syntax").unwrap()
            ));
        }

        let name = BoundedString::from_str(parts[1])
            .map_err(|_| WitParseError::InvalidIdentifier(
                BoundedString::from_str(parts[1]).unwrap_or_default()
            ))?;

        let item_type = parts[2];
        let item = match item_type {
            "func" => {
                let func = self.parse_function(line)?;
                WitItem::Function(func)
            }
            _ => {
                return Err(WitParseError::InvalidSyntax(
                    BoundedString::from_str("Unsupported export type").unwrap()
                ));
            }
        };

        Ok(WitExport { name, item })
    }

    fn parse_function(&mut self, line: &str) -> Result<WitFunction, WitParseError> {
        let mut function = WitFunction {
            name: BoundedString::new(),
            params: BoundedVec::new(),
            results: BoundedVec::new(),
            is_async: line.contains("async"),
        };

        if let Some(colon_pos) = line.find(':') {
            let name_part = &line[..colon_pos].trim();
            let parts: Vec<&str> = name_part.split_whitespace().collect();
            
            if let Some(name) = parts.last() {
                function.name = BoundedString::from_str(name)
                    .map_err(|_| WitParseError::InvalidIdentifier(
                        BoundedString::from_str(name).unwrap_or_default()
                    ))?;
            }
        }

        Ok(function)
    }

    fn parse_type_def(&mut self, line: &str) -> Result<WitTypeDef, WitParseError> {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() < 3 {
            return Err(WitParseError::InvalidSyntax(
                BoundedString::from_str("Invalid type definition").unwrap()
            ));
        }

        let name = BoundedString::from_str(parts[1])
            .map_err(|_| WitParseError::InvalidIdentifier(
                BoundedString::from_str(parts[1]).unwrap_or_default()
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
                    let name = BoundedString::from_str(type_str)
                        .map_err(|_| WitParseError::InvalidIdentifier(
                            BoundedString::from_str(type_str).unwrap_or_default()
                        ))?;
                    Ok(WitType::Named(name))
                }
            }
        }
    }

    fn extract_identifier(&self, line: &str, prefix: &str) -> Result<BoundedString<64>, WitParseError> {
        let remaining = line.strip_prefix(prefix)
            .ok_or_else(|| WitParseError::InvalidSyntax(
                BoundedString::from_str("Missing prefix").unwrap()
            ))?;
        
        let identifier = remaining.split_whitespace().next()
            .ok_or_else(|| WitParseError::InvalidSyntax(
                BoundedString::from_str("Missing identifier").unwrap()
            ))?;

        BoundedString::from_str(identifier)
            .map_err(|_| WitParseError::InvalidIdentifier(
                BoundedString::from_str(identifier).unwrap_or_default()
            ))
    }

    pub fn convert_to_valtype(&self, wit_type: &WitType) -> Result<ValType, Error> {
        match wit_type {
            WitType::Bool => Ok(ValType::Bool),
            WitType::U8 => Ok(ValType::U8),
            WitType::U16 => Ok(ValType::U16),
            WitType::U32 => Ok(ValType::U32),
            WitType::U64 => Ok(ValType::U64),
            WitType::S8 => Ok(ValType::S8),
            WitType::S16 => Ok(ValType::S16),
            WitType::S32 => Ok(ValType::S32),
            WitType::S64 => Ok(ValType::S64),
            WitType::F32 => Ok(ValType::F32),
            WitType::F64 => Ok(ValType::F64),
            WitType::Char => Ok(ValType::Char),
            WitType::String => Ok(ValType::String),
            WitType::List(inner) => {
                let inner_valtype = self.convert_to_valtype(inner)?;
                Ok(ValType::List(Box::new(inner_valtype)))
            }
            WitType::Option(inner) => {
                let inner_valtype = self.convert_to_valtype(inner)?;
                Ok(ValType::Option(Box::new(inner_valtype)))
            }
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
            WitParseError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
            WitParseError::UnknownType(name) => write!(f, "Unknown type: {}", name),
            WitParseError::TooManyItems => write!(f, "Too many items"),
            WitParseError::InvalidIdentifier(name) => write!(f, "Invalid identifier: {}", name),
            WitParseError::DuplicateDefinition(name) => write!(f, "Duplicate definition: {}", name),
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
        assert_eq!(world.name.as_str(), "test-world");
        assert_eq!(world.imports.len(), 1);
        assert_eq!(world.exports.len(), 1);
    }

    #[test]
    fn test_convert_to_valtype() {
        let parser = WitParser::new();
        
        assert_eq!(parser.convert_to_valtype(&WitType::Bool).unwrap(), ValType::Bool);
        assert_eq!(parser.convert_to_valtype(&WitType::U32).unwrap(), ValType::U32);
        assert_eq!(parser.convert_to_valtype(&WitType::String).unwrap(), ValType::String);
        
        let list_wit = WitType::List(Box::new(WitType::U32));
        let list_val = parser.convert_to_valtype(&list_wit).unwrap();
        match list_val {
            ValType::List(inner) => assert_eq!(*inner, ValType::U32),
            _ => panic!("Expected list ValType"),
        }
    }
}