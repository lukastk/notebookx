# notebookx Project Roadmap

## Overview

notebookx is a Rust-based alternative to Python's nbconvert, providing fast, lightweight notebook conversion with support for Rust library, CLI (`nbx`), and Python bindings.

---

## Architecture Decisions

### Core Data Model

The `Notebook` struct is the central representation, closely mirroring the Jupyter `.ipynb` format (nbformat v4):

```
Notebook
├── cells: Vec<Cell>
├── metadata: NotebookMetadata
├── nbformat: u8 (always 4)
└── nbformat_minor: u8

Cell (enum)
├── Code
│   ├── source: String
│   ├── execution_count: Option<u32>
│   ├── outputs: Vec<Output>
│   └── metadata: CellMetadata
├── Markdown
│   ├── source: String
│   └── metadata: CellMetadata
└── Raw
    ├── source: String
    └── metadata: CellMetadata

Output (enum)
├── ExecuteResult { execution_count, data, metadata }
├── DisplayData { data, metadata }
├── Stream { name: stdout|stderr, text }
└── Error { ename, evalue, traceback }

MimeBundle: HashMap<String, MimeData>
MimeData: String | Vec<String> (for multi-line) | Base64 bytes
```

### Format Abstraction

All format-specific logic lives in separate modules:
- `formats/ipynb.rs` - JSON serialization/deserialization
- `formats/percent.rs` - Percent format parsing/generation

The core `Notebook` struct has no knowledge of specific formats.

### Conversion Strategy

```
Input File → Parser (format-specific) → Notebook → Serializer (format-specific) → Output File
```

Parsing and serialization are symmetric operations. Each format implements:
- `parse(input: &str) -> Result<Notebook, ParseError>`
- `serialize(notebook: &Notebook, options: FormatOptions) -> Result<String, SerializeError>`

---

## Milestones

### Milestone 1: Core Data Model & ipynb Support

**Implementation:**
- [ ] Define `Notebook`, `Cell`, `Output`, `Metadata` structs
- [ ] Implement serde serialization for ipynb JSON format
- [ ] Handle all output types: execute_result, display_data, stream, error
- [ ] Support MIME bundles with text, JSON, and base64 binary data

**Testing:**
- [ ] Unit tests for each struct's serialization/deserialization
- [ ] Test parsing of minimal valid notebook (empty cells array)
- [ ] Test parsing of notebook with all cell types (code, markdown, raw)
- [ ] Test parsing of all output types individually
- [ ] Test MIME bundle handling (text/plain, text/html, image/png base64)
- [ ] Test edge cases: empty cells, cells with only whitespace, unicode content
- [ ] Test error handling for malformed JSON
- [ ] Test error handling for invalid notebook structure
- [ ] Round-trip test: parse ipynb → serialize → parse → compare
- [ ] Integration test with `nb_format_examples/World population.ipynb`

### Milestone 2: Percent Format Support

**Implementation:**
- [ ] Implement percent format parser
  - [ ] YAML header extraction (optional, with defaults)
  - [ ] Cell delimiter parsing (`# %%`, `# %% [markdown]`, `# %% [raw]`)
  - [ ] Cell metadata parsing (`# %% tags=["hide"]`)
  - [ ] Markdown cell content (comment-prefixed lines)
- [ ] Implement percent format serializer
  - [ ] YAML header generation (configurable)
  - [ ] Cell delimiter generation
  - [ ] Proper comment wrapping for markdown

**Testing:**
- [ ] Unit tests for YAML header parsing (present, absent, malformed)
- [ ] Unit tests for cell delimiter parsing (all cell types)
- [ ] Unit tests for cell metadata extraction from delimiter line
- [ ] Unit tests for markdown comment prefix stripping/adding
- [ ] Test empty percent file (no cells)
- [ ] Test percent file with only code cells
- [ ] Test percent file with mixed cell types
- [ ] Test edge cases: empty lines between cells, trailing whitespace
- [ ] Test serialization options (header styles: full, minimal, none)
- [ ] Round-trip test: percent → Notebook → percent → compare
- [ ] Cross-format round-trip: ipynb → percent → ipynb (content preservation)
- [ ] Integration test with `nb_format_examples/World population.pct.py`

