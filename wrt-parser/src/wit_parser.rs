//! WIT (WebAssembly Interface Types) parser
//!
//! This module provides comprehensive parsing of WIT (WebAssembly Interface Types)
//! with streaming architecture, ASIL-D memory compliance, and modernized AST interpretation.

use wrt_error::{Error, ErrorCategory, Result, codes};
use crate::bounded_types::{SimpleBoundedVec, SimpleBoundedString};

/// Maximum number of types in a WIT document
pub const MAX_WIT_TYPES: usize = 512;

/// Maximum number of functions in an interface
pub const MAX_WIT_FUNCTIONS: usize = 256;

/// Maximum number of parameters in a function
pub const MAX_WIT_PARAMS: usize = 32;

/// Maximum number of results in a function
pub const MAX_WIT_RESULTS: usize = 16;

/// Maximum number of imports in a world
pub const MAX_WIT_IMPORTS: usize = 128;

/// Maximum number of exports in a world
pub const MAX_WIT_EXPORTS: usize = 128;

/// Maximum identifier length in WIT
pub const MAX_WIT_IDENTIFIER_LEN: usize = 128;

/// Maximum string literal length
pub const MAX_WIT_STRING_LEN: usize = 1024;

/// WIT document representing a complete .wit file
#[derive(Debug, Clone)]
pub struct WitDocument {
    /// Package declaration (optional)
    pub package: Option<WitPackage>,
    
    /// Interfaces defined in this document
    pub interfaces: SimpleBoundedVec<WitInterface, MAX_WIT_TYPES>,
    
    /// Worlds defined in this document
    pub worlds: SimpleBoundedVec<WitWorld, MAX_WIT_TYPES>,
    
    /// Use statements for importing from other packages
    pub uses: SimpleBoundedVec<WitUse, MAX_WIT_IMPORTS>,
}

/// WIT package declaration
#[derive(Debug, Clone)]
pub struct WitPackage {
    /// Package namespace (e.g., "wasi")
    pub namespace: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    
    /// Package name (e.g., "cli")
    pub name: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    
    /// Package version (e.g., "0.2.0")
    pub version: Option<SimpleBoundedString<32>>,
}

/// WIT interface definition
#[derive(Debug, Clone)]
pub struct WitInterface {
    /// Interface name
    pub name: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    
    /// Functions in this interface
    pub functions: SimpleBoundedVec<WitFunction, MAX_WIT_FUNCTIONS>,
    
    /// Type definitions in this interface
    pub types: SimpleBoundedVec<WitTypeDef, MAX_WIT_TYPES>,
    
    /// Use statements for this interface
    pub uses: SimpleBoundedVec<WitUse, MAX_WIT_IMPORTS>,
}

/// WIT world definition
#[derive(Debug, Clone)]
pub struct WitWorld {
    /// World name
    pub name: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    
    /// Imported items
    pub imports: SimpleBoundedVec<WitImport, MAX_WIT_IMPORTS>,
    
    /// Exported items
    pub exports: SimpleBoundedVec<WitExport, MAX_WIT_EXPORTS>,
    
    /// Type definitions in this world
    pub types: SimpleBoundedVec<WitTypeDef, MAX_WIT_TYPES>,
}

/// WIT use statement for importing items
#[derive(Debug, Clone)]
pub struct WitUse {
    /// Source package/interface reference
    pub source: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    
    /// Items being imported
    pub items: SimpleBoundedVec<WitUseItem, 32>,
}

/// Individual item in a use statement
#[derive(Debug, Clone)]
pub struct WitUseItem {
    /// Original name in source
    pub source_name: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    
    /// Local alias (if different from source)
    pub local_name: Option<SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>>,
}

/// WIT import statement
#[derive(Debug, Clone)]
pub struct WitImport {
    /// Import name/key
    pub name: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    
    /// Imported item
    pub item: WitItem,
}

/// WIT export statement
#[derive(Debug, Clone)]
pub struct WitExport {
    /// Export name/key
    pub name: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    
    /// Exported item
    pub item: WitItem,
}

/// WIT item that can be imported or exported
#[derive(Debug, Clone)]
pub enum WitItem {
    /// Function item
    Function(WitFunction),
    
    /// Interface item
    Interface(WitInterface),
    
    /// Type item
    Type(WitType),
    
    /// Instance of an interface
    Instance {
        /// Interface reference
        interface: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    },
}

