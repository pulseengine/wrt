use wrt_parser::{parse_wasm, SimpleParser, ValidationConfig};

#[test]
fn test_empty_module_validation() {
    // Minimal valid WebAssembly module (just header)
    let wasm = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
    let module = parse_wasm(&wasm).unwrap();
    
    // Should have no sections
    assert_eq!(module.types.len(), 0);
    assert_eq!(module.functions.len(), 0);
    assert_eq!(module.imports.len(), 0);
    assert_eq!(module.exports.len(), 0);
    assert_eq!(module.globals.len(), 0);
    assert_eq!(module.memories.len(), 0);
    assert_eq!(module.tables.len(), 0);
    assert_eq!(module.data.len(), 0);
    assert_eq!(module.elements.len(), 0);
    assert_eq!(module.code.len(), 0);
    assert!(module.start.is_none());
}

#[test]
fn test_module_with_global_i32_const() {
    // Module with a global initialized with i32.const 42
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Global section
        0x06, // Section ID
        0x06, // Section size (1 + 1 + 1 + 1 + 1 + 1 = 6)
        0x01, // Number of globals
        0x7F, // i32 type
        0x00, // immutable
        0x41, // i32.const
        0x2A, // 42
        0x0B, // end
    ];
    
    let module = parse_wasm(&wasm).unwrap();
    
    assert_eq!(module.globals.len(), 1);
    let global = module.globals.get(0).unwrap();
    assert_eq!(global.value_type, wrt_parser::ValueType::I32);
    assert!(!global.mutable);
}

#[test]
fn test_module_with_memory() {
    // Module with a memory (min=1, no max)
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Memory section
        0x05, // Section ID
        0x03, // Section size
        0x01, // Number of memories
        0x00, // No maximum
        0x01, // Minimum = 1
    ];
    
    let module = parse_wasm(&wasm).unwrap();
    
    assert_eq!(module.memories.len(), 1);
    let memory = module.memories.get(0).unwrap();
    assert_eq!(memory.limits.min, 1);
    assert_eq!(memory.limits.max, None);
}

#[test]
fn test_module_with_memory_and_max() {
    // Module with a memory (min=1, max=10)
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Memory section
        0x05, // Section ID
        0x04, // Section size
        0x01, // Number of memories
        0x01, // Has maximum
        0x01, // Minimum = 1
        0x0A, // Maximum = 10
    ];
    
    let module = parse_wasm(&wasm).unwrap();
    
    assert_eq!(module.memories.len(), 1);
    let memory = module.memories.get(0).unwrap();
    assert_eq!(memory.limits.min, 1);
    assert_eq!(memory.limits.max, Some(10));
}

#[test]
fn test_module_with_table() {
    // Module with a funcref table (min=1, max=5)
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Table section
        0x04, // Section ID
        0x05, // Section size
        0x01, // Number of tables
        0x70, // funcref type
        0x01, // Has maximum
        0x01, // Minimum = 1
        0x05, // Maximum = 5
    ];
    
    let module = parse_wasm(&wasm).unwrap();
    
    assert_eq!(module.tables.len(), 1);
    let table = module.tables.get(0).unwrap();
    assert_eq!(table.element_type, wrt_parser::ValueType::FuncRef);
    assert_eq!(table.limits.min, 1);
    assert_eq!(table.limits.max, Some(5));
}

#[test]
fn test_module_with_function_and_code() {
    // Module with a single function that takes no params and returns i32
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Type section
        0x01, // Section ID
        0x05, // Section size
        0x01, // Number of types
        0x60, // Function type
        0x00, // Zero parameters
        0x01, // One result
        0x7F, // i32
        // Function section
        0x03, // Section ID
        0x02, // Section size
        0x01, // Number of functions
        0x00, // Type index 0
        // Code section
        0x0A, // Section ID
        0x07, // Section size
        0x01, // Number of function bodies
        0x05, // Function body size
        0x00, // No locals
        0x41, // i32.const
        0x2A, // 42
        0x0B, // end
    ];
    
    let module = parse_wasm(&wasm).unwrap();
    
    assert_eq!(module.types.len(), 1);
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.code.len(), 1);
    
    let func_type = module.types.get(0).unwrap();
    assert_eq!(func_type.params.len(), 0);
    assert_eq!(func_type.results.len(), 1);
    assert_eq!(func_type.results.get(0).unwrap(), &wrt_parser::ValueType::I32);
    
    let function_body = module.code.get(0).unwrap();
    assert_eq!(function_body.locals.len(), 0);
    assert_eq!(function_body.code.len(), 3); // i32.const, 42, end
}

