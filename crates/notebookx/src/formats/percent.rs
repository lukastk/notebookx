//! Python percent format (.pct.py) parser and serializer.
//!
//! The percent format is a plain-text representation of Jupyter notebooks
//! where cells are delimited by `# %%` markers. This format is compatible
//! with Jupytext and various IDEs (VS Code, PyCharm, Spyder).
//!
//! # Format Structure
//!
//! ```text
//! # ---
//! # jupyter:
//! #   kernelspec:
//! #     name: python3
//! # ---
//!
//! # %% [markdown]
//! # # Title
//! # Some markdown content
//!
//! # %%
//! print("Hello, world!")
//!
//! # %% [raw]
//! # Raw cell content
//! ```

use crate::cell::{Cell, CellMetadata};
use crate::metadata::{KernelSpec, NotebookMetadata};
use crate::notebook::Notebook;
use crate::output::MultilineString;
use crate::{ParseError, SerializeError};
use std::collections::HashMap;

/// Options for serializing to percent format.
#[derive(Debug, Clone, Default)]
pub struct PercentOptions {
    /// Style for the YAML header.
    pub header_style: HeaderStyle,
}

/// Style for the YAML header in percent format.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HeaderStyle {
    /// Include full metadata in the header.
    #[default]
    Full,
    /// Include only kernelspec in the header.
    Minimal,
    /// No header at all.
    None,
}

/// Parse a notebook from percent format.
pub fn parse(input: &str) -> Result<Notebook, ParseError> {
    let (metadata, content) = parse_header(input)?;
    let cells = parse_cells(content)?;

    Ok(Notebook {
        cells,
        metadata,
        nbformat: 4,
        nbformat_minor: 5,
    })
}

/// Serialize a notebook to percent format.
pub fn serialize(notebook: &Notebook) -> Result<String, SerializeError> {
    serialize_with_options(notebook, &PercentOptions::default())
}

/// Serialize a notebook to percent format with options.
pub fn serialize_with_options(
    notebook: &Notebook,
    options: &PercentOptions,
) -> Result<String, SerializeError> {
    let mut output = String::new();

    // Write header if requested
    if options.header_style != HeaderStyle::None {
        let header = serialize_header(&notebook.metadata, options.header_style)?;
        if !header.is_empty() {
            output.push_str(&header);
            output.push('\n');
        }
    }

    // Write cells
    for (i, cell) in notebook.cells.iter().enumerate() {
        if i > 0 || !output.is_empty() {
            // Add blank line before cell delimiter (but not at start of file with no header)
            if !output.is_empty() && !output.ends_with("\n\n") {
                output.push('\n');
            }
        }

        match cell {
            Cell::Code {
                source, metadata, ..
            } => {
                output.push_str(&format_cell_delimiter("", metadata));
                output.push('\n');
                let src = source.as_string();
                if !src.is_empty() {
                    output.push_str(&src);
                    if !src.ends_with('\n') {
                        output.push('\n');
                    }
                }
            }
            Cell::Markdown {
                source, metadata, ..
            } => {
                output.push_str(&format_cell_delimiter("[markdown]", metadata));
                output.push('\n');
                let src = source.as_string();
                if !src.is_empty() {
                    output.push_str(&comment_lines(&src));
                    if !output.ends_with('\n') {
                        output.push('\n');
                    }
                }
            }
            Cell::Raw {
                source, metadata, ..
            } => {
                output.push_str(&format_cell_delimiter("[raw]", metadata));
                output.push('\n');
                let src = source.as_string();
                if !src.is_empty() {
                    output.push_str(&comment_lines(&src));
                    if !output.ends_with('\n') {
                        output.push('\n');
                    }
                }
            }
        }
    }

    Ok(output)
}

// ============================================================================
// Header parsing and serialization
// ============================================================================

