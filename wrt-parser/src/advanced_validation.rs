//! Advanced WebAssembly validation with platform limits and streaming support
//!
//! This module provides comprehensive validation capabilities including
//! platform-specific limits, streaming validation, and ASIL-D compliance.

use wrt_error::{Error, ErrorCategory, Result, codes};
use crate::types::{ValueType, FuncType, GlobalType, MemoryType, TableType, Limits};
use crate::simple_module::SimpleModule;
use crate::instruction_parser::{Instruction, InstructionParser, ControlFrame};
use crate::bounded_types::SimpleBoundedVec;

/// Maximum number of functions in a module for ASIL-D compliance
pub const MAX_FUNCTIONS_ASIL_D: usize = 1024;

/// Maximum number of globals in a module for ASIL-D compliance
pub const MAX_GLOBALS_ASIL_D: usize = 256;

/// Maximum memory size (in pages) for ASIL-D compliance
pub const MAX_MEMORY_PAGES_ASIL_D: u32 = 1024; // 64MB

/// Maximum table size for ASIL-D compliance
pub const MAX_TABLE_SIZE_ASIL_D: u32 = 4096;

/// Maximum nesting depth for control structures
pub const MAX_CONTROL_NESTING_DEPTH: usize = 64;

/// Platform-specific limits for WebAssembly validation
#[derive(Debug, Clone)]
pub struct PlatformLimits {
    /// Maximum number of functions
    pub max_functions: usize,
    
    /// Maximum number of function types
    pub max_function_types: usize,
    
    /// Maximum number of globals
    pub max_globals: usize,
    
    /// Maximum number of tables
    pub max_tables: usize,
    
    /// Maximum number of memories
    pub max_memories: usize,
    
    /// Maximum memory size in pages (64KB each)
    pub max_memory_pages: u32,
    
    /// Maximum table size
    pub max_table_size: u32,
    
    /// Maximum nesting depth for control flow
    pub max_control_depth: usize,
    
    /// Maximum number of locals per function
    pub max_locals_per_function: usize,
    
    /// Maximum function body size in bytes
    pub max_function_body_size: usize,
    
    /// Maximum number of imports
    pub max_imports: usize,
    
    /// Maximum number of exports
    pub max_exports: usize,
    
    /// Enable strict ASIL-D compliance
    pub asil_d_compliance: bool,
}

impl Default for PlatformLimits {
    fn default() -> Self {
        Self {
            max_functions: 10000,
            max_function_types: 1000,
            max_globals: 1000,
            max_tables: 100,
            max_memories: 100,
            max_memory_pages: 65536, // 4GB
            max_table_size: 10000000,
            max_control_depth: 1024,
            max_locals_per_function: 50000,
            max_function_body_size: 7654321,
            max_imports: 100000,
            max_exports: 100000,
            asil_d_compliance: false,
        }
    }
}

impl PlatformLimits {
    /// Create ASIL-D compliant limits
    pub fn asil_d() -> Self {
        Self {
            max_functions: MAX_FUNCTIONS_ASIL_D,
            max_function_types: 128,
            max_globals: MAX_GLOBALS_ASIL_D,
            max_tables: 4,
            max_memories: 1,
            max_memory_pages: MAX_MEMORY_PAGES_ASIL_D,
            max_table_size: MAX_TABLE_SIZE_ASIL_D,
            max_control_depth: MAX_CONTROL_NESTING_DEPTH,
            max_locals_per_function: 256,
            max_function_body_size: 8192,
            max_imports: 64,
            max_exports: 64,
            asil_d_compliance: true,
        }
    }
    
    /// Create relaxed limits for development
    pub fn development() -> Self {
        Self {
            max_functions: 100000,
            max_function_types: 10000,
            max_globals: 10000,
            max_tables: 1000,
            max_memories: 1000,
            max_memory_pages: 1048576, // 64GB
            max_table_size: 100000000,
            max_control_depth: 2048,
            max_locals_per_function: 100000,
            max_function_body_size: 16777216, // 16MB
            max_imports: 1000000,
            max_exports: 1000000,
            asil_d_compliance: false,
        }
    }
}

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Validation issue with detailed information
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub code: u32,
    pub message: String,
    pub location: ValidationLocation,
}

