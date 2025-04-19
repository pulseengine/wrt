//! Tests for WebAssembly Component Model value encoding and decoding

use wrt_decoder::component::{decode_component, encode_component};
use wrt_error::Result;
use wrt_format::component::{ValType, Value};

/// Create a simple component with values of different types
fn create_test_component() -> Result<wrt_format::component::Component> {
    let mut component = wrt_format::component::Component::new();

    // Add test values
    component.values = vec![
        // Boolean value (true)
        Value {
            ty: ValType::Bool,
            data: vec![0x01],
        },
        // Boolean value (false)
        Value {
            ty: ValType::Bool,
            data: vec![0x00],
        },
        // S32 value (42)
        Value {
            ty: ValType::S32,
            data: vec![0x2A, 0x00, 0x00, 0x00],
        },
        // String value ("Hello, WebAssembly!")
        Value {
            ty: ValType::String,
            data: "Hello, WebAssembly!".as_bytes().to_vec(),
        },
        // List of S32 values
        Value {
            ty: ValType::List(Box::new(ValType::S32)),
            data: {
                // List with 3 items: [1, 2, 3]
                let mut data = vec![];

                // List length (3)
                data.extend_from_slice(&[0x03, 0x00, 0x00, 0x00]);

                // Item 1: 1
                data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
                // Item 2: 2
                data.extend_from_slice(&[0x02, 0x00, 0x00, 0x00]);
                // Item 3: 3
                data.extend_from_slice(&[0x03, 0x00, 0x00, 0x00]);

                data
            },
        },
        // Option type (Some(42))
        Value {
            ty: ValType::Option(Box::new(ValType::S32)),
            data: {
                // Some variant (tag = 1)
                let mut data = vec![0x01];
                // Value (42)
                data.extend_from_slice(&[0x2A, 0x00, 0x00, 0x00]);
                data
            },
        },
        // Option type (None)
        Value {
            ty: ValType::Option(Box::new(ValType::S32)),
            data: vec![0x00], // None variant (tag = 0)
        },
        // Result type (Ok(42))
        Value {
            ty: ValType::Result(Box::new(ValType::S32)),
            data: {
                // Ok variant (tag = 0)
                let mut data = vec![0x00];
                // Value (42)
                data.extend_from_slice(&[0x2A, 0x00, 0x00, 0x00]);
                data
            },
        },
        // Result type (Error)
        Value {
            ty: ValType::Result(Box::new(ValType::S32)),
            data: vec![0x01], // Error variant (tag = 1)
        },
    ];

    Ok(component)
}

#[test]
fn test_component_value_encoding_decoding() -> Result<()> {
    // Create a test component with various values
    let component = create_test_component()?;

    // Encode the component to binary
    let binary = encode_component(&component)?;

    // Decode the binary back to a component
    let decoded = decode_component(&binary)?;

    // Verify the values were preserved
    assert_eq!(component.values.len(), decoded.values.len());

    for (i, (original, decoded)) in component
        .values
        .iter()
        .zip(decoded.values.iter())
        .enumerate()
    {
        assert_eq!(
            original.ty, decoded.ty,
            "Value type mismatch at index {}",
            i
        );
        assert_eq!(
            original.data, decoded.data,
            "Value data mismatch at index {}",
            i
        );
    }

    Ok(())
}
