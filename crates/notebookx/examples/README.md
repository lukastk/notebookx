# notebookx Examples

This directory contains examples demonstrating how to use the `notebookx` library.

## Prerequisites

Make sure you're in the repository root directory when running examples, as some examples rely on notebook files in `nb_format_examples/`.

## Running Examples

All examples can be run using cargo:

```bash
# From the repository root
cargo run --example <example_name>

# Or from the crates/notebookx directory
cd crates/notebookx
cargo run --example <example_name>
```

## Examples

### 00_basic_notebook

**Creating a notebook programmatically**

Demonstrates the basics of creating a new notebook from scratch:
- Creating an empty notebook
- Adding markdown and code cells
- Serializing to ipynb and percent formats

```bash
cargo run --example 00_basic_notebook
```

### 01_parse_ipynb

**Parsing an existing ipynb file**

Shows how to read and parse a Jupyter notebook file:
- Reading notebook files
- Accessing notebook metadata (kernel info, nbformat version)
- Iterating over cells and counting cell types
- Accessing cell source and outputs

```bash
cargo run --example 01_parse_ipynb
```

### 02_convert_formats

**Converting between notebook formats**

Demonstrates format conversion capabilities:
- Converting from ipynb to percent format
- Round-trip verification (ipynb → percent → ipynb)
- Format detection from file paths

```bash
cargo run --example 02_convert_formats
```

### 03_inspect_notebook

**Inspecting notebook contents in detail**

Deep dive into notebook internals:
- Examining notebook metadata
- Inspecting kernel and language information
- Analyzing cell outputs (ExecuteResult, DisplayData, Stream, Error)
- Working with MIME bundles

```bash
cargo run --example 03_inspect_notebook
```

### 04_modify_notebook

**Modifying notebook contents**

Shows how to manipulate notebooks:
- Adding cells at specific positions
- Removing cells
- Modifying cell content
- Filtering cells by type
- Creating derived notebooks (e.g., code-only version)
- Setting notebook metadata

```bash
cargo run --example 04_modify_notebook
```

### 05_clean_notebook

**Cleaning notebooks**

Demonstrates how to clean notebooks for version control:
- Removing outputs and execution counts
- Using preset cleaning options (`for_vcs`, `strip_all`)
- Custom cleaning with specific options
- Verifying immutability (original unchanged)

```bash
cargo run --example 05_clean_notebook
```

## Example Output

Running `00_basic_notebook` produces output like:

```
Created notebook with 5 cells:
  Cell 0: markdown (56 chars)
  Cell 1: code (22 chars)
  Cell 2: markdown (46 chars)
  Cell 3: code (52 chars)
  Cell 4: markdown (72 chars)

--- ipynb output (first 500 chars) ---
{
  "cells": [
    {
      "cell_type": "markdown",
      "metadata": {},
      "source": "# My First Notebook\n\nThis notebook was created with notebookx!"
    },
    ...

--- percent format output ---
# %% [markdown]
# # My First Notebook
#
# This notebook was created with notebookx!

# %%
print('Hello, World!')
...
```

## Adding New Examples

When adding new examples:

1. Create a new file named `NN_example_name.rs` where `NN` is the next number
2. Include a module-level doc comment explaining the example
3. Add a run instruction in the doc comment
4. Update this README with a description
