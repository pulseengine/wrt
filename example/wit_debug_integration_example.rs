//! Example demonstrating WIT debugging integration
//!
//! This example shows how to use the WIT-aware debugger for component-level debugging.

#[cfg(any(feature = "std", feature = "alloc"))]
fn main() {
    println!("WIT Debug Integration Example");
    println!("=============================");
    
    // Note: This example demonstrates the API design but cannot run without
    // a full runtime integration. In a real scenario, this would be integrated
    // with the WRT runtime engine.
    
    #[cfg(feature = "wit-integration")]
    {
        use wrt_debug::{
            WitDebugger, ComponentMetadata, FunctionMetadata, TypeMetadata,
            ComponentId, FunctionId, TypeId, WitStepMode,
        };
        use wrt_foundation::NoStdProvider;
        use wrt_format::ast::SourceSpan;
        
        // Create a WIT-aware debugger
        let mut debugger = WitDebugger::new();
        println!("Created WIT-aware debugger");
        
        // Set up component metadata
        let provider = NoStdProvider::default();
        let component_metadata = ComponentMetadata {
            name: wrt_foundation::BoundedString::from_str("hello-world", provider.clone()).unwrap(),
            source_span: SourceSpan::new(0, 100, 0),
            binary_start: 1000,
            binary_end: 2000,
            exports: vec![FunctionId(1)],
            imports: vec![],
        };
        
        let component_id = ComponentId(1);
        debugger.add_component(component_id, component_metadata);
        println!("Added component metadata for component {:?}", component_id);
        
        // Set up function metadata
        let function_metadata = FunctionMetadata {
            name: wrt_foundation::BoundedString::from_str("greet", provider.clone()).unwrap(),
            source_span: SourceSpan::new(10, 50, 0),
            binary_offset: 1200,
            param_types: vec![TypeId(1)],
            return_types: vec![],
            is_async: false,
        };
        
        let function_id = FunctionId(1);
        debugger.add_function(function_id, function_metadata);
        println!("Added function metadata for function {:?}", function_id);
        
        // Set up type metadata
        let type_metadata = TypeMetadata {
            name: wrt_foundation::BoundedString::from_str("string", provider.clone()).unwrap(),
            source_span: SourceSpan::new(5, 11, 0),
            kind: wrt_debug::WitTypeKind::Primitive,
            size: Some(4), // pointer size
        };
        
        let type_id = TypeId(1);
        debugger.add_type(type_id, type_metadata);
        println!("Added type metadata for type {:?}", type_id);
        
        // Add source file
        let wit_source = r#"package hello:world@1.0.0;

interface greeter {
    greet: func(name: string);
}

world hello-world {
    export greeter;
}
"#;
        
        debugger.add_source_file(0, "hello.wit", wit_source).expect("Failed to add source file");
        println!("Added source file: hello.wit");
        
        // Demonstrate source-level breakpoint
        let breakpoint_span = SourceSpan::new(10, 50, 0); // Function span
        match debugger.add_source_breakpoint(breakpoint_span) {
            Ok(bp_id) => println!("Added source breakpoint with ID: {}", bp_id),
            Err(e) => println!("Failed to add breakpoint: {:?}", e),
        }
        
        // Set step mode
        debugger.set_step_mode(WitStepMode::SourceLine);
        println!("Set step mode to source line stepping");
        
        // Demonstrate address-to-component mapping
        let test_address = 1500u32;
        if let Some(found_component) = debugger.find_component_for_address(test_address) {
            println!("Address {} belongs to component {:?}", test_address, found_component);
        } else {
            println!("Address {} not found in any component", test_address);
        }
        
        // Demonstrate address-to-function mapping
        if let Some(found_function) = debugger.find_function_for_address(test_address) {
            println!("Address {} belongs to function {:?}", test_address, found_function);
            
            // Get function name
            if let Some(func_name) = debugger.wit_function_name(found_function) {
                println!("Function name: {}", func_name.as_str().unwrap_or("<invalid>"));
            }
        } else {
            println!("Address {} not found in any function", test_address);
        }
        
        // Demonstrate source context retrieval
        if let Some(source_context) = debugger.source_context_for_address(test_address, 2) {
            println!("Source context for address {}:", test_address);
            println!("File: {}", source_context.file_path.as_str().unwrap_or("<invalid>"));
            for line in source_context.lines {
                let marker = if line.is_highlighted { ">" } else { " " };
                println!("{} {:3}: {}", marker, line.line_number, 
                        line.content.as_str().unwrap_or("<invalid>"));
            }
        } else {
            println!("No source context available for address {}", test_address);
        }
        
        println!("\nWIT debugging integration example completed!");
        println!("In a real application, this debugger would be attached to the runtime");
        println!("and receive debugging events during component execution.");
    }
    
    #[cfg(not(feature = "wit-integration"))]
    {
        println!("This example requires the wit-integration feature to be enabled.");
        println!("Run with: cargo run --example wit_debug_integration_example --features wit-integration");
    }
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
fn main() {
    println!("This example requires std or alloc features");
}