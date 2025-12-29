//! Integration tests for notebook cleaning using real notebook files.

use notebookx::{CleanOptions, NotebookFormat};

const EXAMPLE_IPYNB: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../nb_format_examples/World population.ipynb"
);

#[test]
fn test_clean_real_notebook_outputs() {
    let content = std::fs::read_to_string(EXAMPLE_IPYNB).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    // Verify the notebook has outputs
    let original_output_count: usize = notebook
        .cells
        .iter()
        .filter_map(|c| c.outputs())
        .map(|o| o.len())
        .sum();
    assert!(
        original_output_count > 0,
        "Test notebook should have outputs"
    );

    // Clean outputs
    let options = CleanOptions {
        remove_outputs: true,
        ..Default::default()
    };
    let cleaned = notebook.clean(&options);

    // All outputs should be removed
    let cleaned_output_count: usize = cleaned
        .cells
        .iter()
        .filter_map(|c| c.outputs())
        .map(|o| o.len())
        .sum();
    assert_eq!(cleaned_output_count, 0);

    // Cell content should be preserved
    assert_eq!(cleaned.cells.len(), notebook.cells.len());
    for (original, cleaned) in notebook.cells.iter().zip(cleaned.cells.iter()) {
        assert_eq!(original.source_string(), cleaned.source_string());
    }
}

#[test]
fn test_clean_real_notebook_execution_counts() {
    let content = std::fs::read_to_string(EXAMPLE_IPYNB).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    // Verify the notebook has execution counts
    let has_execution_counts = notebook.cells.iter().any(|c| c.execution_count().is_some());
    assert!(
        has_execution_counts,
        "Test notebook should have execution counts"
    );

    // Clean execution counts
    let options = CleanOptions {
        remove_execution_counts: true,
        ..Default::default()
    };
    let cleaned = notebook.clean(&options);

    // All execution counts should be removed
    let has_cleaned_execution_counts = cleaned.cells.iter().any(|c| c.execution_count().is_some());
    assert!(!has_cleaned_execution_counts);

    // Outputs should be preserved
    let original_output_count: usize = notebook
        .cells
        .iter()
        .filter_map(|c| c.outputs())
        .map(|o| o.len())
        .sum();
    let cleaned_output_count: usize = cleaned
        .cells
        .iter()
        .filter_map(|c| c.outputs())
        .map(|o| o.len())
        .sum();
    assert_eq!(original_output_count, cleaned_output_count);
}

#[test]
fn test_clean_for_vcs() {
    let content = std::fs::read_to_string(EXAMPLE_IPYNB).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    // Verify the notebook has outputs before cleaning
    let original_output_count: usize = notebook
        .cells
        .iter()
        .filter_map(|c| c.outputs())
        .map(|o| o.len())
        .sum();
    assert!(
        original_output_count > 0,
        "Test notebook should have outputs"
    );

    let cleaned = notebook.clean(&CleanOptions::for_vcs());

    // Execution counts should be removed
    for cell in &cleaned.cells {
        assert!(
            cell.execution_count().is_none(),
            "VCS clean should remove execution counts"
        );
    }

    // Outputs should be preserved (for_vcs doesn't remove outputs, only their metadata)
    let cleaned_output_count: usize = cleaned
        .cells
        .iter()
        .filter_map(|c| c.outputs())
        .map(|o| o.len())
        .sum();
    assert_eq!(
        original_output_count, cleaned_output_count,
        "VCS clean should preserve outputs"
    );

    // Cell metadata should be removed
    for cell in &cleaned.cells {
        let metadata = cell.metadata();
        assert!(
            metadata.tags.is_none(),
            "VCS clean should remove cell metadata"
        );
        assert!(metadata.collapsed.is_none());
        assert!(metadata.name.is_none());
    }

    // Kernel info should be preserved
    assert!(cleaned.metadata.kernelspec.is_some());
}

