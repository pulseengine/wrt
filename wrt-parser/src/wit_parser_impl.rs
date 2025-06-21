//! WIT parser implementation
//!
//! High-performance WIT parser with modernized AST interpretation and streaming support.

use wrt_error::{Error, ErrorCategory, Result, codes};
use crate::wit_lexer::{WitLexer, Token, Position, Span};
use crate::wit_parser::*;
use crate::bounded_types::{SimpleBoundedString, SimpleBoundedVec};

/// WIT parser with advanced features
#[derive(Debug)]
pub struct WitParser<'a> {
    /// Lexer for tokenizing input
    lexer: WitLexer<'a>,
    
    /// Current token
    current_token: Option<Token>,
    
    /// Current position for error reporting
    current_position: Position,
    
    /// Enable streaming mode for large files
    streaming_mode: bool,
}

impl<'a> WitParser<'a> {
    /// Create a new WIT parser
    pub fn new(source: &'a str) -> Result<Self> {
        let mut lexer = WitLexer::new(source);
        let current_token = Some(lexer.next_token()?);
        let current_position = lexer.position();
        
        Ok(Self {
            lexer,
            current_token,
            current_position,
            streaming_mode: false,
        })
    }
    
    /// Enable streaming mode for large files
    pub fn enable_streaming(&mut self) {
        self.streaming_mode = true;
    }
    
    /// Parse a complete WIT document
    pub fn parse_document(&mut self) -> Result<WitDocument> {
        let mut document = WitDocument::default();
        
        // Skip initial whitespace and comments
        self.skip_trivia()?;
        
        // Parse optional package declaration
        if self.check_token(&Token::Package) {
            document.package = Some(self.parse_package()?);
        }
        
        // Parse top-level items
        while !self.is_at_end() {
            self.skip_trivia()?;
            
            if self.is_at_end() {
                break;
            }
            
            match self.current_token.as_ref() {
                Some(Token::Use) => {
                    let use_stmt = self.parse_use()?;
                    document.uses.push(use_stmt).map_err(|_| self.error("Too many use statements"))?;
                }
                Some(Token::Interface) => {
                    let interface = self.parse_interface()?;
                    document.interfaces.push(interface).map_err(|_| self.error("Too many interfaces"))?;
                }
                Some(Token::World) => {
                    let world = self.parse_world()?;
                    document.worlds.push(world).map_err(|_| self.error("Too many worlds"))?;
                }
                _ => {
                    return Err(self.error("Expected interface, world, or use statement"));
                }
            }
        }
        
        Ok(document)
    }
    
    /// Parse a package declaration
    fn parse_package(&mut self) -> Result<WitPackage> {
        self.expect_token(&Token::Package)?;
        
        // Parse namespace
        let namespace = self.expect_identifier()?;
        
        // Expect colon
        self.expect_token(&Token::Colon)?;
        
        // Parse name
        let name = self.expect_identifier()?;
        
        // Check for optional version
        let version = if self.match_token(&Token::At)? {
            let version_id = self.expect_identifier()?;
            Some(SimpleBoundedString::from_str(version_id.as_str()))
        } else {
            None
        };
        
        Ok(WitPackage {
            namespace,
            name,
            version,
        })
    }
    
    /// Parse a use statement
    fn parse_use(&mut self) -> Result<WitUse> {
        self.expect_token(&Token::Use)?;
        
        let source = self.expect_identifier()?;
        
        self.expect_token(&Token::Dot)?;
        self.expect_token(&Token::LeftBrace)?;
        
        let mut items = SimpleBoundedVec::new();
        
        while !self.check_token(&Token::RightBrace) {
            let source_name = self.expect_identifier()?;
            let local_name = if self.match_token(&Token::As)? {
                Some(self.expect_identifier()?)
            } else {
                None
            };
            
            let item = WitUseItem { source_name, local_name };
            items.push(item).map_err(|_| self.error("Too many use items"))?;
            
            if !self.match_token(&Token::Comma)? {
                break;
            }
        }
        
        self.expect_token(&Token::RightBrace)?;
        
        Ok(WitUse { source, items })
    }
    
