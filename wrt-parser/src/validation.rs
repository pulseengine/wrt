//! WebAssembly validation utilities
//!
//! This module provides validation functionality for WebAssembly modules
//! and components during parsing.

use wrt_error::{Error, ErrorCategory, Result, codes};

/// Validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Enable type checking
    pub enable_types: bool,
    /// Enable memory validation
    pub enable_memory: bool,
    /// Enable control flow validation
    pub enable_control_flow: bool,
    /// Maximum stack depth for validation
    pub max_stack_depth: usize,
    /// Maximum number of locals per function
    pub max_locals: usize,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        ValidationConfig {
            enable_types: true,
            enable_memory: true,
            enable_control_flow: true,
            max_stack_depth: 1024,
            max_locals: 1024,
        }
    }
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: &'static str,
    pub offset: Option<usize>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(message: &'static str) -> Self {
        ValidationError {
            message,
            offset: None,
        }
    }
    
    /// Create a validation error with offset
    pub fn with_offset(message: &'static str, offset: usize) -> Self {
        ValidationError {
            message,
            offset: Some(offset),
        }
    }
}

impl From<ValidationError> for Error {
    fn from(err: ValidationError) -> Self {
        Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            err.message
        )
    }
}

/// Module validator
#[derive(Debug)]
pub struct ModuleValidator {
    config: ValidationConfig,
}

impl ModuleValidator {
    /// Create a new module validator
    pub fn new(config: ValidationConfig) -> Self {
        ModuleValidator { config }
    }
    
    /// Validate a WebAssembly module
    pub fn validate(&self, module: &crate::simple_module::SimpleModule) -> Result<()> {
        // Cross-reference validation
        self.validate_function_types(module)?;
        self.validate_imports(module)?;
        self.validate_exports(module)?;
        self.validate_globals(module)?;
        self.validate_elements(module)?;
        self.validate_data(module)?;
        self.validate_start_function(module)?;
        
        // Count validation
        self.validate_function_count(module)?;
        self.validate_memory_limits(module)?;
        self.validate_table_limits(module)?;
        
        Ok(())
    }
    
    /// Validate function types
    fn validate_function_types(&self, module: &crate::simple_module::SimpleModule) -> Result<()> {
        if !self.config.enable_types {
            return Ok(());
        }
        
        // Check that all function type indices are valid
        for function_type_idx in module.functions.iter() {
            if *function_type_idx as usize >= module.types.len() {
                return Err(ValidationError::new("Invalid function type index").into());
            }
        }
        
        Ok(())
    }
    
