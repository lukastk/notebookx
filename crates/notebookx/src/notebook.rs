//! The core Notebook struct and its methods.

use crate::cell::Cell;
use crate::metadata::NotebookMetadata;

/// A Jupyter notebook.
///
/// This is the central representation of a notebook in notebookx.
/// It closely mirrors the structure of the Jupyter `.ipynb` format
/// (nbformat version 4).
///
/// # Example
///
/// ```
/// use notebookx::{Notebook, Cell};
///
/// let mut notebook = Notebook::new();
/// notebook.cells.push(Cell::markdown("# Hello World"));
/// notebook.cells.push(Cell::code("print('Hello!')"));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Notebook {
    /// The cells in this notebook.
    pub cells: Vec<Cell>,
    /// Notebook-level metadata.
    pub metadata: NotebookMetadata,
    /// Major version of the notebook format (always 4).
    pub nbformat: u8,
    /// Minor version of the notebook format.
    pub nbformat_minor: u8,
}

impl Notebook {
    /// Create a new empty notebook.
    ///
    /// The notebook is created with nbformat version 4.5 (the current version
    /// with cell ID support) and empty metadata.
    pub fn new() -> Self {
        Notebook {
            cells: Vec::new(),
            metadata: NotebookMetadata::default(),
            nbformat: 4,
            nbformat_minor: 5,
        }
    }

    /// Create a notebook with the specified cells.
    pub fn with_cells(cells: Vec<Cell>) -> Self {
        Notebook {
            cells,
            metadata: NotebookMetadata::default(),
            nbformat: 4,
            nbformat_minor: 5,
        }
    }

    /// Get the number of cells in this notebook.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Check if this notebook has no cells.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Iterate over the cells in this notebook.
    pub fn iter(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter()
    }

    /// Iterate over the cells mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Cell> {
        self.cells.iter_mut()
    }

    /// Get only the code cells.
    pub fn code_cells(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter().filter(|c| c.is_code())
    }

    /// Get only the markdown cells.
    pub fn markdown_cells(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter().filter(|c| c.is_markdown())
    }
}

impl Default for Notebook {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Notebook {
    type Item = Cell;
    type IntoIter = std::vec::IntoIter<Cell>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

impl<'a> IntoIterator for &'a Notebook {
    type Item = &'a Cell;
    type IntoIter = std::slice::Iter<'a, Cell>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}