    /// Parse an interface definition
    fn parse_interface(&mut self) -> Result<WitInterface> {
        self.expect_token(&Token::Interface)?;
        
        let name = self.expect_identifier()?;
        
        self.expect_token(&Token::LeftBrace)?;
        
        let mut interface = WitInterface {
            name,
            ..Default::default()
        };
        
        while !self.check_token(&Token::RightBrace) {
            self.skip_trivia()?;
            
            if self.check_token(&Token::RightBrace) {
                break;
            }
            
            match self.current_token.as_ref() {
                Some(Token::Use) => {
                    let use_stmt = self.parse_use()?;
                    interface.uses.push(use_stmt).map_err(|_| self.error("Too many use statements"))?;
                }
                Some(Token::Type) => {
                    let type_def = self.parse_type_def()?;
                    interface.types.push(type_def).map_err(|_| self.error("Too many type definitions"))?;
                }
                Some(Token::Func) => {
                    let function = self.parse_function()?;
                    interface.functions.push(function).map_err(|_| self.error("Too many functions"))?;
                }
                _ => {
                    return Err(self.error("Expected use, type, or function in interface"));
                }
            }
        }
        
        self.expect_token(&Token::RightBrace)?;
        
        Ok(interface)
    }
    
    /// Parse a world definition
    fn parse_world(&mut self) -> Result<WitWorld> {
        self.expect_token(&Token::World)?;
        
        let name = self.expect_identifier()?;
        
        self.expect_token(&Token::LeftBrace)?;
        
        let mut world = WitWorld {
            name,
            ..Default::default()
        };
        
        while !self.check_token(&Token::RightBrace) {
            self.skip_trivia()?;
            
            if self.check_token(&Token::RightBrace) {
                break;
            }
            
            match self.current_token.as_ref() {
                Some(Token::Import) => {
                    let import = self.parse_import()?;
                    world.imports.push(import).map_err(|_| self.error("Too many imports"))?;
                }
                Some(Token::Export) => {
                    let export = self.parse_export()?;
                    world.exports.push(export).map_err(|_| self.error("Too many exports"))?;
                }
                Some(Token::Type) => {
                    let type_def = self.parse_type_def()?;
                    world.types.push(type_def).map_err(|_| self.error("Too many type definitions"))?;
                }
                _ => {
                    return Err(self.error("Expected import, export, or type in world"));
                }
            }
        }
        
        self.expect_token(&Token::RightBrace)?;
        
        Ok(world)
    }
    
    /// Parse an import statement
    fn parse_import(&mut self) -> Result<WitImport> {
        self.expect_token(&Token::Import)?;
        
        let name = self.expect_identifier()?;
        self.expect_token(&Token::Colon)?;
        let item = self.parse_item()?;
        
        Ok(WitImport { name, item })
    }
    
    /// Parse an export statement
    fn parse_export(&mut self) -> Result<WitExport> {
        self.expect_token(&Token::Export)?;
        
        let name = self.expect_identifier()?;
        self.expect_token(&Token::Colon)?;
        let item = self.parse_item()?;
        
        Ok(WitExport { name, item })
    }
    
    /// Parse an item (function, interface, type, or instance)
    fn parse_item(&mut self) -> Result<WitItem> {
        match self.current_token.as_ref() {
            Some(Token::Func) => {
                let function = self.parse_function()?;
                Ok(WitItem::Function(function))
            }
            Some(Token::Interface) => {
                let interface = self.parse_interface()?;
                Ok(WitItem::Interface(interface))
            }
            Some(Token::Identifier(_)) => {
                // Could be a type reference or instance
                let type_name = self.expect_identifier()?;
                
                // Check if this is followed by something that indicates it's an instance
                if self.match_token(&Token::LeftBrace)? {
                    // This is an instance specification
                    self.expect_token(&Token::RightBrace)?; // For now, empty instances
                    Ok(WitItem::Instance {
                        interface: type_name,
                    })
                } else {
                    // This is a type reference
                    Ok(WitItem::Type(WitType::Named(type_name)))
                }
            }
            _ => {
                let wit_type = self.parse_type()?;
                Ok(WitItem::Type(wit_type))
            }
        }
    }
    
