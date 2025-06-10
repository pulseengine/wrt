//! Comprehensive tests for Component Instantiation and Linking System
//!
//! This module provides extensive test coverage for the WebAssembly Component Model
//! instantiation and linking functionality, including edge cases, error conditions,
//! and cross-environment compatibility.

#[cfg(test)]
mod tests {
    use super::super::canonical_abi::ComponentType;
    use super::super::component_instantiation::*;
    use super::super::component_linker::*;
    use wrt_error::ErrorCategory;

    // ====== COMPONENT INSTANCE TESTS ======

    #[test]
    fn test_instance_creation_with_exports() {
        let config = InstanceConfig::default();
        let exports = vec![
            create_component_export(
                "add".to_string(),
                ExportType::Function(create_function_signature(
                    "add".to_string(),
                    vec![ComponentType::S32, ComponentType::S32],
                    vec![ComponentType::S32],
                )),
            ),
            create_component_export(
                "memory".to_string(),
                ExportType::Memory(MemoryConfig {
                    initial_pages: 2,
                    max_pages: Some(10),
                    protected: true,
                }),
            ),
        ];

        let instance =
            ComponentInstance::new(1, "math_component".to_string(), config, exports, vec![]);

        assert!(instance.is_ok());
        let instance = instance.unwrap();
        assert_eq!(instance.id, 1);
        assert_eq!(instance.name, "math_component");
        assert_eq!(instance.state, InstanceState::Initializing);
        assert_eq!(instance.exports.len(), 2);
    }

    #[test]
    fn test_instance_creation_with_imports() {
        let config = InstanceConfig::default();
        let imports = vec![
            create_component_import(
                "log".to_string(),
                "env".to_string(),
                ImportType::Function(create_function_signature(
                    "log".to_string(),
                    vec![ComponentType::String],
                    vec![],
                )),
            ),
            create_component_import(
                "allocate".to_string(),
                "memory".to_string(),
                ImportType::Function(create_function_signature(
                    "allocate".to_string(),
                    vec![ComponentType::U32],
                    vec![ComponentType::U32],
                )),
            ),
        ];

        let instance = ComponentInstance::new(2, "calculator".to_string(), config, vec![], imports);

        assert!(instance.is_ok());
        let instance = instance.unwrap();
        assert_eq!(instance.id, 2);
        assert_eq!(instance.name, "calculator");
        assert_eq!(instance.imports.len(), 0); // Imports start unresolved
    }

    #[test]
    fn test_instance_initialization() {
        let config = InstanceConfig::default();
        let exports = vec![create_component_export(
            "test_func".to_string(),
            ExportType::Function(create_function_signature(
                "test_func".to_string(),
                vec![ComponentType::Bool],
                vec![ComponentType::S32],
            )),
        )];

        let mut instance =
            ComponentInstance::new(3, "test_component".to_string(), config, exports, vec![])
                .unwrap();

        assert_eq!(instance.state, InstanceState::Initializing);

        let result = instance.initialize();
        assert!(result.is_ok());
        assert_eq!(instance.state, InstanceState::Ready);
    }

    #[test]
    fn test_instance_function_call() {
        let config = InstanceConfig::default();
        let exports = vec![create_component_export(
            "test_func".to_string(),
            ExportType::Function(create_function_signature(
                "test_func".to_string(),
                vec![ComponentType::S32],
                vec![ComponentType::S32],
            )),
        )];

        let mut instance =
            ComponentInstance::new(4, "test_component".to_string(), config, exports, vec![])
                .unwrap();

        instance.initialize().unwrap();

        let args = vec![ComponentValue::S32(42)];
        let result = instance.call_function("test_func", &args);

        assert!(result.is_ok());
        let return_values = result.unwrap();
        assert_eq!(return_values.len(), 1);
    }