/// Parse the YAML header from percent format.
///
/// Returns the parsed metadata and the remaining content after the header.
fn parse_header(input: &str) -> Result<(NotebookMetadata, &str), ParseError> {
    let trimmed = input.trim_start();

    // Check if there's a header
    if !trimmed.starts_with("# ---") {
        // No header, use defaults
        return Ok((default_metadata(), input));
    }

    // Find the end of the header
    let header_start = input.find("# ---").unwrap();
    let after_first_marker = &input[header_start + 5..];

    // Find the closing marker
    let header_end =
        after_first_marker
            .find("\n# ---")
            .ok_or_else(|| ParseError::InvalidStructure {
                message: "YAML header not closed (missing '# ---')".to_string(),
            })?;

    let header_content = &after_first_marker[..header_end];
    let remaining = &after_first_marker[header_end + 6..]; // Skip "\n# ---"

    // Skip any trailing newlines after the header
    let remaining = remaining.trim_start_matches('\n');

    // Parse the YAML content
    let metadata = parse_yaml_header(header_content)?;

    Ok((metadata, remaining))
}

/// Parse YAML header content into NotebookMetadata.
fn parse_yaml_header(content: &str) -> Result<NotebookMetadata, ParseError> {
    // Remove the `# ` prefix from each line
    let yaml_lines: Vec<&str> = content
        .lines()
        .map(|line| {
            if let Some(stripped) = line.strip_prefix("# ") {
                stripped
            } else if let Some(stripped) = line.strip_prefix("#") {
                stripped
            } else {
                line
            }
        })
        .collect();

    let yaml_str = yaml_lines.join("\n");

    // Parse as JSON (serde_json can parse simple YAML-like structures)
    // For full YAML support, we'd need the `serde_yaml` crate
    // For now, we'll parse the structure manually for the key fields we care about
    parse_simple_yaml_metadata(&yaml_str)
}

/// Parse simple YAML-like metadata structure.
///
/// This is a simplified parser that handles the common jupytext header format.
/// For full YAML support, consider adding the `serde_yaml` crate.
fn parse_simple_yaml_metadata(yaml: &str) -> Result<NotebookMetadata, ParseError> {
    let mut metadata = NotebookMetadata::default();

    let mut kernelspec_name = None;
    let mut kernelspec_display_name = None;
    let mut kernelspec_language = None;

    let mut in_kernelspec = false;

    for line in yaml.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("kernelspec:") {
            in_kernelspec = true;
            continue;
        }

        if in_kernelspec {
            if !line.starts_with("    ") && !line.starts_with("\t") && !trimmed.is_empty() {
                // Exited kernelspec section
                in_kernelspec = false;
            } else if let Some(rest) = trimmed.strip_prefix("name:") {
                kernelspec_name = Some(rest.trim().to_string());
            } else if let Some(rest) = trimmed.strip_prefix("display_name:") {
                kernelspec_display_name = Some(rest.trim().to_string());
            } else if let Some(rest) = trimmed.strip_prefix("language:") {
                kernelspec_language = Some(rest.trim().to_string());
            }
        }
    }

    // Build kernelspec if we have the required fields
    if kernelspec_name.is_some() || kernelspec_display_name.is_some() {
        metadata.kernelspec = Some(KernelSpec {
            name: kernelspec_name.unwrap_or_else(|| "python3".to_string()),
            display_name: kernelspec_display_name.unwrap_or_else(|| "Python 3".to_string()),
            language: kernelspec_language.unwrap_or_else(|| "python".to_string()),
        });
    }

    Ok(metadata)
}

/// Create default metadata for notebooks without a header.
fn default_metadata() -> NotebookMetadata {
    NotebookMetadata {
        kernelspec: Some(KernelSpec {
            name: "python3".to_string(),
            display_name: "Python 3".to_string(),
            language: "python".to_string(),
        }),
        language_info: None,
        extra: HashMap::new(),
    }
}

/// Serialize notebook metadata to a YAML header.
fn serialize_header(
    metadata: &NotebookMetadata,
    style: HeaderStyle,
) -> Result<String, SerializeError> {
    if style == HeaderStyle::None {
        return Ok(String::new());
    }

    let mut lines = vec!["# ---".to_string()];

    if let Some(ref kernelspec) = metadata.kernelspec {
        lines.push("# jupyter:".to_string());
        lines.push("#   kernelspec:".to_string());
        lines.push(format!("#     display_name: {}", kernelspec.display_name));
        lines.push(format!("#     language: {}", kernelspec.language));
        lines.push(format!("#     name: {}", kernelspec.name));
    }

    lines.push("# ---".to_string());

    Ok(lines.join("\n"))
}

