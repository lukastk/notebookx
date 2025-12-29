//! Integration tests for ipynb format using real notebook files.

use notebookx::{NotebookFormat, Output, StreamName};
use std::path::Path;

const EXAMPLE_NOTEBOOK: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../nb_format_examples/World population.ipynb"
);

#[test]
fn test_parse_real_notebook() {
    let path = Path::new(EXAMPLE_NOTEBOOK);
    assert!(path.exists(), "Example notebook not found at {:?}", path);

    let content = std::fs::read_to_string(path).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    // Verify basic structure
    assert!(!notebook.cells.is_empty());
    assert_eq!(notebook.nbformat, 4);

    // Check metadata
    let kernelspec = notebook.metadata.kernelspec.as_ref().unwrap();
    assert_eq!(kernelspec.name, "python3");
    assert_eq!(kernelspec.language, "python");
}

#[test]
fn test_real_notebook_cell_types() {
    let content = std::fs::read_to_string(EXAMPLE_NOTEBOOK).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    // Count cell types
    let code_count = notebook.cells.iter().filter(|c| c.is_code()).count();
    let markdown_count = notebook.cells.iter().filter(|c| c.is_markdown()).count();

    // The World population notebook has both code and markdown cells
    assert!(code_count > 0, "Expected code cells");
    assert!(markdown_count > 0, "Expected markdown cells");
}

#[test]
fn test_real_notebook_has_outputs() {
    let content = std::fs::read_to_string(EXAMPLE_NOTEBOOK).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    // Find cells with outputs
    let cells_with_outputs: Vec<_> = notebook
        .cells
        .iter()
        .filter_map(|c| c.outputs())
        .filter(|outputs| !outputs.is_empty())
        .collect();

    assert!(
        !cells_with_outputs.is_empty(),
        "Expected cells with outputs"
    );
}

#[test]
fn test_real_notebook_output_types() {
    let content = std::fs::read_to_string(EXAMPLE_NOTEBOOK).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    let mut has_execute_result = false;
    let mut has_display_data = false;
    let mut has_stream = false;

    for cell in &notebook.cells {
        if let Some(outputs) = cell.outputs() {
            for output in outputs {
                match output {
                    Output::ExecuteResult { .. } => has_execute_result = true,
                    Output::DisplayData { .. } => has_display_data = true,
                    Output::Stream { name, .. } => {
                        has_stream = true;
                        // Verify stream name is valid
                        assert!(
                            matches!(name, StreamName::Stdout | StreamName::Stderr),
                            "Invalid stream name"
                        );
                    }
                    Output::Error { .. } => {}
                }
            }
        }
    }

    // The World population notebook should have these output types
    assert!(has_execute_result, "Expected execute_result outputs");
    assert!(has_display_data, "Expected display_data outputs (plots)");
    assert!(has_stream, "Expected stream outputs");
}

#[test]
fn test_real_notebook_round_trip() {
    let content = std::fs::read_to_string(EXAMPLE_NOTEBOOK).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    // Serialize and reparse
    let serialized = NotebookFormat::Ipynb.serialize(&notebook).unwrap();
    let reparsed = NotebookFormat::Ipynb.parse(&serialized).unwrap();

    // Verify cell count is preserved
    assert_eq!(notebook.cells.len(), reparsed.cells.len());

    // Verify each cell's source is preserved
    for (original, reparsed) in notebook.cells.iter().zip(reparsed.cells.iter()) {
        assert_eq!(
            original.source_string(),
            reparsed.source_string(),
            "Cell source mismatch"
        );
    }

    // Verify metadata is preserved
    assert_eq!(notebook.metadata.kernelspec, reparsed.metadata.kernelspec);
}

#[test]
fn test_real_notebook_mime_bundle() {
    let content = std::fs::read_to_string(EXAMPLE_NOTEBOOK).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    let mut found_html = false;
    let mut found_plain = false;
    let mut found_image = false;

    for cell in &notebook.cells {
        if let Some(outputs) = cell.outputs() {
            for output in outputs {
                let data = match output {
                    Output::ExecuteResult { data, .. } => Some(data),
                    Output::DisplayData { data, .. } => Some(data),
                    _ => None,
                };

                if let Some(data) = data {
                    if data.contains_key("text/html") {
                        found_html = true;
                    }
                    if data.contains_key("text/plain") {
                        found_plain = true;
                    }
                    if data.contains_key("image/png") {
                        found_image = true;
                    }
                }
            }
        }
    }

    // The World population notebook has DataFrames (HTML) and plots (PNG)
    assert!(found_plain, "Expected text/plain outputs");
    assert!(found_html, "Expected text/html outputs (DataFrames)");
    assert!(found_image, "Expected image/png outputs (plots)");
}

#[test]
fn test_real_notebook_preserves_extra_metadata() {
    let content = std::fs::read_to_string(EXAMPLE_NOTEBOOK).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    // The World population notebook has jupytext and toc metadata
    assert!(
        notebook.metadata.extra.contains_key("jupytext")
            || notebook.metadata.extra.contains_key("toc"),
        "Expected extra metadata fields to be preserved"
    );
}

#[test]
fn test_format_inference_from_path() {
    let path = Path::new("notebook.ipynb");
    assert_eq!(NotebookFormat::from_path(path), Some(NotebookFormat::Ipynb));

    let path = Path::new("notebook.pct.py");
    assert_eq!(
        NotebookFormat::from_path(path),
        Some(NotebookFormat::Percent)
    );

    let path = Path::new("regular.py");
    assert_eq!(NotebookFormat::from_path(path), None);
}
