//! Example demonstrating WIT component lowering integration
//!
//! This example shows how to use the enhanced component lowering system
//! to convert WIT interfaces to component model representations.

#[cfg(any(feature = "std", feature = "alloc"))]
fn main() {
    // Note: This example would use wrt-component features if they were available
    println!("WIT Component Lowering Example");
    println!("===============================");
    
    // Create a sample WIT document programmatically
    use wrt_format::ast::*;
    use wrt_foundation::NoStdProvider;
    
    let provider = NoStdProvider::<1024>::new();
    
    // Create interface declaration
    let interface_name = wrt_format::wit_parser::WitBoundedString::from_str("greeter", provider.clone())
        .expect("Failed to create interface name");
    let interface_ident = Identifier::new(interface_name, SourceSpan::new(10, 17, 0));
    
    // Create function parameter
    let param_name = wrt_format::wit_parser::WitBoundedString::from_str("name", provider.clone())
        .expect("Failed to create param name");
    let param_ident = Identifier::new(param_name, SourceSpan::new(25, 29, 0));
    
    let param = Param {
        name: param_ident,
        ty: TypeExpr::Primitive(PrimitiveType {
            kind: PrimitiveKind::String,
            span: SourceSpan::new(31, 37, 0),
        }),
        span: SourceSpan::new(25, 37, 0),
    };
    
    // Create function
    let func_name = wrt_format::wit_parser::WitBoundedString::from_str("greet", provider.clone())
        .expect("Failed to create function name");
    let func_ident = Identifier::new(func_name, SourceSpan::new(43, 48, 0));
    
    let function = Function {
        params: vec![param],
        results: FunctionResults::Single(TypeExpr::Primitive(PrimitiveType {
            kind: PrimitiveKind::String,
            span: SourceSpan::new(52, 58, 0),
        })),
        is_async: false,
        span: SourceSpan::new(25, 58, 0),
    };
    
    let func_decl = FunctionDecl {
        name: func_ident,
        func: function,
        docs: None,
        span: SourceSpan::new(43, 58, 0),
    };
    
    // Create interface
    let interface = InterfaceDecl {
        name: interface_ident,
        items: vec![InterfaceItem::Function(func_decl)],
        docs: None,
        span: SourceSpan::new(10, 60, 0),
    };
    
    // Create WIT document
    let document = WitDocument {
        package: None,
        use_items: vec![],
        items: vec![TopLevelItem::Interface(interface)],
        span: SourceSpan::new(0, 60, 0),
    };
    
    println!("✓ Created WIT document with interface 'greeter'");
    
    #[cfg(feature = "component-integration")]
    {
        // This would use the WIT component integration
        use wrt_component::{ComponentLowering, ComponentConfig};
        
        println!("\n--- Component Lowering ---");
        
        // Configure component lowering
        let config = ComponentConfig {
            debug_info: true,
            optimize: false,
            memory_limit: Some(1024 * 1024), // 1MB
            stack_limit: Some(64 * 1024),    // 64KB
            async_support: false,
        };
        
        match ComponentLowering::lower_document_with_config(document, config) {
            Ok(context) => {
                println!("✓ Document lowered successfully");
                
                // Show interface mappings
                for (name, interface) in context.interfaces() {
                    println!("  Interface: {} (ID: {})", name, interface.component_id);
                    println!("    Functions: {}", interface.functions.len());
                    println!("    Types: {}", interface.types.len());
                }
                
                // Show type mappings
                for (name, type_mapping) in context.types() {
                    println!("  Type: {} -> {:?}", name, type_mapping.component_type);
                    if let Some(size) = type_mapping.size {
                        println!("    Size: {} bytes", size);
                    }
                    if let Some(align) = type_mapping.alignment {
                        println!("    Alignment: {} bytes", align);
                    }
                }
                
                // Show function mappings
                for (name, func_mapping) in context.functions() {
                    println!("  Function: {} (Index: {})", name, func_mapping.function_index);
                    println!("    Parameters: {}", func_mapping.param_types.len());
                    println!("    Returns: {}", func_mapping.return_types.len());
                    println!("    Async: {}", func_mapping.is_async);
                }
                
                // Validate mappings
                match ComponentLowering::validate_mappings(&context) {
                    Ok(()) => println!("✓ All mappings validated successfully"),
                    Err(e) => println!("✗ Validation failed: {:?}", e),
                }
            }
            Err(e) => println!("✗ Failed to lower document: {:?}", e),
        }
    }
    
    #[cfg(not(feature = "component-integration"))]
    {
        println!("\n--- Component Integration Demo ---");
        println!("The actual component integration would:");
        println!("1. Convert WIT types to component model types");
        println!("2. Map functions to component function indices");
        println!("3. Generate interface mappings");
        println!("4. Calculate type sizes and alignments");
        println!("5. Validate all mappings for consistency");
        println!("6. Enable efficient component instantiation");
        println!("");
        println!("Example mappings:");
        println!("  WIT 'string' -> ComponentType::String");
        println!("  WIT 'u32' -> ComponentType::U32 (4 bytes, 4-byte aligned)");
        println!("  WIT function 'greet' -> Component function index 0");
        println!("  WIT interface 'greeter' -> Component interface ID 0");
    }
    
    println!("\n--- Integration Benefits ---");
    println!("1. Type-safe lowering from WIT to component model");
    println!("2. Automatic size and alignment calculation");
    println!("3. Validation of component mappings");
    println!("4. Memory-efficient representation");
    println!("5. Debugging support with source locations");
    println!("6. Configurable optimization levels");
    
    println!("\nComponent lowering example completed!");
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
fn main() {
    println!("This example requires std or alloc features");
}