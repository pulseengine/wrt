use core::fmt;

// Re-export types from wrt-runtime
pub use wrt_runtime::{FuncType, GlobalType, MemoryType, TableType};
// Import from wrt-types
pub use wrt_types::types::{Limits, ValueType};

use crate::error::kinds;
use crate::error::{Error, Result};
use crate::module::Module;
use crate::resource::ResourceType;

// Import std when available
#[cfg(feature = "std")]
use std::{boxed::Box, string::String, vec::Vec};

// Import alloc for no_std
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};

/// Represents a WebAssembly external type
#[derive(Debug, Clone, PartialEq)]
pub enum ExternType {
    /// Function type
    Function(FuncType),
    /// Table type
    Table(TableType),
    /// Memory type
    Memory(MemoryType),
    /// Global type
    Global(GlobalType),
    /// Resource type (Component Model)
    Resource(ResourceType),
    /// Instance type (Component Model)
    Instance(InstanceType),
    /// Component type (Component Model)
    Component(ComponentTypeRef),
}

/// Represents a component model type
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    /// Primitive type
    Primitive(ValueType),
    /// Record type
    Record(Vec<(String, Box<ComponentType>)>),
    /// Tuple type
    Tuple(Vec<Box<ComponentType>>),
    /// List type
    List(Box<ComponentType>),
    /// Flags type
    Flags(Vec<String>),
    /// Variant type
    Variant(Vec<(String, Option<Box<ComponentType>>)>),
    /// Enum type
    Enum(Vec<String>),
    /// Union type
    Union(Vec<Box<ComponentType>>),
    /// Option type
    Option(Box<ComponentType>),
    /// Result type
    Result {
        /// Ok type
        ok: Option<Box<ComponentType>>,
        /// Error type
        err: Option<Box<ComponentType>>,
    },
    /// Future type
    Future(Box<ComponentType>),
    /// Stream type
    Stream {
        /// Element type
        element: Box<ComponentType>,
        /// End type
        end: Option<Box<ComponentType>>,
    },
    /// Resource type
    Resource(ResourceType),
    /// Borrowed type
    Borrowed(Box<ComponentType>),
    /// Own type
    Own(Box<ComponentType>),
    /// Handle type
    Handle(u32),
    /// Type reference (for recursive types)
    TypeRef(u32),
    /// Unknown type
    Unknown,
}

/// Component function type
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentFuncType {
    /// Parameter types
    pub params: Vec<(String, ComponentType)>,
    /// Result types
    pub results: ComponentType,
}

/// Component module type
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentModuleType {
    /// Module imports
    pub imports: Vec<(String, ExternType)>,
    /// Module exports
    pub exports: Vec<(String, ExternType)>,
}

/// Component instance type
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceType {
    /// Instance exports
    pub exports: Vec<(String, ExternType)>,
}

/// Reference to a component type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentTypeRef {
    /// Type index
    pub type_idx: u32,
}

/// Represents a component model resource type with a name and version
pub struct ComponentResourceType {
    /// The name of the resource type
    pub name: String,
    /// The version of the resource type
    pub version: u32,
}

/// Represents a WebAssembly block type for control flow instructions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    /// No values are returned
    Empty,
    /// A single value of the specified type is returned
    Type(ValueType),
    /// A single value of the specified type is returned (alternative version)
    Value(ValueType),
    /// Multiple values are returned according to the function type
    FuncType(FuncType),
    /// Reference to a function type by index
    TypeIndex(u32),
}

impl ComponentType {
    /// Creates a new component primitive type from a value type
    #[must_use]
    pub const fn from_value_type(value_type: ValueType) -> Self {
        Self::Primitive(value_type)
    }

    /// Returns whether the component type is a primitive type
    #[must_use]
    pub const fn is_primitive(&self) -> bool {
        matches!(self, Self::Primitive(_))
    }

    /// Returns whether the component type is a reference type
    #[must_use]
    pub const fn is_ref(&self) -> bool {
        match self {
            Self::Primitive(value_type) => match value_type {
                ValueType::FuncRef | ValueType::ExternRef => true,
                _ => false,
            },
            Self::Resource(_) => true,
            Self::Borrowed(_) | Self::Own(_) => true,
            _ => false,
        }
    }

