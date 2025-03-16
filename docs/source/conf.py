import os
import sys
import platform
sys.path.insert(0, os.path.abspath('../..'))

project = 'WRT'
copyright = '2024, WRT Contributors'
author = 'WRT Contributors'
release = '0.1.0'

extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.viewcode',
    'sphinx.ext.napoleon',
    'sphinx_needs',
    'myst_parser',
    'sphinxcontrib.plantuml',
    "sphinxcontrib_rust",
]

templates_path = ['_templates']
exclude_patterns = []

html_theme = 'sphinx_book_theme'
html_static_path = ['_static']

# PlantUML configuration
# Using the installed plantuml executable
plantuml = 'plantuml'
plantuml_output_format = 'svg'

# Make PlantUML work cross-platform
if platform.system() == "Windows":
    # Windows may need the full path to the plantuml.jar or plantuml.bat
    plantuml = os.environ.get('PLANTUML_PATH', 'plantuml')
elif platform.system() == "Darwin":  # macOS
    # macOS typically uses Homebrew installation
    plantuml = os.environ.get('PLANTUML_PATH', 'plantuml')
elif platform.system() == "Linux":
    # Linux installation path
    plantuml = os.environ.get('PLANTUML_PATH', 'plantuml')

# Allow customization through environment variables
plantuml_output_format = os.environ.get('PLANTUML_FORMAT', 'svg')

# Sphinx-needs configuration
needs_types = [
    dict(directive="req", title="Requirement", prefix="R_", color="#BFD8D2", style="node"),
    dict(directive="spec", title="Specification", prefix="S_", color="#FEDCD2", style="node"),
    dict(directive="impl", title="Implementation", prefix="I_", color="#DF744A", style="node"),
    dict(directive="test", title="Test Case", prefix="T_", color="#DCB239", style="node"),
]

needs_id_length = 7
needs_title_optional = True
needs_file_pattern = '**/*.rst' 

source_suffix = {
    ".rst": "restructuredtext",
    ".md": "markdown",
    ".txt": "markdown", # Optional
}

# See docs/compatibility for details on these extensions.
myst_enable_extensions = {
    "attrs_block",
    "colon_fence",
    "html_admonition",
    "replacements",
    "smartquotes",
    "strikethrough",
    "tasklist",
}
rust_crates = {
    "wrt": "wrt",
    "wrtd": "wrtd",
}
rust_doc_dir = "docs/source/"
rust_rustdoc_fmt = "md"