    /// Validate imports section
    fn validate_imports(&self, module: &crate::simple_module::SimpleModule) -> Result<()> {
        for import in module.imports.iter() {
            // Validate import names are valid UTF-8
            validate_name(&import.module.iter().copied().collect::<Vec<_>>())?;
            validate_name(&import.name.iter().copied().collect::<Vec<_>>())?;
            
            // Validate import descriptors
            match &import.desc {
                crate::simple_module::ImportDesc::Func(type_idx) => {
                    if *type_idx as usize >= module.types.len() {
                        return Err(ValidationError::new("Import function type index out of bounds").into());
                    }
                }
                crate::simple_module::ImportDesc::Global(global_type) => {
                    // Global types are always valid (only contain value types)
                    let _ = global_type;
                }
                crate::simple_module::ImportDesc::Memory(_) => {
                    // Memory limits are validated separately
                }
                crate::simple_module::ImportDesc::Table(_) => {
                    // Table limits are validated separately
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate exports section
    fn validate_exports(&self, module: &crate::simple_module::SimpleModule) -> Result<()> {
        let total_functions = module.imports.iter()
            .filter(|i| matches!(i.desc, crate::simple_module::ImportDesc::Func(_)))
            .count() + module.functions.len();
            
        let total_globals = module.imports.iter()
            .filter(|i| matches!(i.desc, crate::simple_module::ImportDesc::Global(_)))
            .count() + module.globals.len();
            
        let total_memories = module.imports.iter()
            .filter(|i| matches!(i.desc, crate::simple_module::ImportDesc::Memory(_)))
            .count() + module.memories.len();
            
        let total_tables = module.imports.iter()
            .filter(|i| matches!(i.desc, crate::simple_module::ImportDesc::Table(_)))
            .count() + module.tables.len();
        
        for export in module.exports.iter() {
            // Validate export name is valid UTF-8
            validate_name(&export.name.iter().copied().collect::<Vec<_>>())?;
            
            // Validate export indices
            match export.kind {
                crate::simple_module::ExportKind::Func => {
                    if export.index as usize >= total_functions {
                        return Err(ValidationError::new("Export function index out of bounds").into());
                    }
                }
                crate::simple_module::ExportKind::Global => {
                    if export.index as usize >= total_globals {
                        return Err(ValidationError::new("Export global index out of bounds").into());
                    }
                }
                crate::simple_module::ExportKind::Memory => {
                    if export.index as usize >= total_memories {
                        return Err(ValidationError::new("Export memory index out of bounds").into());
                    }
                }
                crate::simple_module::ExportKind::Table => {
                    if export.index as usize >= total_tables {
                        return Err(ValidationError::new("Export table index out of bounds").into());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate globals section
    fn validate_globals(&self, module: &crate::simple_module::SimpleModule) -> Result<()> {
        // For now, just validate that globals don't exceed limits
        if module.globals.len() > 100_000 {
            return Err(ValidationError::new("Too many globals in module").into());
        }
        
        // TODO: Validate global initialization expressions against imported globals
        Ok(())
    }
    
    /// Validate elements section
    fn validate_elements(&self, module: &crate::simple_module::SimpleModule) -> Result<()> {
        let total_tables = module.imports.iter()
            .filter(|i| matches!(i.desc, crate::simple_module::ImportDesc::Table(_)))
            .count() + module.tables.len();
            
        let total_functions = module.imports.iter()
            .filter(|i| matches!(i.desc, crate::simple_module::ImportDesc::Func(_)))
            .count() + module.functions.len();
        
        for element in module.elements.iter() {
            // Validate table index
            if element.table_index as usize >= total_tables {
                return Err(ValidationError::new("Element table index out of bounds").into());
            }
            
            // Validate function indices in element initializer
            for func_idx in element.init.iter() {
                if *func_idx as usize >= total_functions {
                    return Err(ValidationError::new("Element function index out of bounds").into());
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate data section
    fn validate_data(&self, module: &crate::simple_module::SimpleModule) -> Result<()> {
        let total_memories = module.imports.iter()
            .filter(|i| matches!(i.desc, crate::simple_module::ImportDesc::Memory(_)))
            .count() + module.memories.len();
        
        for data_segment in module.data.iter() {
            // Validate memory index
            if data_segment.memory_index as usize >= total_memories {
                return Err(ValidationError::new("Data memory index out of bounds").into());
            }
        }
        
        Ok(())
    }
    
    /// Validate start function
    fn validate_start_function(&self, module: &crate::simple_module::SimpleModule) -> Result<()> {
        if let Some(start_idx) = module.start {
            let total_functions = module.imports.iter()
                .filter(|i| matches!(i.desc, crate::simple_module::ImportDesc::Func(_)))
                .count() + module.functions.len();
                
            if start_idx as usize >= total_functions {
                return Err(ValidationError::new("Start function index out of bounds").into());
            }
            
            // TODO: Validate that start function has type [] -> []
        }
        
        Ok(())
    }
    
    /// Validate memory limits
    fn validate_memory_limits(&self, module: &crate::simple_module::SimpleModule) -> Result<()> {
        if !self.config.enable_memory {
            return Ok(());
        }
        
        let total_memories = module.imports.iter()
            .filter(|i| matches!(i.desc, crate::simple_module::ImportDesc::Memory(_)))
            .count() + module.memories.len();
            
        // WebAssembly 1.0 allows at most one memory
        if total_memories > 1 {
            return Err(ValidationError::new("Multiple memories not allowed").into());
        }
        
        // Validate memory limits
        for memory_type in module.memories.iter() {
            let limits = &memory_type.limits;
            if let Some(max) = limits.max {
                if max < limits.min {
                    return Err(ValidationError::new("Memory maximum less than minimum").into());
                }
                if max > 65536 {
                    return Err(ValidationError::new("Memory maximum exceeds limit").into());
                }
            }
            if limits.min > 65536 {
                return Err(ValidationError::new("Memory minimum exceeds limit").into());
            }
        }
        
        Ok(())
    }
    
    /// Validate table limits
    fn validate_table_limits(&self, module: &crate::simple_module::SimpleModule) -> Result<()> {
        let total_tables = module.imports.iter()
            .filter(|i| matches!(i.desc, crate::simple_module::ImportDesc::Table(_)))
            .count() + module.tables.len();
            
        // WebAssembly 1.0 allows at most one table
        if total_tables > 1 {
            return Err(ValidationError::new("Multiple tables not allowed").into());
        }
        
        // Validate table limits
        for table_type in module.tables.iter() {
            let limits = &table_type.limits;
            if let Some(max) = limits.max {
                if max < limits.min {
                    return Err(ValidationError::new("Table maximum less than minimum").into());
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate function count
    fn validate_function_count(&self, module: &crate::simple_module::SimpleModule) -> Result<()> {
        // Check that we don't exceed reasonable limits
        if module.functions.len() > 1_000_000 {
            return Err(ValidationError::new("Too many functions in module").into());
        }
        
        Ok(())
    }
}

/// Component validator
pub struct ComponentValidator {
    config: ValidationConfig,
}

impl ComponentValidator {
    /// Create a new component validator
    pub fn new(config: ValidationConfig) -> Self {
        ComponentValidator { config }
    }
    
    /// Validate a WebAssembly component
    pub fn validate(&self, _component: &()) -> Result<()> {
        // Basic component validation - placeholder for now
        Ok(())
    }
}

/// Validate a WebAssembly name (UTF-8 encoded)
pub fn validate_name(data: &[u8]) -> Result<()> {
    match core::str::from_utf8(data) {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError::new("Invalid UTF-8 in name").into()),
    }
}