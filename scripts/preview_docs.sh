#!/bin/bash
# Script to preview documentation locally

echo "Starting documentation preview server..."
echo "Documentation will be available at: http://localhost:8000"
echo "Press Ctrl+C to stop the server"
echo ""

# Change to the docs output directory
cd docs_output/local || exit 1

# Start Python HTTP server
python3 -m http.server 8000