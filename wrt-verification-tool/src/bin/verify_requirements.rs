use std::env;
use std::process;
use wrt_verification_tool::requirements_file::RequirementsFile;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <requirements.toml>", args[0]);
        process::exit(1);
    }
    
    let requirements_path = &args[1];
    
    match RequirementsFile::load(requirements_path) {
        Ok(req_file) => {
            println!("{}", req_file.generate_report());
            
            let missing_files = req_file.verify_files_exist();
            if !missing_files.is_empty() {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error loading requirements file: {}", e);
            process::exit(1);
        }
    }
}