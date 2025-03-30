use core::fmt;

use crate::resource::ResourceType;

// Import std when available
#[cfg(feature = "std")]
use std::{boxed::Box, string::String, vec::Vec};

// Import alloc for no_std
#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};

/// Represents a WebAssembly value type
#[derive(Debug, Clone, PartialEq, Copy, Eq, Hash)]
pub enum ValueType {
    /// 32-bit integer
    I32,
    /// 64-bit integer
    I64,
    /// 32-bit float
    F32,
    /// 64-bit float
    F64,
    /// Function reference
    FuncRef,
    /// External reference
    ExternRef,
    /// 128-bit vector for SIMD operations
    V128,
    /// Any reference type (reference types proposal)
    AnyRef,
}

/// Represents a WebAssembly function type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    /// Parameter types
    pub params: Vec<ValueType>,
    /// Result types
    pub results: Vec<ValueType>,
}

/// Represents a WebAssembly table type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableType {
    /// Element type
    pub element_type: ValueType,
    /// Minimum size
    pub min: u32,
    /// Maximum size (optional)
    pub max: Option<u32>,
}

/// Represents a WebAssembly memory type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryType {
    /// Minimum size in pages
    pub min: u32,
    /// Maximum size in pages (optional)
    pub max: Option<u32>,
}

/// Represents a WebAssembly global type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalType {
    /// Content type
    pub content_type: ValueType,
    /// Whether the global is mutable
    pub mutable: bool,
}

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

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
            Self::FuncRef => write!(f, "funcref"),
            Self::ExternRef => write!(f, "externref"),
            Self::V128 => write!(f, "v128"),
            Self::AnyRef => write!(f, "anyref"),
        }
    }
}

impl ValueType {
    /// Returns the size of the value type in bytes
    #[must_use]
    pub const fn size(&self) -> usize {
        match self {
            Self::I32 | Self::F32 => 4,
            Self::I64 | Self::F64 => 8,
            Self::FuncRef | Self::ExternRef | Self::AnyRef => 8,
            Self::V128 => 16,
        }
    }

    /// Returns whether the value type is a reference type
    #[must_use]
    pub const fn is_ref(&self) -> bool {
        matches!(self, Self::FuncRef | Self::ExternRef | Self::AnyRef)
    }

    /// Returns whether the value type is a numeric type
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::I32 | Self::I64 | Self::F32 | Self::F64 | Self::V128
        )
    }

    /// Returns whether the value type is a vector type
    #[must_use]
    pub const fn is_vector(&self) -> bool {
        matches!(self, Self::V128)
    }
}

impl ComponentType {
    /// Creates a new component primitive type from a value type
    #[must_use]
    pub const fn from_value_type(value_type: ValueType) -> Self {
        Self::Primitive(value_type)
    }

    /// Checks if this type is a primitive type
    #[must_use]
    pub const fn is_primitive(&self) -> bool {
        matches!(self, Self::Primitive(_))
    }

    /// Checks if this type is a reference type
    #[must_use]
    pub const fn is_ref(&self) -> bool {
        match self {
            Self::Primitive(value_type) => value_type.is_ref(),
            Self::Resource(_) | Self::Borrowed(_) | Self::Own(_) => true,
            _ => false,
        }
    }

    /// Checks if this type is a numeric type
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

    /// Checks if this type is a record type
    #[must_use]
    pub const fn is_record(&self) -> bool {
        matches!(self, Self::Record(_))
    }

    /// Checks if this type is a tuple type
    #[must_use]
    pub const fn is_tuple(&self) -> bool {
        matches!(self, Self::Tuple(_))
    }

    /// Checks if this type is a list type
    #[must_use]
    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Checks if this type is a flags type
    #[must_use]
    pub const fn is_flags(&self) -> bool {
        matches!(self, Self::Flags(_))
    }

    /// Checks if this type is a variant type
    #[must_use]
    pub const fn is_variant(&self) -> bool {
        matches!(self, Self::Variant(_))
    }

    /// Checks if this type is an enum type
    #[must_use]
    pub const fn is_enum(&self) -> bool {
        matches!(self, Self::Enum(_))
    }

    /// Checks if this type is a union type
    #[must_use]
    pub const fn is_union(&self) -> bool {
        matches!(self, Self::Union(_))
    }

    /// Checks if this type is an option type
    #[must_use]
    pub const fn is_option(&self) -> bool {
        matches!(self, Self::Option(_))
    }

    /// Checks if this type is a result type
    #[must_use]
    pub const fn is_result(&self) -> bool {
        matches!(self, Self::Result { .. })
    }

    /// Checks if this type is a future type
    #[must_use]
    pub const fn is_future(&self) -> bool {
        matches!(self, Self::Future(_))
    }

    /// Checks if this type is a stream type
    #[must_use]
    pub const fn is_stream(&self) -> bool {
        matches!(self, Self::Stream { .. })
    }

