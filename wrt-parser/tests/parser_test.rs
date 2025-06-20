use wrt_parser::{parse_wasm, SimpleModule};

#[test]
fn test_empty_module() {
    // Minimal valid WebAssembly module (just header)
    let wasm = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
    let module = parse_wasm(&wasm).unwrap();
    
    assert_eq!(module.types.len(), 0);
    assert_eq!(module.functions.len(), 0);
    assert_eq!(module.imports.len(), 0);
    assert_eq!(module.exports.len(), 0);
}

#[test]
fn test_module_with_type_section() {
    // Module with a single function type: () -> i32
    let wasm = [
        // Header
        0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00,
        // Type section
        0x01, // Section ID
        0x05, // Section size (corrected: 1 + 1 + 1 + 1 + 1 = 5)
        0x01, // Number of types
        0x60, // Function type
        0x00, // Zero parameters
        0x01, // One result
        0x7F, // i32
    ];
    
    let module = parse_wasm(&wasm).unwrap();
    
    assert_eq!(module.types.len(), 1);
    let func_type = &module.types.get(0).unwrap();
    assert_eq!(func_type.params.len(), 0);
    assert_eq!(func_type.results.len(), 1);
}

#[test]
fn test_module_with_import() {
    // Module that imports a function
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
        // Import section
        0x02, // Section ID
        0x0C, // Section size (1 + 1 + 3 + 1 + 4 + 1 + 1 = 12)
        0x01, // Number of imports (1 byte)
        0x03, // Module name length (1 byte)
        b'e', b'n', b'v', // "env" (3 bytes)
        0x04, // Field name length (1 byte)
        b'f', b'u', b'n', b'c', // "func" (4 bytes)
        0x00, // Import kind (function) (1 byte)
        0x00, // Type index (1 byte)
    ];
    
    let module = parse_wasm(&wasm).unwrap();
    
    assert_eq!(module.imports.len(), 1);
    let import = module.imports.get(0).unwrap();
    
    // Check module name
    let module_name: Vec<u8> = import.module.iter().copied().collect();
    assert_eq!(module_name, b"env");
    
    // Check field name
    let field_name: Vec<u8> = import.name.iter().copied().collect();
    assert_eq!(field_name, b"func");
}

#[test]
fn test_module_with_export() {
    // Module with a function and export
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
        0x00, // Type index
        // Export section
        0x07, // Section ID
        0x08, // Section size (1 + 1 + 4 + 1 + 1 = 8)
        0x01, // Number of exports
        0x04, // Name length
        b'm', b'a', b'i', b'n', // "main"
        0x00, // Export kind (function)
        0x00, // Function index
    ];
    
    let module = parse_wasm(&wasm).unwrap();
    
    assert_eq!(module.exports.len(), 1);
    let export = module.exports.get(0).unwrap();
    
    // Check export name
    let name: Vec<u8> = export.name.iter().copied().collect();
    assert_eq!(name, b"main");
    assert_eq!(export.index, 0);
}