### Milestone 3: Clean Functionality

**Implementation:**
- [ ] Define `CleanOptions` struct with granular controls:
  - `remove_outputs: bool`
  - `remove_execution_counts: bool`
  - `remove_cell_metadata: bool`
  - `remove_notebook_metadata: bool`
  - `remove_kernel_info: bool`
  - `preserve_cell_ids: bool`
  - `allowed_metadata_keys: Option<Vec<String>>`
- [ ] Implement `Notebook::clean(options: CleanOptions) -> Notebook`
- [ ] Ensure clean creates a new copy, not mutation

**Testing:**
- [ ] Test each clean option individually (outputs, exec counts, cell meta, etc.)
- [ ] Test combinations of clean options
- [ ] Test that original notebook is unchanged after clean (immutability)
- [ ] Test clean with empty options (should return equivalent notebook)
- [ ] Test `allowed_metadata_keys` whitelist behavior
- [ ] Test idempotency: `clean(clean(nb, opts), opts) == clean(nb, opts)`
- [ ] Test clean on notebook with no outputs (no-op for remove_outputs)
- [ ] Test clean preserves cell content integrity
- [ ] Integration test: clean real notebook, verify outputs removed

### Milestone 4: CLI (`nbx`)

**Implementation:**
- [ ] Set up clap-based CLI structure
- [ ] Implement format inference from file extensions
- [ ] Commands:
  - `nbx <input> --to <output>` (convert with format inference)
  - `nbx <input> --from-fmt <fmt> --to <output> --to-fmt <fmt>` (explicit formats)
  - `nbx clean <input> [--output <output>] [--remove-outputs] [--remove-metadata] ...`
- [ ] Stdin/stdout support (`nbx - --from-fmt ipynb --to - --to-fmt percent`)
- [ ] Error handling with helpful messages
- [ ] Exit codes for scripting

**Testing:**
- [ ] Test format inference from file extensions (.ipynb, .pct.py)
- [ ] Test explicit format flags override inference
- [ ] Test conversion: ipynb → percent (file to file)
- [ ] Test conversion: percent → ipynb (file to file)
- [ ] Test stdin/stdout conversion
- [ ] Test clean command with each flag
- [ ] Test clean --in-place modifies file correctly
- [ ] Test error exit codes for: missing file, parse error, invalid args
- [ ] Test helpful error messages for common mistakes
- [ ] Test --help output for all commands
- [ ] End-to-end test: convert real notebook via CLI, verify output

### Milestone 5: Python Bindings

**Project Setup:**
- [ ] Create `pyproject.toml` at repository root (maturin config)
- [ ] Create `python/notebookx/` directory structure
- [ ] Create `crates/notebookx-py/` PyO3 crate
- [ ] Add `notebookx-py` to workspace members
- [ ] Set up `python/notebookx/__init__.py` with re-exports
- [ ] Create `python/notebookx/py.typed` marker file

**Implementation:**
- [ ] Implement PyO3 bindings in `crates/notebookx-py/src/lib.rs`
- [ ] Expose `Notebook` class:
  - `Notebook.from_file(path, format=None)`
  - `Notebook.from_string(content, format)`
  - `Notebook.to_file(path, format=None)`
  - `Notebook.to_string(format)`
  - `Notebook.clean(options=None)`
- [ ] Expose `CleanOptions` as Python dataclass/dict
- [ ] Expose format enum: `Format.IPYNB`, `Format.PERCENT`
- [ ] Convenience functions:
  - `convert(input_path, output_path, from_fmt=None, to_fmt=None)`
  - `clean_notebook(path, output_path=None, **options)`
- [ ] Python type stubs (`python/notebookx/__init__.pyi`)
- [ ] PyPI packaging via maturin

**Testing (in `tests/python/`):**
- [ ] Set up pytest configuration
- [ ] Test `Notebook.from_file()` with valid ipynb
- [ ] Test `Notebook.from_file()` with valid percent file
- [ ] Test `Notebook.from_string()` with both formats
- [ ] Test `Notebook.to_file()` writes correct content
- [ ] Test `Notebook.to_string()` returns correct string
- [ ] Test format inference in Python API
- [ ] Test `Notebook.clean()` with various options
- [ ] Test `CleanOptions` construction from kwargs
- [ ] Test error handling: FileNotFoundError, ValueError for parse errors
- [ ] Test `convert()` convenience function
- [ ] Test `clean_notebook()` convenience function
- [ ] Test type stubs are correct (mypy/pyright check)
- [ ] Integration test: round-trip through Python API