/// Location information for validation issues
#[derive(Debug, Clone)]
pub struct ValidationLocation {
    pub section: Option<String>,
    pub function_index: Option<u32>,
    pub instruction_offset: Option<usize>,
    pub byte_offset: Option<usize>,
}

impl Default for ValidationLocation {
    fn default() -> Self {
        Self {
            section: None,
            function_index: None,
            instruction_offset: None,
            byte_offset: None,
        }
    }
}

/// Advanced WebAssembly validator with streaming support
#[derive(Debug)]
pub struct AdvancedValidator {
    /// Platform-specific limits
    limits: PlatformLimits,
    
    /// Instruction parser for bytecode validation
    instruction_parser: InstructionParser,
    
    /// Collected validation issues
    issues: SimpleBoundedVec<ValidationIssue, 1024>,
    
    /// Current validation context
    current_location: ValidationLocation,
    
    /// Enable streaming validation
    streaming_mode: bool,
}

impl AdvancedValidator {
    /// Create a new advanced validator
    pub fn new(limits: PlatformLimits) -> Self {
        Self {
            limits,
            instruction_parser: InstructionParser::new(),
            issues: SimpleBoundedVec::new(),
            current_location: ValidationLocation::default(),
            streaming_mode: false,
        }
    }
    
    /// Create validator with ASIL-D compliance
    pub fn asil_d() -> Self {
        Self::new(PlatformLimits::asil_d())
    }
    
    /// Enable streaming validation mode
    pub fn enable_streaming(&mut self) {
        self.streaming_mode = true;
    }
    
