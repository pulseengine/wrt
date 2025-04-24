use anyhow::{Context, Result};
use serde::Serialize;
use std::env;
use std::fs::{self, File};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

/// Generate the version switcher JSON file for documentation
pub fn generate_switcher_json(is_local: bool) -> Result<()> {
    let versions = get_versions()?;
    let current_version = env::var("DOCS_VERSION").unwrap_or_else(|_| "main".to_string());

    // Determine the base URL based on environment
    let base_url = if is_local {
        "http://localhost:8080/"
    } else {
        "https://avrabe.github.io/wrt/"
    };

    // Create entries for the switcher JSON
    let mut entries = Vec::new();
    for version in &versions {
        // Format the name based on version
        let name = if version == "main" {
            "main (development)".to_string()
        } else if version == versions.last().unwrap() && version != "main" {
            format!("v{} (stable)", version)
        } else {
            format!("v{}", version)
        };

        // Create the entry
        let mut entry = SwitcherEntry {
            name,
            version: version.clone(),
            url: format!("{}{}/", base_url, version),
            preferred: None,
        };

        // Mark the latest release as preferred (for warning banners)
        if version == versions.last().unwrap() && version != "main" {
            entry.preferred = Some(true);
        }

        entries.push(entry);
    }

    // Create the output directory if it doesn't exist
    let output_dir = PathBuf::from("docs/_build/versioned");
    fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    // Write the JSON file
    let output_path = output_dir.join("switcher.json");
    let file = File::create(&output_path).context("Failed to create switcher.json file")?;
    serde_json::to_writer_pretty(file, &entries).context("Failed to write JSON data")?;

    println!("Generated switcher.json at {}", output_path.display());
    println!("Available versions: {}", versions.join(", "));
    println!("Current version: {}", current_version);

    Ok(())
}

/// Start a local HTTP server for documentation
pub fn serve_docs() -> Result<()> {
    // Check if the versioned docs directory exists
    let versioned_dir = PathBuf::from("docs/_build/versioned");
    if !versioned_dir.exists() {
        println!("Error: {} does not exist.", versioned_dir.display());
        println!("Please build the documentation first with 'just docs-versioned'");
        return Ok(());
    }

    // Generate local switcher.json
    println!("Generating local switcher.json...");
    generate_switcher_json(true)?;

    // Start the HTTP server using tiny_http
    let addr = SocketAddr::from_str("0.0.0.0:8080").unwrap();
    let server = tiny_http::Server::http(addr)
        .map_err(|e| anyhow::anyhow!("Failed to start HTTP server: {}", e))?;

    println!("Starting local server at http://localhost:8080");
    println!("Serving documentation from {}", versioned_dir.display());
    println!("Press Ctrl+C to stop the server");

    for request in server.incoming_requests() {
        // Convert the request path to a file path
        let url_path = request.url();
        let file_path = if url_path == "/" {
            versioned_dir.join("index.html")
        } else {
            versioned_dir.join(url_path.trim_start_matches("/"))
        };

        // Skip logging for switcher.json requests
        if !url_path.contains("switcher.json") {
            println!("{} {}", request.method(), url_path);
        }

        // Handle the request
        let response = if file_path.exists() && file_path.is_file() {
            // Determine content type based on file extension
            let content_type = match file_path.extension().and_then(|e| e.to_str()) {
                Some("html") => "text/html",
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("json") => "application/json",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("svg") => "image/svg+xml",
                _ => "application/octet-stream",
            };

            // Open the file and create a response
            match fs::read(&file_path) {
                Ok(content) => {
                    let mut response = tiny_http::Response::from_data(content);
                    response.add_header(
                        tiny_http::Header::from_str(&format!("Content-Type: {}", content_type))
                            .unwrap(),
                    );
                    response
                }
                Err(_) => {
                    let mut response =
                        tiny_http::Response::from_string("500 Internal Server Error");
                    response.add_header(
                        tiny_http::Header::from_str("Content-Type: text/plain").unwrap(),
                    );
                    response.with_status_code(500)
                }
            }
        } else {
            // File not found
            let mut response = tiny_http::Response::from_string("404 Not Found");
            response.add_header(tiny_http::Header::from_str("Content-Type: text/plain").unwrap());
            response.with_status_code(404)
        };

        if let Err(e) = request.respond(response) {
            println!("Error responding to request: {}", e);
        }
    }

    Ok(())
}

/// Get all available versions from git tags and the main branch
fn get_versions() -> Result<Vec<String>> {
    let mut versions = vec!["main".to_string()];

    // Get all tags from git
    let output = Command::new("git")
        .args(["tag"])
        .output()
        .context("Failed to execute git tag command")?;

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for tag in output_str.lines() {
            // Handle tags with optional 'v' prefix (v0.1.0 or 0.1.0)
            let clean_tag = tag.trim_start_matches('v');

            // Only include semantic version tags (x.y.z)
            if clean_tag.matches('.').count() == 2
                && clean_tag.split('.').all(|part| part.parse::<u32>().is_ok())
            {
                versions.push(clean_tag.to_string());
            }
        }
    }

    // Sort versions (keep 'main' at the beginning)
    let main_idx = versions.iter().position(|v| v == "main").unwrap();
    versions.swap(0, main_idx); // Move 'main' to the beginning

    // Sort the rest of the versions
    let rest = &mut versions[1..];
    rest.sort_by(|a, b| {
        let a_parts: Vec<u32> = a.split('.').map(|part| part.parse().unwrap_or(0)).collect();
        let b_parts: Vec<u32> = b.split('.').map(|part| part.parse().unwrap_or(0)).collect();
        a_parts.cmp(&b_parts)
    });

    Ok(versions)
}

#[derive(Serialize)]
struct SwitcherEntry {
    name: String,
    version: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    preferred: Option<bool>,
}