    #[test]
    fn test_instance_function_call_invalid_state() {
        let config = InstanceConfig::default();
        let exports = vec![create_component_export(
            "test_func".to_string(),
            ExportType::Function(create_function_signature(
                "test_func".to_string(),
                vec![],
                vec![ComponentType::S32],
            )),
        )];

        let mut instance =
            ComponentInstance::new(5, "test_component".to_string(), config, exports, vec![])
                .unwrap();

        // Don't initialize - should fail
        let result = instance.call_function("test_func", &[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Runtime);
    }

    #[test]
    fn test_instance_function_call_not_found() {
        let config = InstanceConfig::default();
        let mut instance =
            ComponentInstance::new(6, "test_component".to_string(), config, vec![], vec![])
                .unwrap();

        instance.initialize().unwrap();

        let result = instance.call_function("nonexistent", &[]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Runtime);
    }

    #[test]
    fn test_instance_memory_operations() {
        let config = InstanceConfig {
            memory_config: MemoryConfig { initial_pages: 2, max_pages: Some(5), protected: true },
            ..Default::default()
        };

        let instance =
            ComponentInstance::new(7, "memory_test".to_string(), config, vec![], vec![]).unwrap();

        let memory = instance.get_memory();
        assert!(memory.is_some());
        let memory = memory.unwrap();
        assert_eq!(memory.size_pages(), 2);
        assert_eq!(memory.current_size, 2 * 65536);
    }

    #[test]
    fn test_instance_termination() {
        let config = InstanceConfig::default();
        let mut instance =
            ComponentInstance::new(8, "test_component".to_string(), config, vec![], vec![])
                .unwrap();

        instance.initialize().unwrap();
        assert_eq!(instance.state, InstanceState::Ready);

        instance.terminate();
        assert_eq!(instance.state, InstanceState::Terminated);
    }

    // ====== MEMORY TESTS ======

    #[test]
    fn test_memory_creation_and_growth() {
        let config = MemoryConfig { initial_pages: 1, max_pages: Some(10), protected: true };

        let mut memory = ComponentMemory::new(0, config).unwrap();
        assert_eq!(memory.size_pages(), 1);
        assert_eq!(memory.current_size, 65536);

        let old_pages = memory.grow(3).unwrap();
        assert_eq!(old_pages, 1);
        assert_eq!(memory.size_pages(), 4);
        assert_eq!(memory.current_size, 4 * 65536);
    }

    #[test]
    fn test_memory_growth_limit() {
        let config = MemoryConfig { initial_pages: 1, max_pages: Some(3), protected: true };

        let mut memory = ComponentMemory::new(0, config).unwrap();

        // Try to grow beyond maximum
        let result = memory.grow(5);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Runtime);
    }

    #[test]
    fn test_memory_read_write_operations() {
        let config = MemoryConfig { initial_pages: 1, max_pages: Some(2), protected: true };

        let mut memory = ComponentMemory::new(0, config).unwrap();

        // Test basic read/write
        let test_data = vec![1, 2, 3, 4, 5];
        memory.write_bytes(100, &test_data).unwrap();

        let read_data = memory.read_bytes(100, 5).unwrap();
        assert_eq!(read_data, test_data);

        // Test individual byte operations
        memory.write_u8(200, 42).unwrap();
        assert_eq!(memory.read_u8(200).unwrap(), 42);

        // Test multi-byte operations
        memory.write_u32_le(300, 0x12345678).unwrap();
        assert_eq!(memory.read_u32_le(300).unwrap(), 0x12345678);
    }

    #[test]
    fn test_memory_bounds_checking() {
        let config = MemoryConfig { initial_pages: 1, max_pages: Some(1), protected: true };

        let memory = ComponentMemory::new(0, config).unwrap();

        // Try to read beyond bounds
        let result = memory.read_bytes(65535, 2);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Memory);

