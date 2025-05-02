use wrt_decoder::parser::{ImportSectionReader, Parser, Payload};
use wrt_format::module::ImportDesc;
use wrt_format::types::ValueType;

// Create a minimal valid WebAssembly module with an import section
fn create_test_module() -> Vec<u8> {
    // WebAssembly header
    let mut module = vec![
        0x00, 0x61, 0x73, 0x6D, // magic ("\0asm")
        0x01, 0x00, 0x00, 0x00, // version 1
    ];

    // Import section (id = 2)
    module.push(0x02); // section id

    // Import section contents
    let mut section_contents = Vec::new();
    section_contents.push(0x01); // 1 import

    // Import: wasi_builtin.random
    // Module name: "wasi_builtin"
    section_contents.push(0x0B); // name length
    section_contents.extend_from_slice(b"wasi_builtin");

    // Field name: "random"
    section_contents.push(0x06); // name length
    section_contents.extend_from_slice(b"random");

    // Import kind: function
    section_contents.push(0x00);

    // Function type index
    section_contents.push(0x00);

    // Write section size
    module.push(section_contents.len() as u8); // section size (simple LEB128 encoding)

    // Add section contents
    module.extend_from_slice(&section_contents);

    module
}

#[test]
fn test_import_section_reader() {
    let module_bytes = create_test_module();

    // Parse the module
    let parser = Parser::new(&module_bytes);
    let payloads: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // Should have 3 payloads: Version, ImportSection, End
    assert_eq!(payloads.len(), 3);

    // Extract import section data
    let import_section_data = match &payloads[1] {
        Payload::ImportSection(data, _) => data,
        _ => panic!("Expected ImportSection payload"),
    };

    // Create import section reader
    let import_reader = ImportSectionReader::new(import_section_data).unwrap();
    let imports: Vec<_> = import_reader.collect::<Result<Vec<_>, _>>().unwrap();

    // Verify import data
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].module, "wasi_builtin");
    assert_eq!(imports[0].name, "random");

    match &imports[0].desc {
        ImportDesc::Function(type_idx) => assert_eq!(*type_idx, 0),
        _ => panic!("Expected Function import"),
    }
}
