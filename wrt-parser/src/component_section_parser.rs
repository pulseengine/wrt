//! Component Model section parser
//!
//! This module extends the section parser to handle Component Model sections
//! with streaming parsing and ASIL-D compliant memory management.

use wrt_error::{Error, ErrorCategory, Result, codes};
use crate::{binary_constants, leb128};
use crate::component_types::{ComponentType, ComponentTypeDefinition, ComponentValueType, TypeRef};
use crate::component_registry::{ComponentRegistry, ComponentParserState};
use crate::enhanced_module::{
    EnhancedModule, ComponentModel, ComponentImport, ComponentExport, ComponentInstance,
    ComponentFunction, ComponentValue, ComponentStart, ImportName, InstantiationArg,
    CanonicalOptions, StringEncoding, ItemKind, ComponentAlias
};
use crate::bounded_types::{SimpleBoundedVec, SimpleBoundedString};

/// Component Model binary format section IDs
/// 
/// These IDs are used in Component Model binaries to identify different
/// section types beyond the core WebAssembly sections.
pub mod component_section_ids {
    /// Core module section (embedded core modules)
    pub const CORE_MODULE: u8 = 1;
    /// Core instance section (core module instantiations)
    pub const CORE_INSTANCE: u8 = 2;
    /// Core type section (core type definitions)
    pub const CORE_TYPE: u8 = 3;
    /// Component section (nested components)
    pub const COMPONENT: u8 = 4;
    /// Instance section (component instantiations)
    pub const INSTANCE: u8 = 5;
    /// Alias section (type and instance aliases)
    pub const ALIAS: u8 = 6;
    /// Type section (component type definitions)
    pub const TYPE: u8 = 7;
    /// Canon section (canonical function definitions)
    pub const CANON: u8 = 8;
    /// Start section (component start function)
    pub const START: u8 = 9;
    /// Import section (component imports)
    pub const IMPORT: u8 = 10;
    /// Export section (component exports)
    pub const EXPORT: u8 = 11;
    /// Value section (component values)
    pub const VALUE: u8 = 12;
}

/// Component Model section parser
/// 
/// Handles parsing of Component Model specific sections while maintaining
/// streaming architecture and ASIL-D memory compliance.
#[derive(Debug)]
pub struct ComponentSectionParser {
    /// Component registry for type management
    registry: ComponentRegistry,
    
    /// Current parsing state
    parser_state: ComponentParserState,
}