/// WIT function definition
#[derive(Debug, Clone)]
pub struct WitFunction {
    /// Function name
    pub name: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    
    /// Function parameters
    pub params: SimpleBoundedVec<WitParam, MAX_WIT_PARAMS>,
    
    /// Function results
    pub results: SimpleBoundedVec<WitResult, MAX_WIT_RESULTS>,
    
    /// Whether function is async
    pub is_async: bool,
    
    /// Whether function is static (no implicit self parameter)
    pub is_static: bool,
}

/// WIT function parameter
#[derive(Debug, Clone)]
pub struct WitParam {
    /// Parameter name
    pub name: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    
    /// Parameter type
    pub ty: WitType,
}

/// WIT function result
#[derive(Debug, Clone)]
pub struct WitResult {
    /// Optional result name
    pub name: Option<SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>>,
    
    /// Result type
    pub ty: WitType,
}

/// WIT type definition
#[derive(Debug, Clone)]
pub struct WitTypeDef {
    /// Type name
    pub name: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    
    /// Type definition body
    pub ty: WitTypeDefKind,
}

/// Kind of type definition
#[derive(Debug, Clone)]
pub enum WitTypeDefKind {
    /// Record type (struct-like)
    Record(WitRecord),
    
    /// Variant type (union-like)
    Variant(WitVariant),
    
    /// Enum type (simple union of names)
    Enum(WitEnum),
    
    /// Flags type (bitflags)
    Flags(WitFlags),
    
    /// Resource type
    Resource(WitResource),
    
    /// Type alias
    Type(WitType),
}

/// WIT record type
#[derive(Debug, Clone)]
pub struct WitRecord {
    /// Record fields
    pub fields: SimpleBoundedVec<WitRecordField, MAX_WIT_TYPES>,
}

/// WIT record field
#[derive(Debug, Clone)]
pub struct WitRecordField {
    /// Field name
    pub name: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    
    /// Field type
    pub ty: WitType,
}

/// WIT variant type
#[derive(Debug, Clone)]
pub struct WitVariant {
    /// Variant cases
    pub cases: SimpleBoundedVec<WitVariantCase, MAX_WIT_TYPES>,
}

/// WIT variant case
#[derive(Debug, Clone)]
pub struct WitVariantCase {
    /// Case name
    pub name: SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>,
    
    /// Optional case payload type
    pub ty: Option<WitType>,
}

/// WIT enum type
#[derive(Debug, Clone)]
pub struct WitEnum {
    /// Enum cases
    pub cases: SimpleBoundedVec<SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>, MAX_WIT_TYPES>,
}

/// WIT flags type
#[derive(Debug, Clone)]
pub struct WitFlags {
    /// Flag names
    pub flags: SimpleBoundedVec<SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>, MAX_WIT_TYPES>,
}

/// WIT resource type
#[derive(Debug, Clone)]
pub struct WitResource {
    /// Resource constructor (optional)
    pub constructor: Option<WitFunction>,
    
    /// Resource methods
    pub methods: SimpleBoundedVec<WitFunction, MAX_WIT_FUNCTIONS>,
    
    /// Static methods
    pub static_methods: SimpleBoundedVec<WitFunction, MAX_WIT_FUNCTIONS>,
}

/// WIT type expressions
#[derive(Debug, Clone)]
pub enum WitType {
    /// Primitive types
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
    Tuple(SimpleBoundedVec<WitType, 8>),
    
    /// Resource handle types
    Own(SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>),
    Borrow(SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>),
    
    /// Named type reference
    Named(SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>),
}

impl Default for WitDocument {
    fn default() -> Self {
        Self {
            package: None,
            interfaces: SimpleBoundedVec::new(),
            worlds: SimpleBoundedVec::new(),
            uses: SimpleBoundedVec::new(),
        }
    }
}

impl Default for WitInterface {
    fn default() -> Self {
        Self {
            name: SimpleBoundedString::new(),
            functions: SimpleBoundedVec::new(),
            types: SimpleBoundedVec::new(),
            uses: SimpleBoundedVec::new(),
        }
    }
}

impl Default for WitWorld {
    fn default() -> Self {
        Self {
            name: SimpleBoundedString::new(),
            imports: SimpleBoundedVec::new(),
            exports: SimpleBoundedVec::new(),
            types: SimpleBoundedVec::new(),
        }
    }
}

impl Default for WitFunction {
    fn default() -> Self {
        Self {
            name: SimpleBoundedString::new(),
            params: SimpleBoundedVec::new(),
            results: SimpleBoundedVec::new(),
            is_async: false,
            is_static: false,
        }
    }
}