// ============================================================================
// Cell parsing
// ============================================================================

/// Parse cells from the content after the header.
fn parse_cells(content: &str) -> Result<Vec<Cell>, ParseError> {
    let mut cells = Vec::new();

    // Split on cell delimiters
    let parts: Vec<&str> = split_on_cell_markers(content);

    for part in parts {
        if part.trim().is_empty() {
            continue;
        }

        let cell = parse_cell(part)?;
        cells.push(cell);
    }

    Ok(cells)
}

/// Split content on `# %%` markers, keeping the marker with its content.
fn split_on_cell_markers(content: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut last_end = 0;

    // Find all `# %%` markers at the start of a line
    for (i, _) in content.match_indices("\n# %%") {
        if last_end < i {
            let part = &content[last_end..i];
            if !part.trim().is_empty() {
                parts.push(part);
            }
        }
        last_end = i + 1; // Skip the newline, keep `# %%`
    }

    // Check if content starts with `# %%`
    if content.starts_with("# %%") {
        // Handle the case where the first cell starts at the beginning
        if last_end == 0 {
            parts.push(content);
        } else {
            // Add the remaining content
            if last_end < content.len() {
                parts.push(&content[last_end..]);
            }
        }
    } else if last_end == 0 {
        // No markers found, treat entire content as one part (if not empty)
        if !content.trim().is_empty() {
            // Content before any marker - could be code without explicit marker
            parts.push(content);
        }
    } else {
        // Add remaining content after last marker
        if last_end < content.len() {
            parts.push(&content[last_end..]);
        }
    }

    parts
}

