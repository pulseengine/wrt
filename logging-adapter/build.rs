use std::fs;
use std::path::Path;

fn main() {
    // Tell Cargo to rerun this build script if the WIT files change
    println!("cargo:rerun-if-changed=wit/adapter.wit");
    println!("cargo:rerun-if-changed=wit/deps/wasi/logging/logging.wit");
    println!("cargo:rerun-if-changed=wit/deps/wasi/cli/stderr.wit");
    println!("cargo:rerun-if-changed=wit/deps/wasi/io/streams.wit");

    // Make sure the necessary directories exist
    let directories = [
        "wit/deps/wasi/logging",
        "wit/deps/wasi/cli",
        "wit/deps/wasi/io",
    ];

    for dir in directories.iter() {
        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            fs::create_dir_all(dir_path)
                .unwrap_or_else(|_| panic!("Failed to create directory: {}", dir));
        }
    }

    // Copy the WASI logging WIT file if it doesn't exist
    ensure_wit_file(
        "../wit/wasi/logging/logging.wit",
        "wit/deps/wasi/logging/logging.wit",
        "wit/adapter.wit",
    );

    // Create minimal CLI stderr interface
    let stderr_path = Path::new("wit/deps/wasi/cli/stderr.wit");
    if !stderr_path.exists() {
        let stderr_content = r#"package wasi:cli@0.2.0;
import wasi:io/streams@0.2.0.{output-stream};

interface stderr {
    get-stderr: func() -> output-stream;
}
"#;
        fs::write(stderr_path, stderr_content).expect("Failed to create minimal stderr.wit");
    }

    // Create minimal IO streams interface
    let streams_path = Path::new("wit/deps/wasi/io/streams.wit");
    if !streams_path.exists() {
        let streams_content = r#"package wasi:io@0.2.0;

interface streams {
    /// An output stream of bytes.
    resource output-stream {
        /// Write bytes to a stream.
        write: func(contents: list<u8>) -> result<u64, stream-error>;
        
        /// Indicate that the application is done writing to this stream.
        flush: func() -> result<_, stream-error>;
    }

    /// Errors which may occur when operating on a stream.
    variant stream-error {
        /// The last operation failed.
        last-operation-failed(string),
        /// The stream is closed.
        closed,
    }
}
"#;
        fs::write(streams_path, streams_content).expect("Failed to create minimal streams.wit");
    }
}

fn ensure_wit_file(source_path: &str, target_path: &str, _reference_content: &str) {
    let target = Path::new(target_path);
    if !target.exists() {
        let source = Path::new(source_path);
        if source.exists() {
            fs::copy(source, target)
                .unwrap_or_else(|_| panic!("Failed to copy WIT file from {}", source_path));
        } else {
            // If we're handling logging.wit specifically
            if target_path.contains("logging.wit") {
                let logging_content = r#"// WASI Logging Interface
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
                fs::write(target, logging_content)
                    .unwrap_or_else(|_| panic!("Failed to create {}", target_path));
            }
        }
    }
}
