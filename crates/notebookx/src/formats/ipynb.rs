//! Jupyter notebook (.ipynb) format parser and serializer.
//!
//! The ipynb format is a JSON-based format that stores notebooks as a single
//! JSON object with cells, metadata, and format version information.

use crate::cell::{Cell, CellMetadata};
use crate::metadata::NotebookMetadata;
use crate::notebook::Notebook;
use crate::output::{MimeBundle, MimeData, MultilineString, Output, OutputMetadata, StreamName};
use crate::{ParseError, SerializeError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parse a notebook from ipynb JSON format.
pub fn parse(input: &str) -> Result<Notebook, ParseError> {
    let raw: RawNotebook = serde_json::from_str(input)?;
    raw.into_notebook()
}

/// Serialize a notebook to ipynb JSON format.
pub fn serialize(notebook: &Notebook) -> Result<String, SerializeError> {
    let raw = RawNotebook::from_notebook(notebook);
    let json = serde_json::to_string_pretty(&raw)?;
    // Ensure trailing newline (Jupyter convention)
    Ok(json + "\n")
}

// ============================================================================
// Raw types for JSON serialization
// ============================================================================
//
// These types mirror the exact JSON structure of .ipynb files.
// They handle the various quirks of the format (e.g., source as string or array).

#[derive(Debug, Serialize, Deserialize)]
struct RawNotebook {
    cells: Vec<RawCell>,
    metadata: NotebookMetadata,
    nbformat: u8,
    nbformat_minor: u8,
}

impl RawNotebook {
    fn into_notebook(self) -> Result<Notebook, ParseError> {
        let cells = self
            .cells
            .into_iter()
            .map(|c| c.into_cell())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Notebook {
            cells,
            metadata: self.metadata,
            nbformat: self.nbformat,
            nbformat_minor: self.nbformat_minor,
        })
    }

    fn from_notebook(notebook: &Notebook) -> Self {
        RawNotebook {
            cells: notebook.cells.iter().map(RawCell::from_cell).collect(),
            metadata: notebook.metadata.clone(),
            nbformat: notebook.nbformat,
            nbformat_minor: notebook.nbformat_minor,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cell_type", rename_all = "lowercase")]
enum RawCell {
    Code {
        source: RawSource,
        execution_count: Option<u32>,
        #[serde(default)]
        outputs: Vec<RawOutput>,
        #[serde(default)]
        metadata: CellMetadata,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    Markdown {
        source: RawSource,
        #[serde(default)]
        metadata: CellMetadata,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    Raw {
        source: RawSource,
        #[serde(default)]
        metadata: CellMetadata,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
}

impl RawCell {
    fn into_cell(self) -> Result<Cell, ParseError> {
        match self {
            RawCell::Code {
                source,
                execution_count,
                outputs,
                metadata,
                id,
            } => {
                let outputs = outputs
                    .into_iter()
                    .map(|o| o.into_output())
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Cell::Code {
                    source: source.into_multiline_string(),
                    execution_count,
                    outputs,
                    metadata,
                    id,
                })
            }
            RawCell::Markdown {
                source,
                metadata,
                id,
            } => Ok(Cell::Markdown {
                source: source.into_multiline_string(),
                metadata,
                id,
            }),
            RawCell::Raw {
                source,
                metadata,
                id,
            } => Ok(Cell::Raw {
                source: source.into_multiline_string(),
                metadata,
                id,
            }),
        }
    }

    fn from_cell(cell: &Cell) -> Self {
        match cell {
            Cell::Code {
                source,
                execution_count,
                outputs,
                metadata,
                id,
            } => RawCell::Code {
                source: RawSource::from_multiline_string(source),
                execution_count: *execution_count,
                outputs: outputs.iter().map(RawOutput::from_output).collect(),
                metadata: metadata.clone(),
                id: id.clone(),
            },
            Cell::Markdown {
                source,
                metadata,
                id,
            } => RawCell::Markdown {
                source: RawSource::from_multiline_string(source),
                metadata: metadata.clone(),
                id: id.clone(),
            },
            Cell::Raw {
                source,
                metadata,
                id,
            } => RawCell::Raw {
                source: RawSource::from_multiline_string(source),
                metadata: metadata.clone(),
                id: id.clone(),
            },
        }
    }
}

/// Source can be either a single string or an array of strings.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum RawSource {
    String(String),
    Lines(Vec<String>),
}

impl RawSource {
    fn into_multiline_string(self) -> MultilineString {
        match self {
            RawSource::String(s) => MultilineString::String(s),
            RawSource::Lines(lines) => MultilineString::Lines(lines),
        }
    }

    fn from_multiline_string(ms: &MultilineString) -> Self {
        // Always serialize as lines (Jupyter convention)
        RawSource::Lines(ms.to_lines())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "output_type", rename_all = "snake_case")]
enum RawOutput {
    ExecuteResult {
        execution_count: Option<u32>,
        data: RawMimeBundle,
        #[serde(default)]
        metadata: OutputMetadata,
    },
    DisplayData {
        data: RawMimeBundle,
        #[serde(default)]
        metadata: OutputMetadata,
    },
    Stream {
        name: StreamName,
        text: RawSource,
    },
    Error {
        ename: String,
        evalue: String,
        traceback: Vec<String>,
    },
}

impl RawOutput {
    fn into_output(self) -> Result<Output, ParseError> {
        match self {
            RawOutput::ExecuteResult {
                execution_count,
                data,
                metadata,
            } => Ok(Output::ExecuteResult {
                execution_count,
                data: data.into_mime_bundle(),
                metadata,
            }),
            RawOutput::DisplayData { data, metadata } => Ok(Output::DisplayData {
                data: data.into_mime_bundle(),
                metadata,
            }),
            RawOutput::Stream { name, text } => Ok(Output::Stream {
                name,
                text: text.into_multiline_string(),
            }),
            RawOutput::Error {
                ename,
                evalue,
                traceback,
            } => Ok(Output::Error {
                ename,
                evalue,
                traceback,
            }),
        }
    }

    fn from_output(output: &Output) -> Self {
        match output {
            Output::ExecuteResult {
                execution_count,
                data,
                metadata,
            } => RawOutput::ExecuteResult {
                execution_count: *execution_count,
                data: RawMimeBundle::from_mime_bundle(data),
                metadata: metadata.clone(),
            },
            Output::DisplayData { data, metadata } => RawOutput::DisplayData {
                data: RawMimeBundle::from_mime_bundle(data),
                metadata: metadata.clone(),
            },
            Output::Stream { name, text } => RawOutput::Stream {
                name: *name,
                text: RawSource::from_multiline_string(text),
            },
            Output::Error {
                ename,
                evalue,
                traceback,
            } => RawOutput::Error {
                ename: ename.clone(),
                evalue: evalue.clone(),
                traceback: traceback.clone(),
            },
        }
    }
}

/// Raw MIME bundle that handles both string and array values.
type RawMimeBundle = HashMap<String, RawMimeData>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum RawMimeData {
    String(String),
    Lines(Vec<String>),
    Json(serde_json::Value),
}

impl RawMimeData {
    fn into_mime_data(self) -> MimeData {
        match self {
            RawMimeData::String(s) => MimeData::String(s),
            RawMimeData::Lines(lines) => MimeData::Lines(lines),
            RawMimeData::Json(value) => MimeData::Json(value),
        }
    }

    fn from_mime_data(data: &MimeData) -> Self {
        match data {
            MimeData::String(s) => RawMimeData::String(s.clone()),
            MimeData::Lines(lines) => RawMimeData::Lines(lines.clone()),
            MimeData::Json(value) => RawMimeData::Json(value.clone()),
        }
    }
}

trait MimeBundleExt {
    fn into_mime_bundle(self) -> MimeBundle;
    fn from_mime_bundle(bundle: &MimeBundle) -> Self;
}

impl MimeBundleExt for RawMimeBundle {
    fn into_mime_bundle(self) -> MimeBundle {
        self.into_iter()
            .map(|(k, v)| (k, v.into_mime_data()))
            .collect()
    }

    fn from_mime_bundle(bundle: &MimeBundle) -> Self {
        bundle
            .iter()
            .map(|(k, v)| (k.clone(), RawMimeData::from_mime_data(v)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_minimal_notebook() {
        let json = r#"{
            "cells": [],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        assert!(notebook.cells.is_empty());
        assert_eq!(notebook.nbformat, 4);
        assert_eq!(notebook.nbformat_minor, 5);
    }

    #[test]
    fn test_parse_code_cell() {
        let json = r#"{
            "cells": [{
                "cell_type": "code",
                "source": ["print('hello')\n", "print('world')"],
                "execution_count": 1,
                "outputs": [],
                "metadata": {}
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        assert_eq!(notebook.cells.len(), 1);

        let cell = &notebook.cells[0];
        assert!(cell.is_code());
        assert_eq!(cell.source_string(), "print('hello')\nprint('world')");
        assert_eq!(cell.execution_count(), Some(1));
    }

    #[test]
    fn test_parse_markdown_cell() {
        let json = r##"{
            "cells": [{
                "cell_type": "markdown",
                "source": "# Hello World",
                "metadata": {}
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"##;

        let notebook = parse(json).unwrap();
        assert_eq!(notebook.cells.len(), 1);

        let cell = &notebook.cells[0];
        assert!(cell.is_markdown());
        assert_eq!(cell.source_string(), "# Hello World");
    }

    #[test]
    fn test_parse_raw_cell() {
        let json = r#"{
            "cells": [{
                "cell_type": "raw",
                "source": "raw content",
                "metadata": {}
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        assert_eq!(notebook.cells.len(), 1);

        let cell = &notebook.cells[0];
        assert!(cell.is_raw());
        assert_eq!(cell.source_string(), "raw content");
    }

    #[test]
    fn test_parse_execute_result_output() {
        let json = r#"{
            "cells": [{
                "cell_type": "code",
                "source": "1 + 1",
                "execution_count": 1,
                "outputs": [{
                    "output_type": "execute_result",
                    "execution_count": 1,
                    "data": {
                        "text/plain": "2"
                    },
                    "metadata": {}
                }],
                "metadata": {}
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        let outputs = notebook.cells[0].outputs().unwrap();
        assert_eq!(outputs.len(), 1);

        match &outputs[0] {
            Output::ExecuteResult {
                execution_count,
                data,
                ..
            } => {
                assert_eq!(*execution_count, Some(1));
                assert_eq!(data.get("text/plain").unwrap().as_string(), "2");
            }
            _ => panic!("Expected ExecuteResult"),
        }
    }

    #[test]
    fn test_parse_display_data_output() {
        let json = r#"{
            "cells": [{
                "cell_type": "code",
                "source": "",
                "execution_count": 1,
                "outputs": [{
                    "output_type": "display_data",
                    "data": {
                        "text/html": "<b>bold</b>",
                        "text/plain": "bold"
                    },
                    "metadata": {}
                }],
                "metadata": {}
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        let outputs = notebook.cells[0].outputs().unwrap();

        match &outputs[0] {
            Output::DisplayData { data, .. } => {
                assert_eq!(data.get("text/html").unwrap().as_string(), "<b>bold</b>");
                assert_eq!(data.get("text/plain").unwrap().as_string(), "bold");
            }
            _ => panic!("Expected DisplayData"),
        }
    }

    #[test]
    fn test_parse_stream_output() {
        let json = r#"{
            "cells": [{
                "cell_type": "code",
                "source": "print('hello')",
                "execution_count": 1,
                "outputs": [{
                    "output_type": "stream",
                    "name": "stdout",
                    "text": "hello\n"
                }],
                "metadata": {}
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        let outputs = notebook.cells[0].outputs().unwrap();

        match &outputs[0] {
            Output::Stream { name, text } => {
                assert_eq!(*name, StreamName::Stdout);
                assert_eq!(text.as_string(), "hello\n");
            }
            _ => panic!("Expected Stream"),
        }
    }

    #[test]
    fn test_parse_error_output() {
        let json = r#"{
            "cells": [{
                "cell_type": "code",
                "source": "raise ValueError('oops')",
                "execution_count": 1,
                "outputs": [{
                    "output_type": "error",
                    "ename": "ValueError",
                    "evalue": "oops",
                    "traceback": ["line 1", "line 2"]
                }],
                "metadata": {}
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        let outputs = notebook.cells[0].outputs().unwrap();

        match &outputs[0] {
            Output::Error {
                ename,
                evalue,
                traceback,
            } => {
                assert_eq!(ename, "ValueError");
                assert_eq!(evalue, "oops");
                assert_eq!(traceback, &vec!["line 1", "line 2"]);
            }
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_parse_notebook_metadata() {
        let json = r#"{
            "cells": [],
            "metadata": {
                "kernelspec": {
                    "display_name": "Python 3",
                    "language": "python",
                    "name": "python3"
                },
                "language_info": {
                    "name": "python",
                    "version": "3.9.0"
                }
            },
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        let kernelspec = notebook.metadata.kernelspec.as_ref().unwrap();
        assert_eq!(kernelspec.display_name, "Python 3");
        assert_eq!(kernelspec.language, "python");
        assert_eq!(kernelspec.name, "python3");

        let lang_info = notebook.metadata.language_info.as_ref().unwrap();
        assert_eq!(lang_info.name, "python");
        assert_eq!(lang_info.version, Some("3.9.0".to_string()));
    }

    #[test]
    fn test_serialize_round_trip() {
        let json = r#"{
            "cells": [{
                "cell_type": "code",
                "source": ["print('hello')"],
                "execution_count": 1,
                "outputs": [{
                    "output_type": "stream",
                    "name": "stdout",
                    "text": ["hello\n"]
                }],
                "metadata": {}
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        let serialized = serialize(&notebook).unwrap();
        let reparsed = parse(&serialized).unwrap();

        assert_eq!(notebook.cells.len(), reparsed.cells.len());
        assert_eq!(
            notebook.cells[0].source_string(),
            reparsed.cells[0].source_string()
        );
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_cells() {
        let json = r#"{"metadata": {}, "nbformat": 4, "nbformat_minor": 5}"#;
        let result = parse(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_source_as_string() {
        let json = r#"{
            "cells": [{
                "cell_type": "code",
                "source": "single string source",
                "execution_count": null,
                "outputs": [],
                "metadata": {}
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        assert_eq!(notebook.cells[0].source_string(), "single string source");
    }

    #[test]
    fn test_parse_cell_with_id() {
        let json = r#"{
            "cells": [{
                "cell_type": "code",
                "id": "abc123",
                "source": "",
                "execution_count": null,
                "outputs": [],
                "metadata": {}
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        assert_eq!(notebook.cells[0].id(), Some("abc123"));
    }

    #[test]
    fn test_parse_cell_metadata_tags() {
        let json = r#"{
            "cells": [{
                "cell_type": "code",
                "source": "",
                "execution_count": null,
                "outputs": [],
                "metadata": {
                    "tags": ["hide", "skip"]
                }
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        let tags = notebook.cells[0].metadata().tags.as_ref().unwrap();
        assert_eq!(tags, &vec!["hide".to_string(), "skip".to_string()]);
    }

    #[test]
    fn test_parse_mime_bundle_with_lines() {
        let json = r#"{
            "cells": [{
                "cell_type": "code",
                "source": "",
                "execution_count": 1,
                "outputs": [{
                    "output_type": "execute_result",
                    "execution_count": 1,
                    "data": {
                        "text/plain": ["line1\n", "line2\n", "line3"]
                    },
                    "metadata": {}
                }],
                "metadata": {}
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        let outputs = notebook.cells[0].outputs().unwrap();

        match &outputs[0] {
            Output::ExecuteResult { data, .. } => {
                assert_eq!(
                    data.get("text/plain").unwrap().as_string(),
                    "line1\nline2\nline3"
                );
            }
            _ => panic!("Expected ExecuteResult"),
        }
    }

    #[test]
    fn test_parse_empty_cell() {
        let json = r#"{
            "cells": [{
                "cell_type": "code",
                "source": "",
                "execution_count": null,
                "outputs": [],
                "metadata": {}
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"#;

        let notebook = parse(json).unwrap();
        assert_eq!(notebook.cells[0].source_string(), "");
    }

    #[test]
    fn test_parse_unicode_content() {
        let json = r##"{
            "cells": [{
                "cell_type": "markdown",
                "source": "# 你好世界 🌍\n\nこんにちは",
                "metadata": {}
            }],
            "metadata": {},
            "nbformat": 4,
            "nbformat_minor": 5
        }"##;

        let notebook = parse(json).unwrap();
        assert_eq!(
            notebook.cells[0].source_string(),
            "# 你好世界 🌍\n\nこんにちは"
        );
    }

    #[test]
    fn test_serialize_preserves_cell_order() {
        let mut notebook = Notebook::new();
        notebook.cells.push(Cell::markdown("# First"));
        notebook.cells.push(Cell::code("second()"));
        notebook.cells.push(Cell::markdown("# Third"));

        let serialized = serialize(&notebook).unwrap();
        let reparsed = parse(&serialized).unwrap();

        assert_eq!(reparsed.cells.len(), 3);
        assert_eq!(reparsed.cells[0].source_string(), "# First");
        assert_eq!(reparsed.cells[1].source_string(), "second()");
        assert_eq!(reparsed.cells[2].source_string(), "# Third");
    }
}