/// Parse a single cell from its content (including the `# %%` marker line).
fn parse_cell(content: &str) -> Result<Cell, ParseError> {
    let content = content.trim_start_matches('\n');

    // Check if it starts with a cell marker
    if !content.starts_with("# %%") {
        // No marker - treat as code cell
        return Ok(Cell::Code {
            source: MultilineString::from_string(content.to_string()),
            execution_count: None,
            outputs: Vec::new(),
            metadata: CellMetadata::default(),
            id: None,
        });
    }

    // Parse the marker line
    let first_newline = content.find('\n').unwrap_or(content.len());
    let marker_line = &content[..first_newline];
    let cell_content = if first_newline < content.len() {
        &content[first_newline + 1..]
    } else {
        ""
    };

    // Parse cell type and metadata from marker
    let (cell_type, metadata) = parse_cell_marker(marker_line)?;

    // Build the cell
    match cell_type {
        CellType::Code => {
            // Remove trailing empty lines but preserve internal structure
            let source = cell_content.trim_end_matches('\n').to_string();
            Ok(Cell::Code {
                source: MultilineString::from_string(source),
                execution_count: None,
                outputs: Vec::new(),
                metadata,
                id: None,
            })
        }
        CellType::Markdown => {
            let source = uncomment_lines(cell_content);
            Ok(Cell::Markdown {
                source: MultilineString::from_string(source),
                metadata,
                id: None,
            })
        }
        CellType::Raw => {
            let source = uncomment_lines(cell_content);
            Ok(Cell::Raw {
                source: MultilineString::from_string(source),
                metadata,
                id: None,
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellType {
    Code,
    Markdown,
    Raw,
}

/// Parse a cell marker line like `# %% [markdown] tags=["hide"]`.
fn parse_cell_marker(line: &str) -> Result<(CellType, CellMetadata), ParseError> {
    let line = line.trim();

    // Remove the `# %%` prefix
    let rest = line
        .strip_prefix("# %%")
        .ok_or_else(|| ParseError::InvalidStructure {
            message: format!("Invalid cell marker: {}", line),
        })?
        .trim();

    // Determine cell type
    let (cell_type, rest) = if rest.starts_with("[markdown]") {
        (
            CellType::Markdown,
            rest.strip_prefix("[markdown]").unwrap().trim(),
        )
    } else if rest.starts_with("[md]") {
        (
            CellType::Markdown,
            rest.strip_prefix("[md]").unwrap().trim(),
        )
    } else if rest.starts_with("[raw]") {
        (CellType::Raw, rest.strip_prefix("[raw]").unwrap().trim())
    } else {
        (CellType::Code, rest)
    };

    // Parse metadata from remaining content (e.g., `tags=["hide"]`)
    let metadata = parse_cell_metadata(rest)?;

    Ok((cell_type, metadata))
}

/// Parse cell metadata from the marker line (after cell type).
fn parse_cell_metadata(content: &str) -> Result<CellMetadata, ParseError> {
    let mut metadata = CellMetadata::default();

    if content.is_empty() {
        return Ok(metadata);
    }

    // Simple parsing for common metadata patterns
    // Format: key=value or key="value" or key=['a', 'b']

    // Look for tags
    if let Some(tags_start) = content.find("tags=") {
        let rest = &content[tags_start + 5..];
        if rest.starts_with('[') {
            if let Some(end) = rest.find(']') {
                let tags_str = &rest[1..end];
                let tags: Vec<String> = tags_str
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !tags.is_empty() {
                    metadata.tags = Some(tags);
                }
            }
        }
    }

    Ok(metadata)
}

// ============================================================================
// Comment handling
// ============================================================================

/// Add `# ` prefix to each line for markdown/raw cells.
fn comment_lines(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            if line.is_empty() {
                "#".to_string()
            } else {
                format!("# {}", line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Remove `# ` prefix from each line for markdown/raw cells.
fn uncomment_lines(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    // Find where the actual content ends (trim trailing empty comment lines)
    let mut end = lines.len();
    while end > 0 && (lines[end - 1] == "#" || lines[end - 1] == "# " || lines[end - 1].is_empty())
    {
        end -= 1;
    }

    lines[..end]
        .iter()
        .map(|line| {
            if let Some(stripped) = line.strip_prefix("# ") {
                stripped
            } else if *line == "#" {
                ""
            } else if let Some(stripped) = line.strip_prefix("#") {
                stripped
            } else {
                *line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format a cell delimiter line.
fn format_cell_delimiter(cell_type_marker: &str, metadata: &CellMetadata) -> String {
    let mut result = if cell_type_marker.is_empty() {
        "# %%".to_string()
    } else {
        format!("# %% {}", cell_type_marker)
    };

    // Add metadata if present
    if let Some(ref tags) = metadata.tags {
        if !tags.is_empty() {
            let tags_str: Vec<String> = tags.iter().map(|t| format!("\"{}\"", t)).collect();
            result.push_str(&format!(" tags=[{}]", tags_str.join(", ")));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal() {
        let input = "# %%\nprint('hello')";
        let notebook = parse(input).unwrap();

        assert_eq!(notebook.cells.len(), 1);
        assert!(notebook.cells[0].is_code());
        assert_eq!(notebook.cells[0].source_string(), "print('hello')");
    }

    #[test]
    fn test_parse_markdown_cell() {
        let input = "# %% [markdown]\n# # Title\n# Some text";
        let notebook = parse(input).unwrap();

        assert_eq!(notebook.cells.len(), 1);
        assert!(notebook.cells[0].is_markdown());
        assert_eq!(notebook.cells[0].source_string(), "# Title\nSome text");
    }

    #[test]
    fn test_parse_mixed_cells() {
        let input = r#"# %% [markdown]
# # Hello

# %%
print('world')

# %% [raw]
# raw content
"#;
        let notebook = parse(input).unwrap();

        assert_eq!(notebook.cells.len(), 3);
        assert!(notebook.cells[0].is_markdown());
        assert!(notebook.cells[1].is_code());
        assert!(notebook.cells[2].is_raw());
    }

    #[test]
    fn test_parse_with_header() {
        let input = r#"# ---
# jupyter:
#   kernelspec:
#     display_name: Python 3
#     language: python
#     name: python3
# ---

# %%
print('hello')
"#;
        let notebook = parse(input).unwrap();

        assert_eq!(notebook.cells.len(), 1);
        let ks = notebook.metadata.kernelspec.as_ref().unwrap();
        assert_eq!(ks.name, "python3");
        assert_eq!(ks.display_name, "Python 3");
        assert_eq!(ks.language, "python");
    }

    #[test]
    fn test_parse_no_header() {
        let input = "# %%\nprint('hello')";
        let notebook = parse(input).unwrap();

        // Should have default kernelspec
        let ks = notebook.metadata.kernelspec.as_ref().unwrap();
        assert_eq!(ks.name, "python3");
    }

    #[test]
    fn test_parse_cell_with_tags() {
        let input = "# %% [markdown] tags=[\"hide\", \"skip\"]\n# Content";
        let notebook = parse(input).unwrap();

        let tags = notebook.cells[0].metadata().tags.as_ref().unwrap();
        assert_eq!(tags, &vec!["hide".to_string(), "skip".to_string()]);
    }

    #[test]
    fn test_comment_lines() {
        assert_eq!(comment_lines("hello\nworld"), "# hello\n# world");
        assert_eq!(comment_lines("line1\n\nline2"), "# line1\n#\n# line2");
        assert_eq!(comment_lines(""), "");
    }

    #[test]
    fn test_uncomment_lines() {
        assert_eq!(uncomment_lines("# hello\n# world"), "hello\nworld");
        assert_eq!(uncomment_lines("# line1\n#\n# line2"), "line1\n\nline2");
        assert_eq!(uncomment_lines("#hello"), "hello");
    }

    #[test]
    fn test_serialize_minimal() {
        let mut notebook = Notebook::new();
        notebook.cells.push(Cell::code("print('hello')"));

        let output = serialize(&notebook).unwrap();
        assert!(output.contains("# %%"));
        assert!(output.contains("print('hello')"));
    }

    #[test]
    fn test_serialize_markdown() {
        let mut notebook = Notebook::new();
        notebook.cells.push(Cell::markdown("# Title\nSome text"));

        let output = serialize(&notebook).unwrap();
        assert!(output.contains("# %% [markdown]"));
        assert!(output.contains("# # Title"));
        assert!(output.contains("# Some text"));
    }

    #[test]
    fn test_serialize_no_header() {
        let mut notebook = Notebook::new();
        notebook.cells.push(Cell::code("x = 1"));

        let options = PercentOptions {
            header_style: HeaderStyle::None,
        };
        let output = serialize_with_options(&notebook, &options).unwrap();
        assert!(!output.contains("# ---"));
        assert!(output.starts_with("# %%"));
    }

    #[test]
    fn test_round_trip() {
        let input = r#"# ---
# jupyter:
#   kernelspec:
#     display_name: Python 3
#     language: python
#     name: python3
# ---

# %% [markdown]
# # Title
# Some text

# %%
print('hello')
x = 1

# %% [raw]
# Raw content
"#;
        let notebook = parse(input).unwrap();
        let output = serialize(&notebook).unwrap();
        let reparsed = parse(&output).unwrap();

        assert_eq!(notebook.cells.len(), reparsed.cells.len());
        for (orig, new) in notebook.cells.iter().zip(reparsed.cells.iter()) {
            assert_eq!(orig.source_string(), new.source_string());
        }
    }

    #[test]
    fn test_empty_markdown_lines() {
        let input = "# %% [markdown]\n# Line 1\n#\n# Line 2";
        let notebook = parse(input).unwrap();

        assert_eq!(notebook.cells[0].source_string(), "Line 1\n\nLine 2");
    }

    #[test]
    fn test_md_shorthand() {
        let input = "# %% [md]\n# Content";
        let notebook = parse(input).unwrap();

        assert!(notebook.cells[0].is_markdown());
        assert_eq!(notebook.cells[0].source_string(), "Content");
    }
}
