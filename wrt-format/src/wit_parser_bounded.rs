//! Bounded WIT (WebAssembly Interface Types) parser for no_std environments
//!
//! This module provides basic WIT parsing capabilities using bounded collections,
//! enabling WIT support in pure no_std environments without allocation.

use wrt_foundation::{BoundedVec, BoundedString, MemoryProvider, NoStdProvider};
use wrt_foundation::traits::{Checksummable, ToBytes, FromBytes};
use wrt_error::{Error, Result};
use crate::{MAX_BOUNDED_AST_NODES, MAX_BOUNDED_TOKENS, MAX_WASM_STRING_SIZE};

/// Bounded WIT name for no_std environments
pub type BoundedWitName<P> = BoundedString<MAX_WASM_STRING_SIZE, P>;

/// Bounded WIT parser for no_std environments
#[derive(Debug, Clone)]
pub struct BoundedWitParser<P: MemoryProvider + Default + Clone + PartialEq + Eq = NoStdProvider<4096>> {
    /// Current parsing position
    position: usize,
    /// Input text being parsed
    input: BoundedString<8192, P>, // 8KB input buffer
    /// Current tokens
    tokens: BoundedVec<BoundedToken<P>, MAX_BOUNDED_TOKENS, P>,
    /// Parsed world definitions
    worlds: BoundedVec<BoundedWitWorld<P>, 16, P>,
    /// Parsed interface definitions
    interfaces: BoundedVec<BoundedWitInterface<P>, 32, P>,
    /// Memory provider
    provider: P,
}

/// Bounded WIT token for no_std parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedToken<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Token type
    pub kind: TokenKind,
    /// Token text
    pub text: BoundedWitName<P>,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
}

impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> Default for BoundedToken<P> {
    fn default() -> Self {
        Self {
            kind: TokenKind::Eof,
            text: BoundedWitName::new(P::default()).unwrap_or_else(|_| {
                // Fallback - should not happen in practice
                unsafe { core::mem::zeroed() }
            }),
            line: 0,
            column: 0,
        }
    }
}

impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> Checksummable for BoundedToken<P> {
    fn checksum(&self) -> wrt_foundation::Checksum {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.kind as u8).to_le_bytes());
        bytes.extend_from_slice(&self.text.checksum().as_bytes());
        bytes.extend_from_slice(&self.line.to_le_bytes());
        bytes.extend_from_slice(&self.column.to_le_bytes());
        wrt_foundation::Checksum::from_bytes(&bytes)
    }
}

impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> ToBytes for BoundedToken<P> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self.kind as u8).to_le_bytes());
        bytes.extend_from_slice(&self.text.to_bytes());
        bytes.extend_from_slice(&self.line.to_le_bytes());
        bytes.extend_from_slice(&self.column.to_le_bytes());
        bytes
    }
}

impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> FromBytes for BoundedToken<P> {
    fn from_bytes(bytes: &[u8]) -> wrt_foundation::Result<Self> {
        if bytes.len() < 9 {
            return Err(wrt_foundation::Error::new(
                wrt_foundation::ErrorCategory::Parse,
                wrt_error::codes::PARSE_ERROR,
                "Insufficient bytes for BoundedToken"
            ));
        }
        
        let kind = match bytes[0] {
            0 => TokenKind::Identifier,
            1 => TokenKind::Keyword,
            2 => TokenKind::TypeName,
            3 => TokenKind::Operator,
            4 => TokenKind::StringLiteral,
            5 => TokenKind::Number,
            6 => TokenKind::Comment,
            7 => TokenKind::Newline,
            _ => TokenKind::Eof,
        };
        
        let text_bytes = &bytes[1..bytes.len()-8];
        let text = BoundedWitName::from_bytes(text_bytes)?;
        
        let line = u32::from_le_bytes([bytes[bytes.len()-8], bytes[bytes.len()-7], bytes[bytes.len()-6], bytes[bytes.len()-5]]);
        let column = u32::from_le_bytes([bytes[bytes.len()-4], bytes[bytes.len()-3], bytes[bytes.len()-2], bytes[bytes.len()-1]]);
        
        Ok(Self {
            kind,
            text,
            line,
            column,
        })
    }
}