    /// Returns whether the component type is a numeric type
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        match self {
            Self::Primitive(value_type) => match value_type {
                ValueType::I32 | ValueType::I64 | ValueType::F32 | ValueType::F64 => true,
                _ => false,
            },
            _ => false,
        }
    }

    /// Returns whether the component type is a record type
    #[must_use]
    pub const fn is_record(&self) -> bool {
        matches!(self, Self::Record(_))
    }

    /// Returns whether the component type is a tuple type
    #[must_use]
    pub const fn is_tuple(&self) -> bool {
        matches!(self, Self::Tuple(_))
    }

    /// Returns whether the component type is a list type
    #[must_use]
    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Returns whether the component type is a flags type
    #[must_use]
    pub const fn is_flags(&self) -> bool {
        matches!(self, Self::Flags(_))
    }

    /// Returns whether the component type is a variant type
    #[must_use]
    pub const fn is_variant(&self) -> bool {
        matches!(self, Self::Variant(_))
    }

    /// Returns whether the component type is an enum type
    #[must_use]
    pub const fn is_enum(&self) -> bool {
        matches!(self, Self::Enum(_))
    }

    /// Returns whether the component type is a union type
    #[must_use]
    pub const fn is_union(&self) -> bool {
        matches!(self, Self::Union(_))
    }

    /// Returns whether the component type is an option type
    #[must_use]
    pub const fn is_option(&self) -> bool {
        matches!(self, Self::Option(_))
    }

    /// Returns whether the component type is a result type
    #[must_use]
    pub const fn is_result(&self) -> bool {
        matches!(self, Self::Result { .. })
    }

    /// Returns whether the component type is a future type
    #[must_use]
    pub const fn is_future(&self) -> bool {
        matches!(self, Self::Future(_))
    }

    /// Returns whether the component type is a stream type
    #[must_use]
    pub const fn is_stream(&self) -> bool {
        matches!(self, Self::Stream { .. })
    }

    /// Returns whether the component type is a resource type
    #[must_use]
    pub const fn is_resource(&self) -> bool {
        matches!(self, Self::Resource(_))
    }

    /// Returns whether the component type is a borrowed type
    #[must_use]
    pub const fn is_borrowed(&self) -> bool {
        matches!(self, Self::Borrowed(_))
    }

    /// Returns whether the component type is an own type
    #[must_use]
    pub const fn is_own(&self) -> bool {
        matches!(self, Self::Own(_))
    }
}

