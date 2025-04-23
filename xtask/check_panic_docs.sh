#!/bin/bash
set -e

echo "Checking for undocumented panics across all crates..."

# Run clippy with the missing_panics_doc lint enabled
cargo clippy --workspace -- -W clippy::missing_panics_doc 

echo ""
echo "To fix any issues, add a '# Panics' section to the function documentation."
echo "Example:"
echo "/// # Panics"
echo "///"
echo "/// This function will panic if [condition]."
echo "///"
echo "/// Safety impact: [LOW|MEDIUM|HIGH]"
echo "/// Tracking: WRTQ-XXX"
echo ""
echo "See docs/PANIC_DOCUMENTATION.md for more details." 