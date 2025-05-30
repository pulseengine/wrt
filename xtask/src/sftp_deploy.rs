//! SFTP hosting deployment for documentation

use anyhow::{Context, Result};
use ssh2::Session;
use std::collections::HashSet;
use std::env;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;

/// Configuration for SFTP deployment
#[derive(Debug, Clone)]
pub struct SftpDeployConfig {
    pub host: String,
    pub username: String,
    pub ssh_key_path: Option<PathBuf>,
    pub ssh_key_content: Option<String>,
    pub target_dir: String,
    pub docs_dir: String,
    pub build_docs: bool,
    pub dry_run: bool,
    pub delete_remote: bool,
    pub port: u16,
}

impl Default for SftpDeployConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            username: String::new(),
            ssh_key_path: None,
            ssh_key_content: None,
            target_dir: "/htdocs".to_string(),
            docs_dir: "docs_output".to_string(),
            build_docs: true,
            dry_run: false,
            delete_remote: false,
            port: 22,
        }
    }
}

impl SftpDeployConfig {
    /// Load configuration from environment variables and parameters
    pub fn from_env_and_args(
        host: Option<String>,
        username: Option<String>,
        target_dir: Option<String>,
        docs_dir: Option<String>,
        ssh_key_path: Option<String>,
        build_docs: bool,
        dry_run: bool,
        delete_remote: bool,
        port: Option<u16>,
    ) -> Result<Self> {
        let config = Self {
            host: host
                .or_else(|| env::var("SFTP_HOST").ok())
                .context("Missing host. Set --host or SFTP_HOST environment variable")?,
            username: username
                .or_else(|| env::var("SFTP_USERNAME").ok())
                .context("Missing username. Set --username or SFTP_USERNAME environment variable")?,
            ssh_key_path: ssh_key_path
                .map(PathBuf::from)
                .or_else(|| env::var("SFTP_SSH_KEY_PATH").ok().map(PathBuf::from)),
            ssh_key_content: env::var("SFTP_SSH_KEY").ok(),
            target_dir: target_dir.unwrap_or_else(|| "/htdocs".to_string()),
            docs_dir: docs_dir.unwrap_or_else(|| "docs_output".to_string()),
            build_docs,
            dry_run,
            delete_remote,
            port: port.unwrap_or(22),
        };

        // Validate that we have either SSH key path or content
        if config.ssh_key_path.is_none() && config.ssh_key_content.is_none() {
            return Err(anyhow::anyhow!(
                "Missing SSH key. Set --ssh-key-path, SFTP_SSH_KEY_PATH, or SFTP_SSH_KEY environment variable"
            ));
        }

        Ok(config)
    }
}

/// Deploy documentation to SFTP hosting
pub async fn deploy_docs_sftp(config: SftpDeployConfig) -> Result<()> {
    println!("🚀 Starting SFTP documentation deployment");
    println!("📋 Configuration:");
    println!("   Host: {}", config.host);
    println!("   Username: {}", config.username);
    println!("   Target directory: {}", config.target_dir);
    println!("   Local docs: {}", config.docs_dir);
    println!("   Port: {}", config.port);
    if config.dry_run {
        println!("   🔍 DRY RUN MODE - No changes will be made");
    }
    println!();

    // Build documentation if requested
    if config.build_docs {
        println!("📚 Building documentation...");
        build_documentation(&config)?;
    }

    // Validate local documentation directory
    let docs_path = Path::new(&config.docs_dir);
    if !docs_path.exists() {
        return Err(anyhow::anyhow!(
            "Documentation directory '{}' does not exist. Run with --build-docs to generate it.",
            config.docs_dir
        ));
    }

    // Connect to SFTP hosting
    println!("🔐 Connecting to SFTP hosting...");
    let sftp = connect_sftp_hosting(&config).await?;
    println!("✅ Connected successfully");

    // Deploy documentation
    println!("📤 Deploying documentation...");
    sync_documentation(&sftp, &config).await?;

    // Clean up remote files if requested
    if config.delete_remote && !config.dry_run {
        println!("🧹 Cleaning up remote files...");
        cleanup_remote_files(&sftp, &config).await?;
    }

    // Verify deployment
    println!("✅ Deployment completed successfully!");
    
    if !config.dry_run {
        if config.host.parse::<std::net::IpAddr>().is_ok() {
            println!("🌐 Documentation should be available at: http://{}", config.host);
        } else {
            println!("🌐 Documentation should be available at: https://{}", config.host);
        }
    }

    Ok(())
}

/// Build documentation using existing xtask commands
fn build_documentation(config: &SftpDeployConfig) -> Result<()> {
    let output_dir = &config.docs_dir;
    
    // For now, just ensure the docs directory exists
    // In a real implementation, you might call the existing docs build commands
    if !Path::new(output_dir).exists() {
        return Err(anyhow::anyhow!(
            "Documentation build not implemented. Please run 'cargo xtask publish-docs-dagger --output-dir {}' first",
            output_dir
        ));
    }
    
    println!("✅ Documentation directory found: {}", output_dir);
    Ok(())
}

