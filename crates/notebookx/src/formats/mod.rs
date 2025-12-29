//! Notebook format parsers and serializers.
//!
//! This module contains format-specific code for parsing and serializing
//! notebooks. Each format lives in its own submodule.

pub mod ipynb;
pub mod percent;

use crate::{Notebook, ParseError, SerializeError};

/// Supported notebook formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotebookFormat {
    /// Jupyter notebook format (.ipynb).
    Ipynb,
    /// Python percent format (.pct.py).
    Percent,
}

impl NotebookFormat {
    /// Infer the format from a file extension.
    ///
    /// Returns `None` if the extension is not recognized.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "ipynb" => Some(NotebookFormat::Ipynb),
            "pct.py" => Some(NotebookFormat::Percent),
            _ => None,
        }
    }

    /// Infer the format from a file path.
    ///
    /// Handles compound extensions like `.pct.py`.
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        let file_name = path.file_name()?.to_str()?;

        // Check for compound extensions first
        if file_name.ends_with(".pct.py") {
            return Some(NotebookFormat::Percent);
        }

        // Fall back to simple extension
        let ext = path.extension()?.to_str()?;
        Self::from_extension(ext)
    }

    /// Get the canonical file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            NotebookFormat::Ipynb => "ipynb",
            NotebookFormat::Percent => "pct.py",
        }
    }

    /// Parse a notebook from a string in this format.
    pub fn parse(&self, input: &str) -> Result<Notebook, ParseError> {
        match self {
            NotebookFormat::Ipynb => ipynb::parse(input),
            NotebookFormat::Percent => percent::parse(input),
        }
    }

    /// Serialize a notebook to a string in this format.
    pub fn serialize(&self, notebook: &Notebook) -> Result<String, SerializeError> {
        match self {
            NotebookFormat::Ipynb => ipynb::serialize(notebook),
            NotebookFormat::Percent => percent::serialize(notebook),
        }
    }
}

impl std::fmt::Display for NotebookFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotebookFormat::Ipynb => write!(f, "ipynb"),
            NotebookFormat::Percent => write!(f, "percent"),
        }
    }
}
