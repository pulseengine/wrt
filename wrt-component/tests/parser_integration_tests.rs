use wrt_decoder::section_reader::SectionReader;
use wrt_decoder::Parser;
use wrt_decoder::Payload;

// Helper to create a complex test module with multiple section types
fn create_complex_test_module() -> Vec<u8> {
    // WebAssembly module header
    let mut module = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

    // Type section with two function signatures
    // (i32, i32) -> i32 and () -> i32
    module.extend_from_slice(&[
        0x01, 0x0C, // Type section ID and size
        0x02, // Number of types
        0x60, // Function type
        0x02, // Number of params
        0x7F, 0x7F, // i32, i32
        0x01, // Number of results
        0x7F, // i32
        0x60, // Function type
        0x00, // No params
        0x01, // Number of results
        0x7F, // i32
    ]);

    // Import section with one function import
    module.extend_from_slice(&[
        0x02, 0x13, // Import section ID and size
        0x01, // Number of imports
        0x0C, // Module name length
        // "wasi_builtin"
        0x77, 0x61, 0x73, 0x69, 0x5F, 0x62, 0x75, 0x69, 0x6C, 0x74, 0x69, 0x6E,
        0x03, // Field name length
        // "add"
        0x61, 0x64, 0x64, 0x00, // Import kind (function)
        0x00, // Type index
    ]);

    // Function section with two function declarations
    module.extend_from_slice(&[
        0x03, 0x03, // Function section ID and size
        0x02, // Number of functions
        0x00, 0x01, // Type indices
    ]);

    // Memory section with one memory
    module.extend_from_slice(&[
        0x05, 0x03, // Memory section ID and size
        0x01, // Number of memories
        0x00, 0x01, // Min 0, max 1 pages
    ]);

    // Export section with two exports
    module.extend_from_slice(&[
        0x07, 0x13, // Export section ID and size
        0x02, // Number of exports
        0x03, // Export name length
        // "add"
        0x61, 0x64, 0x64, 0x00, // Export kind (function)
        0x01, // Function index
        0x06, // Export name length
        // "memory"
        0x6D, 0x65, 0x6D, 0x6F, 0x72, 0x79, 0x02, // Export kind (memory)
        0x00, // Memory index
    ]);

    // Code section with two function bodies
    module.extend_from_slice(&[
        0x0A, 0x11, // Code section ID and size
        0x02, // Number of functions
        // Function 1 body
        0x07, // Function body size
        0x00, // Local variable count
        0x20, 0x00, // get_local 0
        0x20, 0x01, // get_local 1
        0x6A, // i32.add
        0x0B, // end
        // Function 2 body
        0x05, // Function body size
        0x00, // Local variable count
        0x41, 0x2A, // i32.const 42
        0x0B, // end
    ]);

    module
}

/// Helper to create a malformed module with invalid section sizes
fn create_invalid_section_size_module() -> Vec<u8> {
    // WebAssembly module header
    let mut module = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

    // Add a section with an invalid size (larger than the actual content)
    module.extend_from_slice(&[
        0x01, 0x10, // Type section ID and incorrect size (16 bytes, but only 4 bytes follow)
        0x01, // Number of types
        0x60, // Function type
        0x00, // No params
        0x00, // No results
    ]);

    module
}

/// Helper to create a truncated module
fn create_truncated_module() -> Vec<u8> {
    // WebAssembly module header
    let mut module = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

    // Type section start, but truncated
    module.extend_from_slice(&[
        0x01, 0x05, // Type section ID and size
        0x01, // Number of types
              // Missing data here...
    ]);

    module
}

#[test]
fn test_full_module_parsing() {
    // Create a test module with multiple section types
    let module = create_complex_test_module();

    // Verify parsing of the entire module works without errors
    let parser = Parser::new(&module);
    let payloads: Result<Vec<_>, _> = parser.collect();
    assert!(payloads.is_ok());

    let payloads = payloads.unwrap();

    // Check that we have the expected number of sections
    // +1 for the Version payload
    assert_eq!(payloads.len(), 6 + 1);
}

#[test]
fn test_section_reader() {
    let module = create_complex_test_module();

    // Create a section reader
    let mut reader = SectionReader::new(&module).unwrap();

    // Test finding the import section
    let import_section = reader.find_section(2); // 2 is the import section ID
    assert!(import_section.is_ok());
    let import_section = import_section.unwrap();
    assert!(import_section.is_some());

    // Verify we can extract the section data
    let (offset, size) = import_section.unwrap();
    let section_data = &module[offset..offset + size];
    assert_eq!(section_data[0], 0x01); // Number of imports should be 1

    // Test iterating through sections
    let mut reader = SectionReader::new(&module).unwrap();
    let mut sections_found = 0;

    while let Ok(Some((id, _, _))) = reader.next_section() {
        sections_found += 1;
        // Make sure we don't get any section IDs outside the expected range
        assert!(id <= 11, "Invalid section ID found: {}", id);
    }

    // Verify we found the right number of sections
    assert_eq!(sections_found, 6);
}

#[test]
fn test_invalid_section_size() {
    let module = create_invalid_section_size_module();

    // The section reader should detect the invalid section size
    let mut reader = SectionReader::new(&module).unwrap();
    let result = reader.next_section();

    // The first section (ID 1, type section) should be readable but will have an invalid size
    assert!(result.is_ok());

    // Reading all sections should fail
    let parser = Parser::new(&module);
    let result: Result<Vec<_>, _> = parser.collect();
    assert!(result.is_err());
}

#[test]
fn test_truncated_module() {
    let module = create_truncated_module();

    // The parser should detect the truncated module
    let parser = Parser::new(&module);
    let result: Result<Vec<_>, _> = parser.collect();

    // We expect an error due to the truncated module
    assert!(result.is_err());
}

#[test]
fn test_empty_module() {
    // Just the WebAssembly module header, with no sections
    let empty_module = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

    // The parser should handle an empty module without errors
    let parser = Parser::new(&empty_module);
    let result: Result<Vec<_>, _> = parser.collect();

    assert!(result.is_ok());
    let payloads = result.unwrap();

    // We expect only the Version payload
    assert_eq!(payloads.len(), 1);
}

#[test]
fn test_invalid_wasm_header() {
    // Create an invalid WebAssembly module (wrong magic bytes)
    let invalid_module = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];

    // The parser should reject this as not a valid WebAssembly module
    let parser = Parser::new(&invalid_module);
    let result: Result<Vec<_>, _> = parser.collect();

    // We expect an error due to invalid magic bytes
    assert!(result.is_err());
}

#[test]
fn test_import_section_scanning() {
    let module = create_complex_test_module();

    // Find the import section
    let mut reader = SectionReader::new(&module).unwrap();
    let import_section = reader.find_section(2).unwrap().unwrap(); // 2 is the import section ID

    // Extract the import section data
    let (offset, size) = import_section;
    let import_data = &module[offset..offset + size];

    // Verify the import data
    assert_eq!(import_data[0], 0x01); // Number of imports is 1

    // Check the module name length and value
    assert_eq!(import_data[1], 0x0C); // Module name length (12)
    let module_name = &import_data[2..14];
    assert_eq!(std::str::from_utf8(module_name).unwrap(), "wasi_builtin");

    // Check the field name length and value
    assert_eq!(import_data[14], 0x03); // Field name length (3)
    let field_name = &import_data[15..18];
    assert_eq!(std::str::from_utf8(field_name).unwrap(), "add");
}
