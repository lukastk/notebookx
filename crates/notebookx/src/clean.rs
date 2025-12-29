//! Notebook cleaning functionality.
//!
//! This module provides the `CleanOptions` struct and the implementation
//! of the `clean` method for removing various types of content from notebooks.

use crate::cell::{Cell, CellMetadata};
use crate::metadata::NotebookMetadata;
use crate::notebook::Notebook;
use crate::output::Output;
use std::collections::HashSet;

/// Options for cleaning a notebook.
///
/// All options default to `false`, meaning no cleaning is performed by default.
/// Enable specific options to remove the corresponding content.
///
/// # Example
///
/// ```
/// use notebookx::{Notebook, Cell, CleanOptions};
///
/// let mut notebook = Notebook::new();
/// notebook.cells.push(Cell::code("print('hello')"));
///
/// let options = CleanOptions {
///     remove_outputs: true,
///     remove_execution_counts: true,
///     ..Default::default()
/// };
///
/// let cleaned = notebook.clean(&options);
/// ```
#[derive(Debug, Clone, Default)]
pub struct CleanOptions {
    /// Remove all outputs from code cells.
    pub remove_outputs: bool,

    /// Remove execution counts from code cells.
    pub remove_execution_counts: bool,

    /// Remove cell-level metadata (tags, collapsed, scrolled, name, extra).
    pub remove_cell_metadata: bool,

    /// Remove notebook-level metadata (language_info, extra fields).
    /// Note: kernelspec is controlled separately by `remove_kernel_info`.
    pub remove_notebook_metadata: bool,

    /// Remove kernel specification from notebook metadata.
    pub remove_kernel_info: bool,

    /// Preserve cell IDs even when cleaning.
    /// If false (default), cell IDs are removed during cleaning.
    pub preserve_cell_ids: bool,

    /// If set, only these metadata keys are preserved in cell metadata.
    /// Other keys are removed. If None, all keys are preserved
    /// (unless `remove_cell_metadata` is true).
    pub allowed_cell_metadata_keys: Option<HashSet<String>>,

    /// If set, only these metadata keys are preserved in notebook metadata.
    /// Other keys are removed. If None, all keys are preserved
    /// (unless `remove_notebook_metadata` is true).
    pub allowed_notebook_metadata_keys: Option<HashSet<String>>,

    /// Remove metadata from outputs (ExecuteResult, DisplayData).
    pub remove_output_metadata: bool,

    /// Remove execution counts from ExecuteResult outputs.
    pub remove_output_execution_counts: bool,
}

impl CleanOptions {
    /// Create a new CleanOptions with all options disabled (no cleaning).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create options that remove cell metadata and execution counts, and output metadata and execution counts.
    ///
    /// This is useful for preparing notebooks for version control.
    pub fn for_vcs() -> Self {
        Self {
            remove_cell_metadata: true,
            remove_execution_counts: true,
            remove_output_metadata: true,
            remove_output_execution_counts: true,
            ..Default::default()
        }
    }

    /// Create options that strip all metadata and outputs.
    ///
    /// This produces a minimal notebook with only cell content.
    pub fn strip_all() -> Self {
        Self {
            remove_outputs: true,
            remove_execution_counts: true,
            remove_cell_metadata: true,
            remove_notebook_metadata: true,
            remove_kernel_info: true,
            preserve_cell_ids: false,
            allowed_cell_metadata_keys: None,
            allowed_notebook_metadata_keys: None,
            remove_output_metadata: true,
            remove_output_execution_counts: true,
        }
    }
}

