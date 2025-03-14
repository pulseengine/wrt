use std::fmt;

use crate::{String, Vec};

/// Represents a WebAssembly value type
#[derive(Debug, Clone, PartialEq)]
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
}

/// Represents a WebAssembly function type
#[derive(Debug, Clone, PartialEq)]
pub struct FuncType {
    /// Parameter types
    pub params: Vec<ValueType>,
    /// Result types
    pub results: Vec<ValueType>,
}

/// Represents a WebAssembly table type
#[derive(Debug, Clone, PartialEq)]
pub struct TableType {
    /// Element type
    pub element_type: ValueType,
    /// Minimum size
    pub min: u32,
    /// Maximum size (optional)
    pub max: Option<u32>,
}

/// Represents a WebAssembly memory type
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryType {
    /// Minimum size in pages
    pub min: u32,
    /// Maximum size in pages (optional)
    pub max: Option<u32>,
}

/// Represents a WebAssembly global type
#[derive(Debug, Clone, PartialEq)]
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
}

/// Represents a component model type
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    /// Primitive type
    Primitive(ValueType),
    /// Record type
    Record(Vec<(String, ValueType)>),
    /// Tuple type
    Tuple(Vec<ValueType>),
    /// List type
    List(ValueType),
    /// Flags type
    Flags(Vec<String>),
    /// Variant type
    Variant(Vec<(String, Option<ValueType>)>),
    /// Enum type
    Enum(Vec<String>),
    /// Union type
    Union(Vec<ValueType>),
    /// Option type
    Option(ValueType),
    /// Result type
    Result {
        /// Ok type
        ok: Option<ValueType>,
        /// Error type
        err: Option<ValueType>,
    },
    /// Future type
    Future(ValueType),
    /// Stream type
    Stream {
        /// Element type
        element: ValueType,
        /// End type
        end: Option<ValueType>,
    },
    /// Unknown type
    Unknown,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::I32 => write!(f, "i32"),
            ValueType::I64 => write!(f, "i64"),
            ValueType::F32 => write!(f, "f32"),
            ValueType::F64 => write!(f, "f64"),
            ValueType::FuncRef => write!(f, "funcref"),
            ValueType::ExternRef => write!(f, "externref"),
        }
    }
}

impl ValueType {
    /// Returns the size of the value type in bytes
    pub fn size(&self) -> usize {
        match self {
            ValueType::I32 | ValueType::F32 => 4,
            ValueType::I64 | ValueType::F64 => 8,
            ValueType::FuncRef | ValueType::ExternRef => 8,
        }
    }

    /// Returns whether the value type is a reference type
    pub fn is_ref(&self) -> bool {
        matches!(self, ValueType::FuncRef | ValueType::ExternRef)
    }

    /// Returns whether the value type is a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            ValueType::I32 | ValueType::I64 | ValueType::F32 | ValueType::F64
        )
    }
}

impl ComponentType {
    /// Returns whether the component type is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(self, ComponentType::Primitive(_))
    }

    /// Returns whether the component type is a reference type
    pub fn is_ref(&self) -> bool {
        matches!(
            self,
            ComponentType::Primitive(ValueType::FuncRef | ValueType::ExternRef)
        )
    }

    /// Returns whether the component type is a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            ComponentType::Primitive(
                ValueType::I32 | ValueType::I64 | ValueType::F32 | ValueType::F64
            )
        )
    }

    /// Returns whether the component type is a record type
    pub fn is_record(&self) -> bool {
        matches!(self, ComponentType::Record(_))
    }

    /// Returns whether the component type is a tuple type
    pub fn is_tuple(&self) -> bool {
        matches!(self, ComponentType::Tuple(_))
    }

    /// Returns whether the component type is a list type
    pub fn is_list(&self) -> bool {
        matches!(self, ComponentType::List(_))
    }

    /// Returns whether the component type is a flags type
    pub fn is_flags(&self) -> bool {
        matches!(self, ComponentType::Flags(_))
    }

    /// Returns whether the component type is a variant type
    pub fn is_variant(&self) -> bool {
        matches!(self, ComponentType::Variant(_))
    }

    /// Returns whether the component type is an enum type
    pub fn is_enum(&self) -> bool {
        matches!(self, ComponentType::Enum(_))
    }

    /// Returns whether the component type is a union type
    pub fn is_union(&self) -> bool {
        matches!(self, ComponentType::Union(_))
    }

    /// Returns whether the component type is an option type
    pub fn is_option(&self) -> bool {
        matches!(self, ComponentType::Option(_))
    }

    /// Returns whether the component type is a result type
    pub fn is_result(&self) -> bool {
        matches!(self, ComponentType::Result { .. })
    }

    /// Returns whether the component type is a future type
    pub fn is_future(&self) -> bool {
        matches!(self, ComponentType::Future(_))
    }

    /// Returns whether the component type is a stream type
    pub fn is_stream(&self) -> bool {
        matches!(self, ComponentType::Stream { .. })
    }
}
