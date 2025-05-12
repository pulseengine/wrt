use wrt::{
    module::Module,
    types::{ExternType, ValueType},
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test parsing of imports section from a WebAssembly binary
    #[test]
    fn test_import_parsing() {
        // A simple WebAssembly module with various types of imports
        let module_bytes = wat::parse_str(
            r#"
            (module
              (import "env" "func" (func (param i32) (result i32)))
              (import "env" "table" (table 10 funcref))
              (import "env" "memory" (memory 1))
              (import "env" "global" (global i32))
            )
            "#,
        )
        .unwrap();

        // Parse the module
        let module = Module::from_bytes(&module_bytes).unwrap();

        // Verify imports were parsed correctly
        assert_eq!(module.imports.len(), 4, "Expected 4 imports");
    }

    /// Test parsing of the element section from a WebAssembly binary
    #[test]
    fn test_element_parsing() {
        // A WebAssembly module with an element section
        let module_bytes = wat::parse_str(
            r#"
            (module
              (table 1 funcref)
              (func $f1 (result i32) (i32.const 42))
              (func $f2 (result i32) (i32.const 43))
              (elem (i32.const 0) $f1 $f2)
            )
            "#,
        )
        .unwrap();

        // Parse the module
        let module = Module::from_bytes(&module_bytes).unwrap();

        // Verify elements were parsed correctly
        assert_eq!(module.elements.len(), 1, "Expected 1 element segment");
    }

    /// Test parsing of the data section from a WebAssembly binary
    #[test]
    fn test_data_parsing() {
        // A WebAssembly module with a data section
        let module_bytes = wat::parse_str(
            r#"
            (module
              (memory 1)
              (data (i32.const 0) "Hello, World!")
            )
            "#,
        )
        .unwrap();

        // Parse the module
        let module = Module::from_bytes(&module_bytes).unwrap();

        // Verify data segments were parsed correctly
        assert_eq!(module.data.len(), 1, "Expected 1 data segment");
    }
}
