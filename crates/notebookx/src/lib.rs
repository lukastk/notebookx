//! notebookx - Fast, lightweight notebook conversion library
//!
//! This library provides a unified representation for Jupyter notebooks
//! and supports conversion between different notebook formats.
//!
//! # Example
//!
//! ```
//! use notebookx::{Notebook, Cell, NotebookFormat};
//!
//! // Create a notebook programmatically
//! let mut notebook = Notebook::new();
//! notebook.cells.push(Cell::markdown("# Hello World"));
//! notebook.cells.push(Cell::code("print('Hello!')"));
//!
//! // Serialize to ipynb format
//! let json = NotebookFormat::Ipynb.serialize(&notebook).unwrap();
//! ```

mod cell;
mod clean;
mod error;
mod metadata;
mod notebook;
mod output;

pub mod formats;

#[cfg(feature = "cli")]
pub mod cli;

pub use cell::{Cell, CellMetadata};
pub use clean::CleanOptions;
pub use error::{NotebookError, ParseError, SerializeError};
pub use formats::NotebookFormat;
pub use metadata::{KernelSpec, LanguageInfo, NotebookMetadata};
pub use notebook::Notebook;
pub use output::{MimeBundle, MimeData, MultilineString, Output, StreamName};