    /// Parse a function definition
    fn parse_function(&mut self) -> Result<WitFunction> {
        let is_static = self.match_token(&Token::Static)?;
        self.expect_token(&Token::Func)?;
        
        let name = self.expect_identifier()?;
        
        self.expect_token(&Token::LeftParen)?;
        
        let mut params = SimpleBoundedVec::new();
        
        while !self.check_token(&Token::RightParen) {
            let param_name = self.expect_identifier()?;
            self.expect_token(&Token::Colon)?;
            let param_type = self.parse_type()?;
            
            let param = WitParam {
                name: param_name,
                ty: param_type,
            };
            params.push(param).map_err(|_| self.error("Too many parameters"))?;
            
            if !self.match_token(&Token::Comma)? {
                break;
            }
        }
        
        self.expect_token(&Token::RightParen)?;
        
        let mut results = SimpleBoundedVec::new();
        
        if self.match_token(&Token::Arrow)? {
            if self.check_token(&Token::LeftParen) {
                // Multiple results: -> (result1: type1, result2: type2)
                self.expect_token(&Token::LeftParen)?;
                
                while !self.check_token(&Token::RightParen) {
                    let result_name = if self.peek_is_identifier_followed_by_colon() {
                        Some(self.expect_identifier()?)
                    } else {
                        None
                    };
                    
                    if result_name.is_some() {
                        self.expect_token(&Token::Colon)?;
                    }
                    
                    let result_type = self.parse_type()?;
                    
                    let result = WitResult {
                        name: result_name,
                        ty: result_type,
                    };
                    results.push(result).map_err(|_| self.error("Too many results"))?;
                    
                    if !self.match_token(&Token::Comma)? {
                        break;
                    }
                }
                
                self.expect_token(&Token::RightParen)?;
            } else {
                // Single result: -> type
                let result_type = self.parse_type()?;
                let result = WitResult {
                    name: None,
                    ty: result_type,
                };
                results.push(result).map_err(|_| self.error("Too many results"))?;
            }
        }
        
        Ok(WitFunction {
            name,
            params,
            results,
            is_async: false, // TODO: Parse async keyword
            is_static,
        })
    }
    
    /// Parse a type definition
    fn parse_type_def(&mut self) -> Result<WitTypeDef> {
        self.expect_token(&Token::Type)?;
        
        let name = self.expect_identifier()?;
        
        self.expect_token(&Token::Equals)?;
        
        let ty = match self.current_token.as_ref() {
            Some(Token::Record) => {
                WitTypeDefKind::Record(self.parse_record()?)
            }
            Some(Token::Variant) => {
                WitTypeDefKind::Variant(self.parse_variant()?)
            }
            Some(Token::Enum) => {
                WitTypeDefKind::Enum(self.parse_enum()?)
            }
            Some(Token::Flags) => {
                WitTypeDefKind::Flags(self.parse_flags()?)
            }
            Some(Token::Resource) => {
                WitTypeDefKind::Resource(self.parse_resource()?)
            }
            _ => {
                let wit_type = self.parse_type()?;
                WitTypeDefKind::Type(wit_type)
            }
        };
        
        Ok(WitTypeDef { name, ty })
    }
    
    /// Parse a record type
    fn parse_record(&mut self) -> Result<WitRecord> {
        self.expect_token(&Token::Record)?;
        self.expect_token(&Token::LeftBrace)?;
        
        let mut fields = SimpleBoundedVec::new();
        
        while !self.check_token(&Token::RightBrace) {
            self.skip_trivia()?;
            
            if self.check_token(&Token::RightBrace) {
                break;
            }
            
            let field_name = self.expect_identifier()?;
            self.expect_token(&Token::Colon)?;
            let field_type = self.parse_type()?;
            
            let field = WitRecordField {
                name: field_name,
                ty: field_type,
            };
            fields.push(field).map_err(|_| self.error("Too many record fields"))?;
            
            if !self.match_token(&Token::Comma)? {
                break;
            }
        }
        
        self.skip_trivia()?;
        self.expect_token(&Token::RightBrace)?;
        
        Ok(WitRecord { fields })
    }
    
    /// Parse a variant type
    fn parse_variant(&mut self) -> Result<WitVariant> {
        self.expect_token(&Token::Variant)?;
        self.expect_token(&Token::LeftBrace)?;
        
        let mut cases = SimpleBoundedVec::new();
        
        while !self.check_token(&Token::RightBrace) {
            self.skip_trivia()?;
            
            if self.check_token(&Token::RightBrace) {
                break;
            }
            
            let case_name = self.expect_identifier()?;
            let case_type = if self.match_token(&Token::LeftParen)? {
                let ty = self.parse_type()?;
                self.expect_token(&Token::RightParen)?;
                Some(ty)
            } else {
                None
            };
            
            let case = WitVariantCase {
                name: case_name,
                ty: case_type,
            };
            cases.push(case).map_err(|_| self.error("Too many variant cases"))?;
            
            if !self.match_token(&Token::Comma)? {
                break;
            }
        }
        
        self.skip_trivia()?;
        self.expect_token(&Token::RightBrace)?;
        
        Ok(WitVariant { cases })
    }
    
    /// Parse an enum type
    fn parse_enum(&mut self) -> Result<WitEnum> {
        self.expect_token(&Token::Enum)?;
        self.expect_token(&Token::LeftBrace)?;
        
        let mut cases = SimpleBoundedVec::new();
        
        while !self.check_token(&Token::RightBrace) {
            let case_name = self.expect_identifier()?;
            cases.push(case_name).map_err(|_| self.error("Too many enum cases"))?;
            
            if !self.match_token(&Token::Comma)? {
                break;
            }
        }
        
        self.expect_token(&Token::RightBrace)?;
        
        Ok(WitEnum { cases })
    }
    
