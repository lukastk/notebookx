#!/usr/bin/env python3
"""Converting notebook files with notebookx.

This example demonstrates:
- Using the convert() convenience function
- Using the clean_notebook() convenience function
- File format inference
"""

import os
import tempfile
from notebookx import convert, clean_notebook, Notebook, Format

# Create a temporary directory for output files
with tempfile.TemporaryDirectory() as tmpdir:
    # Convert ipynb to percent format
    input_file = "nb_format_examples/World population.ipynb"
    percent_output = os.path.join(tmpdir, "notebook.pct.py")

    convert(input_file, percent_output)
    print(f"Converted {input_file} -> {percent_output}")

    # Read and show first few lines
    with open(percent_output) as f:
        content = f.read()
    print(f"\nPercent format preview (first 300 chars):")
    print(content[:300])

    # Convert back to ipynb
    ipynb_output = os.path.join(tmpdir, "restored.ipynb")
    convert(percent_output, ipynb_output)
    print(f"\nConverted back: {percent_output} -> {ipynb_output}")

    # Verify round-trip
    original = Notebook.from_file(input_file)
    restored = Notebook.from_file(ipynb_output)
    print(f"Cell count: {len(original)} -> {len(restored)}")

    # Clean a notebook to a new file
    clean_output = os.path.join(tmpdir, "clean.ipynb")
    clean_notebook(input_file, clean_output, remove_outputs=True)
    print(f"\nCleaned notebook saved to: {clean_output}")

    # Verify cleaning
    clean_nb = Notebook.from_file(clean_output)
    import json
    clean_json = json.loads(clean_nb.to_string(Format.Ipynb))
    code_cells = [c for c in clean_json["cells"] if c["cell_type"] == "code"]
    has_outputs = any(c.get("outputs") for c in code_cells)
    print(f"Clean notebook has outputs: {has_outputs}")

print("\nDone!")
