//! Unified WebAssembly parser with automatic format detection
//!
//! This module provides a single parser that can handle both Core WebAssembly
//! modules and Component Model binaries, automatically detecting the format
//! and routing to the appropriate parser implementation.

use wrt_error::{Error, ErrorCategory, Result, codes};
use crate::{
    format_detection::{FormatDetector, BinaryFormat, BinaryInfo},
    simple_parser::SimpleParser,
    component_section_parser::ComponentSectionParser,
    enhanced_module::{EnhancedModule, ParserMode},
    component_registry::ComponentRegistry,
    component_types::ComponentMemoryBudget,
    simple_module::SimpleModule,
};

/// Unified parser configuration
#[derive(Debug, Clone)]
pub struct UnifiedParserConfig {
    /// Parser mode for enhanced modules
    pub parser_mode: ParserMode,
    
    /// Force a specific format (skip auto-detection)
    pub force_format: Option<BinaryFormat>,
    
    /// Enable validation during parsing
    pub validate_during_parse: bool,
    
    /// Component Model memory budget (when enabled)
    pub component_memory_budget: usize,
    
    /// Maximum number of component types
    pub max_component_types: usize,
}

impl Default for UnifiedParserConfig {
    fn default() -> Self {
        Self {
            parser_mode: ParserMode::CoreOnly,
            force_format: None,
            validate_during_parse: true,
            component_memory_budget: 64 * 1024, // 64KB
            max_component_types: 512,
        }
    }
}

impl UnifiedParserConfig {
    /// Create config for Core WebAssembly only
    pub fn core_only() -> Self {
        Self {
            parser_mode: ParserMode::CoreOnly,
            ..Default::default()
        }
    }
    
    /// Create config with Component Model support
    pub fn with_components() -> Self {
        Self {
            parser_mode: ParserMode::ComponentAware {
                type_budget: 64 * 1024,
                max_types: 512,
            },
            ..Default::default()
        }
    }
    
    /// Create config with custom Component Model settings
    pub fn with_component_budget(memory_budget: usize, max_types: usize) -> Self {
        Self {
            parser_mode: ParserMode::ComponentAware {
                type_budget: memory_budget,
                max_types,
            },
            component_memory_budget: memory_budget,
            max_component_types: max_types,
            ..Default::default()
        }
    }
    
    /// Force parsing as a specific format
    pub fn force_format(mut self, format: BinaryFormat) -> Self {
        self.force_format = Some(format);
        self
    }
    
    /// Disable validation during parsing (for performance)
    pub fn no_validation(mut self) -> Self {
        self.validate_during_parse = false;
        self
    }
}

/// Unified WebAssembly parser with automatic format detection
/// 
/// This parser can handle both Core WebAssembly modules and Component Model
/// binaries, automatically detecting the format and using the appropriate
/// parsing strategy.
#[derive(Debug)]
pub struct UnifiedParser {
    /// Configuration for this parser
    config: UnifiedParserConfig,
    
    /// Format detector for automatic format identification
    format_detector: FormatDetector,
    
    /// Core WebAssembly parser
    core_parser: SimpleParser,
    
    /// Component Model section parser
    component_parser: ComponentSectionParser,
    
    /// Last detected binary info
    last_binary_info: Option<BinaryInfo>,
}

