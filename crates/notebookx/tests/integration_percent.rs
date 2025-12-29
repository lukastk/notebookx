//! Integration tests for percent format using real notebook files.

use notebookx::NotebookFormat;
use std::path::Path;

const EXAMPLE_PERCENT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../nb_format_examples/World population.pct.py"
);

const EXAMPLE_IPYNB: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../nb_format_examples/World population.ipynb"
);

#[test]
fn test_parse_real_percent_file() {
    let path = Path::new(EXAMPLE_PERCENT);
    assert!(
        path.exists(),
        "Example percent file not found at {:?}",
        path
    );

    let content = std::fs::read_to_string(path).unwrap();
    let notebook = NotebookFormat::Percent.parse(&content).unwrap();

    // Verify basic structure
    assert!(!notebook.cells.is_empty());

    // Check metadata - should have kernelspec from header
    let kernelspec = notebook.metadata.kernelspec.as_ref().unwrap();
    assert_eq!(kernelspec.name, "python3");
    assert_eq!(kernelspec.language, "python");
}

#[test]
fn test_real_percent_cell_types() {
    let content = std::fs::read_to_string(EXAMPLE_PERCENT).unwrap();
    let notebook = NotebookFormat::Percent.parse(&content).unwrap();

    // Count cell types
    let code_count = notebook.cells.iter().filter(|c| c.is_code()).count();
    let markdown_count = notebook.cells.iter().filter(|c| c.is_markdown()).count();

    // The World population notebook has both code and markdown cells
    assert!(code_count > 0, "Expected code cells");
    assert!(markdown_count > 0, "Expected markdown cells");
}

#[test]
fn test_real_percent_round_trip() {
    let content = std::fs::read_to_string(EXAMPLE_PERCENT).unwrap();
    let notebook = NotebookFormat::Percent.parse(&content).unwrap();

    // Serialize and reparse
    let serialized = NotebookFormat::Percent.serialize(&notebook).unwrap();
    let reparsed = NotebookFormat::Percent.parse(&serialized).unwrap();

    // Verify cell count is preserved
    assert_eq!(notebook.cells.len(), reparsed.cells.len());

    // Verify each cell's source is preserved
    for (original, reparsed) in notebook.cells.iter().zip(reparsed.cells.iter()) {
        assert_eq!(
            original.source_string().trim(),
            reparsed.source_string().trim(),
            "Cell source mismatch"
        );
    }
}

#[test]
fn test_cross_format_ipynb_to_percent() {
    let content = std::fs::read_to_string(EXAMPLE_IPYNB).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    // Convert to percent format
    let percent_str = NotebookFormat::Percent.serialize(&notebook).unwrap();

    // Parse back
    let reparsed = NotebookFormat::Percent.parse(&percent_str).unwrap();

    // Verify cell count is preserved
    assert_eq!(notebook.cells.len(), reparsed.cells.len());

    // Verify cell types are preserved
    for (original, reparsed) in notebook.cells.iter().zip(reparsed.cells.iter()) {
        assert_eq!(original.is_code(), reparsed.is_code());
        assert_eq!(original.is_markdown(), reparsed.is_markdown());
        assert_eq!(original.is_raw(), reparsed.is_raw());
    }

    // Verify source content is preserved
    for (original, reparsed) in notebook.cells.iter().zip(reparsed.cells.iter()) {
        assert_eq!(
            original.source_string().trim(),
            reparsed.source_string().trim(),
            "Cell source mismatch during ipynb -> percent conversion"
        );
    }
}

#[test]
fn test_cross_format_percent_to_ipynb() {
    let content = std::fs::read_to_string(EXAMPLE_PERCENT).unwrap();
    let notebook = NotebookFormat::Percent.parse(&content).unwrap();

    // Convert to ipynb format
    let ipynb_str = NotebookFormat::Ipynb.serialize(&notebook).unwrap();

    // Parse back
    let reparsed = NotebookFormat::Ipynb.parse(&ipynb_str).unwrap();

    // Verify cell count is preserved
    assert_eq!(notebook.cells.len(), reparsed.cells.len());

    // Verify source content is preserved
    for (original, reparsed) in notebook.cells.iter().zip(reparsed.cells.iter()) {
        assert_eq!(
            original.source_string().trim(),
            reparsed.source_string().trim(),
            "Cell source mismatch during percent -> ipynb conversion"
        );
    }
}

#[test]
fn test_cross_format_round_trip_ipynb_percent_ipynb() {
    let content = std::fs::read_to_string(EXAMPLE_IPYNB).unwrap();
    let original = NotebookFormat::Ipynb.parse(&content).unwrap();

    // ipynb -> percent -> ipynb
    let percent_str = NotebookFormat::Percent.serialize(&original).unwrap();
    let from_percent = NotebookFormat::Percent.parse(&percent_str).unwrap();
    let back_to_ipynb = NotebookFormat::Ipynb.serialize(&from_percent).unwrap();
    let final_notebook = NotebookFormat::Ipynb.parse(&back_to_ipynb).unwrap();

    // Verify cell count
    assert_eq!(original.cells.len(), final_notebook.cells.len());

    // Verify source content (this is what matters for round-trip)
    for (orig, final_cell) in original.cells.iter().zip(final_notebook.cells.iter()) {
        assert_eq!(
            orig.source_string().trim(),
            final_cell.source_string().trim(),
            "Cell source lost during round-trip"
        );
    }
}

#[test]
fn test_percent_preserves_markdown_formatting() {
    let content = std::fs::read_to_string(EXAMPLE_PERCENT).unwrap();
    let notebook = NotebookFormat::Percent.parse(&content).unwrap();

    // Find a markdown cell with a heading
    let markdown_cells: Vec<_> = notebook.cells.iter().filter(|c| c.is_markdown()).collect();
    assert!(!markdown_cells.is_empty());

    // The first markdown cell should have a heading
    let first_markdown = markdown_cells[0].source_string();
    assert!(
        first_markdown.contains("# ") || first_markdown.starts_with('#'),
        "Expected markdown heading to be preserved"
    );
}

#[test]
fn test_percent_output_is_valid_python() {
    // When serializing to percent format, the output should be syntactically valid Python
    // (assuming the code cells contain valid Python)
    let content = std::fs::read_to_string(EXAMPLE_IPYNB).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    let percent_str = NotebookFormat::Percent.serialize(&notebook).unwrap();

    // Check that comments are properly formatted
    for line in percent_str.lines() {
        if line.starts_with('#') {
            // Valid Python comment
            continue;
        }
        // Non-comment lines should be either empty or valid Python syntax
        // (we can't fully validate Python syntax here, but we can check basics)
        assert!(
            line.is_empty()
                || !line.starts_with(' ')
                || line.starts_with("    ")
                || line.starts_with('\t'),
            "Unexpected indentation in line: {}",
            line
        );
    }
}

#[test]
fn test_empty_percent_file() {
    let notebook = NotebookFormat::Percent.parse("").unwrap();
    assert!(notebook.cells.is_empty());
}

#[test]
fn test_percent_with_only_header() {
    let input = r#"# ---
# jupyter:
#   kernelspec:
#     name: python3
# ---
"#;
    let notebook = NotebookFormat::Percent.parse(input).unwrap();
    assert!(notebook.cells.is_empty());
    assert_eq!(
        notebook.metadata.kernelspec.as_ref().unwrap().name,
        "python3"
    );
}
