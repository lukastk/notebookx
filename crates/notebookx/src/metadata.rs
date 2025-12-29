//! Notebook-level metadata structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Notebook-level metadata.
///
/// This struct captures the metadata section of a Jupyter notebook,
/// including kernel specification, language info, and any additional
/// metadata added by extensions or tools.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NotebookMetadata {
    /// Kernel specification (display name, language, name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernelspec: Option<KernelSpec>,

    /// Language information for the notebook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_info: Option<LanguageInfo>,

    /// Additional metadata fields not explicitly modeled.
    /// This preserves any extension-specific or custom metadata.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Kernel specification metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KernelSpec {
    /// Human-readable name for the kernel (e.g., "Python 3").
    pub display_name: String,

    /// Programming language (e.g., "python").
    pub language: String,

    /// Internal kernel name (e.g., "python3").
    pub name: String,
}

/// Language information metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageInfo {
    /// Programming language name.
    pub name: String,

    /// Codemirror mode for syntax highlighting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codemirror_mode: Option<CodemirrorMode>,

    /// File extension (e.g., ".py").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_extension: Option<String>,

    /// MIME type (e.g., "text/x-python").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// nbconvert exporter name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbconvert_exporter: Option<String>,

    /// Pygments lexer name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pygments_lexer: Option<String>,

    /// Language version (e.g., "3.9.0").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Codemirror mode specification.
///
/// Can be either a simple string (e.g., "python") or an object
/// with name and version (e.g., {"name": "ipython", "version": 3}).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CodemirrorMode {
    /// Simple string mode name.
    Simple(String),
    /// Object with name and optional version.
    Object {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<u32>,
    },
}
