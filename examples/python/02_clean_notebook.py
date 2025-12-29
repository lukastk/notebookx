#!/usr/bin/env python3
"""Cleaning notebooks with notebookx.

This example demonstrates:
- Removing outputs and execution counts
- Using preset cleaning options
- Cleaning notebooks for version control
"""

import json
from notebookx import Notebook, Format, CleanOptions

# Load a notebook with outputs
nb = Notebook.from_file("nb_format_examples/World population.ipynb")
print(f"Original notebook: {len(nb)} cells")

# Check original state (serialize to inspect)
original_json = json.loads(nb.to_string(Format.Ipynb))
code_cells = [c for c in original_json["cells"] if c["cell_type"] == "code"]
outputs_count = sum(len(c.get("outputs", [])) for c in code_cells)
exec_counts = sum(1 for c in code_cells if c.get("execution_count") is not None)
print(f"  - Cells with outputs: {sum(1 for c in code_cells if c.get('outputs'))}")
print(f"  - Total outputs: {outputs_count}")
print(f"  - Cells with execution count: {exec_counts}")

# Clean for version control (removes outputs and execution counts)
clean_nb = nb.clean(CleanOptions.for_vcs())

# Verify cleaning
clean_json = json.loads(clean_nb.to_string(Format.Ipynb))
clean_code_cells = [c for c in clean_json["cells"] if c["cell_type"] == "code"]
clean_outputs = sum(len(c.get("outputs", [])) for c in clean_code_cells)
clean_exec = sum(1 for c in clean_code_cells if c.get("execution_count") is not None)

print(f"\nAfter cleaning (for_vcs):")
print(f"  - Total outputs: {clean_outputs}")
print(f"  - Cells with execution count: {clean_exec}")

# Custom cleaning options
custom_options = CleanOptions(
    remove_outputs=True,
    remove_execution_counts=True,
    remove_cell_metadata=True,
)
custom_clean = nb.clean(custom_options)
print(f"\nCustom cleaning also works: {len(custom_clean)} cells")

# Strip all (most aggressive)
stripped = nb.clean(CleanOptions.strip_all())
stripped_json = json.loads(stripped.to_string(Format.Ipynb))
print(f"\nAfter strip_all:")
print(f"  - kernelspec present: {'kernelspec' in stripped_json.get('metadata', {})}")
