#!/usr/bin/env python3
"""Basic usage of the notebookx Python library.

This example demonstrates:
- Loading notebooks from files
- Converting between formats
- Basic notebook inspection
"""

from notebookx import Notebook, Format

# Load a notebook from file
nb = Notebook.from_file("nb_format_examples/World population.ipynb")

# Inspect the notebook
print(f"Loaded notebook with {len(nb)} cells")
print(f"  - Code cells: {nb.code_cell_count}")
print(f"  - Markdown cells: {nb.markdown_cell_count}")
print(f"  - Raw cells: {nb.raw_cell_count}")
print(f"  - nbformat: {nb.nbformat}.{nb.nbformat_minor}")

# Convert to percent format
percent_content = nb.to_string(Format.Percent)
print(f"\n--- Percent format (first 500 chars) ---")
print(percent_content[:500])

# Convert back to ipynb
nb2 = Notebook.from_string(percent_content, Format.Percent)
print(f"\nRound-trip: {len(nb)} cells -> {len(nb2)} cells")
