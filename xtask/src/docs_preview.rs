//! Documentation preview server for xtask

use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::process::{Command, Stdio};
use tiny_http::{Server, Response, Header};

/// Configuration for documentation preview
#[derive(Debug, Clone)]
pub struct DocsPreviewConfig {
    pub port: u16,
    pub host: IpAddr,
    pub docs_dir: String,
    pub open_browser: bool,
}

impl Default for DocsPreviewConfig {
    fn default() -> Self {
        Self {
            port: 8000,
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            docs_dir: "docs_output/local".to_string(),
            open_browser: false,
        }
    }
}

/// Start documentation preview server
pub fn run_docs_preview(config: DocsPreviewConfig) -> Result<()> {
    let docs_path = Path::new(&config.docs_dir);
    
    if !docs_path.exists() {
        println!("❌ Documentation directory '{}' does not exist", config.docs_dir);
        println!("💡 Try running: xtask docs");
        return Err(anyhow::anyhow!("Documentation directory not found"));
    }
    
    let addr = SocketAddr::new(config.host, config.port);
    
    println!("🌐 Starting documentation preview server...");
    println!("📁 Serving from: {}", docs_path.display());
    println!("🔗 Documentation available at: http://{}", addr);
    println!("⏹️  Press Ctrl+C to stop the server");
    println!();
    
    // Open browser if requested
    if config.open_browser {
        open_browser(&format!("http://{}", addr))?;
    }
    
    // Start the HTTP server using tiny_http
    let server = Server::http(addr)
        .map_err(|e| anyhow::anyhow!("Failed to start HTTP server: {}", e))?;
    
    println!("✅ Server started successfully");
    
    // Serve files
    for request in server.incoming_requests() {
        match serve_file(&request, docs_path) {
            Ok(response) => {
                let _ = request.respond(response);
            }
            Err(e) => {
                println!("⚠️  Error serving request: {}", e);
                let response = Response::from_string("Internal Server Error")
                    .with_status_code(500);
                let _ = request.respond(response);
            }
        }
    }
    
    Ok(())
}

/// Serve a file based on the HTTP request
fn serve_file(request: &tiny_http::Request, docs_path: &Path) -> Result<Response<std::io::Cursor<Vec<u8>>>> {
    let url_path = request.url().trim_start_matches('/');
    
    // Default to index.html if path is empty
    let file_path = if url_path.is_empty() || url_path == "/" {
        docs_path.join("index.html")
    } else {
        docs_path.join(url_path)
    };
    
    // Security check: ensure the file is within docs_path
    let canonical_docs = docs_path.canonicalize()
        .map_err(|e| anyhow::anyhow!("Failed to canonicalize docs path: {}", e))?;
    let canonical_file = file_path.canonicalize()
        .unwrap_or(file_path.clone());
    
    if !canonical_file.starts_with(&canonical_docs) {
        return Ok(Response::from_string("Forbidden").with_status_code(403));
    }
    
    // Check if file exists
    if !file_path.exists() {
        return Ok(Response::from_string("Not Found").with_status_code(404));
    }
    
    // Read file content
    let content = std::fs::read(&file_path)
        .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;
    
    // Determine content type
    let content_type = match file_path.extension().and_then(|s| s.to_str()) {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        _ => "text/plain",
    };
    
    let header = Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to create content-type header: {:?}", e))?;
    
    Ok(Response::from_data(content).with_header(header))
}

/// Open browser to the given URL
fn open_browser(url: &str) -> Result<()> {
    println!("🌐 Opening browser to: {}", url);
    
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(target_os = "linux")]
    let cmd = "xdg-open";
    #[cfg(target_os = "windows")]
    let cmd = "start";
    
    let result = Command::new(cmd)
        .arg(url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    
    match result {
        Ok(_) => Ok(()),
        Err(_) => {
            println!("⚠️  Could not automatically open browser");
            println!("💡 Please manually open: {}", url);
            Ok(())
        }
    }
}