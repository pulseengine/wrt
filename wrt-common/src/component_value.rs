//! WebAssembly Component Model value types
//!
//! This module defines the runtime value types used in WebAssembly Component Model
//! implementations.

#[cfg(feature = "std")]
use std::{collections::HashMap, string::String, vec::Vec};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{
    boxed::Box,
    collections::BTreeMap as HashMap,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::component_type::ValType;
use wrt_error::Result;

/// A Component Model value used at runtime
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValue {
    /// Boolean value
    Bool(bool),
    /// Signed 8-bit integer
    S8(i8),
    /// Unsigned 8-bit integer
    U8(u8),
    /// Signed 16-bit integer
    S16(i16),
    /// Unsigned 16-bit integer
    U16(u16),
    /// Signed 32-bit integer
    S32(i32),
    /// Unsigned 32-bit integer
    U32(u32),
    /// Signed 64-bit integer
    S64(i64),
    /// Unsigned 64-bit integer
    U64(u64),
    /// 32-bit floating point
    F32(f32),
    /// 64-bit floating point
    F64(f64),
    /// Unicode character
    Char(char),
    /// UTF-8 string
    String(String),
    /// List of values
    List(Vec<ComponentValue>),
    /// Record with named fields
    Record(HashMap<String, ComponentValue>),
    /// Variant with case name and optional value
    Variant {
        case: u32,
        value: Option<Box<ComponentValue>>,
    },
    /// Tuple of values
    Tuple(Vec<ComponentValue>),
    /// Flags (set of named boolean flags)
    Flags(HashMap<String, bool>),
    /// Enumeration value
    Enum(u32),
    /// Option type
    Option(Option<Box<ComponentValue>>),
    /// Result type with ok value
    Result(Result<Option<Box<ComponentValue>>, Option<Box<ComponentValue>>>),
    /// Resource handle (owned)
    Own(u32),
    /// Resource handle (borrowed)
    Borrow(u32),
}

// Implement Eq for ComponentValue
// Note: This means we can't use floating point equality comparisons directly
impl Eq for ComponentValue {}

impl ComponentValue {
    /// Get the type of this component value
    pub fn get_type(&self) -> ValType {
        match self {
            Self::Bool(_) => ValType::Bool,
            Self::S8(_) => ValType::S8,
            Self::U8(_) => ValType::U8,
            Self::S16(_) => ValType::S16,
            Self::U16(_) => ValType::U16,
            Self::S32(_) => ValType::S32,
            Self::U32(_) => ValType::U32,
            Self::S64(_) => ValType::S64,
            Self::U64(_) => ValType::U64,
            Self::F32(_) => ValType::F32,
            Self::F64(_) => ValType::F64,
            Self::Char(_) => ValType::Char,
            Self::String(_) => ValType::String,
            Self::List(items) => {
                if let Some(first) = items.first() {
                    ValType::List(Box::new(first.get_type()))
                } else {
                    // Empty list, use a placeholder type
                    ValType::List(Box::new(ValType::Bool))
                }
            }
            Self::Record(fields) => {
                let mut field_types = Vec::new();
                for (name, value) in fields {
                    field_types.push((name.clone(), value.get_type()));
                }
                ValType::Record(field_types)
            }
            Self::Variant { case, value } => {
                let cases = vec![(case.to_string(), value.as_ref().map(|v| v.get_type()))];
                ValType::Variant(cases)
            }
            Self::Tuple(items) => {
                let item_types = items.iter().map(|v| v.get_type()).collect();
                ValType::Tuple(item_types)
            }
            Self::Flags(flags) => {
                let names = flags.keys().cloned().collect();
                ValType::Flags(names)
            }
            Self::Enum(variant) => {
                let variants = vec![variant.to_string()];
                ValType::Enum(variants)
            }
            Self::Option(opt) => {
                if let Some(val) = opt {
                    ValType::Option(Box::new(val.get_type()))
                } else {
                    ValType::Option(Box::new(ValType::Bool)) // Placeholder
                }
            }
            Self::Result(val) => ValType::Result(Box::new(if let Ok(ok) = val {
                if let Some(v) = ok {
                    v.get_type()
                } else {
                    ValType::Bool // Placeholder for None
                }
            } else {
                ValType::Bool // Placeholder for Err
            })),
            Self::Own(idx) => ValType::Own(*idx),
            Self::Borrow(idx) => ValType::Borrow(*idx),
        }
    }

    /// Check if this value matches the specified type
    pub fn matches_type(&self, value_type: &ValType) -> bool {
        match (self, value_type) {
            // Simple primitive type checks
            (ComponentValue::Bool(_), ValType::Bool) => true,
            (ComponentValue::S8(_), ValType::S8) => true,
            (ComponentValue::U8(_), ValType::U8) => true,
            (ComponentValue::S16(_), ValType::S16) => true,
            (ComponentValue::U16(_), ValType::U16) => true,
            (ComponentValue::S32(_), ValType::S32) => true,
            (ComponentValue::U32(_), ValType::U32) => true,
            (ComponentValue::S64(_), ValType::S64) => true,
            (ComponentValue::U64(_), ValType::U64) => true,
            (ComponentValue::F32(_), ValType::F32) => true,
            (ComponentValue::F64(_), ValType::F64) => true,
            (ComponentValue::Char(_), ValType::Char) => true,
            (ComponentValue::String(_), ValType::String) => true,

            // Complex type checks
            (ComponentValue::List(items), ValType::List(list_type)) => {
                items.iter().all(|item| item.matches_type(list_type))
            }

            (ComponentValue::Record(fields), ValType::Record(record_types)) => {
                // Check if all fields in the record type are present in the value
                // and that their types match
                if fields.len() != record_types.len() {
                    return false;
                }

                for (field_name, field_type) in record_types {
                    if let Some(field_value) = fields.get(field_name) {
                        if !field_value.matches_type(field_type) {
                            return false;
                        }
                    } else {
                        return false; // Missing field
                    }
                }

                true
            }

            (ComponentValue::Variant { case, value }, ValType::Variant(cases)) => {
                // Find the case in the variant type
                if let Some((_, case_type)) =
                    cases.iter().find(|(name, _)| name == &case.to_string())
                {
                    // Check if the value matches the case type
                    match (value, case_type) {
                        (Some(value), Some(ty)) => value.matches_type(ty),
                        (None, None) => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }

            (ComponentValue::Tuple(items), ValType::Tuple(item_types)) => {
                // Check if tuple lengths match
                if items.len() != item_types.len() {
                    return false;
                }

                // Check if each item matches its corresponding type
                items
                    .iter()
                    .zip(item_types.iter())
                    .all(|(item, item_type)| item.matches_type(item_type))
            }

            (ComponentValue::Flags(flags), ValType::Flags(flag_names)) => {
                // Check if all flag names are present in the value
                if flags.len() != flag_names.len() {
                    return false;
                }

                for name in flag_names {
                    if !flags.contains_key(name) {
                        return false;
                    }
                }

                true
            }

            (ComponentValue::Enum(value), ValType::Enum(variants)) => {
                variants.contains(&value.to_string())
            }

            (ComponentValue::Option(value), ValType::Option(option_type)) => match value {
                Some(val) => val.matches_type(option_type),
                None => true,
            },

            (ComponentValue::Result(val), ValType::Result(result_type)) => match val {
                Ok(val) => {
                    if let Some(v) = val {
                        v.matches_type(result_type)
                    } else {
                        true
                    }
                }
                Err(_) => true,
            },

            (ComponentValue::Own(handle), ValType::Own(id)) => handle == id,
            (ComponentValue::Borrow(handle), ValType::Borrow(id)) => handle == id,

            // All other combinations don't match
            _ => false,
        }
    }
}
