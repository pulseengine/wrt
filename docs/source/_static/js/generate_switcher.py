#!/usr/bin/env python3
"""
Generate a version switcher JSON file for the PyData Sphinx theme.
This script creates a switcher.json file at the root of the docs directory
that contains information about all available documentation versions.
"""

import json
import subprocess
import re
import os
import sys

def get_versions():
    """Get all available versions from git tags and the main branch."""
    versions = ['main']
    try:
        # Get all tags
        result = subprocess.run(['git', 'tag'], stdout=subprocess.PIPE, universal_newlines=True)
        if result.returncode == 0:
            # Only include semantic version tags (x.y.z)
            tags = result.stdout.strip().split('\n')
            for tag in tags:
                if re.match(r'^\d+\.\d+\.\d+$', tag):
                    versions.append(tag)
    except Exception as e:
        print(f"Error getting versions: {e}")
    
    # Sort versions (keep 'main' at the beginning)
    sorted_versions = sorted([v for v in versions if v != 'main'], 
                           key=lambda v: [int(x) for x in v.split('.')])
    sorted_versions.insert(0, 'main')
    
    return sorted_versions

def generate_switcher_json(output_path, is_local=False):
    """Generate the switcher.json file."""
    versions = get_versions()
    
    # Get the current version from environment or default to 'main'
    current_version = os.environ.get('DOCS_VERSION', 'main')
    
    # Determine if we're building for local or GitHub Pages
    base_url = "http://localhost:8080/" if is_local else "https://avrabe.github.io/wrt/"
    
    # Create version entries for the JSON file
    entries = []
    for version in versions:
        # Format name (e.g., "v1.0.0 (stable)" for the latest version)
        if version == 'main':
            name = "main (development)"
        elif version == versions[-1] and version != 'main':
            name = f"v{version} (stable)"
        else:
            name = f"v{version}"
        
        # Create entry
        entry = {
            "name": name,
            "version": version,
            "url": f"{base_url}{version}/"
        }
        
        # Mark the latest release as preferred (for warning banners)
        if version == versions[-1] and version != 'main':
            entry["preferred"] = True
            
        entries.append(entry)
    
    # Write the JSON file
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    with open(output_path, 'w') as f:
        json.dump(entries, f, indent=4)
    
    print(f"Generated switcher.json at {output_path}")
    print(f"Available versions: {', '.join(versions)}")
    print(f"Current version: {current_version}")

if __name__ == "__main__":
    # Determine if we're building for local or GitHub Pages
    is_local = len(sys.argv) > 1 and sys.argv[1] == "local"
    
    # Default output path
    output_dir = "docs/_build/versioned"
    os.makedirs(output_dir, exist_ok=True)
    
    generate_switcher_json(f"{output_dir}/switcher.json", is_local) 