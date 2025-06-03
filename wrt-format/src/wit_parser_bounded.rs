//! Bounded WIT (WebAssembly Interface Types) parser for no_std environments
//!
//! This module provides basic WIT parsing capabilities using bounded collections,
//! enabling WIT support in pure no_std environments without allocation.

use wrt_foundation::{BoundedString, MemoryProvider, NoStdProvider};
use wrt_error::{Error, Result};
use crate::MAX_WASM_STRING_SIZE;

/// Bounded WIT name for no_std environments
pub type BoundedWitName<P> = BoundedString<MAX_WASM_STRING_SIZE, P>;

/// Simple bounded WIT parser for no_std environments
#[derive(Debug, Clone)]
pub struct BoundedWitParser<P: MemoryProvider + Default + Clone + PartialEq + Eq = NoStdProvider<4096>> {
    /// Input text being parsed
    input: BoundedString<8192, P>, // 8KB input buffer
    /// Parsed worlds (simplified)
    worlds: [Option<BoundedWitWorld<P>>; 4], // Maximum 4 worlds
    /// Parsed interfaces (simplified)
    interfaces: [Option<BoundedWitInterface<P>>; 8], // Maximum 8 interfaces
    /// Number of parsed worlds
    world_count: usize,
    /// Number of parsed interfaces
    interface_count: usize,
    /// Memory provider
    provider: P,
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

/// Simple bounded WIT world definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWitWorld<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// World name
    pub name: BoundedWitName<P>,
    /// Simple import/export counters for basic functionality
    pub import_count: u32,
    pub export_count: u32,
}

/// Simple bounded WIT interface definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWitInterface<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Interface name
    pub name: BoundedWitName<P>,
    /// Simple function counter for basic functionality
    pub function_count: u32,
}

/// Simple bounded WIT function definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWitFunction<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Function name
    pub name: BoundedWitName<P>,
    /// Parameter count (simplified)
    pub param_count: u32,
    /// Result count (simplified)
    pub result_count: u32,
}

/// Simple bounded WIT type definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundedWitType<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Primitive types
    Bool,
    U8, U16, U32, U64,
    S8, S16, S32, S64,
    F32, F64,
    Char,
    String,
    
    /// Named type reference
    Named {
        name: BoundedWitName<P>,
    },
    
    /// Unknown/unsupported type
    Unknown,
}

/// Simple bounded import definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWitImport<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Import name
    pub name: BoundedWitName<P>,
    /// Import is a function (simplified)
    pub is_function: bool,
}

/// Simple bounded export definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedWitExport<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    /// Export name
    pub name: BoundedWitName<P>,
    /// Export is a function (simplified)
    pub is_function: bool,
}

impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> BoundedWitParser<P> {
    /// Create a new bounded WIT parser
    pub fn new(provider: P) -> Result<Self> {
        Ok(Self {
            input: BoundedString::from_str("", provider.clone())
                .map_err(|_| Error::new(crate::ErrorCategory::Runtime, wrt_error::codes::MEMORY_ERROR, "Failed to create input buffer"))?,
            worlds: [None, None, None, None],
            interfaces: [None, None, None, None, None, None, None, None],
            world_count: 0,
            interface_count: 0,
            provider,
        })
    }

    /// Parse WIT text input (simplified)
    pub fn parse(&mut self, input: &str) -> Result<()> {
        // Store input in bounded buffer
        self.input = BoundedString::from_str(input, self.provider.clone())
            .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Input too large for bounded buffer"))?;
        
        // Reset parser state
        self.worlds = [None, None, None, None];
        self.interfaces = [None, None, None, None, None, None, None, None];
        self.world_count = 0;
        self.interface_count = 0;

        // Simple parsing - look for "world" and "interface" keywords
        self.simple_parse()?;
        
        Ok(())
    }

    /// Simple parsing implementation
    fn simple_parse(&mut self) -> Result<()> {
        // Extract the input data without holding a borrow
        let input_bytes = {
            let input_str = self.input.as_str()
                .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Invalid UTF-8 in input"))?;
            let bytes = input_str.as_bytes();
            let mut bounded_bytes = wrt_foundation::bounded::BoundedVec::<u8, 8192, P>::new_with_provider(self.provider.clone())
                .map_err(|_| Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Failed to create byte buffer"))?;
            for &byte in bytes.iter().take(bounded_bytes.capacity()) {
                if bounded_bytes.try_push(byte).is_err() {
                    break;
                }
            }
            bounded_bytes
        };
        
        let mut position = 0;
        
        while position < input_bytes.len() {
            // Skip whitespace
            while position < input_bytes.len() && input_bytes[position].is_ascii_whitespace() {
                position += 1;
            }
            
            if position >= input_bytes.len() {
                break;
            }
            
            // Look for keywords
            if let Some(word) = self.read_word(&input_bytes, &mut position) {
                if let Ok(word_str) = word.as_str() {
                    match word_str {
                        "world" => {
                            if let Some(name) = self.read_word(&input_bytes, &mut position) {
                                self.add_world(name)?;
                            }
                        }
                        "interface" => {
                            if let Some(name) = self.read_word(&input_bytes, &mut position) {
                                self.add_interface(name)?;
                            }
                        }
                        _ => {
                            // Skip unknown words
                        }
                    }
                }
            }
            
            position += 1;
        }
        
        Ok(())
    }

    /// Read a word from byte stream (no_std compatible)
    fn read_word(&self, bytes: &wrt_foundation::bounded::BoundedVec<u8, 8192, P>, position: &mut usize) -> Option<BoundedWitName<P>> {
        // Skip whitespace
        while *position < bytes.len() && bytes[*position].is_ascii_whitespace() {
            *position += 1;
        }
        
        if *position >= bytes.len() {
            return None;
        }
        
        let start = *position;
        
        // Read alphanumeric characters, hyphens, and underscores
        while *position < bytes.len() {
            let byte = bytes[*position];
            if byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' {
                *position += 1;
            } else {
                break;
            }
        }
        
        if *position > start {
            // Convert bytes to bounded string (ASCII safe)
            let word_bytes = &bytes[start..*position];
            if let Ok(word_str) = core::str::from_utf8(word_bytes) {
                BoundedWitName::from_str(word_str, self.provider.clone()).ok()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Add a world to the parser
    fn add_world(&mut self, name: BoundedWitName<P>) -> Result<()> {
        if self.world_count >= self.worlds.len() {
            return Err(Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Too many worlds"));
        }
        
        let world = BoundedWitWorld {
            name,
            import_count: 0,
            export_count: 0,
        };
        
        self.worlds[self.world_count] = Some(world);
        self.world_count += 1;
        
        Ok(())
    }

    /// Add an interface to the parser
    fn add_interface(&mut self, name: BoundedWitName<P>) -> Result<()> {
        if self.interface_count >= self.interfaces.len() {
            return Err(Error::new(crate::ErrorCategory::Parse, wrt_error::codes::PARSE_ERROR, "Too many interfaces"));
        }
        
        let interface = BoundedWitInterface {
            name,
            function_count: 0,
        };
        
        self.interfaces[self.interface_count] = Some(interface);
        self.interface_count += 1;
        
        Ok(())
    }

    /// Get parsed worlds
    pub fn worlds(&self) -> impl Iterator<Item = &BoundedWitWorld<P>> {
        self.worlds.iter().filter_map(|w| w.as_ref())
    }

    /// Get parsed interfaces
    pub fn interfaces(&self) -> impl Iterator<Item = &BoundedWitInterface<P>> {
        self.interfaces.iter().filter_map(|i| i.as_ref())
    }

    /// Get world count
    pub fn world_count(&self) -> usize {
        self.world_count
    }

    /// Get interface count
    pub fn interface_count(&self) -> usize {
        self.interface_count
    }
}

impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> Default for BoundedWitParser<P> {
    fn default() -> Self {
        Self::new(P::default()).unwrap_or_else(|_| {
            // Fallback to empty parser if creation fails
            Self {
                input: BoundedString::from_str("", P::default()).unwrap(),
                worlds: [None, None, None, None],
                interfaces: [None, None, None, None, None, None, None, None],
                world_count: 0,
                interface_count: 0,
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
        assert_eq!(parser.world_count(), 0);
        assert_eq!(parser.interface_count(), 0);
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
        assert_eq!(parser.world_count(), 1);

        let mut worlds = parser.worlds();
        let world = worlds.next().unwrap();
        assert_eq!(world.name.as_str().unwrap(), "test-world");
    }

    #[test]
    fn test_interface_parsing() {
        let wit_text = r#"
            interface test-interface {
                test-func: func(a: u32, b: string) -> bool
            }
        "#;

        let result = parse_wit_bounded(wit_text);
        assert!(result.is_ok());

        let parser = result.unwrap();
        assert_eq!(parser.interface_count(), 1);

        let mut interfaces = parser.interfaces();
        let interface = interfaces.next().unwrap();
        assert_eq!(interface.name.as_str().unwrap(), "test-interface");
    }

    #[test]
    fn test_multiple_definitions() {
        let wit_text = r#"
            world world1 {}
            interface interface1 {}
            world world2 {}
            interface interface2 {}
        "#;

        let result = parse_wit_bounded(wit_text);
        assert!(result.is_ok());

        let parser = result.unwrap();
        assert_eq!(parser.world_count(), 2);
        assert_eq!(parser.interface_count(), 2);
    }

    #[test]
    fn test_bounded_capacity_limits() {
        // Test that parser respects bounded collection limits
        let mut parser = BoundedWitParser::new(TestProvider::default()).unwrap();
        
        // Create input with many worlds (should hit limit)
        let mut large_input = String::new();
        for i in 0..10 {
            large_input.push_str(&format!("world world{} {{}}\n", i));
        }
        
        let result = parser.parse(&large_input);
        assert!(result.is_ok());
        
        // Should have parsed up to the limit
        assert!(parser.world_count() <= 4);
    }

    #[test]
    fn test_error_handling() {
        let invalid_wit = "invalid wit syntax {{{";
        let result = parse_wit_bounded(invalid_wit);
        
        // Should handle gracefully (may parse partially or succeed with no results)
        assert!(result.is_ok());
        let parser = result.unwrap();
        assert_eq!(parser.world_count(), 0);
        assert_eq!(parser.interface_count(), 0);
    }

    #[test]
    fn test_empty_input() {
        let result = parse_wit_bounded("");
        assert!(result.is_ok());

        let parser = result.unwrap();
        assert_eq!(parser.world_count(), 0);
        assert_eq!(parser.interface_count(), 0);
    }

    #[test]
    fn test_whitespace_handling() {
        let wit_text = "   world   test-world   {}   ";

        let result = parse_wit_bounded(wit_text);
        assert!(result.is_ok());

        let parser = result.unwrap();
        assert_eq!(parser.world_count(), 1);

        let mut worlds = parser.worlds();
        let world = worlds.next().unwrap();
        assert_eq!(world.name.as_str().unwrap(), "test-world");
    }
}