use std::{fs, path::Path};

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
        // First try to copy from the project root wit directory
        let root_wit = Path::new("../wit/wasi/logging/logging.wit");
        if root_wit.exists() {
            fs::copy(root_wit, logging_wit).expect("Failed to copy logging.wit from root");
        } else {
            // Create a minimal logging.wit file
            let minimal_logging_wit = r#"// WASI Logging Interface
package wasi:logging@0.2.0;

/// Log levels, ordered by severity.
enum level {
    /// Trace-level message (lowest severity).
    trace,
    /// Debug-level message.
    debug,
    /// Info-level message.
    info,
    /// Warn-level message.
    warn,
    /// Error-level message (highest severity).
    error,
}

/// Logging context information.
record context {
    /// The span ID, if provided.
    span-id: option<u64>,
}

/// Logging interface to output messages at different severity levels.
interface logging {
    /// Log a message at the specified level.
    log: func(level: level, context: context, message: string);
}
"#;
            fs::write(logging_wit, minimal_logging_wit)
                .expect("Failed to create minimal logging.wit");
        }
    }

    // Output any WIT files we find
    let wit_dir = Path::new("wit");
    if wit_dir.exists() {
        for entry in fs::read_dir(wit_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "wit") {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}
