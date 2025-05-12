use std::collections::HashSet;

use wrt_component::parser;
use wrt_error::Result;
use wrt_types::builtin::BuiltinType;

// Helper to create a minimal test module with an import
fn create_test_module(module_name: &str, import_name: &str) -> Vec<u8> {
    // WebAssembly module header
    let mut module = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

    // Type section (empty)
    module.extend_from_slice(&[0x01, 0x04, 0x01, 0x60, 0x00, 0x00]);

    // Import section with one import
    let module_name_len = module_name.len() as u8;
    let import_name_len = import_name.len() as u8;

    // Import section header
    module.push(0x02); // Import section ID
    module.push(0x07 + module_name_len + import_name_len); // Section size
    module.push(0x01); // Number of imports

    // Import entry
    module.push(module_name_len); // Module name length
    module.extend_from_slice(module_name.as_bytes()); // Module name
    module.push(import_name_len); // Import name length
    module.extend_from_slice(import_name.as_bytes()); // Import name
    module.push(0x00); // Import kind (function)
    module.push(0x00); // Type index

    module
}

#[test]
fn test_scan_for_builtins() {
    // Create a minimal test module with a wasi_builtin import
    let module = create_test_module("wasi_builtin", "resource.create");

    // Test that we can find the built-in import
    let builtin_names = parser::scan_for_builtins(&module).unwrap();
    assert_eq!(builtin_names.len(), 1);
    assert_eq!(builtin_names[0], "resource.create");
}

#[test]
fn test_non_builtin_imports() {
    // Create a test module with an import that is not from wasi_builtin
    let module = create_test_module("other_module", "other_import");

    // We should not find any built-in imports
    let builtin_names = parser::scan_for_builtins(&module).unwrap();
    assert_eq!(builtin_names.len(), 0);
}

// Test that works for both implementations (with or without wasmparser)
#[test]
fn test_get_required_builtins() {
    // Create a test module with a wasi_builtin import for resource.create
    let module = create_test_module("wasi_builtin", "resource.create");

    // Test the mapping to built-in types using the abstraction layer
    let required_builtins = get_required_builtins(&module).unwrap();
    assert!(required_builtins.contains(&BuiltinType::ResourceCreate));
    assert_eq!(required_builtins.len(), 1);
}

// Test that works for both implementations (with or without wasmparser)
#[test]
fn test_random_builtin_import() {
    // Create a test module with a random_get_bytes import
    let module = create_test_module("wasi_builtin", "random_get_bytes");

    // Random get bytes imports should not map to any built-in type
    let required_builtins = get_required_builtins(&module).unwrap();
    assert!(required_builtins.is_empty());
}

// Test multiple built-in imports in a single module
#[test]
fn test_multiple_builtins() {
    // First create a module with resource.create
    let mut module = create_test_module("wasi_builtin", "resource.create");

    // Now manually add another import for resource.drop
    // We need to update the section size and count
    let import_section_size_offset = 11; // Position of import section size
    let import_count_offset = 12; // Position of import count

    // Increase the import count
    module[import_count_offset] = 0x02; // Now 2 imports

    // Add the second import (resource.drop)
    let second_import = &[
        0x0c, // Module name length (12)
        0x77, 0x61, 0x73, 0x69, 0x5f, 0x62, 0x75, 0x69, 0x6c, 0x74, 0x69,
        0x6e, // "wasi_builtin"
        0x0c, // Import name length (12)
        0x72, 0x65, 0x73, 0x6f, 0x75, 0x72, 0x63, 0x65, 0x2e, 0x64, 0x72, 0x6f,
        0x70, // "resource.drop"
        0x00, // Import kind (function)
        0x00, // Type index
    ];

    // Append the second import
    module.extend_from_slice(second_import);

    // Update the import section size
    let new_section_size = module[import_section_size_offset] as usize + second_import.len();
    module[import_section_size_offset] = new_section_size as u8;

    // Test that both built-ins are detected
    let required_builtins = get_required_builtins(&module).unwrap();
    assert!(required_builtins.contains(&BuiltinType::ResourceCreate));
    assert!(required_builtins.contains(&BuiltinType::ResourceDrop));
    assert_eq!(required_builtins.len(), 2);
}

// Test for error handling with malformed modules
#[test]
fn test_malformed_module() {
    // Create an invalid WebAssembly module
    let invalid_module = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00]; // Missing last byte of magic

    // The parser should return an error
    let result = get_required_builtins(&invalid_module);
    assert!(result.is_err());
}
