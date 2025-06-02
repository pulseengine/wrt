//! Example demonstrating WIT incremental parsing
//!
//! This example shows how to use the incremental parser for efficient
//! re-parsing of WIT files when changes are made.

#[cfg(any(feature = "std", feature = "alloc"))]
fn main() {
    use wrt_format::incremental_parser::{
        IncrementalParser, IncrementalParserCache, ChangeType, SourceChange,
    };
    use wrt_foundation::{BoundedString, NoStdProvider};
    
    println!("WIT Incremental Parser Example");
    println!("==============================");
    
    // Create an incremental parser
    let mut parser = IncrementalParser::new();
    
    // Initial WIT source
    let initial_source = r#"package hello:world@1.0.0;

interface greeter {
    greet: func(name: string) -> string;
}

world hello-world {
    export greeter;
}
"#;
    
    // Set initial source
    match parser.set_source(initial_source) {
        Ok(()) => println!("✓ Initial parse successful"),
        Err(e) => println!("✗ Initial parse failed: {:?}", e),
    }
    
    // Check statistics
    let stats = parser.stats();
    println!("\nInitial parse statistics:");
    println!("  Total parses: {}", stats.total_parses);
    println!("  Full re-parses: {}", stats.full_reparses);
    
    // Simulate a change: Add a new function
    let provider = NoStdProvider::<1024>::new();
    let new_text = BoundedString::from_str("    goodbye: func() -> string;\n", provider)
        .expect("Failed to create bounded string");
    
    let change = SourceChange {
        change_type: ChangeType::Insert {
            offset: 80, // After the greet function
            length: new_text.as_str().map(|s| s.len() as u32).unwrap_or(0),
        },
        text: Some(new_text),
    };
    
    println!("\nApplying change: Adding 'goodbye' function");
    match parser.apply_change(change) {
        Ok(()) => println!("✓ Incremental parse successful"),
        Err(e) => println!("✗ Incremental parse failed: {:?}", e),
    }
    
    // Check updated statistics
    let stats = parser.stats();
    println!("\nUpdated parse statistics:");
    println!("  Total parses: {}", stats.total_parses);
    println!("  Incremental parses: {}", stats.incremental_parses);
    println!("  Nodes reused: {}", stats.nodes_reused);
    println!("  Nodes re-parsed: {}", stats.nodes_reparsed);
    
    // Demonstrate parser cache for multiple files
    println!("\n--- Multi-file Parser Cache ---");
    
    let mut cache = IncrementalParserCache::new();
    
    // Add parsers for multiple files
    let parser1 = cache.get_parser(0); // file_id = 0
    parser1.set_source("interface file1 { test: func(); }").ok();
    
    let parser2 = cache.get_parser(1); // file_id = 1
    parser2.set_source("interface file2 { run: func() -> u32; }").ok();
    
    // Get global statistics
    let global_stats = cache.global_stats();
    println!("\nGlobal statistics across all files:");
    println!("  Total parses: {}", global_stats.total_parses);
    println!("  Full re-parses: {}", global_stats.full_reparses);
    
    // Demonstrate change types
    println!("\n--- Change Types ---");
    
    let delete_change = ChangeType::Delete {
        offset: 50,
        length: 10,
    };
    println!("Delete change: Remove 10 characters at offset 50");
    
    let replace_change = ChangeType::Replace {
        offset: 100,
        old_length: 5,
        new_length: 8,
    };
    println!("Replace change: Replace 5 characters with 8 at offset 100");
    
    println!("\n--- Incremental Parsing Benefits ---");
    println!("1. Efficient re-parsing: Only affected nodes are re-parsed");
    println!("2. Memory efficient: Reuses existing parse tree nodes");
    println!("3. LSP-ready: Designed for language server protocol integration");
    println!("4. Multi-file support: Cache manages parsers for multiple files");
    
    println!("\nIncremental parser example completed!");
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
fn main() {
    println!("This example requires std or alloc features");
}