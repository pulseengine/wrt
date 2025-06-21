//! WIT lexer/tokenizer
//!
//! Provides efficient tokenization of WIT source code with memory-bounded operation.

use wrt_error::{Error, ErrorCategory, Result, codes};
use crate::bounded_types::SimpleBoundedString;

/// Maximum token length
pub const MAX_TOKEN_LEN: usize = 128;

/// Position in source code
#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    /// Line number (1-based)
    pub line: u32,
    /// Column number (1-based)
    pub column: u32,
    /// Byte offset in source
    pub offset: usize,
}

/// Source span covering a range in the source
#[derive(Debug, Clone, Copy)]
pub struct Span {
    /// Start position
    pub start: Position,
    /// End position
    pub end: Position,
}

/// WIT token types
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Identifier(SimpleBoundedString<MAX_TOKEN_LEN>),
    StringLiteral(SimpleBoundedString<MAX_TOKEN_LEN>),
    IntegerLiteral(u64),
    
    // Keywords
    Package,
    Interface,
    World,
    Use,
    Type,
    Record,
    Variant,
    Enum,
    Flags,
    Resource,
    Func,
    Constructor,
    Static,
    Import,
    Export,
    As,
    From,
    With,
    Include,
    
    // Primitive types
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
    
    // Compound type keywords
    List,
    Option,
    Result,
    Tuple,
    Own,
    Borrow,
    
    // Punctuation
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    LeftAngle,      // <
    RightAngle,     // >
    Comma,          // ,
    Semicolon,      // ;
    Colon,          // :
    DoubleColon,    // ::
    Arrow,          // ->
    Dot,            // .
    At,             // @
    Equals,         // =
    Pipe,           // |
    Star,           // *
    
    // Whitespace and comments
    Whitespace,
    Comment(SimpleBoundedString<MAX_TOKEN_LEN>),
    
    // Special tokens
    NewLine,
    EndOfFile,
}

/// WIT lexer state
#[derive(Debug)]
pub struct WitLexer<'a> {
    /// Source code
    source: &'a str,
    /// Current position in source
    current: usize,
    /// Current line
    line: u32,
    /// Current column
    column: u32,
    /// Whether we've reached end of file
    at_eof: bool,
}

