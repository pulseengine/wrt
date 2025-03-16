use std::fs;
use std::path::Path;

fn main() {
    // Tell Cargo to rerun this build script if the WIT files change
    println!("cargo:rerun-if-changed=wit/example.wit");
    println!("cargo:rerun-if-changed=wit/wasi/logging/logging.wit");

    // Make sure the necessary directories exist
    let logging_dir = Path::new("wit/wasi/logging");
    if !logging_dir.exists() {
        fs::create_dir_all(logging_dir).expect("Failed to create WIT directories");
    }

    // Copy the WASI logging WIT file if it doesn't exist
    let logging_wit = logging_dir.join("logging.wit");
    if !logging_wit.exists() {
        let source_wit = Path::new("../wasi-logging/wit/logging.wit");
        if source_wit.exists() {
            fs::copy(source_wit, logging_wit).expect("Failed to copy logging.wit");
        }
    }
}