    /// Parse a flags type
    fn parse_flags(&mut self) -> Result<WitFlags> {
        self.expect_token(&Token::Flags)?;
        self.expect_token(&Token::LeftBrace)?;
        
        let mut flags = SimpleBoundedVec::new();
        
        while !self.check_token(&Token::RightBrace) {
            let flag_name = self.expect_identifier()?;
            flags.push(flag_name).map_err(|_| self.error("Too many flags"))?;
            
            if !self.match_token(&Token::Comma)? {
                break;
            }
        }
        
        self.expect_token(&Token::RightBrace)?;
        
        Ok(WitFlags { flags })
    }
    
    /// Parse a resource type
    fn parse_resource(&mut self) -> Result<WitResource> {
        self.expect_token(&Token::Resource)?;
        self.expect_token(&Token::LeftBrace)?;
        
        let mut constructor = None;
        let mut methods = SimpleBoundedVec::new();
        let mut static_methods = SimpleBoundedVec::new();
        
        while !self.check_token(&Token::RightBrace) {
            if self.check_token(&Token::Constructor) {
                self.advance()?;
                constructor = Some(self.parse_function()?);
            } else if self.check_token(&Token::Static) {
                let function = self.parse_function()?;
                static_methods.push(function).map_err(|_| self.error("Too many static methods"))?;
            } else if self.check_token(&Token::Func) {
                let function = self.parse_function()?;
                methods.push(function).map_err(|_| self.error("Too many methods"))?;
            } else {
                return Err(self.error("Expected constructor, static function, or method in resource"));
            }
        }
        
        self.expect_token(&Token::RightBrace)?;
        
        Ok(WitResource {
            constructor,
            methods,
            static_methods,
        })
    }
    
    /// Parse a type expression
    fn parse_type(&mut self) -> Result<WitType> {
        match self.current_token.as_ref() {
            // Primitive types
            Some(Token::Bool) => { self.advance()?; Ok(WitType::Bool) }
            Some(Token::U8) => { self.advance()?; Ok(WitType::U8) }
            Some(Token::U16) => { self.advance()?; Ok(WitType::U16) }
            Some(Token::U32) => { self.advance()?; Ok(WitType::U32) }
            Some(Token::U64) => { self.advance()?; Ok(WitType::U64) }
            Some(Token::S8) => { self.advance()?; Ok(WitType::S8) }
            Some(Token::S16) => { self.advance()?; Ok(WitType::S16) }
            Some(Token::S32) => { self.advance()?; Ok(WitType::S32) }
            Some(Token::S64) => { self.advance()?; Ok(WitType::S64) }
            Some(Token::F32) => { self.advance()?; Ok(WitType::F32) }
            Some(Token::F64) => { self.advance()?; Ok(WitType::F64) }
            Some(Token::Char) => { self.advance()?; Ok(WitType::Char) }
            Some(Token::String) => { self.advance()?; Ok(WitType::String) }
            
            // Compound types
            Some(Token::List) => {
                self.advance()?;
                self.expect_token(&Token::LeftAngle)?;
                let element_type = Box::new(self.parse_type()?);
                self.expect_token(&Token::RightAngle)?;
                Ok(WitType::List(element_type))
            }
            
            Some(Token::Option) => {
                self.advance()?;
                self.expect_token(&Token::LeftAngle)?;
                let inner_type = Box::new(self.parse_type()?);
                self.expect_token(&Token::RightAngle)?;
                Ok(WitType::Option(inner_type))
            }
            
            Some(Token::Result) => {
                self.advance()?;
                self.expect_token(&Token::LeftAngle)?;
                
                let ok_type = if !self.check_token(&Token::Comma) && !self.check_token(&Token::RightAngle) {
                    Some(Box::new(self.parse_type()?))
                } else {
                    None
                };
                
                let err_type = if self.match_token(&Token::Comma)? {
                    Some(Box::new(self.parse_type()?))
                } else {
                    None
                };
                
                self.expect_token(&Token::RightAngle)?;
                Ok(WitType::Result { ok: ok_type, err: err_type })
            }
            
            Some(Token::Tuple) => {
                self.advance()?;
                self.expect_token(&Token::LeftAngle)?;
                
                let mut types = SimpleBoundedVec::new();
                
                while !self.check_token(&Token::RightAngle) {
                    let ty = self.parse_type()?;
                    types.push(ty).map_err(|_| self.error("Too many tuple elements"))?;
                    
                    if !self.match_token(&Token::Comma)? {
                        break;
                    }
                }
                
                self.expect_token(&Token::RightAngle)?;
                Ok(WitType::Tuple(types))
            }
            
            Some(Token::Own) => {
                self.advance()?;
                self.expect_token(&Token::LeftAngle)?;
                let resource_name = self.expect_identifier()?;
                self.expect_token(&Token::RightAngle)?;
                Ok(WitType::Own(resource_name))
            }
            
            Some(Token::Borrow) => {
                self.advance()?;
                self.expect_token(&Token::LeftAngle)?;
                let resource_name = self.expect_identifier()?;
                self.expect_token(&Token::RightAngle)?;
                Ok(WitType::Borrow(resource_name))
            }
            
            Some(Token::Identifier(_)) => {
                let type_name = self.expect_identifier()?;
                Ok(WitType::Named(type_name))
            }
            
            _ => Err(self.error("Expected type"))
        }
    }
    