#[test]
fn test_clean_strip_all() {
    let content = std::fs::read_to_string(EXAMPLE_IPYNB).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    let cleaned = notebook.clean(&CleanOptions::strip_all());

    // Outputs, execution counts, and metadata should be removed
    for cell in &cleaned.cells {
        if let Some(outputs) = cell.outputs() {
            assert!(outputs.is_empty());
        }
        assert!(cell.execution_count().is_none());
        assert!(cell.id().is_none());
    }

    // Notebook metadata should be removed
    assert!(cleaned.metadata.kernelspec.is_none());
    assert!(cleaned.metadata.language_info.is_none());

    // Cell content should still be preserved
    assert_eq!(cleaned.cells.len(), notebook.cells.len());
    for (original, cleaned) in notebook.cells.iter().zip(cleaned.cells.iter()) {
        assert_eq!(original.source_string(), cleaned.source_string());
    }
}

#[test]
fn test_clean_preserves_round_trip() {
    let content = std::fs::read_to_string(EXAMPLE_IPYNB).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    // Clean for VCS
    let cleaned = notebook.clean(&CleanOptions::for_vcs());

    // Serialize and parse back
    let serialized = NotebookFormat::Ipynb.serialize(&cleaned).unwrap();
    let reparsed = NotebookFormat::Ipynb.parse(&serialized).unwrap();

    // Verify structure is preserved
    assert_eq!(cleaned.cells.len(), reparsed.cells.len());
    for (c1, c2) in cleaned.cells.iter().zip(reparsed.cells.iter()) {
        assert_eq!(c1.source_string(), c2.source_string());
        assert_eq!(c1.is_code(), c2.is_code());
        assert_eq!(c1.is_markdown(), c2.is_markdown());
    }
}

#[test]
fn test_clean_then_convert_to_percent() {
    let content = std::fs::read_to_string(EXAMPLE_IPYNB).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    // Clean and convert to percent
    let cleaned = notebook.clean(&CleanOptions::for_vcs());
    let percent = NotebookFormat::Percent.serialize(&cleaned).unwrap();

    // Parse back and verify
    let reparsed = NotebookFormat::Percent.parse(&percent).unwrap();
    assert_eq!(cleaned.cells.len(), reparsed.cells.len());

    // Cell content should be preserved
    for (c1, c2) in cleaned.cells.iter().zip(reparsed.cells.iter()) {
        assert_eq!(c1.source_string().trim(), c2.source_string().trim());
    }
}

#[test]
fn test_clean_original_unchanged() {
    let content = std::fs::read_to_string(EXAMPLE_IPYNB).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    let original_output_count: usize = notebook
        .cells
        .iter()
        .filter_map(|c| c.outputs())
        .map(|o| o.len())
        .sum();

    // Clean with strip_all
    let _cleaned = notebook.clean(&CleanOptions::strip_all());

    // Original should be unchanged
    let after_output_count: usize = notebook
        .cells
        .iter()
        .filter_map(|c| c.outputs())
        .map(|o| o.len())
        .sum();
    assert_eq!(original_output_count, after_output_count);
}

#[test]
fn test_clean_removes_kernel_info() {
    let content = std::fs::read_to_string(EXAMPLE_IPYNB).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    assert!(
        notebook.metadata.kernelspec.is_some(),
        "Test notebook should have kernelspec"
    );

    let options = CleanOptions {
        remove_kernel_info: true,
        ..Default::default()
    };
    let cleaned = notebook.clean(&options);

    assert!(cleaned.metadata.kernelspec.is_none());
}

#[test]
fn test_clean_idempotent_on_real_notebook() {
    let content = std::fs::read_to_string(EXAMPLE_IPYNB).unwrap();
    let notebook = NotebookFormat::Ipynb.parse(&content).unwrap();

    let options = CleanOptions::strip_all();
    let cleaned_once = notebook.clean(&options);
    let cleaned_twice = cleaned_once.clean(&options);

    // Should produce identical results
    assert_eq!(cleaned_once.cells.len(), cleaned_twice.cells.len());
    for (c1, c2) in cleaned_once.cells.iter().zip(cleaned_twice.cells.iter()) {
        assert_eq!(c1.source_string(), c2.source_string());
        assert_eq!(c1.outputs(), c2.outputs());
        assert_eq!(c1.execution_count(), c2.execution_count());
        assert_eq!(c1.id(), c2.id());
    }
    assert_eq!(
        cleaned_once.metadata.kernelspec,
        cleaned_twice.metadata.kernelspec
    );
}