### Milestone 6: CI/CD & Wheel Building

**GitHub Actions CI:**
- [x] Create `.github/workflows/ci.yml` for continuous integration
  - Run Rust tests on push/PR
  - Run Python tests on push/PR
  - Test on ubuntu-latest, macos-latest, windows-latest

**Release Workflow & Wheel Building:**
- [x] Create `.github/workflows/release.yml` triggered on git tags
- [x] Build wheels for all major platforms using maturin:
  - Linux x86_64 (manylinux)
  - Linux ARM64 (manylinux)
  - macOS x86_64 (Intel)
  - macOS ARM64 (Apple Silicon)
  - Windows x86_64
- [x] Build source distribution (sdist)
- [x] Publish to PyPI on release
- [x] Publish to crates.io on release

**Notes:**
- Using `abi3-py38` means one wheel per platform works for Python 3.8+
- Use `maturin build --release` for optimized builds
- Use `maturin publish` for PyPI upload (requires PYPI_API_TOKEN secret)

### Milestone 7: Documentation

**Documentation:**
- [ ] README with installation and usage examples
- [ ] API documentation (rustdoc)
- [ ] Python docstrings and API docs
- [ ] Examples directory with common use cases

---

## Format Parsing Strategy

### ipynb Format

**Parsing:**
1. Deserialize JSON using serde_json
2. Map to internal structures with validation
3. Handle both string and array source formats (normalize to String internally)
4. Preserve unknown metadata fields as `serde_json::Value`

**Serialization:**
1. Serialize cells with source as array of lines (Jupyter convention)
2. Pretty-print JSON with 1-space indentation (matching Jupyter default)
3. Ensure trailing newline

**Edge Cases:**
- Empty cells
- Cells with only whitespace
- Binary outputs (base64 encoded)
- Very large outputs
- Malformed JSON (helpful error messages)

### Percent Format

**Parsing:**
1. Check for optional YAML header (`# ---` ... `# ---`)
2. Split on cell delimiters (`# %%`)
3. For each cell:
   - Parse cell type from delimiter (`[markdown]`, `[raw]`, or code)
   - Extract optional metadata from delimiter line
   - For markdown/raw: strip `# ` prefix from each line
   - For code: keep as-is
4. Infer kernel from YAML header or default to Python 3

**Serialization:**
1. Generate YAML header (configurable):
   - Full header with all metadata
   - Minimal header (kernelspec only)
   - No header
2. For each cell:
   - Write delimiter with type marker if needed
   - For markdown: prefix each line with `# `
   - For code: write source directly
3. Ensure single trailing newline

**Configuration Options:**
- `include_yaml_header: bool`
- `yaml_header_style: Full | Minimal | None`
- `preserve_outputs_as_comments: bool` (future)

---

## CLI Design

### Command Structure

```
nbx [OPTIONS] <INPUT> --to <OUTPUT>
nbx [OPTIONS] <INPUT> --from-fmt <FORMAT> --to <OUTPUT> --to-fmt <FORMAT>
nbx clean [OPTIONS] <INPUT> [--output <OUTPUT>]
```

### Format Inference

| Extension | Format |
|-----------|--------|
| `.ipynb` | ipynb |
| `.pct.py` | percent |
| `.py` (with `# %%`) | percent (detection) |

### Options

**Conversion:**
- `--from-fmt <FORMAT>` - Explicit input format
- `--to-fmt <FORMAT>` - Explicit output format
- `--strip-outputs` - Remove outputs during conversion
- `--strip-metadata` - Remove metadata during conversion

**Cleaning:**
- `--remove-outputs` / `-o`
- `--remove-execution-counts` / `-e`
- `--remove-cell-metadata`
- `--remove-notebook-metadata`
- `--remove-kernel-info`
- `--keep-only <keys>` - Whitelist specific metadata keys
- `--in-place` / `-i` - Modify file in place

