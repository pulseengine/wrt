use wrt_component::parser;

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

#[test]
fn test_multiple_imports() {
    // This test just verifies that we can find a builtin import
    // even if it's among other imports
    let module = create_test_module("wasi_builtin", "resource.drop");

    let builtin_names = parser::scan_for_builtins(&module).unwrap();
    assert_eq!(builtin_names.len(), 1);
    assert_eq!(builtin_names[0], "resource.drop");
}
