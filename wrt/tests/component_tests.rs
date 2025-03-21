use wrt::{
    CanonicalABI, Component, Error, InterfaceValue, ResourceTable, Result, ValueType,
};
use wrt::types::*;
use wrt::resource::ResourceRepresentation;
use wrt::resource::ResourceType;
use wrt::module::Module;

use std::sync::Arc;

#[test]
fn test_component_instantiation() -> Result<()> {
    // Create a simple component with a basic component type
    let mut component_type = wrt::component::ComponentType {
        imports: Vec::new(),
        exports: vec![
            (
                "add".to_string(),
                ExternType::Function(FuncType {
                    params: vec![ValueType::I32, ValueType::I32],
                    results: vec![ValueType::I32],
                }),
            ),
            (
                "hello".to_string(),
                ExternType::Function(FuncType {
                    params: Vec::new(),
                    results: vec![ValueType::I32],
                }),
            ),
        ],
        instances: Vec::new(),
    };

    let mut component = Component::new(component_type);
    component.instantiate(Vec::new())?;

    // Check that the exports are accessible
    let add_export = component.get_export("add")?;
    assert_eq!(add_export.name, "add");
    match &add_export.ty {
        ExternType::Function(func_type) => {
            assert_eq!(func_type.params.len(), 2);
            assert_eq!(func_type.results.len(), 1);
        }
        _ => return Err(Error::Validation("Expected function type".into())),
    };

    let hello_export = component.get_export("hello")?;
    assert_eq!(hello_export.name, "hello");
    match &hello_export.ty {
        ExternType::Function(func_type) => {
            assert_eq!(func_type.params.len(), 0);
            assert_eq!(func_type.results.len(), 1);
        }
        _ => return Err(Error::Validation("Expected function type".into())),
    };

    Ok(())
}

#[test]
fn test_canonical_abi_conversion() -> Result<()> {
    // Test lifting/lowering of primitive values
    let i32_val = wrt::Value::I32(42);
    let i32_type = ComponentType::Primitive(ValueType::I32);
    let interface_val = CanonicalABI::lift(i32_val.clone(), &i32_type, None, None)?;
    
    assert!(matches!(interface_val, InterfaceValue::S32(42)));
    
    let lowered_val = CanonicalABI::lower(interface_val, None, None)?;
    assert_eq!(lowered_val, i32_val);
    
    Ok(())
}

#[test]
fn test_resource_handling() -> Result<()> {
    // Create a resource table
    let mut table = ResourceTable::new();
    
    // Define a resource type
    let resource_type = ResourceType {
        name: "test:resource".to_string(),
        representation: ResourceRepresentation::Handle32,
        nullable: false,
        borrowable: true,
    };
    
    // Create some test resource data
    struct TestResourceData {
        value: String,
    }
    
    impl wrt::ResourceData for TestResourceData {
        // ResourceData requires Debug + Send + Sync and as_any
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    
    impl std::fmt::Debug for TestResourceData {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "TestResourceData {{ value: {} }}", self.value)
        }
    }
    
    // Allocate a resource
    let data = Arc::new(TestResourceData {
        value: "test value".to_string(),
    });
    
    let id = table.allocate(resource_type.clone(), data);
    
    // Get the resource back
    let resource = table.get(id)?;
    
    // Check the resource
    assert_eq!(resource.id, id);
    assert_eq!(resource.resource_type.name, "test:resource");
    
    // Add some references
    table.add_ref(id)?;
    table.add_ref(id)?;
    
    // Drop references
    table.drop_ref(id)?;
    table.drop_ref(id)?;
    
    // Resource should still exist
    assert!(table.get(id).is_ok());
    
    // Drop the last reference
    table.drop_ref(id)?;
    
    // Resource should no longer exist
    assert!(table.get(id).is_err());
    
    Ok(())
}

#[test]
fn test_component_types() {
    // Create nested component types
    
    // 1. Record type
    let record_type = ComponentType::Record(vec![
        ("name".to_string(), Box::new(ComponentType::Primitive(ValueType::I32))),
        ("age".to_string(), Box::new(ComponentType::Primitive(ValueType::I32))),
    ]);
    
    assert!(record_type.is_record());
    
    // 2. List of records
    let list_type = ComponentType::List(Box::new(record_type.clone()));
    
    assert!(list_type.is_list());
    
    // 3. Result type with record as ok
    let result_type = ComponentType::Result {
        ok: Some(Box::new(record_type)),
        err: Some(Box::new(ComponentType::Primitive(ValueType::I32))),
    };
    
    assert!(result_type.is_result());
    
    // 4. Option type with list as value
    let option_type = ComponentType::Option(Box::new(list_type));
    
    assert!(option_type.is_option());
}

#[test]
fn test_component_binary_parsing() -> Result<()> {
    // Create a minimal component binary with just the header and mandatory component-type-section
    let component_binary = [
        // Component magic number and version
        0x00, 0x61, 0x73, 0x6D, // magic
        0x0D, 0x00, 0x01, 0x00, // component model version

        // Component Type Section (section code 1)
        0x01, // section code
        0x02, // section size
        0x00, // number of types (0)
        0x00, // padding byte to meet minimum length
    ];
    
    // Create a module and load the component binary
    let module = wrt::module::Module::new();
    let loaded_module = module.load_from_binary(&component_binary)?;
    
    // Verify that the module contains component-model-info section
    let component_info = loaded_module.custom_sections.iter()
        .find(|section| section.name == "component-model-info")
        .expect("Component model info section not found");
    
    // Verify it's marked as a component
    assert_eq!(component_info.data, vec![0x01]);
    
    Ok(())
}

#[test]
fn test_component_validation() -> Result<()> {
    // Create an invalid component binary (incorrect magic number)
    let invalid_binary = [
        // Invalid magic
        0x00, 0x61, 0x73, 0x00, // wrong magic
        0x0D, 0x00, 0x01, 0x00, // component model version
    ];
    
    // Loading should fail
    let module = wrt::module::Module::new();
    let result = module.load_from_binary(&invalid_binary);
    assert!(result.is_err());
    
    // Create an invalid component binary (no core module or type section)
    let invalid_binary = [
        // Component magic number and version
        0x00, 0x61, 0x73, 0x6D, // magic
        0x0D, 0x00, 0x01, 0x00, // component model version
        
        // Just a custom section
        0x00, // section code
        0x02, // section size
        0x00, // name length
        0x00, // empty data
    ];
    
    // Loading should fail
    let result = module.load_from_binary(&invalid_binary);
    assert!(result.is_err());
    
    Ok(())
} 