### Exit Codes

- `0` - Success
- `1` - Parse error
- `2` - Serialization error
- `3` - I/O error
- `4` - Invalid arguments

---

## Python Bindings Plan

### Repository Structure for Python

```
notebookx/                      # Repository root
├── Cargo.toml                  # Workspace configuration
├── pyproject.toml              # Python package config (maturin)
├── python/
│   └── notebookx/
│       ├── __init__.py         # Re-exports from Rust extension
│       ├── __init__.pyi        # Type stubs
│       └── py.typed            # PEP 561 marker
├── crates/
│   ├── notebookx/              # Core Rust library
│   ├── nbx/                    # CLI binary
│   └── notebookx-py/           # PyO3 bindings crate
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs          # PyO3 module definition
└── tests/
    └── python/                 # Python test suite (pytest)
        └── test_notebookx.py
```

### Maturin Configuration

The `pyproject.toml` at root configures maturin to:
- Build the `notebookx-py` crate as a native extension
- Include the `python/notebookx/` package
- Generate wheels for multiple platforms

### API Design

```python
from notebookx import Notebook, Format, CleanOptions

# Load from file (format inferred)
nb = Notebook.from_file("example.ipynb")

# Load from string (format required)
nb = Notebook.from_string(content, Format.IPYNB)

# Convert to different format
percent_str = nb.to_string(Format.PERCENT)
nb.to_file("example.pct.py")

# Clean notebook
options = CleanOptions(
    remove_outputs=True,
    remove_execution_counts=True,
)
clean_nb = nb.clean(options)

# Convenience functions
from notebookx import convert, clean

convert("input.ipynb", "output.pct.py")
clean("notebook.ipynb", output="clean.ipynb", remove_outputs=True)
```

### PyO3 Implementation Notes

- Use `#[pyclass]` for `Notebook`, `CleanOptions`, `Format`
- Use `#[pymethods]` for instance methods
- Use `#[pyfunction]` for module-level convenience functions
- Return Python exceptions for Rust errors
- Support both Path objects and strings for file paths

---

## Testing Strategy

### Unit Tests

- Each struct/enum has serialization/deserialization tests
- Each format parser has dedicated test module
- Edge case coverage for malformed inputs
- Clean options tested individually and in combination

### Integration Tests

- Real notebook files from `nb_format_examples/`
- Round-trip conversion tests (A → B → A)
- Cross-format conversion tests (ipynb → percent → ipynb)
- CLI end-to-end tests

### Property-Based Tests

Using `proptest` or `quickcheck`:
- Arbitrary valid notebooks round-trip correctly
- Clean is idempotent: `clean(clean(nb)) == clean(nb)`
- Format conversions preserve cell content integrity

### Benchmarks

Using `criterion`:
- Parse time for various notebook sizes
- Serialize time for various notebook sizes
- Compare with Python nbconvert/jupytext (external benchmark)

---

## Incremental Priorities

### Phase 1: MVP (ipynb ↔ pct.py)
1. Core data model with unit tests
2. ipynb parsing/serialization with round-trip tests
3. Percent format parsing/serialization with round-trip tests
4. Cross-format integration tests
5. Basic CLI with format conversion and CLI tests
6. Basic cleaning (outputs, metadata) with cleaning tests

### Phase 2: Production Ready
1. Comprehensive error handling with error case tests
2. Full CLI feature set with end-to-end tests
3. Python bindings with Python test suite
4. Documentation
5. Benchmarks comparing with nbconvert/jupytext
6. PyPI/crates.io publishing

### Phase 3: Extended Formats (Future)
1. Light format (`.lgt.py`)
2. MyST Markdown (`.myst.md`)
3. Quarto (`.qmd`)
4. R Markdown (`.Rmd`)

---

## Open Questions / Future Considerations

1. **Cell IDs**: nbformat 4.5+ supports cell IDs. Should we generate them if missing?
2. **Validation**: Should we validate notebook structure strictly or be lenient?
3. **Streaming**: For very large notebooks, should we support streaming parse/serialize?
4. **Diff-friendly output**: Option to sort metadata keys for deterministic output?
5. **Widget state**: How to handle Jupyter widget state in metadata?
