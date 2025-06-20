use wrt_parser::{parse_wasm, validate_header, ValueType};

#[test]
fn test_value_type_parsing() {
    assert_eq!(ValueType::from_byte(0x7F).unwrap(), ValueType::I32);
    assert_eq!(ValueType::from_byte(0x7E).unwrap(), ValueType::I64);
    assert_eq!(ValueType::from_byte(0x7D).unwrap(), ValueType::F32);
    assert_eq!(ValueType::from_byte(0x7C).unwrap(), ValueType::F64);
    assert_eq!(ValueType::from_byte(0x70).unwrap(), ValueType::FuncRef);
    assert_eq!(ValueType::from_byte(0x6F).unwrap(), ValueType::ExternRef);
}

#[test]
fn test_value_type_to_byte() {
    assert_eq!(ValueType::I32.to_byte(), 0x7F);
    assert_eq!(ValueType::I64.to_byte(), 0x7E);
    assert_eq!(ValueType::F32.to_byte(), 0x7D);
    assert_eq!(ValueType::F64.to_byte(), 0x7C);
    assert_eq!(ValueType::FuncRef.to_byte(), 0x70);
    assert_eq!(ValueType::ExternRef.to_byte(), 0x6F);
}

#[test]
fn test_header_validation() {
    // Valid WebAssembly header
    let valid_header = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
    assert!(validate_header(&valid_header).is_ok());
    
    // Invalid magic
    let invalid_magic = [0x00, 0x61, 0x73, 0x6E, 0x01, 0x00, 0x00, 0x00];
    assert!(validate_header(&invalid_magic).is_err());
    
    // Invalid version
    let invalid_version = [0x00, 0x61, 0x73, 0x6D, 0x02, 0x00, 0x00, 0x00];
    assert!(validate_header(&invalid_version).is_err());
    
    // Too short
    let too_short = [0x00, 0x61, 0x73];
    assert!(validate_header(&too_short).is_err());
}

#[test]
fn test_basic_wasm_parsing() {
    // Minimal valid WebAssembly module (just header)
    let minimal_wasm = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];
    let module = parse_wasm(&minimal_wasm).unwrap();
    
    // For now, just verify the module was created
    assert_eq!(module.functions.len(), 0);
    assert_eq!(module.types.len(), 0);
}