# CLAUDE.md - notebookx Contributor Guide

This document provides guidance for contributors (human and AI) working on the notebookx project.

---

## Project Goals

**notebookx** is a fast, lightweight notebook conversion tool that serves as a Rust-based alternative to Python's nbconvert.

### Primary Goals

1. **Performance**: Significantly faster than Python-based alternatives
2. **Correctness**: Lossless round-trip conversion where format allows
3. **Simplicity**: Clean, minimal API with sensible defaults
4. **Usability**: Available as Rust library, CLI tool, and Python package

### Non-Goals

1. **Feature parity with nbconvert**: We focus on core conversion, not export to PDF/HTML/LaTeX
2. **Notebook execution**: We don't execute notebooks, only convert them
3. **Every format**: Focus on high-value formats (ipynb, percent) before expanding
4. **GUI**: This is a library and CLI tool only

---

## Architectural Principles

### 1. Single Source of Truth

The `Notebook` struct is the canonical representation. All formats convert to/from this struct:

```
Format A → Notebook → Format B
```

Never convert directly between formats without going through `Notebook`.

### 2. Format Logic at the Edges

The core `Notebook` struct and its methods should be format-agnostic. Format-specific code lives in:
- `crates/notebookx/src/formats/ipynb.rs`
- `crates/notebookx/src/formats/percent.rs`
- etc.

The core module (`crates/notebookx/src/notebook.rs`) should never import format modules.

### 3. Immutability by Default

Operations like `clean()` return new instances rather than mutating:

```rust
// Good
fn clean(&self, options: CleanOptions) -> Notebook

// Bad
fn clean(&mut self, options: CleanOptions)
```

### 4. Explicit Over Implicit

When there's ambiguity (e.g., format detection), prefer explicit parameters with sensible defaults:

```rust
// Good - explicit with default
pub fn from_file(path: &Path, format: Option<NotebookFormat>) -> Result<Notebook>

// Infer if None, but allow override
```

### 5. Fail Fast with Helpful Errors

Use custom error types with context:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid JSON at line {line}: {message}")]
    InvalidJson { line: usize, message: String },

    #[error("Unknown cell type '{cell_type}' in cell {index}")]
    UnknownCellType { cell_type: String, index: usize },
}
```

---

## Coding Conventions

### Rust Style

- Follow standard Rust conventions (rustfmt, clippy)
- Use `thiserror` for error types
- Use `serde` for serialization
- Prefer `&str` over `String` in function parameters where possible
- Use builder pattern for complex configuration structs

### Naming

- Types: `PascalCase` (e.g., `Notebook`, `CellMetadata`)
- Functions/methods: `snake_case` (e.g., `from_file`, `to_string`)
- Constants: `SCREAMING_SNAKE_CASE`
- Format enum variants: Match common naming (e.g., `Ipynb`, `Percent`)

### Repository Structure

```
notebookx/                          # Repository root
├── Cargo.toml                      # Workspace configuration
├── pyproject.toml                  # Python package config (maturin)
├── python/
│   └── notebookx/
│       ├── __init__.py             # Re-exports from Rust extension
│       ├── __init__.pyi            # Type stubs
│       └── py.typed                # PEP 561 marker
├── crates/
│   ├── notebookx/                  # Core Rust library + CLI
│   │   └── src/
│   │       ├── lib.rs              # Public API re-exports
│   │       ├── main.rs             # CLI entry point (requires "cli" feature)
│   │       ├── cli.rs              # CLI implementation (optional)
│   │       ├── notebook.rs         # Core Notebook struct
│   │       ├── cell.rs             # Cell types
│   │       ├── output.rs           # Output types
│   │       ├── metadata.rs         # Metadata handling
│   │       ├── clean.rs            # Cleaning functionality
│   │       ├── error.rs            # Error types
│   │       └── formats/
│   │           ├── mod.rs          # Format enum and traits
│   │           ├── ipynb.rs        # ipynb JSON format
│   │           └── percent.rs      # Percent format
│   └── notebookx-py/               # PyO3 bindings crate
│       └── src/
│           └── lib.rs              # PyO3 module definition
├── tests/
│   └── python/                     # Python test suite (pytest)
│       └── test_notebookx.py
└── nb_format_examples/             # Example notebook files
```

### Testing

- Rust unit tests in the same file as the code (`#[cfg(test)]`)
- Rust integration tests in `crates/notebookx/tests/` directory
- Python tests in `tests/python/` using pytest
- Use `insta` for snapshot testing of serialized output
- Name tests descriptively: `test_parse_empty_notebook`, `test_clean_removes_outputs`

### Documentation

- All public items must have doc comments
- Include examples in doc comments for key APIs
- Use `#[doc(hidden)]` for internal-but-public items

---

## Adding New Formats

To add support for a new format (e.g., MyST Markdown):

### 1. Add Format Variant

```rust
// crates/notebookx/src/formats/mod.rs
pub enum NotebookFormat {
    Ipynb,
    Percent,
    Myst,  // Add new variant
}
```

### 2. Create Format Module

```rust
// crates/notebookx/src/formats/myst.rs
use crate::{Notebook, ParseError, SerializeError};

pub fn parse(input: &str) -> Result<Notebook, ParseError> {
    // Implementation
}

pub fn serialize(notebook: &Notebook, options: MystOptions) -> Result<String, SerializeError> {
    // Implementation
}

#[derive(Default)]
pub struct MystOptions {
    // Format-specific options
}
```