#[test]
fn test_module_with_start_function() {
    // Module with a start function
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Type section - function type [] -> []
        0x01, // Section ID
        0x04, // Section size
        0x01, // Number of types
        0x60, // Function type
        0x00, // Zero parameters
        0x00, // Zero results
        // Function section
        0x03, // Section ID
        0x02, // Section size
        0x01, // Number of functions
        0x00, // Type index 0
        // Start section
        0x08, // Section ID
        0x01, // Section size
        0x00, // Function index 0
        // Code section
        0x0A, // Section ID
        0x04, // Section size
        0x01, // Number of function bodies
        0x02, // Function body size
        0x00, // No locals
        0x0B, // end
    ];
    
    let module = parse_wasm(&wasm).unwrap();
    
    assert_eq!(module.start, Some(0));
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.code.len(), 1);
}

#[test]
fn test_module_with_data_segment() {
    // Module with memory and data segment
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Memory section
        0x05, // Section ID
        0x03, // Section size
        0x01, // Number of memories
        0x00, // No maximum
        0x01, // Minimum = 1
        // Data section
        0x0B, // Section ID
        0x0B, // Section size (1 + 1 + 1 + 1 + 1 + 5 + 1 = 11)
        0x01, // Number of data segments
        0x00, // Memory index 0
        0x41, // i32.const
        0x00, // 0 (offset)
        0x0B, // end
        0x05, // Data length
        b'h', b'e', b'l', b'l', b'o', // "hello"
    ];
    
    let module = parse_wasm(&wasm).unwrap();
    
    assert_eq!(module.memories.len(), 1);
    assert_eq!(module.data.len(), 1);
    
    let data_segment = module.data.get(0).unwrap();
    assert_eq!(data_segment.memory_index, 0);
    assert_eq!(data_segment.data.len(), 5);
    
    // Check data content
    let data: Vec<u8> = data_segment.data.iter().copied().collect();
    assert_eq!(data, b"hello");
}

#[test]
fn test_module_with_element_segment() {
    // Module with table and element segment
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Type section - function type [] -> []
        0x01, // Section ID
        0x04, // Section size
        0x01, // Number of types
        0x60, // Function type
        0x00, // Zero parameters
        0x00, // Zero results
        // Function section
        0x03, // Section ID
        0x02, // Section size
        0x01, // Number of functions
        0x00, // Type index 0
        // Table section
        0x04, // Section ID
        0x04, // Section size
        0x01, // Number of tables
        0x70, // funcref type
        0x00, // No maximum
        0x01, // Minimum = 1
        // Element section
        0x09, // Section ID
        0x07, // Section size (1 + 1 + 1 + 1 + 1 + 1 + 1 = 7)
        0x01, // Number of element segments
        0x00, // Table index 0
        0x41, // i32.const
        0x00, // 0 (offset)
        0x0B, // end
        0x01, // Number of function indices
        0x00, // Function index 0
        // Code section
        0x0A, // Section ID
        0x04, // Section size
        0x01, // Number of function bodies
        0x02, // Function body size
        0x00, // No locals
        0x0B, // end
    ];
    
    let module = parse_wasm(&wasm).unwrap();
    
    assert_eq!(module.tables.len(), 1);
    assert_eq!(module.elements.len(), 1);
    assert_eq!(module.functions.len(), 1);
    
    let element_segment = module.elements.get(0).unwrap();
    assert_eq!(element_segment.table_index, 0);
    assert_eq!(element_segment.init.len(), 1);
    assert_eq!(element_segment.init.get(0).unwrap(), &0);
}

#[test]
fn test_validation_invalid_function_type_index() {
    // Module with function that references invalid type index
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Type section - one type
        0x01, // Section ID
        0x04, // Section size
        0x01, // Number of types
        0x60, // Function type
        0x00, // Zero parameters
        0x00, // Zero results
        // Function section - reference invalid type index 1
        0x03, // Section ID
        0x02, // Section size
        0x01, // Number of functions
        0x01, // Type index 1 (invalid, only index 0 exists)
    ];
    
    let result = parse_wasm(&wasm);
    assert!(result.is_err());
}

#[test]
fn test_validation_export_invalid_function_index() {
    // Module with export that references invalid function index
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Export section - reference non-existent function
        0x07, // Section ID
        0x08, // Section size
        0x01, // Number of exports
        0x04, // Name length
        b't', b'e', b's', b't', // "test"
        0x00, // Export kind (function)
        0x00, // Function index 0 (but no functions exist)
    ];
    
    let result = parse_wasm(&wasm);
    assert!(result.is_err());
}

#[test]
fn test_validation_multiple_memories() {
    // Module with multiple memories (invalid in WebAssembly 1.0)
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Memory section with 2 memories
        0x05, // Section ID
        0x05, // Section size
        0x02, // Number of memories (invalid - only 1 allowed)
        0x00, // No maximum
        0x01, // Minimum = 1
        0x00, // No maximum
        0x02, // Minimum = 2
    ];
    
    let result = parse_wasm(&wasm);
    assert!(result.is_err());
}

