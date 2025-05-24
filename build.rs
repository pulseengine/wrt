fn main() {
    // If using skeptic for markdown examples
    #[cfg(feature = "test-examples")]
    {
        extern crate skeptic;
        
        // Test all markdown files with code examples
        skeptic::generate_doc_tests(&[
            "README.md",
            "docs/examples/README.md",
            // Add more markdown files here
        ]);
    }
}