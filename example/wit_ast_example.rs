//! Example demonstrating WIT AST usage
//!
//! This example shows how to create and work with WIT AST nodes for 
//! building language tools and analysis.

#[cfg(any(feature = "std", feature = "alloc"))]
use wrt_format::ast::*;
#[cfg(any(feature = "std", feature = "alloc"))]
use wrt_format::wit_parser::{WitBoundedString};
#[cfg(any(feature = "std", feature = "alloc"))]
use wrt_foundation::NoStdProvider;

#[cfg(any(feature = "std", feature = "alloc"))]
fn main() {
    println!("WIT AST Example");
    println!("===============");
    
    // Create a simple identifier
    let provider = NoStdProvider::<1024>::new();
    let name = WitBoundedString::from_str("hello", provider.clone()).unwrap();
    let span = SourceSpan::new(0, 5, 0);
    let ident = Identifier::new(name, span);
    
    println!("Created identifier: {} at span {:?}", ident, span);
    
    // Create a primitive type
    let string_type = TypeExpr::Primitive(PrimitiveType {
        kind: PrimitiveKind::String,
        span: SourceSpan::new(10, 16, 0),
    });
    
    println!("Created string type at span {:?}", string_type.span());
    
    // Create a function parameter
    let param = Param {
        name: ident.clone(),
        ty: string_type,
        span: SourceSpan::new(0, 20, 0),
    };
    
    println!("Created parameter: {} of type string", param.name);
    
    // Create a simple function
    let function = Function {
        #[cfg(any(feature = "std", feature = "alloc"))]
        params: vec![param],
        results: FunctionResults::None,
        is_async: false,
        span: SourceSpan::new(0, 30, 0),
    };
    
    println!("Created function with {} parameters", function.params.len());
    
    // Create a function declaration
    let func_name = WitBoundedString::from_str("greet", provider.clone()).unwrap();
    let func_ident = Identifier::new(func_name, SourceSpan::new(35, 40, 0));
    
    let func_decl = FunctionDecl {
        name: func_ident.clone(),
        func: function,
        docs: None,
        span: SourceSpan::new(35, 60, 0),
    };
    
    println!("Created function declaration: {}", func_decl.name);
    
    // Create an interface
    let interface_name = WitBoundedString::from_str("greeter", provider.clone()).unwrap();
    let interface_ident = Identifier::new(interface_name, SourceSpan::new(70, 77, 0));
    
    let interface = InterfaceDecl {
        name: interface_ident.clone(),
        #[cfg(any(feature = "std", feature = "alloc"))]
        items: vec![InterfaceItem::Function(func_decl)],
        docs: None,
        span: SourceSpan::new(70, 100, 0),
    };
    
    println!("Created interface: {} with {} items", 
             interface.name, interface.items.len());
    
    // Create a WIT document
    let mut document = WitDocument {
        package: None,
        #[cfg(any(feature = "std", feature = "alloc"))]
        use_items: vec![],
        #[cfg(any(feature = "std", feature = "alloc"))]
        items: vec![TopLevelItem::Interface(interface)],
        span: SourceSpan::new(0, 100, 0),
    };
    
    println!("Created WIT document with {} top-level items", document.items.len());
    
    // Demonstrate span merging
    let span1 = SourceSpan::new(0, 10, 0);
    let span2 = SourceSpan::new(5, 15, 0);
    let merged = span1.merge(&span2);
    
    println!("Merged spans [{}, {}] and [{}, {}] -> [{}, {}]",
             span1.start, span1.end, span2.start, span2.end,
             merged.start, merged.end);
    
    println!("\nAST Example completed successfully!");
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
fn main() {
    println!("This example requires std or alloc features");
}