impl Notebook {
    /// Clean the notebook according to the specified options.
    ///
    /// This method returns a new notebook with the requested content removed.
    /// The original notebook is not modified.
    ///
    /// # Example
    ///
    /// ```
    /// use notebookx::{Notebook, Cell, CleanOptions};
    ///
    /// let mut notebook = Notebook::new();
    /// notebook.cells.push(Cell::code("x = 1"));
    ///
    /// let options = CleanOptions {
    ///     remove_outputs: true,
    ///     ..Default::default()
    /// };
    ///
    /// let cleaned = notebook.clean(&options);
    /// assert_eq!(notebook.len(), cleaned.len()); // Original unchanged
    /// ```
    pub fn clean(&self, options: &CleanOptions) -> Notebook {
        let cells = self
            .cells
            .iter()
            .map(|cell| clean_cell(cell, options))
            .collect();
        let metadata = clean_notebook_metadata(&self.metadata, options);

        Notebook {
            cells,
            metadata,
            nbformat: self.nbformat,
            nbformat_minor: self.nbformat_minor,
        }
    }
}

/// Clean a single cell according to the options.
fn clean_cell(cell: &Cell, options: &CleanOptions) -> Cell {
    match cell {
        Cell::Code {
            source,
            execution_count,
            outputs,
            metadata,
            id,
        } => {
            let new_execution_count = if options.remove_execution_counts {
                None
            } else {
                *execution_count
            };

            let new_outputs = if options.remove_outputs {
                Vec::new()
            } else if options.remove_output_metadata || options.remove_output_execution_counts {
                outputs.iter().map(|o| clean_output(o, options)).collect()
            } else {
                outputs.clone()
            };

            let new_metadata = clean_cell_metadata(metadata, options);

            let new_id = if options.preserve_cell_ids {
                id.clone()
            } else {
                None
            };

            Cell::Code {
                source: source.clone(),
                execution_count: new_execution_count,
                outputs: new_outputs,
                metadata: new_metadata,
                id: new_id,
            }
        }
        Cell::Markdown {
            source,
            metadata,
            id,
        } => {
            let new_metadata = clean_cell_metadata(metadata, options);
            let new_id = if options.preserve_cell_ids {
                id.clone()
            } else {
                None
            };

            Cell::Markdown {
                source: source.clone(),
                metadata: new_metadata,
                id: new_id,
            }
        }
        Cell::Raw {
            source,
            metadata,
            id,
        } => {
            let new_metadata = clean_cell_metadata(metadata, options);
            let new_id = if options.preserve_cell_ids {
                id.clone()
            } else {
                None
            };

            Cell::Raw {
                source: source.clone(),
                metadata: new_metadata,
                id: new_id,
            }
        }
    }
}

/// Clean cell metadata according to the options.
fn clean_cell_metadata(metadata: &CellMetadata, options: &CleanOptions) -> CellMetadata {
    if options.remove_cell_metadata {
        return CellMetadata::default();
    }

    // If allowed_cell_metadata_keys is set, filter the metadata
    if let Some(ref allowed_keys) = options.allowed_cell_metadata_keys {
        let mut new_metadata = CellMetadata::default();

        if allowed_keys.contains("tags") {
            new_metadata.tags = metadata.tags.clone();
        }
        if allowed_keys.contains("collapsed") {
            new_metadata.collapsed = metadata.collapsed;
        }
        if allowed_keys.contains("scrolled") {
            new_metadata.scrolled = metadata.scrolled.clone();
        }
        if allowed_keys.contains("name") {
            new_metadata.name = metadata.name.clone();
        }

        // Filter extra fields
        for (key, value) in &metadata.extra {
            if allowed_keys.contains(key) {
                new_metadata.extra.insert(key.clone(), value.clone());
            }
        }

        new_metadata
    } else {
        metadata.clone()
    }
}

