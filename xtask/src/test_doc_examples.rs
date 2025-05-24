use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::Regex;

pub fn test_documentation_examples(docs_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing documentation examples...");
    
    // Find all RST files
    let rst_files = find_rst_files(docs_path)?;
    
    for rst_file in rst_files {
        println!("Checking examples in: {}", rst_file.display());
        let content = fs::read_to_string(&rst_file)?;
        
        // Extract Rust code blocks
        let rust_examples = extract_rust_code_blocks(&content);
        
        for (idx, example) in rust_examples.iter().enumerate() {
            test_example(&rst_file, idx, example)?;
        }
    }
    
    Ok(())
}

fn find_rst_files(path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                files.extend(find_rst_files(&path)?);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rst") {
                files.push(path);
            }
        }
    }
    
    Ok(files)
}

fn extract_rust_code_blocks(content: &str) -> Vec<String> {
    let mut examples = Vec::new();
    
    // Match code blocks with rust language
    let re = Regex::new(r"(?s)\.\. code-block:: rust.*?\n\n((?:   .*\n|\n)+)").unwrap();
    
    for cap in re.captures_iter(content) {
        if let Some(code) = cap.get(1) {
            // Remove RST indentation (3 spaces)
            let code = code.as_str()
                .lines()
                .map(|line| {
                    if line.starts_with("   ") {
                        &line[3..]
                    } else {
                        line
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            
            examples.push(code);
        }
    }
    
    examples
}

fn test_example(rst_file: &Path, idx: usize, code: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary test file
    let test_name = format!("doc_example_{}_{}.rs", 
        rst_file.file_stem().unwrap().to_string_lossy(),
        idx
    );
    
    let test_dir = PathBuf::from("target/doc_tests");
    fs::create_dir_all(&test_dir)?;
    
    let test_file = test_dir.join(&test_name);
    
    // Wrap the example in a test function
    let test_code = format!(
        r#"
#[test]
fn test_example() {{
{}
}}
"#,
        code.lines()
            .map(|line| format!("    {}", line))
            .collect::<Vec<_>>()
            .join("\n")
    );
    
    fs::write(&test_file, &test_code)?;
    
    // Try to compile it
    let output = Command::new("rustc")
        .arg("--test")
        .arg(&test_file)
        .arg("--edition=2021")
        .arg("-L")
        .arg("target/debug/deps")
        .output()?;
    
    if !output.status.success() {
        eprintln!("Failed to compile example from {}:", rst_file.display());
        eprintln!("Code:\n{}", code);
        eprintln!("Error:\n{}", String::from_utf8_lossy(&output.stderr));
        return Err("Compilation failed".into());
    }
    
    println!("  ✓ Example {} compiled successfully", idx);
    
    // Clean up
    fs::remove_file(&test_file)?;
    
    Ok(())
}