### 3. Register in Format Module

```rust
// crates/notebookx/src/formats/mod.rs
mod myst;

impl NotebookFormat {
    pub fn parse(&self, input: &str) -> Result<Notebook, ParseError> {
        match self {
            Self::Ipynb => ipynb::parse(input),
            Self::Percent => percent::parse(input),
            Self::Myst => myst::parse(input),
        }
    }
}
```

### 4. Add File Extension Mapping

```rust
impl NotebookFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "ipynb" => Some(Self::Ipynb),
            "pct.py" => Some(Self::Percent),
            "myst.md" => Some(Self::Myst),
            _ => None,
        }
    }
}
```

### 5. Add Tests

- Round-trip tests: parse → serialize → parse
- Cross-format tests: myst → ipynb → myst
- Edge case tests for format-specific features

### 6. Update CLI

Add format to CLI help text and argument parsing.

---

## Performance Expectations

### Targets

- Parse 1MB ipynb: < 10ms
- Serialize 1MB notebook: < 10ms
- Round-trip conversion: < 20ms
- Memory: ~2x input file size during conversion

### Guidelines

- Avoid unnecessary allocations in hot paths
- Use `Cow<str>` where ownership is conditional
- Pre-allocate vectors when size is known
- Profile before optimizing (use `criterion` benchmarks)

### What Not to Optimize

- Startup time (CLI is already fast)
- Tiny notebooks (< 10KB) - already instant
- Error paths - clarity over speed

---

## Correctness Requirements

### Round-Trip Preservation

When converting ipynb → format → ipynb:

**Must Preserve:**
- All cell content (source code, markdown text)
- Cell types (code, markdown, raw)
- Cell order
- Outputs (unless explicitly stripped)
- Execution counts (unless explicitly stripped)

**May Differ:**
- JSON formatting (whitespace)
- Metadata key ordering
- Array vs string source representation
- Trailing whitespace in cells

### Data Loss Warnings

When a conversion would lose data (e.g., outputs when converting to percent), either:
1. Preserve via format-specific mechanism (e.g., special comments)
2. Warn the user (CLI)
3. Require explicit confirmation (API via options)

Default behavior: preserve data where possible, require opt-in for lossy operations.

---

## Error Handling

### User-Facing Errors

CLI errors should be:
- Clear about what went wrong
- Suggest how to fix it
- Include relevant context (file path, line number)

```
Error: Failed to parse 'notebook.ipynb'
  → Invalid JSON at line 42: expected ',' or '}'

Hint: The file may be corrupted or not a valid Jupyter notebook.
```

### Library Errors

Return `Result<T, Error>` with typed errors:

```rust
pub enum NotebookError {
    Parse(ParseError),
    Serialize(SerializeError),
    Io(std::io::Error),
    UnsupportedFormat(String),
}
```

Never panic in library code. Use `expect()` only for invariants that indicate bugs.

---

## Python Bindings Guidelines

### API Consistency

Python API should feel Pythonic while maintaining Rust semantics:

```python
# Pythonic naming
nb = Notebook.from_file("example.ipynb")  # Not from_file()
clean_nb = nb.clean(remove_outputs=True)  # Keyword args

# But preserve immutability
new_nb = nb.clean(...)  # Returns new, doesn't mutate
```

### Error Mapping

Map Rust errors to appropriate Python exceptions:
- `ParseError` → `ValueError` with message
- `IoError` → `OSError`
- `SerializeError` → `ValueError`

### Type Hints

Provide complete type stubs (`.pyi` files) for IDE support.

---

## Commit and PR Guidelines

### Commit Early and Often

**Important**: Commit changes as you make them. Don't accumulate large uncommitted changes.

- Commit after completing each logical unit of work
- Commit after adding a new feature or fixing a bug
- Commit after adding tests for new functionality
- Commit after refactoring, even if small

This ensures:
- Progress is saved and can be reviewed incrementally
- Easier to bisect and find issues
- Clearer history of what changed and why
- Reduced risk of losing work

### Commit Messages

```
<type>: <short description>

<optional body with more detail>
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

### PR Checklist

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] New code has tests
- [ ] Public APIs have documentation
- [ ] CHANGELOG updated (for features/fixes)
- [ ] No unnecessary dependencies added

---

## Dependencies Policy

### Allowed Dependencies

- `serde`, `serde_json` - Serialization (essential)
- `thiserror` - Error handling
- `clap` - CLI parsing
- `pyo3` - Python bindings
- `regex` - Text parsing (if needed)

### Python Build Tools

- `maturin` - Build tool for PyO3 projects (configured in `pyproject.toml`)
- `pytest` - Python test framework (dev dependency)

### Avoid

- Heavy frameworks
- Async runtimes (not needed for file conversion)
- Unmaintained crates
- Crates with excessive transitive dependencies

### Evaluation Criteria

Before adding a dependency:
1. Is it well-maintained?
2. Does it have reasonable compile time?
3. Can we implement the needed functionality ourselves in < 100 lines?
4. Is it used by other major Rust projects?

---

## Release Process

1. Update version in `Cargo.toml` files
2. Update CHANGELOG.md
3. Create git tag: `v0.1.0`
4. CI builds and publishes to crates.io
5. CI builds and publishes Python wheels to PyPI
6. Create GitHub release with notes
