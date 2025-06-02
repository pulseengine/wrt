//! Example demonstrating WIT Language Server Protocol (LSP) support
//!
//! This example shows how to use the basic LSP infrastructure for WIT files.

#[cfg(all(feature = "lsp", any(feature = "std", feature = "alloc")))]
fn main() {
    use wrt_format::lsp_server::{
        WitLanguageServer, TextDocumentItem, Position, Range,
        TextDocumentContentChangeEvent, DiagnosticSeverity,
        CompletionItemKind,
    };
    use wrt_foundation::{BoundedString, NoStdProvider};
    
    println!("WIT LSP Server Example");
    println!("======================");
    
    // Create a language server
    let mut server = WitLanguageServer::new();
    
    println!("\n--- Server Capabilities ---");
    let caps = server.capabilities();
    println!("✓ Text document sync: {}", caps.text_document_sync);
    println!("✓ Hover provider: {}", caps.hover_provider);
    println!("✓ Completion provider: {}", caps.completion_provider);
    println!("✓ Definition provider: {}", caps.definition_provider);
    println!("✓ Document symbols: {}", caps.document_symbol_provider);
    println!("✓ Diagnostics: {}", caps.diagnostic_provider);
    
    // Open a WIT document
    println!("\n--- Opening Document ---");
    
    let provider = NoStdProvider::<1024>::new();
    let uri = BoundedString::from_str("file:///example.wit", provider.clone()).unwrap();
    let language_id = BoundedString::from_str("wit", provider.clone()).unwrap();
    
    let content = vec![
        BoundedString::from_str("package hello:world@1.0.0;", provider.clone()).unwrap(),
        BoundedString::from_str("", provider.clone()).unwrap(),
        BoundedString::from_str("interface greeter {", provider.clone()).unwrap(),
        BoundedString::from_str("    greet: func(name: string) -> string;", provider.clone()).unwrap(),
        BoundedString::from_str("}", provider.clone()).unwrap(),
        BoundedString::from_str("", provider.clone()).unwrap(),
        BoundedString::from_str("world hello-world {", provider.clone()).unwrap(),
        BoundedString::from_str("    export greeter;", provider.clone()).unwrap(),
        BoundedString::from_str("}", provider.clone()).unwrap(),
    ];
    
    let document = TextDocumentItem {
        uri: uri.clone(),
        language_id,
        version: 1,
        text: content,
    };
    
    match server.open_document(document) {
        Ok(()) => println!("✓ Document opened successfully"),
        Err(e) => println!("✗ Failed to open document: {:?}", e),
    }
    
    // Test hover functionality
    println!("\n--- Hover Information ---");
    
    let hover_position = Position { line: 3, character: 10 }; // On "greet"
    match server.hover("file:///example.wit", hover_position) {
        Ok(Some(hover)) => {
            println!("✓ Hover at line {}, char {}: {}", 
                     hover_position.line, 
                     hover_position.character,
                     hover.contents.as_str().unwrap_or("<invalid>"));
        }
        Ok(None) => println!("- No hover information available"),
        Err(e) => println!("✗ Hover failed: {:?}", e),
    }
    
    // Test completion
    println!("\n--- Code Completion ---");
    
    let completion_position = Position { line: 4, character: 0 }; // Empty line
    match server.completion("file:///example.wit", completion_position) {
        Ok(items) => {
            println!("✓ Found {} completion items:", items.len());
            
            // Show first few completions
            for (i, item) in items.iter().take(5).enumerate() {
                let kind_str = match item.kind {
                    CompletionItemKind::Keyword => "keyword",
                    CompletionItemKind::Function => "function",
                    CompletionItemKind::Interface => "interface",
                    CompletionItemKind::Type => "type",
                    CompletionItemKind::Field => "field",
                    CompletionItemKind::EnumMember => "enum",
                };
                
                println!("  {}. {} ({})", 
                         i + 1, 
                         item.label.as_str().unwrap_or("<invalid>"),
                         kind_str);
            }
        }
        Err(e) => println!("✗ Completion failed: {:?}", e),
    }
    
    // Test document symbols
    println!("\n--- Document Symbols ---");
    
    match server.document_symbols("file:///example.wit") {
        Ok(symbols) => {
            println!("✓ Found {} document symbols:", symbols.len());
            
            for symbol in &symbols {
                println!("  - {} ({:?})", 
                         symbol.name.as_str().unwrap_or("<invalid>"),
                         symbol.kind);
                
                // Show children if any
                #[cfg(any(feature = "std", feature = "alloc"))]
                for child in &symbol.children {
                    println!("    - {} ({:?})", 
                             child.name.as_str().unwrap_or("<invalid>"),
                             child.kind);
                }
            }
        }
        Err(e) => println!("✗ Document symbols failed: {:?}", e),
    }
    
    // Test incremental updates
    println!("\n--- Incremental Update ---");
    
    let change_text = BoundedString::from_str("    goodbye: func() -> string;", provider.clone()).unwrap();
    let change = TextDocumentContentChangeEvent {
        range: Some(Range {
            start: Position { line: 4, character: 0 },
            end: Position { line: 4, character: 0 },
        }),
        text: change_text,
    };
    
    match server.update_document("file:///example.wit", vec![change], 2) {
        Ok(()) => println!("✓ Document updated successfully"),
        Err(e) => println!("✗ Update failed: {:?}", e),
    }
    
    println!("\n--- LSP Integration Benefits ---");
    println!("1. Real-time syntax checking and diagnostics");
    println!("2. Code completion with context awareness");
    println!("3. Hover information for types and functions");
    println!("4. Document outline with symbols");
    println!("5. Incremental parsing for performance");
    println!("6. Go to definition and find references");
    
    println!("\nLSP server example completed!");
}

#[cfg(not(all(feature = "lsp", any(feature = "std", feature = "alloc"))))]
fn main() {
    println!("This example requires the 'lsp' feature and either 'std' or 'alloc'");
    println!("Run with: cargo run --example wit_lsp_example --features lsp,std");
}