        // Try to write beyond bounds
        let mut memory = memory;
        let result = memory.write_bytes(65530, &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Memory);
    }

    // ====== COMPONENT LINKER TESTS ======

    #[test]
    fn test_linker_creation() {
        let linker = ComponentLinker::new();
        assert_eq!(linker.get_stats().components_registered, 0);
        assert_eq!(linker.get_stats().instances_created, 0);
    }

    #[test]
    fn test_linker_add_remove_components() {
        let mut linker = ComponentLinker::new();
        let binary = create_test_component_binary();

        // Add component
        let result = linker.add_component("test_component".to_string(), &binary);
        assert!(result.is_ok());
        assert_eq!(linker.get_stats().components_registered, 1);

        // Remove component
        let result = linker.remove_component(&"test_component".to_string());
        assert!(result.is_ok());
        assert_eq!(linker.get_stats().components_registered, 1); // Stats don't decrease
    }

    #[test]
    fn test_linker_component_instantiation() {
        let mut linker = ComponentLinker::new();
        let binary = create_test_component_binary();

        linker.add_component("test_component".to_string(), &binary).unwrap();

        let instance_id = linker.instantiate(&"test_component".to_string(), None);
        assert!(instance_id.is_ok());

        let instance_id = instance_id.unwrap();
        assert!(linker.get_instance(instance_id).is_some());
        assert_eq!(linker.get_stats().instances_created, 1);
    }

    #[test]
    fn test_linker_component_not_found() {
        let mut linker = ComponentLinker::new();

        let result = linker.instantiate(&"nonexistent".to_string(), None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Runtime);
    }

    #[test]
    fn test_linker_link_all_components() {
        let mut linker = ComponentLinker::new();
        let binary1 = create_test_component_binary();
        let binary2 = create_test_component_binary();

        linker.add_component("component1".to_string(), &binary1).unwrap();
        linker.add_component("component2".to_string(), &binary2).unwrap();

        let instance_ids = linker.link_all();
        assert!(instance_ids.is_ok());

        let instance_ids = instance_ids.unwrap();
        assert_eq!(instance_ids.len(), 2);
        assert_eq!(linker.get_stats().instances_created, 2);
    }

    #[test]
    fn test_linker_dependency_graph() {
        let mut linker = ComponentLinker::new();
        let binary = create_test_component_binary();

        // Add multiple components
        linker.add_component("base".to_string(), &binary).unwrap();
        linker.add_component("middle".to_string(), &binary).unwrap();
        linker.add_component("top".to_string(), &binary).unwrap();

        // Test topological sort
        let sorted = linker.link_graph.topological_sort();
        assert!(sorted.is_ok());

        let sorted = sorted.unwrap();
        assert_eq!(sorted.len(), 3);
        assert!(sorted.contains(&"base".to_string()));
        assert!(sorted.contains(&"middle".to_string()));
        assert!(sorted.contains(&"top".to_string()));
    }

    #[test]
    fn test_linker_max_components() {
        let mut linker = ComponentLinker::new();
        let binary = create_test_component_binary();

        // Try to add too many components
        for i in 0..MAX_LINKED_COMPONENTS {
            let result = linker.add_component("Component not found", &binary);
            assert!(result.is_ok());
        }

        // This should fail
        let result = linker.add_component("overflow".to_string(), &binary);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Resource);
    }

    // ====== FUNCTION SIGNATURE TESTS ======

    #[test]
    fn test_function_signature_creation() {
        let sig = create_function_signature(
            "complex_function".to_string(),
            vec![
                ComponentType::S32,
                ComponentType::String,
                ComponentType::Bool,
                ComponentType::F64,
            ],
            vec![ComponentType::Option(Box::new(ComponentType::S32)), ComponentType::String],
        );

        assert_eq!(sig.name, "complex_function");
        assert_eq!(sig.params.len(), 4);
        assert_eq!(sig.returns.len(), 2);

        match &sig.params[0] {
            ComponentType::S32 => {}
            _ => panic!("Expected S32 parameter"),
        }

        match &sig.returns[0] {
            ComponentType::Option(_) => {}
            _ => panic!("Expected Option return type"),
        }
    }

    #[test]
    fn test_export_type_variants() {
        let function_export = create_component_export(
            "func".to_string(),
            ExportType::Function(create_function_signature(
                "func".to_string(),
                vec![ComponentType::S32],
                vec![ComponentType::S32],
            )),
        );
        assert_eq!(function_export.name, "func");

        let memory_export =
            create_component_export("mem".to_string(), ExportType::Memory(MemoryConfig::default()));
        assert_eq!(memory_export.name, "mem");

        let table_export = create_component_export(
            "table".to_string(),
            ExportType::Table { element_type: ComponentType::S32, size: 100 },
        );
        assert_eq!(table_export.name, "table");

        let global_export = create_component_export(
            "global".to_string(),
            ExportType::Global { value_type: ComponentType::F64, mutable: true },
        );
        assert_eq!(global_export.name, "global");
    }

    #[test]
    fn test_import_type_variants() {
        let function_import = create_component_import(
            "ext_func".to_string(),
            "external".to_string(),
            ImportType::Function(create_function_signature(
                "ext_func".to_string(),
                vec![ComponentType::String],
                vec![ComponentType::Bool],
            )),
        );
        assert_eq!(function_import.name, "ext_func");
        assert_eq!(function_import.module, "external");

        let memory_import = create_component_import(
            "ext_memory".to_string(),
            "external".to_string(),
            ImportType::Memory(MemoryConfig::default()),
        );
        assert_eq!(memory_import.name, "ext_memory");

        let table_import = create_component_import(
            "ext_table".to_string(),
            "external".to_string(),
            ImportType::Table {
                element_type: ComponentType::U32,
                min_size: 10,
                max_size: Some(100),
            },
        );
        assert_eq!(table_import.name, "ext_table");

        let global_import = create_component_import(
            "ext_global".to_string(),
            "external".to_string(),
            ImportType::Global { value_type: ComponentType::F32, mutable: false },
        );
        assert_eq!(global_import.name, "ext_global");
    }

    // ====== CROSS-ENVIRONMENT COMPATIBILITY TESTS ======

    #[cfg(feature = "std")]
    #[test]
    fn test_std_environment_compatibility() {
        let mut linker = ComponentLinker::new();
        let binary = create_test_component_binary();

        // Should work in std environment
        linker.add_component("std_test".to_string(), &binary).unwrap();
        let instance_id = linker.instantiate(&"std_test".to_string(), None).unwrap();
        assert!(linker.get_instance(instance_id).is_some());
    }

    #[cfg(all(not(feature = "std")))]
    #[test]
    fn test_alloc_environment_compatibility() {
        let mut linker = ComponentLinker::new();
        let binary = create_test_component_binary();

        // Binary std/no_std choice
        linker.add_component("alloc_test".to_string(), &binary).unwrap();
        let instance_id = linker.instantiate(&"alloc_test".to_string(), None).unwrap();
        assert!(linker.get_instance(instance_id).is_some());
    }

    #[cfg(not(any(feature = "std", )))]
    #[test]
    fn test_no_std_environment_compatibility() {
        // In pure no_std, we can at least create configurations and validate types
        let config = InstanceConfig::default();
        assert_eq!(config.max_memory_size, 64 * 1024 * 1024);

        let memory_config = MemoryConfig::default();
        assert_eq!(memory_config.initial_pages, 1);
        assert!(memory_config.protected);
    }

    // ====== EDGE CASES AND ERROR CONDITIONS ======

    #[test]
    fn test_instance_creation_empty_name() {
        let config = InstanceConfig::default();
        let result = ComponentInstance::new(
            1,
            "".to_string(), // Empty name
            config,
            vec![],
            vec![],
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Validation);
    }

    #[test]
    fn test_instance_creation_too_many_exports() {
        let config = InstanceConfig::default();
        let mut exports = Vec::new();

        // Create more exports than allowed
        for i in 0..MAX_EXPORTS_PER_COMPONENT + 1 {
            exports.push(create_component_export(
                "Component not found",
                ExportType::Function(create_function_signature(
                    "Component not found",
                    vec![],
                    vec![ComponentType::S32],
                )),
            ));
        }

        let result = ComponentInstance::new(1, "test".to_string(), config, exports, vec![]);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Validation);
    }

    #[test]
    fn test_instance_creation_too_many_imports() {
        let config = InstanceConfig::default();
        let mut imports = Vec::new();

        // Create more imports than allowed
        for i in 0..MAX_IMPORTS_PER_COMPONENT + 1 {
            imports.push(create_component_import(
                "Component not found",
                "env".to_string(),
                ImportType::Function(create_function_signature(
                    "Component not found",
                    vec![],
                    vec![ComponentType::S32],
                )),
            ));
        }

        let result = ComponentInstance::new(1, "test".to_string(), config, vec![], imports);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Validation);
    }

    #[test]
    fn test_memory_creation_invalid_config() {
        let config = MemoryConfig {
            initial_pages: 10,
            max_pages: Some(5), // Initial > max
            protected: true,
        };

        let result = ComponentMemory::new(0, config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Validation);
    }

    #[test]
    fn test_linker_empty_binary() {
        let mut linker = ComponentLinker::new();
        let result = linker.add_component("empty".to_string(), &[]);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Validation);
    }

    #[test]
    fn test_linker_remove_nonexistent_component() {
        let mut linker = ComponentLinker::new();
        let result = linker.remove_component(&"nonexistent".to_string());

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().category(), ErrorCategory::Runtime);
    }

    // ====== PERFORMANCE AND STRESS TESTS ======

    #[test]
    fn test_large_component_creation() {
        let config = InstanceConfig::default();
        let mut exports = Vec::new();

        // Create many exports (but within limits)
        for i in 0..100 {
            exports.push(create_component_export(
                "Component not found",
                ExportType::Function(create_function_signature(
                    "Component not found",
                    vec![ComponentType::S32],
                    vec![ComponentType::S32],
                )),
            ));
        }

        let instance =
            ComponentInstance::new(1, "large_component".to_string(), config, exports, vec![]);

        assert!(instance.is_ok());
        let instance = instance.unwrap();
        assert_eq!(instance.exports.len(), 100);
    }

    #[test]
    fn test_multiple_instance_creation() {
        let mut linker = ComponentLinker::new();
        let binary = create_test_component_binary();

        linker.add_component("base".to_string(), &binary).unwrap();

        // Create multiple instances of the same component
        let mut instance_ids = Vec::new();
        for _ in 0..10 {
            let id = linker.instantiate(&"base".to_string(), None).unwrap();
            instance_ids.push(id);
        }

        assert_eq!(instance_ids.len(), 10);
        assert_eq!(linker.get_stats().instances_created, 10);

        // Verify all instances exist
        for id in instance_ids {
            assert!(linker.get_instance(id).is_some());
        }
    }

    // ====== HELPER FUNCTIONS ======

    fn create_test_component_binary() -> Vec<u8> {
        // Create a minimal valid WebAssembly binary for testing
        vec![
            0x00, 0x61, 0x73, 0x6d, // Magic number "wasm"
            0x01, 0x00, 0x00,
            0x00, // Version 1
                  // Minimal sections would go here in a real implementation
        ]
    }

    #[test]
    fn test_resolved_import_creation() {
        let import = create_component_import(
            "test_func".to_string(),
            "env".to_string(),
            ImportType::Function(create_function_signature(
                "test_func".to_string(),
                vec![ComponentType::S32],
                vec![ComponentType::Bool],
            )),
        );

        let resolved = ResolvedImport {
            import: import.clone(),
            provider_id: 42,
            provider_export: "exported_func".to_string(),
        };

        assert_eq!(resolved.import.name, "test_func");
        assert_eq!(resolved.import.module, "env");
        assert_eq!(resolved.provider_id, 42);
        assert_eq!(resolved.provider_export, "exported_func");
    }

    #[test]
    fn test_instance_state_transitions() {
        let config = InstanceConfig::default();
        let mut instance =
            ComponentInstance::new(1, "state_test".to_string(), config, vec![], vec![]).unwrap();

        // Initial state
        assert_eq!(instance.state, InstanceState::Initializing);

        // Initialize
        instance.initialize().unwrap();
        assert_eq!(instance.state, InstanceState::Ready);

        // Terminate
        instance.terminate();
        assert_eq!(instance.state, InstanceState::Terminated);
    }

    #[test]
    fn test_component_metadata() {
        let metadata = ComponentMetadata {
            name: "test_component".to_string(),
            version: "2.1.0".to_string(),
            description: "A test component for validation".to_string(),
            author: "Test Author".to_string(),
            compiled_at: 1640995200, // 2022-01-01 00:00:00 UTC
        };

        assert_eq!(metadata.name, "test_component");
        assert_eq!(metadata.version, "2.1.0");
        assert_eq!(metadata.description, "A test component for validation");
        assert_eq!(metadata.author, "Test Author");
        assert_eq!(metadata.compiled_at, 1640995200);
    }

    #[test]
    fn test_linker_configuration() {
        let config = LinkerConfig {
            strict_typing: false,
            allow_hot_swap: true,
            max_instance_memory: 128 * 1024 * 1024,
            validate_dependencies: false,
            circular_dependency_mode: CircularDependencyMode::Allow,
        };

        let linker = ComponentLinker::with_config(config.clone());
        assert!(!linker.config.strict_typing);
        assert!(linker.config.allow_hot_swap);
        assert_eq!(linker.config.max_instance_memory, 128 * 1024 * 1024);
        assert!(!linker.config.validate_dependencies);
        assert_eq!(linker.config.circular_dependency_mode, CircularDependencyMode::Allow);
    }
}