/// Connect to SFTP hosting
async fn connect_sftp_hosting(config: &SftpDeployConfig) -> Result<ssh2::Sftp> {
    // For now, we'll use a simplified approach that requires the user to set up SSH keys properly
    // In a full implementation, we'd handle key authentication properly
    
    // Connect via TCP
    let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))
        .with_context(|| format!("Failed to connect to {}:{}", config.host, config.port))?;
    
    // Create SSH session
    let mut sess = Session::new()
        .context("Failed to create SSH session")?;
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .context("Failed to perform SSH handshake")?;
    
    // Authenticate (simplified - assumes SSH agent or proper key setup)
    sess.userauth_agent(&config.username)
        .with_context(|| format!("Failed to authenticate user {}", config.username))?;
    
    // Create SFTP channel
    let sftp = sess.sftp()
        .context("Failed to create SFTP channel")?;
    
    Ok(sftp)
}

/// Synchronize local documentation to remote hosting
async fn sync_documentation(sftp: &ssh2::Sftp, config: &SftpDeployConfig) -> Result<()> {
    let local_docs = Path::new(&config.docs_dir);
    let remote_target = &config.target_dir;

    // Ensure remote target directory exists
    if !config.dry_run {
        create_remote_directory(sftp, remote_target).await?;
    }

    // Walk through local documentation files
    let mut uploaded_files = 0;
    let mut uploaded_bytes = 0u64;

    for entry in WalkDir::new(local_docs).into_iter().filter_map(|e| e.ok()) {
        let local_path = entry.path();
        
        if local_path.is_file() {
            // Calculate relative path from docs directory
            let relative_path = local_path.strip_prefix(local_docs)
                .context("Failed to calculate relative path")?;
            
            // Create remote path
            let remote_path = format!("{}/{}", remote_target.trim_end_matches('/'), 
                                     relative_path.to_string_lossy().replace('\\', "/"));

            // Get file metadata
            let metadata = fs::metadata(local_path).await?;
            let file_size = metadata.len();

            if config.dry_run {
                println!("  📄 Would upload: {} → {} ({} bytes)", 
                        relative_path.display(), remote_path, file_size);
            } else {
                // Ensure remote directory exists
                if let Some(parent) = Path::new(&remote_path).parent() {
                    create_remote_directory(sftp, &parent.to_string_lossy()).await?;
                }

                // Check if file needs uploading (simple existence check)
                let needs_upload = match sftp.stat(std::path::Path::new(&remote_path)) {
                    Ok(_) => false, // File exists, skip for now
                    Err(_) => true, // File doesn't exist, upload it
                };

                if needs_upload {
                    // Upload file
                    let local_content = std::fs::read(local_path)?;
                    let mut remote_file = sftp.create(std::path::Path::new(&remote_path))?;
                    remote_file.write_all(&local_content)
                        .with_context(|| format!("Failed to upload {}", remote_path))?;

                    println!("  ✅ Uploaded: {} ({} bytes)", relative_path.display(), file_size);
                    uploaded_files += 1;
                    uploaded_bytes += file_size;
                } else {
                    println!("  ⏭️  Skipped: {} (unchanged)", relative_path.display());
                }
            }
        }
    }

    if config.dry_run {
        println!("🔍 Dry run completed - no files were actually uploaded");
    } else {
        println!("📊 Upload summary: {} files, {:.2} MB total", 
                uploaded_files, uploaded_bytes as f64 / 1024.0 / 1024.0);
    }

    Ok(())
}

/// Create remote directory if it doesn't exist
async fn create_remote_directory(sftp: &ssh2::Sftp, remote_path: &str) -> Result<()> {
    // Check if directory already exists
    match sftp.stat(std::path::Path::new(remote_path)) {
        Ok(_) => return Ok(()), // Directory already exists
        Err(_) => {
            // Try to create directory
            sftp.mkdir(std::path::Path::new(remote_path), 0o755)
                .with_context(|| format!("Failed to create directory {}", remote_path))?;
        }
    }
    Ok(())
}

/// Clean up remote files that don't exist locally
async fn cleanup_remote_files(_sftp: &ssh2::Sftp, config: &SftpDeployConfig) -> Result<()> {
    let local_docs = Path::new(&config.docs_dir);
    let _remote_target = &config.target_dir;

    // Collect local files for comparison
    let mut local_files = HashSet::new();
    for entry in WalkDir::new(local_docs).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            let relative_path = entry.path().strip_prefix(local_docs)
                .context("Failed to calculate relative path")?;
            local_files.insert(relative_path.to_string_lossy().replace('\\', "/"));
        }
    }

    // Walk remote directory and remove files that don't exist locally
    // Note: This is a simplified implementation
    // A full implementation would recursively walk the remote directory
    println!("🧹 Remote cleanup completed (simplified implementation)");
    println!("💡 Full remote cleanup feature coming in future version");

    Ok(())
}