    /// Validate a complete module
    pub fn validate_module(&mut self, module: &SimpleModule) -> Result<()> {
        self.issues.clear();
        
        // Validate module structure
        self.validate_module_structure(module)?;
        
        // Validate types
        self.validate_types(module)?;
        
        // Validate functions
        self.validate_functions(module)?;
        
        // Validate tables
        self.validate_tables(module)?;
        
        // Validate memories
        self.validate_memories(module)?;
        
        // Validate globals
        self.validate_globals(module)?;
        
        // Validate imports and exports
        self.validate_imports_exports(module)?;
        
        // Validate cross-references
        self.validate_cross_references(module)?;
        
        // Check if any errors occurred
        if self.has_errors() {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Module validation failed"
            ));
        }
        
        Ok(())
    }
    
    /// Validate module structure and limits
    fn validate_module_structure(&mut self, module: &SimpleModule) -> Result<()> {
        self.current_location.section = Some("module".to_string());
        
        // Check function count limits
        if module.functions.len() > self.limits.max_functions {
            self.add_error(
                codes::VALIDATION_ERROR as u32,
                format!("Too many functions: {} > {}", 
                       module.functions.len(), self.limits.max_functions)
            )?;
        }
        
        // Check type count limits
        if module.types.len() > self.limits.max_function_types {
            self.add_error(
                codes::VALIDATION_ERROR as u32,
                format!("Too many function types: {} > {}", 
                       module.types.len(), self.limits.max_function_types)
            )?;
        }
        
        // Check global count limits
        if module.globals.len() > self.limits.max_globals {
            self.add_error(
                codes::VALIDATION_ERROR as u32,
                format!("Too many globals: {} > {}", 
                       module.globals.len(), self.limits.max_globals)
            )?;
        }
        
        // Check table count limits
        if module.tables.len() > self.limits.max_tables {
            self.add_error(
                codes::VALIDATION_ERROR as u32,
                format!("Too many tables: {} > {}", 
                       module.tables.len(), self.limits.max_tables)
            )?;
        }
        
        // Check memory count limits
        if module.memories.len() > self.limits.max_memories {
            self.add_error(
                codes::VALIDATION_ERROR as u32,
                format!("Too many memories: {} > {}", 
                       module.memories.len(), self.limits.max_memories)
            )?;
        }
        
        // ASIL-D specific checks
        if self.limits.asil_d_compliance {
            // Must have at most one memory
            if module.memories.len() > 1 {
                self.add_error(
                    codes::VALIDATION_ERROR as u32,
                    "ASIL-D compliance: multiple memories not allowed".to_string()
                )?;
            }
            
            // Check import/export limits
            if module.imports.len() > self.limits.max_imports {
                self.add_error(
                    codes::VALIDATION_ERROR as u32,
                    format!("Too many imports for ASIL-D: {} > {}", 
                           module.imports.len(), self.limits.max_imports)
                )?;
            }
            
            if module.exports.len() > self.limits.max_exports {
                self.add_error(
                    codes::VALIDATION_ERROR as u32,
                    format!("Too many exports for ASIL-D: {} > {}", 
                           module.exports.len(), self.limits.max_exports)
                )?;
            }
        }
        
        Ok(())
    }
    
    /// Validate function types
    fn validate_types(&mut self, module: &SimpleModule) -> Result<()> {
        self.current_location.section = Some("type".to_string());
        
        for (i, func_type) in module.types.iter().enumerate() {
            self.current_location.function_index = Some(i as u32);
            
            // Validate parameter and result types
            for param_type in func_type.params.iter() {
                self.validate_value_type(*param_type)?;
            }
            
            for result_type in func_type.results.iter() {
                self.validate_value_type(*result_type)?;
            }
            
            // Check parameter count for ASIL-D
            if self.limits.asil_d_compliance && func_type.params.len() > 8 {
                self.add_warning(
                    codes::VALIDATION_ERROR as u32,
                    format!("Function type {} has many parameters: {}", i, func_type.params.len())
                )?;
            }
            
            // Check result count (WebAssembly 1.0 allows at most 1 result)
            if func_type.results.len() > 1 {
                self.add_error(
                    codes::VALIDATION_ERROR as u32,
                    format!("Function type {} has multiple results: {}", i, func_type.results.len())
                )?;
            }
        }
        
        Ok(())
    }
    
    /// Validate function bodies and instructions
    fn validate_functions(&mut self, module: &SimpleModule) -> Result<()> {
        self.current_location.section = Some("code".to_string());
        
        for (i, function_body) in module.code.iter().enumerate() {
            self.current_location.function_index = Some(i as u32);
            
            // Check function body size
            if function_body.code.len() > self.limits.max_function_body_size {
                self.add_error(
                    codes::VALIDATION_ERROR as u32,
                    format!("Function {} body too large: {} bytes > {}", 
                           i, function_body.code.len(), self.limits.max_function_body_size)
                )?;
            }
            
            // Check locals count
            let total_locals: usize = function_body.locals.iter().map(|l| l.count as usize).sum();
            if total_locals > self.limits.max_locals_per_function {
                self.add_error(
                    codes::VALIDATION_ERROR as u32,
                    format!("Function {} has too many locals: {} > {}", 
                           i, total_locals, self.limits.max_locals_per_function)
                )?;
            }
            
            // Validate instruction sequence
            self.validate_function_instructions(i as u32, function_body)?;
        }
        
        Ok(())
    }
    
    /// Validate instruction sequence in a function
    fn validate_function_instructions(&mut self, func_idx: u32, function_body: &crate::simple_module::FunctionBody) -> Result<()> {
        // Build local types vector
        let mut local_types = Vec::new();
        
        // Add function parameters if we have the function type
        // (This would require access to the function type from the module)
        
        // Add local variables
        for local_entry in &function_body.locals {
            for _ in 0..local_entry.count {
                local_types.push(local_entry.value_type);
            }
        }
        
        // Parse and validate instructions
        self.instruction_parser.init_function(&local_types)?;
        
        let instructions = self.instruction_parser.parse_function_body(function_body.code.as_slice(), &local_types)?;
        
        // Validate control flow
        self.validate_control_flow(instructions.as_slice())?;
        
        Ok(())
    }
    
    /// Validate control flow structure
    fn validate_control_flow(&mut self, instructions: &[Instruction]) -> Result<()> {
        let mut control_depth = 0;
        let mut control_stack: SimpleBoundedVec<&Instruction, MAX_CONTROL_NESTING_DEPTH> = SimpleBoundedVec::new();
        
        for (i, instruction) in instructions.iter().enumerate() {
            self.current_location.instruction_offset = Some(i);
            
            match instruction {
                Instruction::Block { .. } | Instruction::Loop { .. } | Instruction::If { .. } => {
                    control_depth += 1;
                    if control_depth > self.limits.max_control_depth {
                        self.add_error(
                            codes::VALIDATION_ERROR as u32,
                            format!("Control flow nesting too deep: {} > {}", 
                                   control_depth, self.limits.max_control_depth)
                        )?;
                    }
                    control_stack.push(instruction)?;
                }
                
                Instruction::End => {
                    if control_stack.is_empty() {
                        self.add_error(
                            codes::VALIDATION_ERROR as u32,
                            "Unexpected 'end' instruction".to_string()
                        )?;
                    } else {
                        control_stack.pop();
                        control_depth = control_depth.saturating_sub(1);
                    }
                }
                
                Instruction::Else => {
                    if control_stack.is_empty() {
                        self.add_error(
                            codes::VALIDATION_ERROR as u32,
                            "Unexpected 'else' instruction".to_string()
                        )?;
                    } else if !matches!(control_stack.last(), Some(Instruction::If { .. })) {
                        self.add_error(
                            codes::VALIDATION_ERROR as u32,
                            "'else' instruction not in 'if' block".to_string()
                        )?;
                    }
                }
                
                Instruction::Br { label_idx } | Instruction::BrIf { label_idx } => {
                    if *label_idx as usize >= control_stack.len() {
                        self.add_error(
                            codes::VALIDATION_ERROR as u32,
                            format!("Invalid branch target: {} >= {}", label_idx, control_stack.len())
                        )?;
                    }
                }
                
                Instruction::BrTable { table } => {
                    let max_label = control_stack.len() as u32;
                    for &target in table.targets.iter() {
                        if target >= max_label {
                            self.add_error(
                                codes::VALIDATION_ERROR as u32,
                                format!("Invalid branch table target: {} >= {}", target, max_label)
                            )?;
                        }
                    }
                    if table.default_target >= max_label {
                        self.add_error(
                            codes::VALIDATION_ERROR as u32,
                            format!("Invalid branch table default: {} >= {}", 
                                   table.default_target, max_label)
                        )?;
                    }
                }
                
                _ => {} // Other instructions don't affect control flow structure
            }
        }
        
        // Check for unmatched control instructions
        if !control_stack.is_empty() {
            self.add_error(
                codes::VALIDATION_ERROR as u32,
                format!("Unmatched control instructions: {} unclosed blocks", control_stack.len())
            )?;
        }
        
        Ok(())
    }
    
    /// Validate tables
    fn validate_tables(&mut self, module: &SimpleModule) -> Result<()> {
        self.current_location.section = Some("table".to_string());
        
        for (i, table) in module.tables.iter().enumerate() {
            // Check table size limits
            if table.limits.min > self.limits.max_table_size {
                self.add_error(
                    codes::VALIDATION_ERROR as u32,
                    format!("Table {} minimum size too large: {} > {}", 
                           i, table.limits.min, self.limits.max_table_size)
                )?;
            }
            
            if let Some(max) = table.limits.max {
                if max > self.limits.max_table_size {
                    self.add_error(
                        codes::VALIDATION_ERROR as u32,
                        format!("Table {} maximum size too large: {} > {}", 
                               i, max, self.limits.max_table_size)
                    )?;
                }
                
                if max < table.limits.min {
                    self.add_error(
                        codes::VALIDATION_ERROR as u32,
                        format!("Table {} maximum size less than minimum: {} < {}", 
                               i, max, table.limits.min)
                    )?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate memories
    fn validate_memories(&mut self, module: &SimpleModule) -> Result<()> {
        self.current_location.section = Some("memory".to_string());
        
        for (i, memory) in module.memories.iter().enumerate() {
            // Check memory size limits
            if memory.limits.min > self.limits.max_memory_pages {
                self.add_error(
                    codes::VALIDATION_ERROR as u32,
                    format!("Memory {} minimum size too large: {} pages > {}", 
                           i, memory.limits.min, self.limits.max_memory_pages)
                )?;
            }
            
            if let Some(max) = memory.limits.max {
                if max > self.limits.max_memory_pages {
                    self.add_error(
                        codes::VALIDATION_ERROR as u32,
                        format!("Memory {} maximum size too large: {} pages > {}", 
                               i, max, self.limits.max_memory_pages)
                    )?;
                }
                
                if max < memory.limits.min {
                    self.add_error(
                        codes::VALIDATION_ERROR as u32,
                        format!("Memory {} maximum size less than minimum: {} < {}", 
                               i, max, memory.limits.min)
                    )?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate globals
    fn validate_globals(&mut self, module: &SimpleModule) -> Result<()> {
        self.current_location.section = Some("global".to_string());
        
        for (i, global) in module.globals.iter().enumerate() {
            // Validate global type
            self.validate_value_type(global.value_type)?;
            
            // ASIL-D compliance: prefer immutable globals
            if self.limits.asil_d_compliance && global.mutable {
                self.add_warning(
                    codes::VALIDATION_ERROR as u32,
                    format!("Global {} is mutable (ASIL-D prefers immutable)", i)
                )?;
            }
        }
        
        Ok(())
    }
    
    /// Validate imports and exports
    fn validate_imports_exports(&mut self, module: &SimpleModule) -> Result<()> {
        // Validate imports
        self.current_location.section = Some("import".to_string());
        for (i, import) in module.imports.iter().enumerate() {
            if import.module.is_empty() {
                self.add_error(
                    codes::VALIDATION_ERROR as u32,
                    format!("Import {} has empty module name", i)
                )?;
            }
            
            if import.name.is_empty() {
                self.add_error(
                    codes::VALIDATION_ERROR as u32,
                    format!("Import {} has empty import name", i)
                )?;
            }
        }
        
        // Validate exports
        self.current_location.section = Some("export".to_string());
        for (i, export) in module.exports.iter().enumerate() {
            if export.name.is_empty() {
                self.add_error(
                    codes::VALIDATION_ERROR as u32,
                    format!("Export {} has empty name", i)
                )?;
            }
        }
        
        Ok(())
    }
    
    /// Validate cross-references between sections
    fn validate_cross_references(&mut self, module: &SimpleModule) -> Result<()> {
        self.current_location.section = Some("cross-ref".to_string());
        
        // Validate function references
        for (i, type_idx) in module.functions.iter().enumerate() {
            if *type_idx as usize >= module.types.len() {
                self.add_error(
                    codes::VALIDATION_ERROR as u32,
                    format!("Function {} references invalid type index: {} >= {}", 
                           i, type_idx, module.types.len())
                )?;
            }
        }
        
        // Validate start function
        if let Some(start_idx) = module.start {
            let total_functions = module.imports.iter()
                .filter(|imp| matches!(imp.desc, crate::simple_module::ImportDesc::Func(_)))
                .count() + module.functions.len();
            
            if start_idx as usize >= total_functions {
                self.add_error(
                    codes::VALIDATION_ERROR as u32,
                    format!("Start function index invalid: {} >= {}", start_idx, total_functions)
                )?;
            }
        }
        
        Ok(())
    }
    
    /// Validate value type
    fn validate_value_type(&mut self, value_type: ValueType) -> Result<()> {
        // All standard value types are valid
        // Could add additional checks for reference types in the future
        match value_type {
            ValueType::I32 | ValueType::I64 | ValueType::F32 | ValueType::F64 => Ok(()),
            ValueType::V128 => {
                if self.limits.asil_d_compliance {
                    self.add_warning(
                        codes::VALIDATION_ERROR as u32,
                        "V128 SIMD types may not be suitable for ASIL-D compliance".to_string()
                    )?;
                }
                Ok(())
            }
            ValueType::FuncRef | ValueType::ExternRef => {
                if self.limits.asil_d_compliance {
                    self.add_warning(
                        codes::VALIDATION_ERROR as u32,
                        "Reference types may not be suitable for ASIL-D compliance".to_string()
                    )?;
                }
                Ok(())
            }
        }
    }
    
    /// Add validation error
    fn add_error(&mut self, code: u32, message: String) -> Result<()> {
        let issue = ValidationIssue {
            severity: ValidationSeverity::Error,
            code,
            message,
            location: self.current_location.clone(),
        };
        self.issues.push(issue)
    }
    
    /// Add validation warning
    fn add_warning(&mut self, code: u32, message: String) -> Result<()> {
        let issue = ValidationIssue {
            severity: ValidationSeverity::Warning,
            code,
            message,
            location: self.current_location.clone(),
        };
        self.issues.push(issue)
    }
    
    /// Check if any errors occurred
    fn has_errors(&self) -> bool {
        self.issues.iter().any(|issue| issue.severity == ValidationSeverity::Error)
    }
    
    /// Get all validation issues
    pub fn get_issues(&self) -> &[ValidationIssue] {
        self.issues.as_slice()
    }
    
    /// Get only error issues
    pub fn get_errors(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(|issue| issue.severity == ValidationSeverity::Error)
    }
    
    /// Get only warning issues
    pub fn get_warnings(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(|issue| issue.severity == ValidationSeverity::Warning)
    }
    
    /// Clear all issues
    pub fn clear_issues(&mut self) {
        self.issues.clear();
    }
}

impl Default for AdvancedValidator {
    fn default() -> Self {
        Self::new(PlatformLimits::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_platform_limits_default() {
        let limits = PlatformLimits::default();
        assert!(!limits.asil_d_compliance);
        assert!(limits.max_functions > 1000);
    }
    
    #[test]
    fn test_platform_limits_asil_d() {
        let limits = PlatformLimits::asil_d();
        assert!(limits.asil_d_compliance);
        assert_eq!(limits.max_functions, MAX_FUNCTIONS_ASIL_D);
        assert_eq!(limits.max_memory_pages, MAX_MEMORY_PAGES_ASIL_D);
    }
    
    #[test]
    fn test_validator_creation() {
        let validator = AdvancedValidator::new(PlatformLimits::default());
        assert_eq!(validator.issues.len(), 0);
        assert!(!validator.streaming_mode);
    }
    
    #[test]
    fn test_asil_d_validator() {
        let validator = AdvancedValidator::asil_d();
        assert!(validator.limits.asil_d_compliance);
    }
    
    #[test]
    fn test_validation_issue_creation() {
        let issue = ValidationIssue {
            severity: ValidationSeverity::Error,
            code: codes::VALIDATION_ERROR as u32,
            message: "Test error".to_string(),
            location: ValidationLocation::default(),
        };
        
        assert_eq!(issue.severity, ValidationSeverity::Error);
        assert_eq!(issue.message, "Test error");
    }
    
    #[test]
    fn test_validate_value_types() {
        let mut validator = AdvancedValidator::new(PlatformLimits::default());
        
        // Standard types should be valid
        assert!(validator.validate_value_type(ValueType::I32).is_ok());
        assert!(validator.validate_value_type(ValueType::I64).is_ok());
        assert!(validator.validate_value_type(ValueType::F32).is_ok());
        assert!(validator.validate_value_type(ValueType::F64).is_ok());
        
        // Reference types should be valid but may generate warnings in ASIL-D mode
        assert!(validator.validate_value_type(ValueType::FuncRef).is_ok());
        assert!(validator.validate_value_type(ValueType::ExternRef).is_ok());
    }
    
    #[test]
    fn test_empty_module_validation() {
        let mut validator = AdvancedValidator::new(PlatformLimits::default());
        let module = SimpleModule::new();
        
        assert!(validator.validate_module(&module).is_ok());
        assert_eq!(validator.get_errors().count(), 0);
    }
}