    // Helper methods
    
    /// Advance to the next token
    fn advance(&mut self) -> Result<()> {
        self.current_token = Some(self.lexer.next_token()?);
        self.current_position = self.lexer.position();
        Ok(())
    }
    
    /// Check if current token matches the given token
    fn check_token(&self, token: &Token) -> bool {
        if let Some(ref current) = self.current_token {
            std::mem::discriminant(current) == std::mem::discriminant(token)
        } else {
            false
        }
    }
    
    /// Match and consume token if it matches
    fn match_token(&mut self, token: &Token) -> Result<bool> {
        if self.check_token(token) {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Expect a specific token
    fn expect_token(&mut self, expected: &Token) -> Result<()> {
        if self.check_token(expected) {
            self.advance()
        } else {
            Err(self.error("Expected token"))
        }
    }
    
    /// Expect an identifier token
    fn expect_identifier(&mut self) -> Result<SimpleBoundedString<MAX_WIT_IDENTIFIER_LEN>> {
        if let Some(Token::Identifier(id)) = &self.current_token {
            let result = id.clone();
            self.advance()?;
            Ok(result)
        } else {
            Err(self.error("Expected identifier"))
        }
    }
    
    /// Check if we're at end of input
    fn is_at_end(&self) -> bool {
        matches!(self.current_token, Some(Token::EndOfFile))
    }
    
    /// Skip whitespace, newlines, and comments
    fn skip_trivia(&mut self) -> Result<()> {
        while let Some(ref token) = self.current_token {
            match token {
                Token::Whitespace | Token::NewLine | Token::Comment(_) => {
                    self.advance()?;
                }
                _ => break,
            }
        }
        Ok(())
    }
    
    /// Check if next token is identifier followed by colon
    fn peek_is_identifier_followed_by_colon(&mut self) -> bool {
        if let Ok(next_token) = self.lexer.peek_token() {
            matches!(next_token, Token::Colon)
        } else {
            false
        }
    }
    
    /// Create an error with current position
    fn error(&self, _message: &str) -> Error {
        Error::new(
            ErrorCategory::Parse,
            codes::PARSE_ERROR,
            "WIT parse error"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_interface() {
        let source = r#"
            interface my-interface {
                func hello() -> string
            }
        "#;
        
        let mut parser = WitParser::new(source).unwrap();
        let document = parser.parse_document().unwrap();
        
        assert_eq!(document.interfaces.len(), 1);
        assert_eq!(document.interfaces[0].name.as_str(), "my-interface");
        assert_eq!(document.interfaces[0].functions.len(), 1);
        assert_eq!(document.interfaces[0].functions[0].name.as_str(), "hello");
    }
    
    #[test]
    fn test_parse_package_declaration() {
        let source = "package wasi:cli@0.2.0";
        
        let mut parser = WitParser::new(source).unwrap();
        let document = parser.parse_document().unwrap();
        
        assert!(document.package.is_some());
        let package = document.package.unwrap();
        assert_eq!(package.namespace.as_str(), "wasi");
        assert_eq!(package.name.as_str(), "cli");
        assert_eq!(package.version.unwrap().as_str(), "0.2.0");
    }
    
    #[test]
    fn test_parse_type_definitions() {
        let source = r#"
            interface types {
                type my-record = record {
                    field1: string,
                    field2: u32
                }
                
                type my-variant = variant {
                    case1,
                    case2(string)
                }
            }
        "#;
        
        let mut parser = WitParser::new(source).unwrap();
        let document = parser.parse_document().unwrap();
        
        assert_eq!(document.interfaces.len(), 1);
        assert_eq!(document.interfaces[0].types.len(), 2);
    }
}