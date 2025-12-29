# Python Examples

This directory contains examples demonstrating how to use the `notebookx` Python library.

## Prerequisites

1. Install the notebookx package:
   ```bash
   pip install notebookx
   ```

   Or for development:
   ```bash
   pip install maturin
   maturin develop
   ```

2. Run examples from the repository root directory (examples use notebook files from `nb_format_examples/`).

## Examples

### 01_basic_usage.py

**Basic usage of the notebookx Python library**

Demonstrates:
- Loading notebooks from files
- Converting between formats
- Basic notebook inspection

```bash
python examples/python/01_basic_usage.py
```

### 02_clean_notebook.py

**Cleaning notebooks with notebookx**

Demonstrates:
- Removing outputs and execution counts
- Using preset cleaning options (`for_vcs`, `strip_all`)
- Cleaning notebooks for version control

```bash
python examples/python/02_clean_notebook.py
```

### 03_convert_files.py

**Converting notebook files with notebookx**

Demonstrates:
- Using the `convert()` convenience function
- Using the `clean_notebook()` convenience function
- File format inference

```bash
python examples/python/03_convert_files.py
```

## Quick Start

```python
from notebookx import Notebook, Format, CleanOptions

# Load a notebook
nb = Notebook.from_file("notebook.ipynb")

# Convert to percent format
nb.to_file("notebook.pct.py")

# Or get as string
content = nb.to_string(Format.Percent)

# Clean for version control
clean_nb = nb.clean(CleanOptions.for_vcs())
clean_nb.to_file("clean.ipynb")
```
