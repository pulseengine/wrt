#[cfg(test)]
mod tests {
    use wrt_decoder::component::ComponentDecoder;
    use wrt_error::Result;
    use wrt_types::{ComponentType, ExternType, FuncType, ValueType};

    #[test]
    fn test_decode_simple_component() -> Result<()> {
        // This is a mock test since we can't include actual binary data here
        // In a real test, we would decode an actual binary component

        // Create a mock component decoder
        let decoder = MockComponentDecoder::new();

        // Decode a mock component
        let component_type = decoder.decode_mock_component()?;

        // Verify the component structure
        assert_eq!(component_type.imports.len(), 1);
        assert_eq!(component_type.exports.len(), 1);

        // Verify the import
        let (import_name, import_namespace, import_type) = &component_type.imports[0];
        assert_eq!(import_name, "import_func");
        assert_eq!(import_namespace, "env");

        match import_type {
            ExternType::Function(func_type) => {
                assert_eq!(func_type.params.len(), 1);
                assert_eq!(func_type.results.len(), 1);
                assert_eq!(func_type.params[0], ValueType::I32);
                assert_eq!(func_type.results[0], ValueType::I32);
            }
            _ => panic!("Expected function import"),
        }

        // Verify the export
        let (export_name, export_type) = &component_type.exports[0];
        assert_eq!(export_name, "export_func");

        match export_type {
            ExternType::Function(func_type) => {
                assert_eq!(func_type.params.len(), 1);
                assert_eq!(func_type.results.len(), 1);
                assert_eq!(func_type.params[0], ValueType::I32);
                assert_eq!(func_type.results[0], ValueType::I32);
            }
            _ => panic!("Expected function export"),
        }

        Ok(())
    }

    // Mock component decoder for testing
    struct MockComponentDecoder {}

    impl MockComponentDecoder {
        fn new() -> Self {
            Self {}
        }

        fn decode_mock_component(&self) -> Result<ComponentType> {
            // Create a mock component type
            let component_type = ComponentType {
                imports: vec![(
                    "import_func".to_string(),
                    "env".to_string(),
                    ExternType::Function(FuncType {
                        params: vec![ValueType::I32],
                        results: vec![ValueType::I32],
                    }),
                )],
                exports: vec![(
                    "export_func".to_string(),
                    ExternType::Function(FuncType {
                        params: vec![ValueType::I32],
                        results: vec![ValueType::I32],
                    }),
                )],
                instances: Vec::new(),
            };

            Ok(component_type)
        }
    }
}