/// Token types for WIT parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// Identifier (world, interface, function names)
    Identifier,
    /// Keywords (world, interface, import, export, etc.)
    Keyword,
    /// Type names (string, u32, etc.)
    TypeName,
    /// Operators and punctuation
    Operator,
    /// String literals
    StringLiteral,
    /// Numbers
    Number,
    /// Comments
    Comment,
    /// Newlines
    Newline,
    /// End of input
    Eof,
}

/// Bounded WIT world definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWitWorld<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// World name
    pub name: BoundedWitName<P>,
    /// World imports
    pub imports: BoundedVec<BoundedWitImport<P>, 64, P>,
    /// World exports
    pub exports: BoundedVec<BoundedWitExport<P>, 64, P>,
}

/// Bounded WIT interface definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWitInterface<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Interface name
    pub name: BoundedWitName<P>,
    /// Interface functions
    pub functions: BoundedVec<BoundedWitFunction<P>, 128, P>,
    /// Interface types
    pub types: BoundedVec<BoundedWitType<P>, 64, P>,
}

/// Bounded WIT import definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWitImport<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Import name
    pub name: BoundedWitName<P>,
    /// Import type
    pub import_type: BoundedImportType<P>,
}

/// Bounded WIT export definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWitExport<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Export name
    pub name: BoundedWitName<P>,
    /// Export type
    pub export_type: BoundedExportType<P>,
}

/// Bounded WIT function definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWitFunction<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Function name
    pub name: BoundedWitName<P>,
    /// Function parameters
    pub params: BoundedVec<BoundedWitParam<P>, 32, P>,
    /// Function results
    pub results: BoundedVec<BoundedWitType<P>, 8, P>,
}

/// Bounded WIT parameter definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWitParam<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Parameter name
    pub name: BoundedWitName<P>,
    /// Parameter type
    pub param_type: BoundedWitType<P>,
}

/// Bounded WIT type definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundedWitType<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Primitive types
    Bool,
    U8, U16, U32, U64,
    S8, S16, S32, S64,
    F32, F64,
    Char,
    String,
    
    /// List type with bounded element type
    List {
        element_type: u32, // Type reference to avoid infinite recursion
    },
    
    /// Record type with bounded fields
    Record {
        fields: BoundedVec<BoundedWitRecordField<P>, 32, P>,
    },
    
    /// Variant type with bounded cases
    Variant {
        cases: BoundedVec<BoundedWitVariantCase<P>, 32, P>,
    },
    
    /// Option type
    Option {
        inner_type: u32, // Type reference
    },
    
    /// Result type
    Result {
        ok_type: Option<u32>, // Type reference
        err_type: Option<u32>, // Type reference
    },
    
    /// Resource handle
    Resource {
        name: BoundedWitName<P>,
    },
    
    /// Named type reference
    Named {
        name: BoundedWitName<P>,
    },
}

/// Bounded WIT record field
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWitRecordField<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Field name
    pub name: BoundedWitName<P>,
    /// Field type
    pub field_type: u32, // Type reference
}

/// Bounded WIT variant case
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWitVariantCase<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Case name
    pub name: BoundedWitName<P>,
    /// Case type (optional)
    pub case_type: Option<u32>, // Type reference
}

/// Bounded import type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundedImportType<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Function import
    Function(BoundedWitFunction<P>),
    /// Interface import
    Interface(BoundedWitName<P>),
    /// Type import
    Type(BoundedWitName<P>),
}

/// Bounded export type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundedExportType<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Function export
    Function(BoundedWitFunction<P>),
    /// Interface export
    Interface(BoundedWitName<P>),
    /// Type export
    Type(BoundedWitName<P>),
}

impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> BoundedWitParser<P> {
    /// Create a new bounded WIT parser
    pub fn new(provider: P) -> Result<Self> {
        Ok(Self {
            position: 0,
            input: BoundedString::new(provider.clone())
                .map_err(|_| Error::new(crate::ErrorCategory::Runtime, wrt_error::codes::MEMORY_ERROR, "Failed to create input buffer"))?,
            tokens: BoundedVec::new(provider.clone())
                .map_err(|_| Error::new(crate::ErrorCategory::Runtime, wrt_error::codes::MEMORY_ERROR, "Failed to create tokens vector"))?,
            worlds: BoundedVec::new(provider.clone())
                .map_err(|_| Error::new(crate::ErrorCategory::Runtime, wrt_error::codes::MEMORY_ERROR, "Failed to create worlds vector"))?,
            interfaces: BoundedVec::new(provider.clone())
                .map_err(|_| Error::new(crate::ErrorCategory::Runtime, wrt_error::codes::MEMORY_ERROR, "Failed to create interfaces vector"))?,
            provider,
        })
    }

    /// Parse WIT text input
    pub fn parse(&mut self, input: &str) -> Result<()> {
        // Store input in bounded buffer
        self.input = BoundedString::from_str(input, self.provider.clone())
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Input too large for bounded buffer"))?;
        
        // Reset parser state
        self.position = 0;
        self.tokens.clear();
        self.worlds.clear();
        self.interfaces.clear();

        // Tokenize input
        self.tokenize()?;
        
        // Parse tokens
        self.parse_definitions()?;
        
        Ok(())
    }

    /// Tokenize input text
    fn tokenize(&mut self) -> Result<()> {
        let input_str = self.input.as_str()
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Invalid UTF-8 in input"))?;
        
        let mut line = 1u32;
        let mut column = 1u32;
        let mut chars = input_str.char_indices().peekable();

        while let Some((pos, ch)) = chars.next() {
            match ch {
                // Whitespace
                ' ' | '\t' => {
                    column += 1;
                    continue;
                }
                '\n' => {
                    self.add_token(TokenKind::Newline, "\n", line, column)?;
                    line += 1;
                    column = 1;
                    continue;
                }
                '\r' => {
                    // Handle \r\n
                    if chars.peek().map(|(_, c)| *c) == Some('\n') {
                        chars.next();
                    }
                    self.add_token(TokenKind::Newline, "\n", line, column)?;
                    line += 1;
                    column = 1;
                    continue;
                }
                
                // Comments
                '/' if chars.peek().map(|(_, c)| *c) == Some('/') => {
                    chars.next(); // consume second '/'
                    let start = pos;
                    
                    // Read until end of line
                    while let Some((_, ch)) = chars.peek() {
                        if *ch == '\n' || *ch == '\r' {
                            break;
                        }
                        chars.next();
                    }
                    
                    let comment_text = &input_str[start..chars.peek().map(|(p, _)| *p).unwrap_or(input_str.len())];
                    self.add_token(TokenKind::Comment, comment_text, line, column)?;
                    column += comment_text.len() as u32;
                    continue;
                }
                
                // String literals
                '"' => {
                    let start = pos;
                    column += 1;
                    
                    // Read until closing quote
                    while let Some((_, ch)) = chars.next() {
                        column += 1;
                        if ch == '"' {
                            break;
                        }
                        if ch == '\\' {
                            // Skip escaped character
                            if chars.next().is_some() {
                                column += 1;
                            }
                        }
                    }
                    
                    let string_text = &input_str[start..chars.peek().map(|(p, _)| *p).unwrap_or(input_str.len())];
                    self.add_token(TokenKind::StringLiteral, string_text, line, column - string_text.len() as u32)?;
                    continue;
                }
                
                // Numbers
                '0'..='9' => {
                    let start = pos;
                    
                    while let Some((_, ch)) = chars.peek() {
                        if !ch.is_ascii_digit() && *ch != '.' {
                            break;
                        }
                        chars.next();
                        column += 1;
                    }
                    
                    let number_text = &input_str[start..chars.peek().map(|(p, _)| *p).unwrap_or(input_str.len())];
                    self.add_token(TokenKind::Number, number_text, line, column - number_text.len() as u32)?;
                    continue;
                }
                
                // Identifiers and keywords
                'a'..='z' | 'A'..='Z' | '_' => {
                    let start = pos;
                    
                    while let Some((_, ch)) = chars.peek() {
                        if !ch.is_alphanumeric() && *ch != '_' && *ch != '-' {
                            break;
                        }
                        chars.next();
                        column += 1;
                    }
                    
                    let ident_text = &input_str[start..chars.peek().map(|(p, _)| *p).unwrap_or(input_str.len())];
                    let token_kind = match ident_text {
                        "world" | "interface" | "import" | "export" | "func" | "type" |
                        "record" | "variant" | "enum" | "flags" | "resource" |
                        "list" | "option" | "result" | "tuple" => TokenKind::Keyword,
                        "bool" | "u8" | "u16" | "u32" | "u64" | "s8" | "s16" | "s32" | "s64" |
                        "f32" | "f64" | "char" | "string" => TokenKind::TypeName,
                        _ => TokenKind::Identifier,
                    };
                    
                    self.add_token(token_kind, ident_text, line, column - ident_text.len() as u32)?;
                    continue;
                }
                
                // Operators and punctuation
                '{' | '}' | '(' | ')' | '[' | ']' | '<' | '>' | ',' | ':' | ';' | '=' | '*' | '%' => {
                    let op_str = &input_str[pos..pos + ch.len_utf8()];
                    self.add_token(TokenKind::Operator, op_str, line, column)?;
                    column += 1;
                }
                
                _ => {
                    // Skip unknown characters
                    column += 1;
                }
            }
        }
        
        // Add EOF token
        self.add_token(TokenKind::Eof, "", line, column)?;
        
        Ok(())
    }

    /// Add a token to the tokens list
    fn add_token(&mut self, kind: TokenKind, text: &str, line: u32, column: u32) -> Result<()> {
        let bounded_text = BoundedWitName::from_str(text, self.provider.clone())
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Token text too long"))?;
        
        let token = BoundedToken {
            kind,
            text: bounded_text,
            line,
            column,
        };
        
        self.tokens.push(token)
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Too many tokens"))?;
        
        Ok(())
    }

    /// Parse top-level definitions (worlds and interfaces)
    fn parse_definitions(&mut self) -> Result<()> {
        self.position = 0;
        
        while self.position < self.tokens.len() {
            let token = &self.tokens[self.position].clone();
            
            match (&token.kind, token.text.as_str().unwrap_or("")) {
                (TokenKind::Keyword, "world") => {
                    self.parse_world()?;
                }
                (TokenKind::Keyword, "interface") => {
                    self.parse_interface()?;
                }
                (TokenKind::Comment, _) | (TokenKind::Newline, _) => {
                    self.position += 1; // Skip comments and newlines
                }
                (TokenKind::Eof, _) => {
                    break;
                }
                _ => {
                    // Skip unknown tokens
                    self.position += 1;
                }
            }
        }
        
        Ok(())
    }

    /// Parse a world definition
    fn parse_world(&mut self) -> Result<()> {
        // Skip 'world' keyword
        self.position += 1;
        
        // Get world name
        let world_name = self.expect_identifier()?;
        
        // Expect '{'
        self.expect_operator("{")?;
        
        // Parse world body
        let mut imports = BoundedVec::new(self.provider.clone())
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Failed to create imports vector"))?;
        let mut exports = BoundedVec::new(self.provider.clone())
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Failed to create exports vector"))?;
        
        while self.position < self.tokens.len() {
            let token = &self.tokens[self.position].clone();
            
            match (&token.kind, token.text.as_str().unwrap_or("")) {
                (TokenKind::Keyword, "import") => {
                    let import = self.parse_import()?;
                    imports.push(import)
                        .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Too many imports"))?;
                }
                (TokenKind::Keyword, "export") => {
                    let export = self.parse_export()?;
                    exports.push(export)
                        .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Too many exports"))?;
                }
                (TokenKind::Operator, "}") => {
                    self.position += 1;
                    break;
                }
                (TokenKind::Comment, _) | (TokenKind::Newline, _) => {
                    self.position += 1; // Skip comments and newlines
                }
                _ => {
                    self.position += 1; // Skip unknown tokens
                }
            }
        }
        
        // Create world
        let world = BoundedWitWorld {
            name: world_name,
            imports,
            exports,
        };
        
        self.worlds.push(world)
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Too many worlds"))?;
        
        Ok(())
    }

    /// Parse an interface definition
    fn parse_interface(&mut self) -> Result<()> {
        // Skip 'interface' keyword
        self.position += 1;
        
        // Get interface name
        let interface_name = self.expect_identifier()?;
        
        // Expect '{'
        self.expect_operator("{")?;
        
        // Parse interface body
        let mut functions = BoundedVec::new(self.provider.clone())
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Failed to create functions vector"))?;
        let mut types = BoundedVec::new(self.provider.clone())
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Failed to create types vector"))?;
        
        while self.position < self.tokens.len() {
            let token = &self.tokens[self.position].clone();
            
            match (&token.kind, token.text.as_str().unwrap_or("")) {
                (TokenKind::Identifier, _) => {
                    // Parse function
                    let function = self.parse_function()?;
                    functions.push(function)
                        .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Too many functions"))?;
                }
                (TokenKind::Keyword, "type") => {
                    let type_def = self.parse_type_definition()?;
                    types.push(type_def)
                        .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Too many types"))?;
                }
                (TokenKind::Operator, "}") => {
                    self.position += 1;
                    break;
                }
                (TokenKind::Comment, _) | (TokenKind::Newline, _) => {
                    self.position += 1; // Skip comments and newlines
                }
                _ => {
                    self.position += 1; // Skip unknown tokens
                }
            }
        }
        
        // Create interface
        let interface = BoundedWitInterface {
            name: interface_name,
            functions,
            types,
        };
        
        self.interfaces.push(interface)
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Too many interfaces"))?;
        
        Ok(())
    }

    /// Parse an import statement
    fn parse_import(&mut self) -> Result<BoundedWitImport<P>> {
        // Skip 'import' keyword
        self.position += 1;
        
        // Get import name
        let import_name = self.expect_identifier()?;
        
        // For simplicity, assume function import
        self.expect_operator(":")?;
        self.expect_keyword("func")?;
        
        let function = self.parse_function_signature()?;
        
        Ok(BoundedWitImport {
            name: import_name,
            import_type: BoundedImportType::Function(function),
        })
    }

    /// Parse an export statement
    fn parse_export(&mut self) -> Result<BoundedWitExport<P>> {
        // Skip 'export' keyword
        self.position += 1;
        
        // Get export name
        let export_name = self.expect_identifier()?;
        
        // For simplicity, assume function export
        self.expect_operator(":")?;
        self.expect_keyword("func")?;
        
        let function = self.parse_function_signature()?;
        
        Ok(BoundedWitExport {
            name: export_name,
            export_type: BoundedExportType::Function(function),
        })
    }

    /// Parse a function definition
    fn parse_function(&mut self) -> Result<BoundedWitFunction<P>> {
        let function_name = self.expect_identifier()?;
        self.parse_function_signature_with_name(function_name)
    }

    /// Parse a function signature
    fn parse_function_signature(&mut self) -> Result<BoundedWitFunction<P>> {
        let function_name = BoundedWitName::from_str("", self.provider.clone())
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Failed to create function name"))?;
        self.parse_function_signature_with_name(function_name)
    }

    /// Parse a function signature with a given name
    fn parse_function_signature_with_name(&mut self, name: BoundedWitName<P>) -> Result<BoundedWitFunction<P>> {
        // Expect '('
        self.expect_operator("(")?;
        
        // Parse parameters
        let mut params = BoundedVec::new(self.provider.clone())
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Failed to create params vector"))?;
        
        while self.position < self.tokens.len() {
            let token = &self.tokens[self.position].clone();
            
            if let (TokenKind::Operator, ")") = (&token.kind, token.text.as_str().unwrap_or("")) {
                self.position += 1;
                break;
            }
            
            if let (TokenKind::Identifier, _) = (&token.kind, token.text.as_str().unwrap_or("")) {
                let param_name = self.expect_identifier()?;
                self.expect_operator(":")?;
                let param_type = self.parse_simple_type()?;
                
                let param = BoundedWitParam {
                    name: param_name,
                    param_type,
                };
                
                params.push(param)
                    .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Too many parameters"))?;
                
                // Skip comma if present
                if self.position < self.tokens.len() {
                    let token = &self.tokens[self.position];
                    if let (TokenKind::Operator, ",") = (&token.kind, token.text.as_str().unwrap_or("")) {
                        self.position += 1;
                    }
                }
            } else {
                self.position += 1; // Skip unknown tokens
            }
        }
        
        // Parse return type (optional)
        let mut results = BoundedVec::new(self.provider.clone())
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Failed to create results vector"))?;
        
        if self.position < self.tokens.len() {
            let token = &self.tokens[self.position];
            if let (TokenKind::Operator, "->") = (&token.kind, token.text.as_str().unwrap_or("")) {
                self.position += 1;
                let result_type = self.parse_simple_type()?;
                results.push(result_type)
                    .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Too many results"))?;
            }
        }
        
        Ok(BoundedWitFunction {
            name,
            params,
            results,
        })
    }

    /// Parse a simple type (primitive types only for now)
    fn parse_simple_type(&mut self) -> Result<BoundedWitType<P>> {
        if self.position >= self.tokens.len() {
            return Err(Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Unexpected end of input"));
        }
        
        let token = &self.tokens[self.position].clone();
        self.position += 1;
        
        match (&token.kind, token.text.as_str().unwrap_or("")) {
            (TokenKind::TypeName, "bool") => Ok(BoundedWitType::Bool),
            (TokenKind::TypeName, "u8") => Ok(BoundedWitType::U8),
            (TokenKind::TypeName, "u16") => Ok(BoundedWitType::U16),
            (TokenKind::TypeName, "u32") => Ok(BoundedWitType::U32),
            (TokenKind::TypeName, "u64") => Ok(BoundedWitType::U64),
            (TokenKind::TypeName, "s8") => Ok(BoundedWitType::S8),
            (TokenKind::TypeName, "s16") => Ok(BoundedWitType::S16),
            (TokenKind::TypeName, "s32") => Ok(BoundedWitType::S32),
            (TokenKind::TypeName, "s64") => Ok(BoundedWitType::S64),
            (TokenKind::TypeName, "f32") => Ok(BoundedWitType::F32),
            (TokenKind::TypeName, "f64") => Ok(BoundedWitType::F64),
            (TokenKind::TypeName, "char") => Ok(BoundedWitType::Char),
            (TokenKind::TypeName, "string") => Ok(BoundedWitType::String),
            (TokenKind::Identifier, _) => {
                // Named type reference
                Ok(BoundedWitType::Named {
                    name: token.text.clone(),
                })
            }
            _ => Err(Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Expected type name")),
        }
    }

    /// Parse a type definition
    fn parse_type_definition(&mut self) -> Result<BoundedWitType<P>> {
        // Skip 'type' keyword
        self.position += 1;
        
        // For now, just parse simple types
        self.parse_simple_type()
    }

    /// Expect an identifier token and return its text
    fn expect_identifier(&mut self) -> Result<BoundedWitName<P>> {
        if self.position >= self.tokens.len() {
            return Err(Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Expected identifier"));
        }
        
        let token = &self.tokens[self.position].clone();
        
        if let TokenKind::Identifier = token.kind {
            self.position += 1;
            Ok(token.text.clone())
        } else {
            Err(Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Expected identifier"))
        }
    }

    /// Expect a keyword token
    fn expect_keyword(&mut self, expected: &str) -> Result<()> {
        if self.position >= self.tokens.len() {
            return Err(Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Expected keyword"));
        }
        
        let token = &self.tokens[self.position];
        
        if let (TokenKind::Keyword, text) = (&token.kind, token.text.as_str().unwrap_or("")) {
            if text == expected {
                self.position += 1;
                Ok(())
            } else {
                Err(Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Unexpected keyword"))
            }
        } else {
            Err(Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Expected keyword"))
        }
    }

    /// Expect an operator token
    fn expect_operator(&mut self, expected: &str) -> Result<()> {
        if self.position >= self.tokens.len() {
            return Err(Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Expected operator"));
        }
        
        let token = &self.tokens[self.position];
        
        if let (TokenKind::Operator, text) = (&token.kind, token.text.as_str().unwrap_or("")) {
            if text == expected {
                self.position += 1;
                Ok(())
            } else {
                Err(Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Unexpected operator"))
            }
        } else {
            Err(Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Expected operator"))
        }
    }

    /// Get parsed worlds
    pub fn worlds(&self) -> &BoundedVec<BoundedWitWorld<P>, 16, P> {
        &self.worlds
    }

    /// Get parsed interfaces
    pub fn interfaces(&self) -> &BoundedVec<BoundedWitInterface<P>, 32, P> {
        &self.interfaces
    }
}

impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> Default for BoundedWitParser<P> {
    fn default() -> Self {
        Self::new(P::default()).unwrap_or_else(|_| {
            // Fallback to empty parser if creation fails
            Self {
                position: 0,
                input: BoundedString::new(P::default()).unwrap(),
                tokens: BoundedVec::new(P::default()).unwrap(),
                worlds: BoundedVec::new(P::default()).unwrap(),
                interfaces: BoundedVec::new(P::default()).unwrap(),
                provider: P::default(),
            }
        })
    }
}

/// Feature detection for bounded WIT parsing
pub const HAS_BOUNDED_WIT_PARSING_NO_STD: bool = true;

/// Convenience function to parse WIT text with default provider
pub fn parse_wit_bounded(input: &str) -> Result<BoundedWitParser<NoStdProvider<4096>>> {
    let mut parser = BoundedWitParser::new(NoStdProvider::<4096>::default())?;
    parser.parse(input)?;
    Ok(parser)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrt_foundation::NoStdProvider;

    type TestProvider = NoStdProvider<4096>;

    #[test]
    fn test_bounded_wit_parser_creation() {
        let provider = TestProvider::default();
        let parser = BoundedWitParser::new(provider);
        assert!(parser.is_ok());

        let parser = parser.unwrap();
        assert_eq!(parser.worlds().len(), 0);
        assert_eq!(parser.interfaces().len(), 0);
    }

    #[test]
    fn test_simple_wit_parsing() {
        let wit_text = r#"
            world test-world {
                import test-func: func(x: u32) -> string
                export main: func() -> u32
            }
        "#;

        let result = parse_wit_bounded(wit_text);
        assert!(result.is_ok());

        let parser = result.unwrap();
        assert_eq!(parser.worlds().len(), 1);

        let world = &parser.worlds()[0];
        assert_eq!(world.name.as_str().unwrap(), "test-world");
        assert_eq!(world.imports.len(), 1);
        assert_eq!(world.exports.len(), 1);
    }

    #[test]
    fn test_tokenization() {
        let mut parser = BoundedWitParser::new(TestProvider::default()).unwrap();
        let input = "world test { import func: func() }";
        
        let result = parser.parse(input);
        assert!(result.is_ok());
        
        // Should have tokenized the input
        assert!(parser.tokens.len() > 0);
    }

    #[test]
    fn test_function_parsing() {
        let wit_text = r#"
            interface test-interface {
                test-func: func(a: u32, b: string) -> bool
            }
        "#;

        let result = parse_wit_bounded(wit_text);
        assert!(result.is_ok());

        let parser = result.unwrap();
        assert_eq!(parser.interfaces().len(), 1);

        let interface = &parser.interfaces()[0];
        assert_eq!(interface.name.as_str().unwrap(), "test-interface");
        assert_eq!(interface.functions.len(), 1);

        let func = &interface.functions[0];
        assert_eq!(func.name.as_str().unwrap(), "test-func");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.results.len(), 1);
    }

    #[test]
    fn test_bounded_capacity_limits() {
        // Test that parser respects bounded collection limits
        let mut parser = BoundedWitParser::new(TestProvider::default()).unwrap();
        
        // Create input that would exceed token limits if not properly bounded
        let mut large_input = String::new();
        for i in 0..1000 {
            large_input.push_str(&format!("token{} ", i));
        }
        
        // Should either parse successfully or fail gracefully
        let result = parser.parse(&large_input);
        // Don't assert success/failure - just ensure no panic
        let _ = result;
    }

    #[test]
    fn test_error_handling() {
        let invalid_wit = "invalid wit syntax {{{";
        let result = parse_wit_bounded(invalid_wit);
        
        // Should handle gracefully (may parse partially or fail)
        let _ = result;
    }

    #[test]
    fn test_primitive_types() {
        let wit_text = r#"
            interface primitives {
                test-bool: func() -> bool
                test-u8: func() -> u8
                test-u16: func() -> u16
                test-u32: func() -> u32
                test-u64: func() -> u64
                test-s8: func() -> s8
                test-s16: func() -> s16
                test-s32: func() -> s32
                test-s64: func() -> s64
                test-f32: func() -> f32
                test-f64: func() -> f64
                test-char: func() -> char
                test-string: func() -> string
            }
        "#;

        let result = parse_wit_bounded(wit_text);
        assert!(result.is_ok());

        let parser = result.unwrap();
        assert_eq!(parser.interfaces().len(), 1);

        let interface = &parser.interfaces()[0];
        assert_eq!(interface.functions.len(), 13);
    }
}