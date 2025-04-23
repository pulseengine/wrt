#!/usr/bin/env python3
"""
Simple HTTP server for testing documentation locally with the version switcher.
This server serves the versioned documentation from docs/_build/versioned
and generates a local version of the switcher.json file.
"""

import os
import sys
import http.server
import socketserver
import subprocess

# Ensure we have the versioned docs directory
versioned_dir = 'docs/_build/versioned'
if not os.path.exists(versioned_dir):
    print(f"Error: {versioned_dir} does not exist.")
    print("Please build the documentation first with 'just docs-versioned'")
    sys.exit(1)

# Generate local switcher.json
print("Generating local switcher.json...")
subprocess.run([sys.executable, 'docs/source/_static/js/generate_switcher.py', 'local'])

# Set up the server
PORT = 8080
DIRECTORY = versioned_dir

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=DIRECTORY, **kwargs)
    
    def log_message(self, format, *args):
        if args[0].startswith('GET /switcher.json'):
            # Don't log switcher.json requests
            return
        super().log_message(format, *args)

print(f"Starting local server at http://localhost:{PORT}")
print(f"Serving documentation from {DIRECTORY}")
print("Press Ctrl+C to stop the server")

with socketserver.TCPServer(("", PORT), Handler) as httpd:
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nServer stopped.")
        sys.exit(0) 