impl<'a> WitLexer<'a> {
    /// Create a new WIT lexer
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            current: 0,
            line: 1,
            column: 1,
            at_eof: false,
        }
    }
    
    /// Get current position
    pub fn position(&self) -> Position {
        Position {
            line: self.line,
            column: self.column,
            offset: self.current,
        }
    }
    
    /// Peek at current character without consuming
    fn peek_char(&self) -> Option<char> {
        self.source.chars().nth(self.current)
    }
    
    /// Consume and return current character
    fn next_char(&mut self) -> Option<char> {
        if let Some(ch) = self.source.chars().nth(self.current) {
            self.current += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            self.at_eof = true;
            None
        }
    }
    
    /// Peek at character at offset without consuming
    fn peek_char_at(&self, offset: usize) -> Option<char> {
        self.source.chars().nth(self.current + offset)
    }
    
    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() && ch != '\n' {
                self.next_char();
            } else {
                break;
            }
        }
    }
    
    /// Read an identifier or keyword
    fn read_identifier(&mut self) -> Result<Token> {
        let mut identifier = String::new();
        
        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                identifier.push(ch);
                self.next_char();
            } else {
                break;
            }
        }
        
        if identifier.len() > MAX_TOKEN_LEN {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Identifier too long"
            ));
        }
        
        // Check for keywords
        let token = match identifier.as_str() {
            "package" => Token::Package,
            "interface" => Token::Interface,
            "world" => Token::World,
            "use" => Token::Use,
            "type" => Token::Type,
            "record" => Token::Record,
            "variant" => Token::Variant,
            "enum" => Token::Enum,
            "flags" => Token::Flags,
            "resource" => Token::Resource,
            "func" => Token::Func,
            "constructor" => Token::Constructor,
            "static" => Token::Static,
            "import" => Token::Import,
            "export" => Token::Export,
            "as" => Token::As,
            "from" => Token::From,
            "with" => Token::With,
            "include" => Token::Include,
            
            // Primitive types
            "bool" => Token::Bool,
            "u8" => Token::U8,
            "u16" => Token::U16,
            "u32" => Token::U32,
            "u64" => Token::U64,
            "s8" => Token::S8,
            "s16" => Token::S16,
            "s32" => Token::S32,
            "s64" => Token::S64,
            "f32" => Token::F32,
            "f64" => Token::F64,
            "char" => Token::Char,
            "string" => Token::String,
            
            // Compound types
            "list" => Token::List,
            "option" => Token::Option,
            "result" => Token::Result,
            "tuple" => Token::Tuple,
            "own" => Token::Own,
            "borrow" => Token::Borrow,
            
            _ => {
                let bounded_id = SimpleBoundedString::from_str(&identifier);
                Token::Identifier(bounded_id)
            }
        };
        
        Ok(token)
    }
    
    /// Read a string literal
    fn read_string_literal(&mut self) -> Result<Token> {
        let mut string_content = String::new();
        
        // Consume opening quote
        self.next_char();
        
        while let Some(ch) = self.peek_char() {
            if ch == '"' {
                self.next_char(); // Consume closing quote
                break;
            } else if ch == '\\' {
                self.next_char(); // Consume backslash
                if let Some(escaped) = self.next_char() {
                    match escaped {
                        'n' => string_content.push('\n'),
                        't' => string_content.push('\t'),
                        'r' => string_content.push('\r'),
                        '\\' => string_content.push('\\'),
                        '"' => string_content.push('"'),
                        _ => {
                            string_content.push('\\');
                            string_content.push(escaped);
                        }
                    }
                } else {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Unexpected end of file in string literal"
                    ));
                }
            } else if ch == '\n' {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Unterminated string literal"
                ));
            } else {
                string_content.push(ch);
                self.next_char();
            }
        }
        
        if string_content.len() > MAX_TOKEN_LEN {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "String literal too long"
            ));
        }
        
        let bounded_string = SimpleBoundedString::from_str(&string_content);
        Ok(Token::StringLiteral(bounded_string))
    }
    
    /// Read an integer literal or version string
    fn read_integer_literal(&mut self) -> Result<Token> {
        let mut number_str = String::new();
        let mut has_dot = false;
        
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                number_str.push(ch);
                self.next_char();
            } else if ch == '.' {
                // Check if this looks like a version string (digit follows dot)
                if let Some(next_ch) = self.peek_char_at(1) {
                    if next_ch.is_ascii_digit() {
                        number_str.push(ch);
                        has_dot = true;
                        self.next_char();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if has_dot {
            // This is a version string, treat as identifier
            let bounded_string = SimpleBoundedString::from_str(&number_str);
            Ok(Token::Identifier(bounded_string))
        } else {
            // This is a pure integer
            let number = number_str.parse::<u64>().map_err(|_| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Invalid integer literal"
            ))?;
            
            Ok(Token::IntegerLiteral(number))
        }
    }
    
    /// Read a comment
    fn read_comment(&mut self) -> Result<Token> {
        let mut comment_content = String::new();
        
        // Skip the initial //
        self.next_char();
        self.next_char();
        
        while let Some(ch) = self.peek_char() {
            if ch == '\n' {
                break;
            } else {
                comment_content.push(ch);
                self.next_char();
            }
        }
        
        let bounded_comment = SimpleBoundedString::from_str(&comment_content);
        Ok(Token::Comment(bounded_comment))
    }
    
    /// Get the next token
    pub fn next_token(&mut self) -> Result<Token> {
        if self.at_eof {
            return Ok(Token::EndOfFile);
        }
        
        self.skip_whitespace();
        
        let ch = match self.peek_char() {
            Some(ch) => ch,
            None => {
                self.at_eof = true;
                return Ok(Token::EndOfFile);
            }
        };
        
        match ch {
            '\n' => {
                self.next_char();
                Ok(Token::NewLine)
            }
            
            // Single character tokens
            '(' => { self.next_char(); Ok(Token::LeftParen) }
            ')' => { self.next_char(); Ok(Token::RightParen) }
            '{' => { self.next_char(); Ok(Token::LeftBrace) }
            '}' => { self.next_char(); Ok(Token::RightBrace) }
            '[' => { self.next_char(); Ok(Token::LeftBracket) }
            ']' => { self.next_char(); Ok(Token::RightBracket) }
            '<' => { self.next_char(); Ok(Token::LeftAngle) }
            '>' => { self.next_char(); Ok(Token::RightAngle) }
            ',' => { self.next_char(); Ok(Token::Comma) }
            ';' => { self.next_char(); Ok(Token::Semicolon) }
            '.' => { self.next_char(); Ok(Token::Dot) }
            '@' => { self.next_char(); Ok(Token::At) }
            '=' => { self.next_char(); Ok(Token::Equals) }
            '|' => { self.next_char(); Ok(Token::Pipe) }
            '*' => { self.next_char(); Ok(Token::Star) }
            
            // Multi-character tokens
            ':' => {
                if self.peek_char_at(1) == Some(':') {
                    self.next_char();
                    self.next_char();
                    Ok(Token::DoubleColon)
                } else {
                    self.next_char();
                    Ok(Token::Colon)
                }
            }
            
            '-' => {
                if self.peek_char_at(1) == Some('>') {
                    self.next_char();
                    self.next_char();
                    Ok(Token::Arrow)
                } else if ch.is_ascii_alphabetic() || ch == '_' {
                    self.read_identifier()
                } else {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Unexpected character '-'"
                    ));
                }
            }
            
            '/' => {
                if self.peek_char_at(1) == Some('/') {
                    self.read_comment()
                } else {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        "Unexpected character '/'"
                    ));
                }
            }
            
            '"' => self.read_string_literal(),
            
            _ if ch.is_ascii_alphabetic() || ch == '_' => self.read_identifier(),
            
            _ if ch.is_ascii_digit() => self.read_integer_literal(),
            
            _ => {
                Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Unexpected character"
                ))
            }
        }
    }
    
    /// Peek at the next token without consuming it
    pub fn peek_token(&mut self) -> Result<Token> {
        let saved_state = WitLexer {
            source: self.source,
            current: self.current,
            line: self.line,
            column: self.column,
            at_eof: self.at_eof,
        };
        
        let token = self.next_token()?;
        
        // Restore state
        self.current = saved_state.current;
        self.line = saved_state.line;
        self.column = saved_state.column;
        self.at_eof = saved_state.at_eof;
        
        Ok(token)
    }
    
    /// Check if we're at end of file
    pub fn is_at_eof(&self) -> bool {
        self.at_eof
    }
}

