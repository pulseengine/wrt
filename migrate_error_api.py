#!/usr/bin/env python3
"""
Error API Migration Script for WRT

This script migrates from the old Error API (Error::ErrorType) to the new API (Error::error_type()).
It handles the most common error patterns found in the WRT codebase.
"""

import os
import re
import sys
from pathlib import Path

# Error mappings from old API to new API
ERROR_MAPPINGS = {
    # Pattern: old_pattern -> (new_function, needs_message)
    r'Error::InvalidInput\("([^"]+)"\)': (r'Error::invalid_input("\1")', False),
    r'Error::InvalidInput\(([^)]+)\)': (r'Error::invalid_input("Invalid input")', False),
    r'Error::ComponentNotFound': ('Error::COMPONENT_NOT_FOUND', True),
    r'Error::OutOfMemory': ('Error::OUT_OF_MEMORY', True),
    r'Error::TooManyComponents': ('Error::TOO_MANY_COMPONENTS', True),
    r'Error::WitInputTooLarge': ('Error::WIT_INPUT_TOO_LARGE', True),
    r'Error::WitWorldLimitExceeded': ('Error::wit_world_limit_exceeded("WIT world limit exceeded")', False),
    r'Error::WitInterfaceLimitExceeded': ('Error::wit_interface_limit_exceeded("WIT interface limit exceeded")', False),
    r'Error::NoWitDefinitionsFound': ('Error::no_wit_definitions_found("No WIT definitions found")', False),
    r'Error::WitParseError\("([^"]+)"\)': (r'Error::wit_parse_error("\1")', False),
    
    # Foundation error mappings
    r'wrt_foundation::WrtError::InvalidInput\("([^"]+)"\.into\(\)\)': (r'wrt_foundation::Error::invalid_input("\1")', False),
    r'wrt_foundation::WrtError::InvalidInput\(([^)]+)\)': (r'wrt_foundation::Error::invalid_input("Invalid input")', False),
}

def migrate_file(file_path):
    """Migrate a single file from old error API to new API."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        changes_made = 0
        
        # Apply each error mapping
        for old_pattern, (new_replacement, needs_message) in ERROR_MAPPINGS.items():
            matches = re.findall(old_pattern, content)
            if matches:
                content = re.sub(old_pattern, new_replacement, content)
                changes_made += len(matches)
                print(f"  Fixed {len(matches)} instances of {old_pattern.split('::')[-1]}")
        
        # Write back if changes were made
        if changes_made > 0:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"✅ Migrated {file_path} ({changes_made} changes)")
            return True
        
        return False
        
    except Exception as e:
        print(f"❌ Error migrating {file_path}: {e}")
        return False

def migrate_directory(directory):
    """Migrate all Rust files in a directory."""
    directory = Path(directory)
    migrated_files = 0
    total_files = 0
    
    print(f"🔍 Scanning {directory} for Rust files...")
    
    for rust_file in directory.rglob("*.rs"):
        # Skip target directory and other generated files
        if "target" in str(rust_file) or ".git" in str(rust_file):
            continue
            
        total_files += 1
        if migrate_file(rust_file):
            migrated_files += 1
    
    print(f"📊 Migration summary: {migrated_files}/{total_files} files migrated")
    return migrated_files > 0

def main():
    """Main migration script."""
    if len(sys.argv) < 2:
        print("Usage: python migrate_error_api.py <crate_path>")
        print("Example: python migrate_error_api.py wrt-component")
        sys.exit(1)
    
    crate_path = sys.argv[1]
    if not os.path.exists(crate_path):
        print(f"❌ Path {crate_path} does not exist")
        sys.exit(1)
    
    print(f"🚀 Starting Error API migration for {crate_path}")
    print("=" * 50)
    
    success = migrate_directory(crate_path)
    
    if success:
        print("\n✅ Migration completed successfully!")
        print("Next steps:")
        print("1. Run 'cargo check' to verify compilation")
        print("2. Run 'cargo test' to verify functionality")
        print("3. Review changes and commit if everything works")
    else:
        print("\n⚠️  No migrations needed or migration failed")

if __name__ == "__main__":
    main()