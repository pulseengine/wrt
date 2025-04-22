#!/bin/bash
echo "Checking for missing panic documentation..."
cargo clippy --workspace -- -W clippy::missing_panics_doc | grep -A 5 "docs for function which may panic" || echo "No missing panic documentation found!"
echo -e "\nTo fix any issues, add a '# Panics' section to the function documentation."
echo "Example:"
echo "/// # Panics"
echo "///"
echo "/// This function will panic if [condition]." 