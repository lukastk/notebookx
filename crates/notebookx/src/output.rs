//! Output types for code cells.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Output from executing a code cell.
///
/// Jupyter notebooks support four types of outputs:
/// - `ExecuteResult`: The result of evaluating the last expression
/// - `DisplayData`: Rich display output (plots, HTML, etc.)
/// - `Stream`: Text output to stdout or stderr
/// - `Error`: Exception/error information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "output_type", rename_all = "snake_case")]
pub enum Output {
    /// Result of evaluating the last expression in a cell.
    ExecuteResult {
        /// Execution count matching the cell's execution count.
        /// Can be None if cleaned or if the cell hasn't been executed.
        execution_count: Option<u32>,
        /// MIME bundle of output representations.
        data: MimeBundle,
        /// Output metadata.
        #[serde(default)]
        metadata: OutputMetadata,
    },

    /// Rich display output (e.g., plots, HTML, images).
    DisplayData {
        /// MIME bundle of output representations.
        data: MimeBundle,
        /// Output metadata.
        #[serde(default)]
        metadata: OutputMetadata,
    },

    /// Text output to stdout or stderr.
    Stream {
        /// Which stream (stdout or stderr).
        name: StreamName,
        /// The text content.
        text: MultilineString,
    },

    /// Error/exception output.
    Error {
        /// Exception class name.
        ename: String,
        /// Exception message.
        evalue: String,
        /// Traceback lines (may include ANSI color codes).
        traceback: Vec<String>,
    },
}

/// Stream name for stream outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamName {
    Stdout,
    Stderr,
}

/// A bundle of MIME-typed data representations.
///
/// Each key is a MIME type (e.g., "text/plain", "image/png"),
/// and the value is the content for that MIME type.
pub type MimeBundle = HashMap<String, MimeData>;

/// Data for a single MIME type in an output.
///
/// The data can be either:
/// - A single string (for small text content)
/// - An array of strings (lines, joined with newlines)
/// - A JSON object (for structured data like plotly)
///
/// For binary data (like images), the content is base64-encoded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MimeData {
    /// Single string value.
    String(String),
    /// Array of strings (typically lines).
    Lines(Vec<String>),
    /// JSON object data (for structured formats like plotly).
    Json(serde_json::Value),
}

impl MimeData {
    /// Get the content as a single string.
    ///
    /// For `Lines` variant, joins with empty string (lines typically
    /// include their own newlines).
    /// For `Json` variant, returns the JSON as a string.
    pub fn as_string(&self) -> String {
        match self {
            MimeData::String(s) => s.clone(),
            MimeData::Lines(lines) => lines.join(""),
            MimeData::Json(value) => serde_json::to_string(value).unwrap_or_default(),
        }
    }

    /// Create MimeData from a string, preserving line structure.
    pub fn from_string(s: String) -> Self {
        MimeData::String(s)
    }

    /// Create MimeData from lines.
    pub fn from_lines(lines: Vec<String>) -> Self {
        MimeData::Lines(lines)
    }

    /// Create MimeData from a JSON value.
    pub fn from_json(value: serde_json::Value) -> Self {
        MimeData::Json(value)
    }

    /// Check if this is JSON data.
    pub fn is_json(&self) -> bool {
        matches!(self, MimeData::Json(_))
    }
}

/// A string that may be serialized as a single string or array of lines.
///
/// In Jupyter notebooks, source code and text outputs can be stored
/// either as a single string or as an array of strings (one per line).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MultilineString {
    /// Single string.
    String(String),
    /// Array of strings (lines).
    Lines(Vec<String>),
}

impl MultilineString {
    /// Get the content as a single string.
    pub fn as_string(&self) -> String {
        match self {
            MultilineString::String(s) => s.clone(),
            MultilineString::Lines(lines) => lines.join(""),
        }
    }

    /// Create from a string.
    pub fn from_string(s: impl Into<String>) -> Self {
        MultilineString::String(s.into())
    }

    /// Create from lines.
    pub fn from_lines(lines: Vec<String>) -> Self {
        MultilineString::Lines(lines)
    }

    /// Convert to lines format (for ipynb serialization).
    pub fn to_lines(&self) -> Vec<String> {
        match self {
            MultilineString::String(s) => split_into_lines(s),
            MultilineString::Lines(lines) => lines.clone(),
        }
    }
}

impl Default for MultilineString {
    fn default() -> Self {
        MultilineString::String(String::new())
    }
}

impl From<String> for MultilineString {
    fn from(s: String) -> Self {
        MultilineString::String(s)
    }
}

impl From<&str> for MultilineString {
    fn from(s: &str) -> Self {
        MultilineString::String(s.to_string())
    }
}

/// Metadata associated with an output.
///
/// This is typically empty but can contain format-specific metadata
/// like image dimensions.
pub type OutputMetadata = HashMap<String, serde_json::Value>;

/// Split a string into lines, preserving trailing newlines.
///
/// Each line includes its trailing newline character, except possibly
/// the last line if the string doesn't end with a newline.
pub fn split_into_lines(s: &str) -> Vec<String> {
    if s.is_empty() {
        return vec![];
    }

    let mut lines = Vec::new();
    let mut start = 0;

    for (i, c) in s.char_indices() {
        if c == '\n' {
            lines.push(s[start..=i].to_string());
            start = i + 1;
        }
    }

    // Handle remaining content after last newline
    if start < s.len() {
        lines.push(s[start..].to_string());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_into_lines_empty() {
        assert_eq!(split_into_lines(""), Vec::<String>::new());
    }

    #[test]
    fn test_split_into_lines_no_newline() {
        assert_eq!(split_into_lines("hello"), vec!["hello"]);
    }

    #[test]
    fn test_split_into_lines_single_newline() {
        assert_eq!(split_into_lines("hello\n"), vec!["hello\n"]);
    }

    #[test]
    fn test_split_into_lines_multiple() {
        assert_eq!(
            split_into_lines("line1\nline2\nline3"),
            vec!["line1\n", "line2\n", "line3"]
        );
    }

    #[test]
    fn test_split_into_lines_trailing_newline() {
        assert_eq!(
            split_into_lines("line1\nline2\n"),
            vec!["line1\n", "line2\n"]
        );
    }

    #[test]
    fn test_multiline_string_round_trip() {
        let original = "line1\nline2\nline3";
        let ms = MultilineString::from_string(original);
        assert_eq!(ms.as_string(), original);

        let lines = ms.to_lines();
        let rejoined: String = lines.join("");
        assert_eq!(rejoined, original);
    }
}