impl ComponentSectionParser {
    /// Create a new component section parser
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
            parser_state: ComponentParserState::Core,
        }
    }
    
    /// Create with custom component registry
    pub fn with_registry(registry: ComponentRegistry) -> Self {
        Self {
            registry,
            parser_state: ComponentParserState::Core,
        }
    }
    
    /// Parse a Component Model section
    pub fn parse_component_section(
        &mut self,
        section_id: u8,
        data: &[u8],
        module: &mut EnhancedModule,
    ) -> Result<()> {
        // Update parser state based on section type
        self.update_parser_state(section_id);
        
        match section_id {
            component_section_ids::TYPE => self.parse_type_section(data, module),
            component_section_ids::IMPORT => self.parse_import_section(data, module),
            component_section_ids::EXPORT => self.parse_export_section(data, module),
            component_section_ids::CANON => self.parse_canon_section(data, module),
            component_section_ids::START => self.parse_start_section(data, module),
            component_section_ids::INSTANCE => self.parse_instance_section(data, module),
            component_section_ids::VALUE => self.parse_value_section(data, module),
            component_section_ids::CORE_MODULE => self.parse_core_module_section(data, module),
            component_section_ids::CORE_INSTANCE => self.parse_core_instance_section(data, module),
            component_section_ids::CORE_TYPE => self.parse_core_type_section(data, module),
            component_section_ids::COMPONENT => self.parse_nested_component_section(data, module),
            component_section_ids::ALIAS => self.parse_alias_section(data, module),
            _ => {
                // Unknown Component Model section, skip it
                Ok(())
            }
        }
    }
    
    /// Parse Component Model type section
    fn parse_type_section(&mut self, data: &[u8], module: &mut EnhancedModule) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        // Ensure component model is available
        let component = module.component_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Component Model not enabled for type section"
            ))?;
        
        for _ in 0..count {
            let (component_type, bytes_consumed) = self.parse_component_type_definition(data, offset)?;
            offset += bytes_consumed;
            
            // Register type in registry
            let type_ref = self.registry.register_type(component_type.clone())?;
            
            // Add to component model
            component.add_type(component_type)?;
        }
        
        Ok(())
    }
    
    /// Parse Component Model import section
    fn parse_import_section(&mut self, data: &[u8], module: &mut EnhancedModule) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        let component = module.component_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Component Model not enabled for import section"
            ))?;
        
        for _ in 0..count {
            let (import, bytes_consumed) = self.parse_component_import(data, offset)?;
            offset += bytes_consumed;
            
            component.add_import(import)?;
        }
        
        Ok(())
    }
    
    /// Parse Component Model export section
    fn parse_export_section(&mut self, data: &[u8], module: &mut EnhancedModule) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        let component = module.component_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Component Model not enabled for export section"
            ))?;
        
        for _ in 0..count {
            let (export, bytes_consumed) = self.parse_component_export(data, offset)?;
            offset += bytes_consumed;
            
            component.add_export(export)?;
        }
        
        Ok(())
    }
    
    /// Parse canonical function section
    fn parse_canon_section(&mut self, data: &[u8], module: &mut EnhancedModule) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        let component = module.component_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Component Model not enabled for canon section"
            ))?;
        
        for _ in 0..count {
            let (function, bytes_consumed) = self.parse_canonical_function(data, offset)?;
            offset += bytes_consumed;
            
            component.add_function(function)?;
        }
        
        Ok(())
    }
    
    /// Parse component start section
    fn parse_start_section(&mut self, data: &[u8], module: &mut EnhancedModule) -> Result<()> {
        let mut offset = 0;
        let (func_index, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        // Parse arguments (simplified - assumes no arguments for now)
        let (arg_count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        let mut args = SimpleBoundedVec::new();
        for _ in 0..arg_count {
            let (arg, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            args.push(arg)?;
        }
        
        let start = ComponentStart {
            func_index,
            args,
        };
        
        let component = module.component_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Component Model not enabled for start section"
            ))?;
        
        component.set_start(start);
        Ok(())
    }
    
    /// Parse a component type definition
    fn parse_component_type_definition(&mut self, data: &[u8], offset: usize) -> Result<(ComponentType, usize)> {
        let mut current_offset = offset;
        
        // Read type kind byte
        if current_offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end while reading component type kind"
            ));
        }
        
        let type_kind = data[current_offset];
        current_offset += 1;
        
        let definition = match type_kind {
            0x40 => {
                // Function type
                let (params, bytes_consumed) = self.parse_function_params(data, current_offset)?;
                current_offset += bytes_consumed;
                
                let (results, bytes_consumed) = self.parse_function_results(data, current_offset)?;
                current_offset += bytes_consumed;
                
                ComponentTypeDefinition::Function { params, results }
            }
            0x41 => {
                // Component type
                let (imports, bytes_consumed) = self.parse_component_imports_type(data, current_offset)?;
                current_offset += bytes_consumed;
                
                let (exports, bytes_consumed) = self.parse_component_exports_type(data, current_offset)?;
                current_offset += bytes_consumed;
                
                ComponentTypeDefinition::Component { imports, exports }
            }
            0x42 => {
                // Instance type
                let (exports, bytes_consumed) = self.parse_component_exports_type(data, current_offset)?;
                current_offset += bytes_consumed;
                
                ComponentTypeDefinition::Instance { exports }
            }
            0x43 => {
                // Value type
                let (value_type, bytes_consumed) = self.parse_component_value_type(data, current_offset)?;
                current_offset += bytes_consumed;
                
                ComponentTypeDefinition::Value(value_type)
            }
            0x3F => {
                // Resource type (simplified)
                ComponentTypeDefinition::Resource {
                    representation: wrt_foundation::resource::ResourceRepresentation::Handle32,
                    nullable: false,
                }
            }
            _ => {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::INVALID_TYPE,
                    "Unknown component type kind"
                ));
            }
        };
        
        Ok((ComponentType { definition }, current_offset - offset))
    }
    
    /// Parse component value type
    fn parse_component_value_type(&mut self, data: &[u8], offset: usize) -> Result<(ComponentValueType, usize)> {
        let mut current_offset = offset;
        
        if current_offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end while reading value type"
            ));
        }
        
        let type_byte = data[current_offset];
        current_offset += 1;
        
        let value_type = match type_byte {
            0x7F => ComponentValueType::Bool,
            0x7E => ComponentValueType::S8,
            0x7D => ComponentValueType::U8,
            0x7C => ComponentValueType::S16,
            0x7B => ComponentValueType::U16,
            0x7A => ComponentValueType::S32,
            0x79 => ComponentValueType::U32,
            0x78 => ComponentValueType::S64,
            0x77 => ComponentValueType::U64,
            0x76 => ComponentValueType::F32,
            0x75 => ComponentValueType::F64,
            0x74 => ComponentValueType::Char,
            0x73 => ComponentValueType::String,
            _ => {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::INVALID_TYPE,
                    "Unknown component value type"
                ));
            }
        };
        
        Ok((value_type, current_offset - offset))
    }
    
    /// Parse function parameters (simplified)
    fn parse_function_params(&mut self, data: &[u8], offset: usize) -> Result<(SimpleBoundedVec<(SimpleBoundedString<64>, TypeRef), 32>, usize)> {
        let mut current_offset = offset;
        let (count, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        let mut params = SimpleBoundedVec::new();
        for _ in 0..count {
            // For now, use placeholder names and type references
            let name = SimpleBoundedString::from_str("param");
            let type_ref = 0; // Placeholder
            params.push((name, type_ref))?;
        }
        
        Ok((params, current_offset - offset))
    }
    
    /// Parse function results (simplified)
    fn parse_function_results(&mut self, data: &[u8], offset: usize) -> Result<(SimpleBoundedVec<TypeRef, 16>, usize)> {
        let mut current_offset = offset;
        let (count, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        let mut results = SimpleBoundedVec::new();
        for _ in 0..count {
            let type_ref = 0; // Placeholder
            results.push(type_ref)?;
        }
        
        Ok((results, current_offset - offset))
    }
    
    /// Parse component import
    fn parse_component_import(&mut self, data: &[u8], offset: usize) -> Result<(ComponentImport, usize)> {
        let mut current_offset = offset;
        
        // Parse import name (simplified)
        let (namespace_len, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        if current_offset + namespace_len as usize > data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Import namespace extends beyond section"
            ));
        }
        
        let namespace = SimpleBoundedString::from_str(
            core::str::from_utf8(&data[current_offset..current_offset + namespace_len as usize])
                .map_err(|_| Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid UTF-8 in import namespace"
                ))?
        );
        current_offset += namespace_len as usize;
        
        let (name_len, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        if current_offset + name_len as usize > data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Import name extends beyond section"
            ));
        }
        
        let name = SimpleBoundedString::from_str(
            core::str::from_utf8(&data[current_offset..current_offset + name_len as usize])
                .map_err(|_| Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid UTF-8 in import name"
                ))?
        );
        current_offset += name_len as usize;
        
        // Parse type reference (simplified)
        let (type_ref, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        let import = ComponentImport {
            name: ImportName {
                namespace,
                name,
            },
            ty: type_ref,
        };
        
        Ok((import, current_offset - offset))
    }
    
    /// Parse component export
    fn parse_component_export(&mut self, data: &[u8], offset: usize) -> Result<(ComponentExport, usize)> {
        let mut current_offset = offset;
        
        // Parse export name
        let (name_len, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        if current_offset + name_len as usize > data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Export name extends beyond section"
            ));
        }
        
        let name = SimpleBoundedString::from_str(
            core::str::from_utf8(&data[current_offset..current_offset + name_len as usize])
                .map_err(|_| Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid UTF-8 in export name"
                ))?
        );
        current_offset += name_len as usize;
        
        // Parse type reference and item index
        let (type_ref, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        let (item_index, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        let export = ComponentExport {
            name,
            ty: type_ref,
            item_index,
        };
        
        Ok((export, current_offset - offset))
    }
    
    /// Parse canonical function
    fn parse_canonical_function(&mut self, data: &[u8], offset: usize) -> Result<(ComponentFunction, usize)> {
        let mut current_offset = offset;
        
        // Parse canonical operation type
        let (canon_kind, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        // Parse type reference
        let (type_ref, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        // Parse core function index (for lift operations)
        let core_func_index = if canon_kind == 0 { // lift
            let (func_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
            current_offset += bytes_read;
            Some(func_idx)
        } else {
            None
        };
        
        let function = ComponentFunction {
            ty: type_ref,
            core_func_index,
            options: CanonicalOptions::default(),
        };
        
        Ok((function, current_offset - offset))
    }
    
    /// Update parser state based on section type
    fn update_parser_state(&mut self, section_id: u8) {
        let new_state = match section_id {
            component_section_ids::TYPE => ComponentParserState::Types,
            component_section_ids::IMPORT | component_section_ids::EXPORT => ComponentParserState::Interface,
            component_section_ids::COMPONENT => ComponentParserState::NestedComponent { depth: 1 },
            _ => ComponentParserState::Component,
        };
        
        self.parser_state = new_state;
        self.registry.set_parser_state(new_state);
    }
    
    // Placeholder implementations for other sections
    fn parse_component_imports_type(&mut self, _data: &[u8], _offset: usize) -> Result<(SimpleBoundedVec<crate::component_types::ComponentImport, 128>, usize)> {
        Ok((SimpleBoundedVec::new(), 0))
    }
    
    fn parse_component_exports_type(&mut self, _data: &[u8], _offset: usize) -> Result<(SimpleBoundedVec<crate::component_types::ComponentExport, 128>, usize)> {
        Ok((SimpleBoundedVec::new(), 0))
    }
    
    fn parse_instance_section(&mut self, data: &[u8], module: &mut EnhancedModule) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        let component = module.component_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Component Model not enabled for instance section"
            ))?;
        
        for _ in 0..count {
            let (instance, bytes_consumed) = self.parse_component_instance(data, offset)?;
            offset += bytes_consumed;
            
            component.add_instance(instance)?;
        }
        
        Ok(())
    }
    
    fn parse_value_section(&mut self, data: &[u8], module: &mut EnhancedModule) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        let component = module.component_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Component Model not enabled for value section"
            ))?;
        
        for _ in 0..count {
            let (value, bytes_consumed) = self.parse_component_value(data, offset)?;
            offset += bytes_consumed;
            
            component.add_value(value)?;
        }
        
        Ok(())
    }
    
    fn parse_core_module_section(&mut self, data: &[u8], module: &mut EnhancedModule) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        let component = module.component_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Component Model not enabled for core module section"
            ))?;
        
        for _ in 0..count {
            // Parse embedded core module
            let (module_size, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            if offset + module_size as usize > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Core module extends beyond section bounds"
                ));
            }
            
            let module_data = &data[offset..offset + module_size as usize];
            offset += module_size as usize;
            
            // Parse the embedded core module using SimpleParser
            let mut core_parser = crate::simple_parser::SimpleParser::new();
            let core_module = core_parser.parse(module_data)?;
            
            // Store the core module in the component
            component.add_core_module(core_module)?;
        }
        
        Ok(())
    }
    
    fn parse_core_instance_section(&mut self, data: &[u8], module: &mut EnhancedModule) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        let component = module.component_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Component Model not enabled for core instance section"
            ))?;
        
        for _ in 0..count {
            // Parse core instance (instantiation of a core module)
            let (module_idx, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            // Parse instantiation arguments
            let (arg_count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            let mut args = SimpleBoundedVec::new();
            for _ in 0..arg_count {
                let (arg, bytes_consumed) = self.parse_instantiation_arg(data, offset)?;
                offset += bytes_consumed;
                args.push(arg)?;
            }
            
            let instance = ComponentInstance {
                module_index: module_idx,
                ty: 0, // TODO: Parse or infer instance type
                args,
            };
            
            component.add_instance(instance)?;
        }
        
        Ok(())
    }
    
    fn parse_core_type_section(&mut self, data: &[u8], module: &mut EnhancedModule) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        let component = module.component_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Component Model not enabled for core type section"
            ))?;
        
        for _ in 0..count {
            // Parse core type definition (function types, etc.)
            let (core_type, bytes_consumed) = self.parse_core_type_definition(data, offset)?;
            offset += bytes_consumed;
            
            component.add_core_type(core_type)?;
        }
        
        Ok(())
    }
    
    fn parse_nested_component_section(&mut self, data: &[u8], module: &mut EnhancedModule) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        let component = module.component_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Component Model not enabled for nested component section"
            ))?;
        
        for _ in 0..count {
            // Parse nested component
            let (component_size, bytes_read) = leb128::read_leb128_u32(data, offset)?;
            offset += bytes_read;
            
            if offset + component_size as usize > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Nested component extends beyond section bounds"
                ));
            }
            
            let component_data = &data[offset..offset + component_size as usize];
            offset += component_size as usize;
            
            // Recursively parse the nested component
            // For now, we'll store it as raw bytes and mark as nested
            component.add_nested_component(component_data.to_vec())?;
        }
        
        Ok(())
    }
    
    fn parse_alias_section(&mut self, data: &[u8], module: &mut EnhancedModule) -> Result<()> {
        let mut offset = 0;
        let (count, bytes_read) = leb128::read_leb128_u32(data, offset)?;
        offset += bytes_read;
        
        let component = module.component_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Component Model not enabled for alias section"
            ))?;
        
        for _ in 0..count {
            // Parse alias definition
            let (alias, bytes_consumed) = self.parse_alias_definition(data, offset)?;
            offset += bytes_consumed;
            
            component.add_alias(alias)?;
        }
        
        Ok(())
    }
    
    /// Parse component instance
    fn parse_component_instance(&mut self, data: &[u8], offset: usize) -> Result<(ComponentInstance, usize)> {
        let mut current_offset = offset;
        
        // Parse instantiation target (component index)
        let (component_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        // Parse instantiation arguments
        let (arg_count, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        let mut args = SimpleBoundedVec::new();
        for _ in 0..arg_count {
            let (arg, bytes_consumed) = self.parse_instantiation_arg(data, current_offset)?;
            current_offset += bytes_consumed;
            args.push(arg)?;
        }
        
        let instance = ComponentInstance {
            module_index: component_idx,
            ty: 0, // TODO: Parse or infer instance type
            args,
        };
        
        Ok((instance, current_offset - offset))
    }
    
    /// Parse component value
    fn parse_component_value(&mut self, data: &[u8], offset: usize) -> Result<(ComponentValue, usize)> {
        let mut current_offset = offset;
        
        // Parse value kind
        if current_offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end while parsing component value"
            ));
        }
        
        let value_kind = data[current_offset];
        current_offset += 1;
        
        let value = match value_kind {
            0x00 => {
                // Instance value
                let (instance_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                ComponentValue::Instance(instance_idx)
            }
            0x01 => {
                // Function value
                let (func_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
                current_offset += bytes_read;
                ComponentValue::Function(func_idx)
            }
            _ => {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::INVALID_TYPE,
                    "Unknown component value kind"
                ));
            }
        };
        
        Ok((value, current_offset - offset))
    }
    
    /// Parse instantiation argument
    fn parse_instantiation_arg(&mut self, data: &[u8], offset: usize) -> Result<(InstantiationArg, usize)> {
        let mut current_offset = offset;
        
        // Parse argument name
        let (name_len, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        if current_offset + name_len as usize > data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Argument name extends beyond section"
            ));
        }
        
        let name = SimpleBoundedString::from_str(
            core::str::from_utf8(&data[current_offset..current_offset + name_len as usize])
                .map_err(|_| Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid UTF-8 in argument name"
                ))?
        );
        current_offset += name_len as usize;
        
        // Parse argument kind and index
        let (kind, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        let (index, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        let item_kind = match kind {
            0x00 => ItemKind::Function,
            0x01 => ItemKind::Table,
            0x02 => ItemKind::Memory,
            0x03 => ItemKind::Global,
            _ => {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::INVALID_TYPE,
                    "Unknown instantiation argument kind"
                ));
            }
        };
        
        let arg = InstantiationArg {
            name,
            kind: item_kind,
            index,
        };
        
        Ok((arg, current_offset - offset))
    }
    
    /// Parse core type definition
    fn parse_core_type_definition(&mut self, data: &[u8], offset: usize) -> Result<(crate::types::FuncType, usize)> {
        let mut current_offset = offset;
        
        // For simplicity, assume this is a function type (0x60)
        if current_offset >= data.len() || data[current_offset] != 0x60 {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::INVALID_TYPE,
                "Expected function type in core type section"
            ));
        }
        current_offset += 1;
        
        // Parse parameter types
        let (param_count, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        let mut params = SimpleBoundedVec::new();
        for _ in 0..param_count {
            let (value_type, bytes_read) = self.parse_value_type(data, current_offset)?;
            current_offset += bytes_read;
            params.push(value_type).map_err(|_| Error::new(
                ErrorCategory::Memory,
                codes::CAPACITY_EXCEEDED,
                "Function parameters capacity exceeded"
            ))?;
        }
        
        // Parse result types
        let (result_count, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        let mut results = SimpleBoundedVec::new();
        for _ in 0..result_count {
            let (value_type, bytes_read) = self.parse_value_type(data, current_offset)?;
            current_offset += bytes_read;
            results.push(value_type).map_err(|_| Error::new(
                ErrorCategory::Memory,
                codes::CAPACITY_EXCEEDED,
                "Function results capacity exceeded"
            ))?;
        }
        
        let func_type = crate::types::FuncType { params, results };
        Ok((func_type, current_offset - offset))
    }
    
    /// Parse alias definition
    fn parse_alias_definition(&mut self, data: &[u8], offset: usize) -> Result<(ComponentAlias, usize)> {
        let mut current_offset = offset;
        
        // Parse alias kind
        if current_offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end while parsing alias"
            ));
        }
        
        let alias_kind = data[current_offset];
        current_offset += 1;
        
        // Parse target instance and item
        let (instance_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        let (item_idx, bytes_read) = leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        let alias = ComponentAlias {
            kind: alias_kind,
            instance_index: instance_idx,
            item_index: item_idx,
        };
        
        Ok((alias, current_offset - offset))
    }
    
    /// Get component registry
    pub fn registry(&self) -> &ComponentRegistry {
        &self.registry
    }
    
    /// Get mutable component registry
    pub fn registry_mut(&mut self) -> &mut ComponentRegistry {
        &mut self.registry
    }
    
    /// Parse core WebAssembly value type
    fn parse_value_type(&self, data: &[u8], offset: usize) -> Result<(crate::types::ValueType, usize)> {
        if offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end while parsing value type"
            ));
        }
        
        let byte = data[offset];
        let value_type = match byte {
            0x7F => crate::types::ValueType::I32,
            0x7E => crate::types::ValueType::I64,
            0x7D => crate::types::ValueType::F32,
            0x7C => crate::types::ValueType::F64,
            0x70 => crate::types::ValueType::FuncRef,
            0x6F => crate::types::ValueType::ExternRef,
            _ => {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::INVALID_TYPE,
                    "Unknown value type"
                ));
            }
        };
        
        Ok((value_type, 1))
    }
}

