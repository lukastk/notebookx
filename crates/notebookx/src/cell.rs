//! Cell types for Jupyter notebooks.

use crate::output::{MultilineString, Output};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A cell in a Jupyter notebook.
///
/// Notebooks contain three types of cells:
/// - `Code`: Executable code with optional outputs
/// - `Markdown`: Formatted text using Markdown
/// - `Raw`: Unformatted text, passed through without processing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "cell_type", rename_all = "lowercase")]
pub enum Cell {
    /// A code cell containing executable source code.
    Code {
        /// The source code content.
        source: MultilineString,
        /// Execution count (None if not yet executed).
        execution_count: Option<u32>,
        /// Outputs from executing this cell.
        #[serde(default)]
        outputs: Vec<Output>,
        /// Cell metadata.
        #[serde(default)]
        metadata: CellMetadata,
        /// Cell ID (optional, nbformat 4.5+).
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },

    /// A markdown cell containing formatted text.
    Markdown {
        /// The markdown content.
        source: MultilineString,
        /// Cell metadata.
        #[serde(default)]
        metadata: CellMetadata,
        /// Cell ID (optional, nbformat 4.5+).
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },

    /// A raw cell containing unprocessed text.
    Raw {
        /// The raw content.
        source: MultilineString,
        /// Cell metadata.
        #[serde(default)]
        metadata: CellMetadata,
        /// Cell ID (optional, nbformat 4.5+).
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
}

impl Cell {
    /// Create a new code cell.
    pub fn code(source: impl Into<String>) -> Self {
        Cell::Code {
            source: MultilineString::from_string(source.into()),
            execution_count: None,
            outputs: Vec::new(),
            metadata: CellMetadata::default(),
            id: None,
        }
    }

    /// Create a new markdown cell.
    pub fn markdown(source: impl Into<String>) -> Self {
        Cell::Markdown {
            source: MultilineString::from_string(source.into()),
            metadata: CellMetadata::default(),
            id: None,
        }
    }

    /// Create a new raw cell.
    pub fn raw(source: impl Into<String>) -> Self {
        Cell::Raw {
            source: MultilineString::from_string(source.into()),
            metadata: CellMetadata::default(),
            id: None,
        }
    }

    /// Get the source content of this cell.
    pub fn source(&self) -> &MultilineString {
        match self {
            Cell::Code { source, .. } => source,
            Cell::Markdown { source, .. } => source,
            Cell::Raw { source, .. } => source,
        }
    }

    /// Get the source content as a string.
    pub fn source_string(&self) -> String {
        self.source().as_string()
    }

    /// Get the cell metadata.
    pub fn metadata(&self) -> &CellMetadata {
        match self {
            Cell::Code { metadata, .. } => metadata,
            Cell::Markdown { metadata, .. } => metadata,
            Cell::Raw { metadata, .. } => metadata,
        }
    }

    /// Get the cell ID if present.
    pub fn id(&self) -> Option<&str> {
        match self {
            Cell::Code { id, .. } => id.as_deref(),
            Cell::Markdown { id, .. } => id.as_deref(),
            Cell::Raw { id, .. } => id.as_deref(),
        }
    }

    /// Check if this is a code cell.
    pub fn is_code(&self) -> bool {
        matches!(self, Cell::Code { .. })
    }

    /// Check if this is a markdown cell.
    pub fn is_markdown(&self) -> bool {
        matches!(self, Cell::Markdown { .. })
    }

    /// Check if this is a raw cell.
    pub fn is_raw(&self) -> bool {
        matches!(self, Cell::Raw { .. })
    }

    /// Get outputs if this is a code cell.
    pub fn outputs(&self) -> Option<&Vec<Output>> {
        match self {
            Cell::Code { outputs, .. } => Some(outputs),
            _ => None,
        }
    }

    /// Get execution count if this is a code cell.
    pub fn execution_count(&self) -> Option<u32> {
        match self {
            Cell::Code {
                execution_count, ..
            } => *execution_count,
            _ => None,
        }
    }
}

/// Metadata associated with a cell.
///
/// Common metadata fields include tags, whether the cell is collapsed,
/// and various editor-specific settings.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CellMetadata {
    /// Tags associated with this cell (e.g., "hide", "skip").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Whether the cell is collapsed in the UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapsed: Option<bool>,

    /// Whether the cell is scrollable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scrolled: Option<ScrolledState>,

    /// Cell name (used by some tools).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Additional metadata fields not explicitly modeled.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Scrolled state for a cell.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScrolledState {
    /// Boolean scrolled state.
    Bool(bool),
    /// String scrolled state (e.g., "auto").
    String(String),
}
