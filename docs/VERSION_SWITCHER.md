# Documentation Version Switcher

This project uses the PyData Sphinx theme's version switcher feature for documentation. This allows users to switch between different versions of the documentation easily.

## How It Works

1. The switcher uses a JSON file (`switcher.json`) that defines all available versions
2. This file is generated from git tags (for releases) and the main branch
3. The root index.html always redirects to the main version
4. Each version's documentation is built separately and stored in versioned directories

## Commands

The documentation system provides the following commands:

### Building Documentation

```bash
# Build documentation for a specific version
just docs-versioned main  # For main branch
just docs-versioned 1.0.0  # For a specific tag

# Serve documentation locally with version switcher support
just docs-serve
```

### Underlying `xtask` Commands

These commands can be used directly if needed:

```bash
# Generate the switcher.json file
cargo xtask docs switcher-json

# Generate the switcher.json file for local development (localhost:8080)
cargo xtask docs switcher-json --local

# Start a local HTTP server for documentation
cargo xtask docs serve
```

## Implementation Details

The version switcher is implemented in the following files:

1. **xtask/src/docs.rs** - Rust module that handles:
   - Generating the switcher.json file based on git tags
   - Serving the documentation locally using a simple HTTP server

2. **docs/source/conf.py** - Configuration of the PyData Sphinx theme:
   ```python
   html_theme_options = {
       "switcher": {
           "json_url": "switcher.json",
           "version_match": current_version,
       },
       "navbar_start": ["navbar-logo", "version-switcher"],
   }
   ```

3. **justfile** - Contains tasks for building and serving documentation:
   - `docs-versioned` - Builds documentation for a specific version
   - `docs-serve` - Starts a local server for preview

4. **.github/workflows/publish.yml** - Handles building and publishing documentation for all versions

## Version Format

The versions are formatted as follows:
- **main** → "main (development)"
- **Latest release** → "v1.0.0 (stable)"
- **Other releases** → "v0.9.0"

The latest release is also marked as "preferred" for warning banners. 