impl Default for Span {
    fn default() -> Self {
        Self {
            start: Position::default(),
            end: Position::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_keywords() {
        let mut lexer = WitLexer::new("interface world package");
        
        assert_eq!(lexer.next_token().unwrap(), Token::Interface);
        assert_eq!(lexer.next_token().unwrap(), Token::World);
        assert_eq!(lexer.next_token().unwrap(), Token::Package);
        assert_eq!(lexer.next_token().unwrap(), Token::EndOfFile);
    }
    
    #[test]
    fn test_lexer_identifiers() {
        let mut lexer = WitLexer::new("my-interface test_world");
        
        if let Token::Identifier(id) = lexer.next_token().unwrap() {
            assert_eq!(id.as_str(), "my-interface");
        } else {
            panic!("Expected identifier");
        }
        
        if let Token::Identifier(id) = lexer.next_token().unwrap() {
            assert_eq!(id.as_str(), "test_world");
        } else {
            panic!("Expected identifier");
        }
    }
    
    #[test]
    fn test_lexer_punctuation() {
        let mut lexer = WitLexer::new("() {} :: ->");
        
        assert_eq!(lexer.next_token().unwrap(), Token::LeftParen);
        assert_eq!(lexer.next_token().unwrap(), Token::RightParen);
        assert_eq!(lexer.next_token().unwrap(), Token::LeftBrace);
        assert_eq!(lexer.next_token().unwrap(), Token::RightBrace);
        assert_eq!(lexer.next_token().unwrap(), Token::DoubleColon);
        assert_eq!(lexer.next_token().unwrap(), Token::Arrow);
    }
    
    #[test]
    fn test_lexer_string_literal() {
        let mut lexer = WitLexer::new(r#""hello world""#);
        
        if let Token::StringLiteral(s) = lexer.next_token().unwrap() {
            assert_eq!(s.as_str(), "hello world");
        } else {
            panic!("Expected string literal");
        }
    }
    
    #[test]
    fn test_lexer_comments() {
        let mut lexer = WitLexer::new("// this is a comment\ninterface");
        
        if let Token::Comment(c) = lexer.next_token().unwrap() {
            assert_eq!(c.as_str(), " this is a comment");
        } else {
            panic!("Expected comment");
        }
        
        assert_eq!(lexer.next_token().unwrap(), Token::NewLine);
        assert_eq!(lexer.next_token().unwrap(), Token::Interface);
    }
}