impl Default for ComponentSectionParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enhanced_module::ParserMode;
    use crate::simple_module::SimpleModule;
    
    #[test]
    fn test_component_section_parser_creation() {
        let parser = ComponentSectionParser::new();
        assert_eq!(parser.parser_state, ComponentParserState::Core);
    }
    
    #[test]
    fn test_parse_component_value_type() {
        let mut parser = ComponentSectionParser::new();
        
        // Test parsing bool type (0x7F)
        let data = [0x7F];
        let (value_type, bytes_consumed) = parser.parse_component_value_type(&data, 0).unwrap();
        assert_eq!(value_type, ComponentValueType::Bool);
        assert_eq!(bytes_consumed, 1);
        
        // Test parsing u32 type (0x79)
        let data = [0x79];
        let (value_type, bytes_consumed) = parser.parse_component_value_type(&data, 0).unwrap();
        assert_eq!(value_type, ComponentValueType::U32);
        assert_eq!(bytes_consumed, 1);
    }
    
    #[test]
    fn test_parse_empty_function_params() {
        let mut parser = ComponentSectionParser::new();
        
        // Empty params (count = 0)
        let data = [0x00];
        let (params, bytes_consumed) = parser.parse_function_params(&data, 0).unwrap();
        assert_eq!(params.len(), 0);
        assert_eq!(bytes_consumed, 1);
    }
    
    #[test]
    fn test_component_section_integration() {
        let mut parser = ComponentSectionParser::new();
        let core = SimpleModule::new();
        let mode = ParserMode::ComponentAware { type_budget: 64 * 1024, max_types: 512 };
        let mut module = EnhancedModule::new_with_component(core, mode).unwrap();
        
        // Test that parser can handle an empty type section
        let data = [0x00]; // count = 0
        let result = parser.parse_type_section(&data, &mut module);
        assert!(result.is_ok());
    }
}