    /// Checks if this type is a resource type
    #[must_use]
    pub const fn is_resource(&self) -> bool {
        matches!(self, Self::Resource(_))
    }

    /// Checks if this type is a borrowed type
    #[must_use]
    pub const fn is_borrowed(&self) -> bool {
        matches!(self, Self::Borrowed(_))
    }

    /// Checks if this type is an own type
    #[must_use]
    pub const fn is_own(&self) -> bool {
        matches!(self, Self::Own(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::Value;
    #[cfg(not(feature = "std"))]
    use alloc::vec;

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
        let func_type = FuncType {
            params: vec![ValueType::I32, ValueType::I64],
            results: vec![ValueType::F32],
        };

        assert_eq!(func_type.params.len(), 2);
        assert_eq!(func_type.results.len(), 1);
        assert_eq!(func_type.params[0], ValueType::I32);
        assert_eq!(func_type.params[1], ValueType::I64);
        assert_eq!(func_type.results[0], ValueType::F32);

        // Test cloning
        let cloned = func_type.clone();
        assert_eq!(func_type, cloned);
    }

    #[test]
    fn test_table_type() {
        let table_type = TableType {
            element_type: ValueType::FuncRef,
            min: 1,
            max: Some(10),
        };

        assert_eq!(table_type.element_type, ValueType::FuncRef);
        assert_eq!(table_type.min, 1);
        assert_eq!(table_type.max, Some(10));

        // Test with no maximum
        let unlimited_table = TableType {
            element_type: ValueType::FuncRef,
            min: 0,
            max: None,
        };
        assert_eq!(unlimited_table.max, None);

        // Test cloning
        let cloned = table_type.clone();
        assert_eq!(table_type, cloned);
    }

    #[test]
    fn test_memory_type() {
        let memory_type = MemoryType {
            min: 1,
            max: Some(10),
        };

        assert_eq!(memory_type.min, 1);
        assert_eq!(memory_type.max, Some(10));

        // Test with no maximum
        let unlimited_memory = MemoryType { min: 0, max: None };
        assert_eq!(unlimited_memory.max, None);

        // Test cloning
        let cloned = memory_type.clone();
        assert_eq!(memory_type, cloned);
    }

    #[test]
    fn test_global_type() {
        let global_type = GlobalType {
            content_type: ValueType::I32,
            mutable: true,
        };

        assert_eq!(global_type.content_type, ValueType::I32);
        assert!(global_type.mutable);

        // Test immutable global
        let immutable_global = GlobalType {
            content_type: ValueType::F64,
            mutable: false,
        };
        assert!(!immutable_global.mutable);

        // Test cloning
        let cloned = global_type.clone();
        assert_eq!(global_type, cloned);
    }

    #[test]
    fn test_extern_type() {
        // Test function external type
        let func_type = FuncType {
            params: vec![ValueType::I32],
            results: vec![ValueType::I32],
        };
        let func_extern = ExternType::Function(func_type.clone());
        match &func_extern {
            ExternType::Function(ft) => assert_eq!(ft, &func_type),
            _ => panic!("Expected Function variant"),
        }

        // Test table external type
        let table_type = TableType {
            element_type: ValueType::FuncRef,
            min: 1,
            max: Some(10),
        };
        let table_extern = ExternType::Table(table_type.clone());
        match &table_extern {
            ExternType::Table(tt) => assert_eq!(tt, &table_type),
            _ => panic!("Expected Table variant"),
        }

        // Test memory external type
        let memory_type = MemoryType {
            min: 1,
            max: Some(10),
        };
        let memory_extern = ExternType::Memory(memory_type.clone());
        match &memory_extern {
            ExternType::Memory(mt) => assert_eq!(mt, &memory_type),
            _ => panic!("Expected Memory variant"),
        }

        // Test global external type
        let global_type = GlobalType {
            content_type: ValueType::I32,
            mutable: true,
        };
        let global_extern = ExternType::Global(global_type.clone());
        match &global_extern {
            ExternType::Global(gt) => assert_eq!(gt, &global_type),
            _ => panic!("Expected Global variant"),
        }

        // Test cloning
        let cloned = func_extern.clone();
        assert_eq!(func_extern, cloned);
    }

    #[test]
    fn test_value_type_operations() {
        // Test value type creation and comparison
        assert_eq!(ValueType::I32, ValueType::I32);
        assert_ne!(ValueType::I32, ValueType::I64);
        assert_ne!(ValueType::F32, ValueType::F64);

        // Test default values for types
        assert!(matches!(
            Value::default_for_type(&ValueType::I32),
            Value::I32(0)
        ));
        assert!(matches!(
            Value::default_for_type(&ValueType::I64),
            Value::I64(0)
        ));
        assert!(matches!(
            Value::default_for_type(&ValueType::F32),
            Value::F32(0.0)
        ));
        assert!(matches!(
            Value::default_for_type(&ValueType::F64),
            Value::F64(0.0)
        ));
    }
}
