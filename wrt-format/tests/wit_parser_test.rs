#[cfg(any(feature = "std", feature = "alloc"))]
#[cfg(test)]
mod tests {
    use wrt_format::wit_parser::*;

    #[test]
    fn test_wit_parser_creation() {
        let parser = WitParser::new();
        assert_eq!(parser.current_position, 0);
    }

    #[test]
    fn test_wit_type_creation() {
        let bool_type = WitType::Bool;
        let u32_type = WitType::U32;
        let string_type = WitType::String;
        
        // Test that these can be compared
        assert_eq!(bool_type, WitType::Bool);
        assert_ne!(u32_type, WitType::String);
        assert_eq!(string_type, WitType::String);
    }

    #[test]
    fn test_wit_function_creation() {
        let function = WitFunction::default();
        assert!(!function.is_async);
        assert_eq!(function.params.len(), 0);
        assert_eq!(function.results.len(), 0);
    }

    #[test]
    fn test_wit_import_export_creation() {
        let import = WitImport::default();
        let export = WitExport::default();
        
        // Test that they can be created and compared
        assert_eq!(import.name.as_str().unwrap_or(""), "");
        assert_eq!(export.name.as_str().unwrap_or(""), "");
    }

    #[test]
    fn test_basic_type_parsing() {
        let mut parser = WitParser::new();
        
        assert_eq!(parser.parse_type("bool").unwrap(), WitType::Bool);
        assert_eq!(parser.parse_type("u32").unwrap(), WitType::U32);
        assert_eq!(parser.parse_type("string").unwrap(), WitType::String);
        assert_eq!(parser.parse_type("f64").unwrap(), WitType::F64);
    }
}