impl UnifiedParser {
    /// Create a new unified parser with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(UnifiedParserConfig::default())
    }
    
    /// Create a unified parser with custom configuration
    pub fn with_config(config: UnifiedParserConfig) -> Result<Self> {
        let component_registry = if matches!(config.parser_mode, ParserMode::ComponentAware { .. }) {
            let budget = ComponentMemoryBudget::with_limits(
                config.component_memory_budget, 
                8 * 1024 // 8KB reserved
            );
            ComponentRegistry::with_memory_budget(budget)
        } else {
            ComponentRegistry::new()
        };
        
        Ok(Self {
            config,
            format_detector: FormatDetector::new(),
            core_parser: SimpleParser::new(),
            component_parser: ComponentSectionParser::with_registry(component_registry),
            last_binary_info: None,
        })
    }
    
    /// Parse a WebAssembly binary with automatic format detection
    /// 
    /// This is the main entry point that replaces both wrt-decoder and
    /// wrt-format parsing functionality.
    pub fn parse(&mut self, binary: &[u8]) -> Result<UnifiedParseResult> {
        // Detect format (unless forced)
        let format = match self.config.force_format {
            Some(forced_format) => {
                // Validate the forced format if validation is enabled
                if self.config.validate_during_parse {
                    self.format_detector.validate_format(binary, forced_format)?;
                }
                forced_format
            }
            None => self.format_detector.detect_format(binary)?,
        };
        
        // Store binary info for debugging/introspection
        self.last_binary_info = Some(BinaryInfo::from_binary(binary)?);
        
        // Route to appropriate parser based on detected format
        match format {
            BinaryFormat::CoreModule => self.parse_core_module(binary),
            BinaryFormat::Component => self.parse_component(binary),
            BinaryFormat::Unknown => Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unknown or invalid WebAssembly binary format"
            )),
        }
    }
    
    /// Parse a Core WebAssembly module
    fn parse_core_module(&mut self, binary: &[u8]) -> Result<UnifiedParseResult> {
        // Parse using existing SimpleParser
        let simple_module = self.core_parser.parse(binary)?;
        
        // Create enhanced module based on parser mode
        let enhanced_module = match self.config.parser_mode {
            ParserMode::CoreOnly => EnhancedModule::new_core_only(simple_module),
            ParserMode::ComponentAware { .. } => {
                EnhancedModule::new_with_component(simple_module, self.config.parser_mode)?
            }
        };
        
        Ok(UnifiedParseResult::CoreModule(enhanced_module))
    }
    
    /// Parse a Component Model binary
    fn parse_component(&mut self, binary: &[u8]) -> Result<UnifiedParseResult> {
        // Ensure Component Model support is enabled
        if matches!(self.config.parser_mode, ParserMode::CoreOnly) {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Component Model binary detected, but Component Model support is disabled"
            ));
        }
        
        // Create enhanced module with Component Model support
        let core_module = SimpleModule::new(); // Empty core for pure components
        let mut enhanced_module = EnhancedModule::new_with_component(
            core_module, 
            self.config.parser_mode
        )?;
        
        // Parse component sections
        self.parse_component_sections(binary, &mut enhanced_module)?;
        
        Ok(UnifiedParseResult::Component(enhanced_module))
    }
    
    /// Parse Component Model sections from binary
    fn parse_component_sections(
        &mut self, 
        binary: &[u8], 
        module: &mut EnhancedModule
    ) -> Result<()> {
        // Skip the header (magic + version + layer)
        let binary_info = self.last_binary_info.as_ref().unwrap();
        let mut offset = binary_info.data_start_offset();
        
        // Parse all sections
        while offset < binary.len() {
            // Read section ID
            if offset >= binary.len() {
                break;
            }
            let section_id = binary[offset];
            offset += 1;
            
            // Read section size (LEB128)
            let (section_size, leb_bytes) = self.read_leb128_u32(&binary[offset..])?;
            offset += leb_bytes;
            
            // Validate section bounds
            if offset + section_size as usize > binary.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Section extends beyond binary bounds"
                ));
            }
            
            // Extract section data
            let section_data = &binary[offset..offset + section_size as usize];
            
            // Parse the section using component parser
            self.component_parser.parse_component_section(section_id, section_data, module)?;
            
            // Move to next section
            offset += section_size as usize;
        }
        
        Ok(())
    }
    
    /// Read LEB128 unsigned 32-bit integer
    fn read_leb128_u32(&self, data: &[u8]) -> Result<(u32, usize)> {
        crate::leb128::read_leb128_u32(data, 0)
    }
    
    /// Get the last detected binary information
    pub fn last_binary_info(&self) -> Option<&BinaryInfo> {
        self.last_binary_info.as_ref()
    }
    
    /// Get the current parser configuration
    pub fn config(&self) -> &UnifiedParserConfig {
        &self.config
    }
    
    /// Get mutable access to the core parser (for advanced usage)
    pub fn core_parser_mut(&mut self) -> &mut SimpleParser {
        &mut self.core_parser
    }
    
    /// Get mutable access to the component parser (for advanced usage)
    pub fn component_parser_mut(&mut self) -> &mut ComponentSectionParser {
        &mut self.component_parser
    }
    
    /// Check if Component Model support is enabled
    pub fn has_component_support(&self) -> bool {
        matches!(self.config.parser_mode, ParserMode::ComponentAware { .. })
    }
    
    /// Get component registry memory usage (if Component Model enabled)
    pub fn component_memory_usage(&self) -> Option<(usize, usize)> {
        if self.has_component_support() {
            Some(self.component_parser.registry().memory_usage())
        } else {
            None
        }
    }
}

