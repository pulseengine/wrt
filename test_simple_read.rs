use std::fs;

fn main() {
    println!("Starting simple file read test...");
    
    let content = fs::read_to_string("simple_test.wast").expect("Failed to read file");
    println!("File content length: {}", content.len());
    println!("Content preview: {}", &content[..content.len().min(100)]);
    
    println!("File read successful!");
}