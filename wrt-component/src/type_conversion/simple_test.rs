#[cfg(test)]
mod simple_tests {
    use wrt_format::component::ValType<NoStdProvider<65536>> as FormatValType<NoStdProvider<65536>>;
    use wrt_foundation::component_value::ValType<NoStdProvider<65536>> as TypesValType<NoStdProvider<65536>>;

    use super::super::registry::TypeConversionRegistry;

    #[test]
    fn test_simple_format_to_types_conversion() {
        let registry = TypeConversionRegistry::with_defaults();

        // Test primitive types
        let bool_type = FormatValType<NoStdProvider<65536>>::Bool;
        let result = registry.convert::<FormatValType<NoStdProvider<65536>>, TypesValType<NoStdProvider<65536>>(&bool_type).unwrap();
        assert!(matches!(result, TypesValType<NoStdProvider<65536>>::Bool));

        let s32_type = FormatValType<NoStdProvider<65536>>::S32;
        let result = registry.convert::<FormatValType<NoStdProvider<65536>>, TypesValType<NoStdProvider<65536>>(&s32_type).unwrap();
        assert!(matches!(result, TypesValType<NoStdProvider<65536>>::S32));
    }

    #[test]
    fn test_simple_types_to_format_conversion() {
        let registry = TypeConversionRegistry::with_defaults();

        // Test primitive types
        let bool_type = TypesValType<NoStdProvider<65536>>::Bool;
        let result = registry.convert::<TypesValType<NoStdProvider<65536>>, FormatValType<NoStdProvider<65536>>(&bool_type).unwrap();
        assert!(matches!(result, FormatValType<NoStdProvider<65536>>::Bool));

        let s32_type = TypesValType<NoStdProvider<65536>>::S32;
        let result = registry.convert::<TypesValType<NoStdProvider<65536>>, FormatValType<NoStdProvider<65536>>(&s32_type).unwrap();
        assert!(matches!(result, FormatValType<NoStdProvider<65536>>::S32));
    }
}
