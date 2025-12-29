//! Error types for notebookx operations.

use thiserror::Error;

/// Top-level error type for all notebookx operations.
#[derive(Debug, Error)]
pub enum NotebookError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Serialization error: {0}")]
    Serialize(#[from] SerializeError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// Errors that can occur during notebook parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid JSON: {message}")]
    InvalidJson { message: String },

    #[error("Invalid notebook structure: {message}")]
    InvalidStructure { message: String },

    #[error("Unknown cell type: {cell_type}")]
    UnknownCellType { cell_type: String },

    #[error("Unknown output type: {output_type}")]
    UnknownOutputType { output_type: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::InvalidJson {
            message: err.to_string(),
        }
    }
}

/// Errors that can occur during notebook serialization.
#[derive(Debug, Error)]
pub enum SerializeError {
    #[error("JSON serialization failed: {message}")]
    JsonError { message: String },

    #[error("Invalid data for format: {message}")]
    InvalidData { message: String },
}

impl From<serde_json::Error> for SerializeError {
    fn from(err: serde_json::Error) -> Self {
        SerializeError::JsonError {
            message: err.to_string(),
        }
    }
}
