# notebookx

A fast, lightweight notebook conversion library written in Rust with Python bindings.

notebookx is a Rust-based alternative to Python's nbconvert, providing fast notebook conversion between formats like `.ipynb` and percent format (`.pct.py`).

## Features

- **Fast**: Written in Rust for maximum performance
- **Multiple interfaces**: Use as a Rust library, CLI tool (`nbx`), or Python package
- **Format conversion**: Convert between ipynb and percent format
- **Notebook cleaning**: Strip outputs, execution counts, and metadata for version control
- **Cross-platform**: Works on Linux, macOS, and Windows

## Installation

### Python

```bash
pip install notebookx-py
```

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
notebookx = "0.1"
```

### CLI

```bash
cargo install notebookx --features cli
```

## Usage

### Python

```python
from notebookx import Notebook, Format, CleanOptions

# Load a notebook
nb = Notebook.from_file("example.ipynb")

# Convert to percent format
nb.to_file("example.pct.py")

# Or get as string
percent_content = nb.to_string(Format.Percent)

# Clean notebook for version control
clean_nb = nb.clean(CleanOptions(
    remove_outputs=True,
    remove_execution_counts=True,
))
clean_nb.to_file("clean.ipynb")

# Convenience functions
from notebookx import convert, clean_notebook

convert("input.ipynb", "output.pct.py")
clean_notebook("notebook.ipynb", remove_outputs=True)
```

### CLI (`nbx`)

```bash
# Convert ipynb to percent format
nbx convert notebook.ipynb --to notebook.pct.py

# Convert percent to ipynb
nbx convert notebook.pct.py --to notebook.ipynb

# Clean a notebook (remove outputs)
nbx clean notebook.ipynb --output clean.ipynb --remove-outputs

# Clean in place
nbx clean notebook.ipynb --in-place --remove-outputs --remove-execution-counts

# Use stdin/stdout
cat notebook.ipynb | nbx convert - --from-fmt ipynb --to - --to-fmt percent
```

### Rust

```rust
use notebookx::{Notebook, NotebookFormat, CleanOptions};

// Parse from file
let content = std::fs::read_to_string("example.ipynb")?;
let notebook = NotebookFormat::Ipynb.parse(&content)?;

// Convert to percent format
let percent = NotebookFormat::Percent.serialize(&notebook)?;

// Clean notebook
let options = CleanOptions {
    remove_outputs: true,
    remove_execution_counts: true,
    ..Default::default()
};
let clean = notebook.clean(&options);

// Save to file
let output = NotebookFormat::Ipynb.serialize(&clean)?;
std::fs::write("clean.ipynb", output)?;
```

## Supported Formats

| Format | Extension | Description |
|--------|-----------|-------------|
| ipynb | `.ipynb` | Standard Jupyter notebook format (JSON) |
| percent | `.pct.py` | Percent format used by Jupytext, VSCode, etc. |

## Clean Options

When cleaning notebooks, you can control what gets removed:

| Option | Description |
|--------|-------------|
| `remove_outputs` | Remove all cell outputs |
| `remove_execution_counts` | Reset execution counts to null |
| `remove_cell_metadata` | Remove cell-level metadata |
| `remove_notebook_metadata` | Remove notebook-level metadata |
| `remove_kernel_info` | Remove kernel specification |
| `preserve_cell_ids` | Keep cell IDs (default: regenerate) |
| `remove_output_metadata` | Remove metadata from outputs |
| `remove_output_execution_counts` | Remove execution counts from outputs |

### Presets

**Python:**
```python
# For version control (removes cell metadata, execution counts, and output metadata)
# Preserves outputs so rendered content remains visible
options = CleanOptions.for_vcs()

# Strip everything (including outputs)
options = CleanOptions.strip_all()
```

## CLI Reference

### `nbx convert`

Convert notebooks between formats.

```
nbx convert <INPUT> --to <OUTPUT> [OPTIONS]

Options:
  --from-fmt <FORMAT>  Input format (ipynb, percent). Inferred from extension if not specified.
  --to-fmt <FORMAT>    Output format (ipynb, percent). Inferred from extension if not specified.
  -h, --help           Print help
```

### `nbx clean`

Clean notebooks by removing outputs and metadata.

```
nbx clean <INPUT> [OPTIONS]

Options:
  -o, --output <FILE>           Output file (default: stdout)
  -i, --in-place                Modify file in place
  -O, --remove-outputs          Remove all outputs
  -e, --remove-execution-counts Remove execution counts
  --remove-cell-metadata        Remove cell metadata
  --remove-notebook-metadata    Remove notebook metadata
  --remove-kernel-info          Remove kernel specification
  -h, --help                    Print help
```

## Development

### Building from source

```bash
# Clone the repository
git clone https://github.com/lukastk/notebookx.git
cd notebookx

# Build Rust library and CLI
cargo build --release

# Build Python package
pip install maturin
maturin develop
```

### Running tests

```bash
# Rust tests
cargo test --workspace

# Python tests
pytest tests/python
```

## License

MIT

## Acknowledgements

Example notebooks in `nb_format_examples/` are from the [Jupytext demo](https://github.com/mwouts/jupytext/tree/main/demo).