impl Default for UnifiedParser {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Result of unified parsing operation
#[derive(Debug)]
pub enum UnifiedParseResult {
    /// Core WebAssembly module
    CoreModule(EnhancedModule),
    /// Component Model binary
    Component(EnhancedModule),
}

impl UnifiedParseResult {
    /// Get the enhanced module regardless of format
    pub fn into_enhanced_module(self) -> EnhancedModule {
        match self {
            UnifiedParseResult::CoreModule(module) => module,
            UnifiedParseResult::Component(module) => module,
        }
    }
    
    /// Get the core module (for backward compatibility)
    pub fn into_core_module(self) -> SimpleModule {
        self.into_enhanced_module().into_core()
    }
    
    /// Check if this is a Component Model result
    pub fn is_component(&self) -> bool {
        matches!(self, UnifiedParseResult::Component(_))
    }
    
    /// Check if this is a Core WebAssembly result
    pub fn is_core_module(&self) -> bool {
        matches!(self, UnifiedParseResult::CoreModule(_))
    }
    
    /// Get a reference to the enhanced module
    pub fn as_enhanced_module(&self) -> &EnhancedModule {
        match self {
            UnifiedParseResult::CoreModule(module) => module,
            UnifiedParseResult::Component(module) => module,
        }
    }
    