/// Clean a single output according to the options.
fn clean_output(output: &Output, options: &CleanOptions) -> Output {
    match output {
        Output::ExecuteResult {
            execution_count,
            data,
            metadata,
        } => {
            let new_execution_count = if options.remove_output_execution_counts {
                None
            } else {
                *execution_count
            };

            let new_metadata = if options.remove_output_metadata {
                Default::default()
            } else {
                metadata.clone()
            };

            Output::ExecuteResult {
                execution_count: new_execution_count,
                data: data.clone(),
                metadata: new_metadata,
            }
        }
        Output::DisplayData { data, metadata } => {
            let new_metadata = if options.remove_output_metadata {
                Default::default()
            } else {
                metadata.clone()
            };

            Output::DisplayData {
                data: data.clone(),
                metadata: new_metadata,
            }
        }
        // Stream and Error outputs don't have metadata or execution_count
        Output::Stream { .. } | Output::Error { .. } => output.clone(),
    }
}

/// Clean notebook metadata according to the options.
fn clean_notebook_metadata(
    metadata: &NotebookMetadata,
    options: &CleanOptions,
) -> NotebookMetadata {
    if options.remove_notebook_metadata && options.remove_kernel_info {
        return NotebookMetadata::default();
    }

    let mut new_metadata = NotebookMetadata::default();

    // Handle kernelspec
    if !options.remove_kernel_info {
        new_metadata.kernelspec = metadata.kernelspec.clone();
    }

    // Handle other metadata (language_info, extra)
    if !options.remove_notebook_metadata {
        new_metadata.language_info = metadata.language_info.clone();

        // If allowed_notebook_metadata_keys is set, filter extra fields
        if let Some(ref allowed_keys) = options.allowed_notebook_metadata_keys {
            for (key, value) in &metadata.extra {
                if allowed_keys.contains(key) {
                    new_metadata.extra.insert(key.clone(), value.clone());
                }
            }
        } else {
            new_metadata.extra = metadata.extra.clone();
        }
    }

    new_metadata
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::KernelSpec;
    use crate::output::{MultilineString, Output, StreamName};

    fn create_test_notebook() -> Notebook {
        let mut notebook = Notebook::new();

        // Add a code cell with outputs
        notebook.cells.push(Cell::Code {
            source: MultilineString::from_string("print('hello')"),
            execution_count: Some(1),
            outputs: vec![Output::Stream {
                name: StreamName::Stdout,
                text: MultilineString::from_string("hello\n"),
            }],
            metadata: CellMetadata {
                tags: Some(vec!["test".to_string()]),
                collapsed: Some(false),
                scrolled: None,
                name: Some("test_cell".to_string()),
                extra: Default::default(),
            },
            id: Some("cell-001".to_string()),
        });

        // Add a markdown cell
        notebook.cells.push(Cell::Markdown {
            source: MultilineString::from_string("# Hello"),
            metadata: CellMetadata {
                tags: Some(vec!["doc".to_string()]),
                ..Default::default()
            },
            id: Some("cell-002".to_string()),
        });

        // Set notebook metadata
        notebook.metadata.kernelspec = Some(KernelSpec {
            name: "python3".to_string(),
            display_name: "Python 3".to_string(),
            language: "python".to_string(),
        });

        notebook
    }

    #[test]
    fn test_clean_default_options_no_change() {
        let notebook = create_test_notebook();
        let options = CleanOptions::default();
        let cleaned = notebook.clean(&options);

        // With default options (except cell IDs are removed by default)
        assert_eq!(cleaned.cells.len(), notebook.cells.len());

        // Cell content should be preserved
        assert_eq!(
            cleaned.cells[0].source_string(),
            notebook.cells[0].source_string()
        );

        // Outputs should be preserved
        assert_eq!(
            cleaned.cells[0].outputs().unwrap().len(),
            notebook.cells[0].outputs().unwrap().len()
        );

        // Execution count should be preserved
        assert_eq!(
            cleaned.cells[0].execution_count(),
            notebook.cells[0].execution_count()
        );

        // Cell IDs are removed by default
        assert!(cleaned.cells[0].id().is_none());
    }

    #[test]
    fn test_clean_remove_outputs() {
        let notebook = create_test_notebook();
        let options = CleanOptions {
            remove_outputs: true,
            ..Default::default()
        };
        let cleaned = notebook.clean(&options);

        // Outputs should be removed
        assert!(cleaned.cells[0].outputs().unwrap().is_empty());

        // Other content should be preserved
        assert_eq!(
            cleaned.cells[0].source_string(),
            notebook.cells[0].source_string()
        );
        assert_eq!(
            cleaned.cells[0].execution_count(),
            notebook.cells[0].execution_count()
        );
    }

    #[test]
    fn test_clean_remove_execution_counts() {
        let notebook = create_test_notebook();
        let options = CleanOptions {
            remove_execution_counts: true,
            ..Default::default()
        };
        let cleaned = notebook.clean(&options);

        // Execution count should be removed
        assert!(cleaned.cells[0].execution_count().is_none());

        // Outputs should be preserved
        assert!(!cleaned.cells[0].outputs().unwrap().is_empty());
    }

    #[test]
    fn test_clean_remove_cell_metadata() {
        let notebook = create_test_notebook();
        let options = CleanOptions {
            remove_cell_metadata: true,
            ..Default::default()
        };
        let cleaned = notebook.clean(&options);

        // Cell metadata should be empty
        let metadata = cleaned.cells[0].metadata();
        assert!(metadata.tags.is_none());
        assert!(metadata.collapsed.is_none());
        assert!(metadata.name.is_none());
    }

    #[test]
    fn test_clean_remove_notebook_metadata() {
        let notebook = create_test_notebook();
        let options = CleanOptions {
            remove_notebook_metadata: true,
            ..Default::default()
        };
        let cleaned = notebook.clean(&options);

        // Language info and extra should be removed
        assert!(cleaned.metadata.language_info.is_none());
        assert!(cleaned.metadata.extra.is_empty());

        // Kernelspec should be preserved (controlled separately)
        assert!(cleaned.metadata.kernelspec.is_some());
    }

    #[test]
    fn test_clean_remove_kernel_info() {
        let notebook = create_test_notebook();
        let options = CleanOptions {
            remove_kernel_info: true,
            ..Default::default()
        };
        let cleaned = notebook.clean(&options);

        // Kernelspec should be removed
        assert!(cleaned.metadata.kernelspec.is_none());
    }

    #[test]
    fn test_clean_preserve_cell_ids() {
        let notebook = create_test_notebook();
        let options = CleanOptions {
            preserve_cell_ids: true,
            ..Default::default()
        };
        let cleaned = notebook.clean(&options);

        // Cell IDs should be preserved
        assert_eq!(cleaned.cells[0].id(), Some("cell-001"));
        assert_eq!(cleaned.cells[1].id(), Some("cell-002"));
    }

    #[test]
    fn test_clean_allowed_cell_metadata_keys() {
        let notebook = create_test_notebook();
        let mut allowed_keys = HashSet::new();
        allowed_keys.insert("tags".to_string());

        let options = CleanOptions {
            allowed_cell_metadata_keys: Some(allowed_keys),
            ..Default::default()
        };
        let cleaned = notebook.clean(&options);

        // Only tags should be preserved
        let metadata = cleaned.cells[0].metadata();
        assert!(metadata.tags.is_some());
        assert!(metadata.collapsed.is_none()); // Not in allowed keys
        assert!(metadata.name.is_none()); // Not in allowed keys
    }

    #[test]
    fn test_clean_for_vcs() {
        let notebook = create_test_notebook();
        let options = CleanOptions::for_vcs();
        let cleaned = notebook.clean(&options);

        // Execution counts should be removed
        assert!(cleaned.cells[0].execution_count().is_none());

        // Outputs should be preserved (for_vcs doesn't remove outputs)
        assert!(!cleaned.cells[0].outputs().unwrap().is_empty());

        // Cell metadata should be removed
        let metadata = cleaned.cells[0].metadata();
        assert!(metadata.tags.is_none());
        assert!(metadata.collapsed.is_none());
        assert!(metadata.name.is_none());

        // Notebook metadata (kernelspec) should be preserved
        assert!(cleaned.metadata.kernelspec.is_some());
    }

    #[test]
    fn test_clean_strip_all() {
        let notebook = create_test_notebook();
        let options = CleanOptions::strip_all();
        let cleaned = notebook.clean(&options);

        // Everything should be stripped except content
        assert!(cleaned.cells[0].outputs().unwrap().is_empty());
        assert!(cleaned.cells[0].execution_count().is_none());
        assert!(cleaned.cells[0].id().is_none());
        assert!(cleaned.cells[0].metadata().tags.is_none());
        assert!(cleaned.metadata.kernelspec.is_none());
    }

    #[test]
    fn test_clean_original_unchanged() {
        let notebook = create_test_notebook();
        let original_output_count = notebook.cells[0].outputs().unwrap().len();
        let original_exec_count = notebook.cells[0].execution_count();

        let options = CleanOptions::strip_all();
        let _cleaned = notebook.clean(&options);

        // Original should be unchanged
        assert_eq!(
            notebook.cells[0].outputs().unwrap().len(),
            original_output_count
        );
        assert_eq!(notebook.cells[0].execution_count(), original_exec_count);
    }

    #[test]
    fn test_clean_idempotent() {
        let notebook = create_test_notebook();
        let options = CleanOptions::strip_all();

        let cleaned_once = notebook.clean(&options);
        let cleaned_twice = cleaned_once.clean(&options);

        // Cleaning twice should produce the same result
        assert_eq!(cleaned_once.cells.len(), cleaned_twice.cells.len());
        for (c1, c2) in cleaned_once.cells.iter().zip(cleaned_twice.cells.iter()) {
            assert_eq!(c1.source_string(), c2.source_string());
            assert_eq!(c1.outputs(), c2.outputs());
            assert_eq!(c1.execution_count(), c2.execution_count());
        }
    }

    #[test]
    fn test_clean_empty_notebook() {
        let notebook = Notebook::new();
        let options = CleanOptions::strip_all();
        let cleaned = notebook.clean(&options);

        assert!(cleaned.is_empty());
    }

    #[test]
    fn test_clean_notebook_without_outputs() {
        let mut notebook = Notebook::new();
        notebook.cells.push(Cell::code("x = 1"));

        let options = CleanOptions {
            remove_outputs: true,
            ..Default::default()
        };
        let cleaned = notebook.clean(&options);

        // Should work fine even without outputs
        assert!(cleaned.cells[0].outputs().unwrap().is_empty());
    }

    fn create_test_notebook_with_output_metadata() -> Notebook {
        use crate::output::{MimeBundle, MimeData, OutputMetadata};

        let mut notebook = Notebook::new();

        // Create output metadata
        let mut output_metadata: OutputMetadata = std::collections::HashMap::new();
        output_metadata.insert("foo".to_string(), serde_json::json!("bar"));

        // Create MIME bundle
        let mut data: MimeBundle = std::collections::HashMap::new();
        data.insert("text/plain".to_string(), MimeData::String("42".to_string()));

        // Add a code cell with ExecuteResult output (has metadata and execution_count)
        notebook.cells.push(Cell::Code {
            source: MultilineString::from_string("40 + 2"),
            execution_count: Some(1),
            outputs: vec![
                Output::ExecuteResult {
                    execution_count: Some(1),
                    data: data.clone(),
                    metadata: output_metadata.clone(),
                },
                Output::DisplayData {
                    data: data.clone(),
                    metadata: output_metadata.clone(),
                },
                Output::Stream {
                    name: StreamName::Stdout,
                    text: MultilineString::from_string("hello\n"),
                },
            ],
            metadata: Default::default(),
            id: Some("cell-001".to_string()),
        });

        notebook
    }

    #[test]
    fn test_clean_remove_output_metadata() {
        let notebook = create_test_notebook_with_output_metadata();
        let options = CleanOptions {
            remove_output_metadata: true,
            ..Default::default()
        };
        let cleaned = notebook.clean(&options);

        let outputs = cleaned.cells[0].outputs().unwrap();
        assert_eq!(outputs.len(), 3);

        // ExecuteResult should have empty metadata
        match &outputs[0] {
            Output::ExecuteResult {
                metadata,
                execution_count,
                ..
            } => {
                assert!(
                    metadata.is_empty(),
                    "ExecuteResult metadata should be empty"
                );
                assert_eq!(
                    *execution_count,
                    Some(1),
                    "execution_count should be preserved"
                );
            }
            _ => panic!("Expected ExecuteResult"),
        }

        // DisplayData should have empty metadata
        match &outputs[1] {
            Output::DisplayData { metadata, .. } => {
                assert!(metadata.is_empty(), "DisplayData metadata should be empty");
            }
            _ => panic!("Expected DisplayData"),
        }

        // Stream should be unchanged (doesn't have metadata)
        match &outputs[2] {
            Output::Stream { name, text } => {
                assert_eq!(*name, StreamName::Stdout);
                assert_eq!(text.as_string(), "hello\n");
            }
            _ => panic!("Expected Stream"),
        }
    }

    #[test]
    fn test_clean_remove_output_execution_counts() {
        let notebook = create_test_notebook_with_output_metadata();
        let options = CleanOptions {
            remove_output_execution_counts: true,
            ..Default::default()
        };
        let cleaned = notebook.clean(&options);

        let outputs = cleaned.cells[0].outputs().unwrap();

        // ExecuteResult should have None execution_count
        match &outputs[0] {
            Output::ExecuteResult {
                execution_count,
                metadata,
                ..
            } => {
                assert!(execution_count.is_none(), "execution_count should be None");
                assert!(!metadata.is_empty(), "metadata should be preserved");
            }
            _ => panic!("Expected ExecuteResult"),
        }

        // Cell's execution_count should be preserved (not output execution_count)
        assert_eq!(cleaned.cells[0].execution_count(), Some(1));
    }

    #[test]
    fn test_clean_remove_both_output_metadata_and_execution_counts() {
        let notebook = create_test_notebook_with_output_metadata();
        let options = CleanOptions {
            remove_output_metadata: true,
            remove_output_execution_counts: true,
            ..Default::default()
        };
        let cleaned = notebook.clean(&options);

        let outputs = cleaned.cells[0].outputs().unwrap();

        // ExecuteResult should have empty metadata and None execution_count
        match &outputs[0] {
            Output::ExecuteResult {
                execution_count,
                metadata,
                ..
            } => {
                assert!(execution_count.is_none(), "execution_count should be None");
                assert!(metadata.is_empty(), "metadata should be empty");
            }
            _ => panic!("Expected ExecuteResult"),
        }

        // DisplayData should have empty metadata
        match &outputs[1] {
            Output::DisplayData { metadata, .. } => {
                assert!(metadata.is_empty(), "metadata should be empty");
            }
            _ => panic!("Expected DisplayData"),
        }
    }

    #[test]
    fn test_clean_for_vcs_cleans_output_metadata() {
        let notebook = create_test_notebook_with_output_metadata();
        let options = CleanOptions::for_vcs();
        let cleaned = notebook.clean(&options);

        // Verify for_vcs cleans output metadata and execution counts
        let outputs = cleaned.cells[0].outputs().unwrap();
        assert_eq!(outputs.len(), 3, "outputs should be preserved");

        match &outputs[0] {
            Output::ExecuteResult {
                execution_count,
                metadata,
                ..
            } => {
                assert!(
                    execution_count.is_none(),
                    "output execution_count should be None"
                );
                assert!(metadata.is_empty(), "output metadata should be empty");
            }
            _ => panic!("Expected ExecuteResult"),
        }
    }
}