#[test]
fn test_validation_memory_limits() {
    // Module with invalid memory limits (max < min)
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Memory section with invalid limits
        0x05, // Section ID
        0x04, // Section size
        0x01, // Number of memories
        0x01, // Has maximum
        0x05, // Minimum = 5
        0x02, // Maximum = 2 (invalid - less than minimum)
    ];
    
    let result = parse_wasm(&wasm);
    assert!(result.is_err());
}

#[test]
fn test_parser_without_validation() {
    // Test that parser can skip validation when configured
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Memory section with invalid limits (would normally fail validation)
        0x05, // Section ID
        0x04, // Section size
        0x01, // Number of memories
        0x01, // Has maximum
        0x05, // Minimum = 5
        0x02, // Maximum = 2 (invalid - less than minimum)
    ];
    
    let mut parser = SimpleParser::without_validation();
    let result = parser.parse(&wasm);
    // Should succeed because validation is disabled
    assert!(result.is_ok());
}

#[test]
fn test_parser_with_custom_validation() {
    // Test parser with custom validation config
    let mut config = ValidationConfig::default();
    config.enable_memory = false; // Disable memory validation
    
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Memory section with invalid limits
        0x05, // Section ID
        0x04, // Section size
        0x01, // Number of memories
        0x01, // Has maximum
        0x05, // Minimum = 5
        0x02, // Maximum = 2 (would be invalid, but memory validation disabled)
    ];
    
    let mut parser = SimpleParser::with_validation(config);
    let result = parser.parse(&wasm);
    // Should succeed because memory validation is disabled
    assert!(result.is_ok());
}

#[test]
fn test_complex_module_with_all_sections() {
    // Complex module with multiple section types
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        
        // Type section - function type [i32] -> [i32]
        0x01, // Section ID
        0x06, // Section size
        0x01, // Number of types
        0x60, // Function type
        0x01, // One parameter
        0x7F, // i32
        0x01, // One result
        0x7F, // i32
        
        // Import section - import a function
        0x02, // Section ID
        0x0C, // Section size
        0x01, // Number of imports
        0x03, // Module name length
        b'e', b'n', b'v', // "env"
        0x04, // Field name length
        b'l', b'o', b'g', b'0', // "log0"
        0x00, // Import kind (function)
        0x00, // Type index 0
        
        // Function section
        0x03, // Section ID
        0x02, // Section size
        0x01, // Number of functions
        0x00, // Type index 0
        
        // Memory section
        0x05, // Section ID
        0x03, // Section size
        0x01, // Number of memories
        0x00, // No maximum
        0x01, // Minimum = 1
        
        // Global section
        0x06, // Section ID
        0x06, // Section size
        0x01, // Number of globals
        0x7F, // i32 type
        0x01, // mutable
        0x41, // i32.const
        0x00, // 0
        0x0B, // end
        
        // Export section
        0x07, // Section ID
        0x0B, // Section size
        0x01, // Number of exports
        0x06, // Name length
        b'a', b'd', b'd', b'O', b'n', b'e', // "addOne"
        0x00, // Export kind (function)
        0x01, // Function index 1 (imported function is 0, our function is 1)
        
        // Code section
        0x0A, // Section ID
        0x09, // Section size
        0x01, // Number of function bodies
        0x07, // Function body size
        0x00, // No locals
        0x20, // local.get
        0x00, // local index 0
        0x41, // i32.const
        0x01, // 1
        0x6A, // i32.add
        0x0B, // end
    ];
    
    let module = parse_wasm(&wasm).unwrap();
    
    // Verify all sections were parsed
    assert_eq!(module.types.len(), 1);
    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.memories.len(), 1);
    assert_eq!(module.globals.len(), 1);
    assert_eq!(module.exports.len(), 1);
    assert_eq!(module.code.len(), 1);
    
    // Verify import
    let import = module.imports.get(0).unwrap();
    let module_name: Vec<u8> = import.module.iter().copied().collect();
    let field_name: Vec<u8> = import.name.iter().copied().collect();
    assert_eq!(module_name, b"env");
    assert_eq!(field_name, b"log0");
    assert!(matches!(import.desc, wrt_parser::simple_module::ImportDesc::Func(0)));
    
    // Verify export
    let export = module.exports.get(0).unwrap();
    let export_name: Vec<u8> = export.name.iter().copied().collect();
    assert_eq!(export_name, b"addOne");
    assert_eq!(export.kind, wrt_parser::simple_module::ExportKind::Func);
    assert_eq!(export.index, 1);
    
    // Verify global
    let global = module.globals.get(0).unwrap();
    assert_eq!(global.value_type, wrt_parser::ValueType::I32);
    assert!(global.mutable);
    
    // Verify function code
    let code = module.code.get(0).unwrap();
    assert_eq!(code.locals.len(), 0);
    assert!(code.code.len() > 0);
}