impl BlockType {
    /// Resolves the parameter and result types for a block type
    pub fn resolve_types(&self, module: &Module) -> Result<(Vec<ValueType>, Vec<ValueType>)> {
        match self {
            Self::Empty => Ok((Vec::new(), Vec::new())),
            Self::Type(value_type) | Self::Value(value_type) => Ok((Vec::new(), vec![*value_type])),
            Self::FuncType(func_type) => Ok((func_type.params.clone(), func_type.results.clone())),
            Self::TypeIndex(type_idx) => match module.get_function_type(*type_idx) {
                Some(func_type) => Ok((func_type.params.clone(), func_type.results.clone())),
                None => Err(Error::new(kinds::ValidationError(format!(
                    "Function type not found: {type_idx}"
                )))),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_equality() {
        assert_eq!(ValueType::I32, ValueType::I32);
        assert_eq!(ValueType::I64, ValueType::I64);
        assert_eq!(ValueType::F32, ValueType::F32);
        assert_eq!(ValueType::F64, ValueType::F64);
        assert_eq!(ValueType::FuncRef, ValueType::FuncRef);
        assert_eq!(ValueType::ExternRef, ValueType::ExternRef);
        assert_ne!(ValueType::I32, ValueType::I64);
        assert_ne!(ValueType::F32, ValueType::F64);
        assert_ne!(ValueType::FuncRef, ValueType::ExternRef);
    }

    #[test]
    fn test_func_type() {
        let func_type = FuncType::new(
            vec![ValueType::I32, ValueType::I64],
            vec![ValueType::F32, ValueType::F64],
        );
        assert_eq!(func_type.params[0], ValueType::I32);
        assert_eq!(func_type.params[1], ValueType::I64);
        assert_eq!(func_type.results[0], ValueType::F32);
        assert_eq!(func_type.results[1], ValueType::F64);
    }

    #[test]
    fn test_table_type() {
        let table_type = TableType {
            limits: Limits {
                min: 10,
                max: Some(20),
            },
            element_type: ValueType::FuncRef,
        };
        assert_eq!(table_type.limits.min, 10);
        assert_eq!(table_type.limits.max, Some(20));
        assert_eq!(table_type.element_type, ValueType::FuncRef);

        let table_type2 = TableType {
            limits: Limits { min: 10, max: None },
            element_type: ValueType::FuncRef,
        };
        assert_eq!(table_type2.limits.min, 10);
        assert_eq!(table_type2.limits.max, None);
        assert_eq!(table_type2.element_type, ValueType::FuncRef);
    }

    #[test]
    fn test_memory_type() {
        let memory_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        assert_eq!(memory_type.limits.min, 1);
        assert_eq!(memory_type.limits.max, Some(2));

        let memory_type2 = MemoryType {
            limits: Limits { min: 1, max: None },
        };
        assert_eq!(memory_type2.limits.min, 1);
        assert_eq!(memory_type2.limits.max, None);
    }

    #[test]
    fn test_global_type() {
        let global_type = GlobalType {
            value_type: ValueType::I32,
            mutable: true,
        };
        assert_eq!(global_type.value_type, ValueType::I32);
        assert!(global_type.mutable);

        let global_type2 = GlobalType {
            value_type: ValueType::I64,
            mutable: false,
        };
        assert_eq!(global_type2.value_type, ValueType::I64);
        assert!(!global_type2.mutable);
    }

    #[test]
    fn test_extern_type() {
        let func_type = FuncType::new(
            vec![ValueType::I32, ValueType::I64],
            vec![ValueType::F32, ValueType::F64],
        );
        let extern_type = ExternType::Function(func_type.clone());
        match extern_type {
            ExternType::Function(ft) => {
                assert_eq!(ft.params[0], ValueType::I32);
                assert_eq!(ft.params[1], ValueType::I64);
                assert_eq!(ft.results[0], ValueType::F32);
                assert_eq!(ft.results[1], ValueType::F64);
            }
            _ => panic!("Expected Function extern type"),
        }

        let table_type = TableType {
            limits: Limits {
                min: 10,
                max: Some(20),
            },
            element_type: ValueType::FuncRef,
        };
        let extern_type2 = ExternType::Table(table_type.clone());
        match extern_type2 {
            ExternType::Table(tt) => {
                assert_eq!(tt.limits.min, 10);
                assert_eq!(tt.limits.max, Some(20));
                assert_eq!(tt.element_type, ValueType::FuncRef);
            }
            _ => panic!("Expected Table extern type"),
        }

        let memory_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let extern_type3 = ExternType::Memory(memory_type.clone());
        match extern_type3 {
            ExternType::Memory(mt) => {
                assert_eq!(mt.limits.min, 1);
                assert_eq!(mt.limits.max, Some(2));
            }
            _ => panic!("Expected Memory extern type"),
        }

        let global_type = GlobalType {
            value_type: ValueType::I32,
            mutable: true,
        };
        let extern_type4 = ExternType::Global(global_type.clone());
        match extern_type4 {
            ExternType::Global(gt) => {
                assert_eq!(gt.value_type, ValueType::I32);
                assert!(gt.mutable);
            }
            _ => panic!("Expected Global extern type"),
        }
    }

    #[test]
    fn test_value_type_operations() {
        // These tests can be simplified since we're now reexporting from wrt-types
        assert!(matches!(ValueType::I32, ValueType::I32));
        assert!(matches!(ValueType::I64, ValueType::I64));
        assert!(matches!(ValueType::F32, ValueType::F32));
        assert!(matches!(ValueType::F64, ValueType::F64));
        assert!(matches!(ValueType::FuncRef, ValueType::FuncRef));
        assert!(matches!(ValueType::ExternRef, ValueType::ExternRef));
    }
}