    /// Get a mutable reference to the enhanced module
    pub fn as_enhanced_module_mut(&mut self) -> &mut EnhancedModule {
        match self {
            UnifiedParseResult::CoreModule(module) => module,
            UnifiedParseResult::Component(module) => module,
        }
    }
}

/// Convenience function to parse any WebAssembly binary
/// 
/// This function provides a simple API that replaces the need for separate
/// wrt-decoder and wrt-format parsing calls.
pub fn parse_wasm_binary(binary: &[u8]) -> Result<UnifiedParseResult> {
    let mut parser = UnifiedParser::new()?;
    parser.parse(binary)
}

/// Parse WebAssembly binary with custom configuration
pub fn parse_wasm_with_config(
    binary: &[u8], 
    config: UnifiedParserConfig
) -> Result<UnifiedParseResult> {
    let mut parser = UnifiedParser::with_config(config)?;
    parser.parse(binary)
}

/// Parse Core WebAssembly module only (equivalent to old SimpleParser)
pub fn parse_core_module(binary: &[u8]) -> Result<SimpleModule> {
    let config = UnifiedParserConfig::core_only().force_format(BinaryFormat::CoreModule);
    let result = parse_wasm_with_config(binary, config)?;
    Ok(result.into_core_module())
}

/// Parse Component Model binary only
pub fn parse_component(binary: &[u8]) -> Result<EnhancedModule> {
    let config = UnifiedParserConfig::with_components().force_format(BinaryFormat::Component);
    let result = parse_wasm_with_config(binary, config)?;
    Ok(result.into_enhanced_module())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary_constants;
    
    #[test]
    fn test_unified_parser_creation() {
        let parser = UnifiedParser::new().unwrap();
        assert!(!parser.has_component_support()); // Default is core only
    }
    
    #[test]
    fn test_unified_parser_with_components() {
        let config = UnifiedParserConfig::with_components();
        let parser = UnifiedParser::with_config(config).unwrap();
        assert!(parser.has_component_support());
    }
    
    #[test]
    fn test_config_creation() {
        let core_config = UnifiedParserConfig::core_only();
        assert!(matches!(core_config.parser_mode, ParserMode::CoreOnly));
        
        let component_config = UnifiedParserConfig::with_components();
        assert!(matches!(component_config.parser_mode, ParserMode::ComponentAware { .. }));
        
        let forced_config = UnifiedParserConfig::default()
            .force_format(BinaryFormat::CoreModule)
            .no_validation();
        assert_eq!(forced_config.force_format, Some(BinaryFormat::CoreModule));
        assert!(!forced_config.validate_during_parse);
    }
    
    #[test]
    fn test_parse_empty_core_module() {
        let mut parser = UnifiedParser::new().unwrap();
        
        // Minimal valid Core WebAssembly module
        let core_binary = [
            0x00, 0x61, 0x73, 0x6D, // Magic
            0x01, 0x00, 0x00, 0x00, // Version 1
            // No sections (empty module)
        ];
        
        let result = parser.parse(&core_binary).unwrap();
        assert!(result.is_core_module());
        
        let enhanced_module = result.into_enhanced_module();
        assert!(!enhanced_module.has_component_model());
    }
    
    #[test]
    fn test_parse_component_without_support() {
        let mut parser = UnifiedParser::new().unwrap(); // Core only
        
        // Component Model binary
        let component_binary = [
            0x00, 0x61, 0x73, 0x6D, // Magic
            0x01, 0x00, 0x00, 0x00, // Version 1
            0x01, 0x00, 0x00, 0x00, // Layer 1 (Component)
        ];
        
        let result = parser.parse(&component_binary);
        assert!(result.is_err()); // Should fail because Component Model not enabled
    }
    
    #[test]
    fn test_parse_component_with_support() {
        let config = UnifiedParserConfig::with_components();
        let mut parser = UnifiedParser::with_config(config).unwrap();
        
        // Component Model binary with empty type section
        let component_binary = [
            0x00, 0x61, 0x73, 0x6D, // Magic
            0x01, 0x00, 0x00, 0x00, // Version 1
            0x01, 0x00, 0x00, 0x00, // Layer 1 (Component)
            0x07,                   // Type section ID
            0x01,                   // Section size
            0x00,                   // Count = 0 (empty)
        ];
        
        let result = parser.parse(&component_binary).unwrap();
        assert!(result.is_component());
        
        let enhanced_module = result.into_enhanced_module();
        assert!(enhanced_module.has_component_model());
    }
    
    #[test]
    fn test_forced_format_parsing() {
        let config = UnifiedParserConfig::core_only()
            .force_format(BinaryFormat::CoreModule);
        let mut parser = UnifiedParser::with_config(config).unwrap();
        
        let core_binary = [
            0x00, 0x61, 0x73, 0x6D, // Magic
            0x01, 0x00, 0x00, 0x00, // Version 1
        ];
        
        let result = parser.parse(&core_binary).unwrap();
        assert!(result.is_core_module());
    }
    
    #[test]
    fn test_convenience_functions() {
        let core_binary = [
            0x00, 0x61, 0x73, 0x6D, // Magic
            0x01, 0x00, 0x00, 0x00, // Version 1
        ];
        
        // Test main convenience function
        let result = parse_wasm_binary(&core_binary).unwrap();
        assert!(result.is_core_module());
        
        // Test core-specific function
        let core_module = parse_core_module(&core_binary).unwrap();
        assert_eq!(core_module.functions.len(), 0); // Empty module
    }
    
    #[test]
    fn test_invalid_binary() {
        let mut parser = UnifiedParser::new().unwrap();
        
        let invalid_binary = [0xFF, 0xFF, 0xFF, 0xFF]; // Invalid magic
        
        let result = parser.parse(&invalid_binary);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_binary_info_tracking() {
        let mut parser = UnifiedParser::new().unwrap();
        
        let core_binary = [
            0x00, 0x61, 0x73, 0x6D, // Magic
            0x01, 0x00, 0x00, 0x00, // Version 1
        ];
        
        assert!(parser.last_binary_info().is_none());
        
        let _result = parser.parse(&core_binary).unwrap();
        
        let info = parser.last_binary_info().unwrap();
        assert_eq!(info.format, BinaryFormat::CoreModule);
        assert_eq!(info.version, 1);
        assert_eq!(info.size, 8);
    }
}