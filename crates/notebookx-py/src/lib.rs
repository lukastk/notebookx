//! Python bindings for notebookx.
//!
//! This module provides PyO3 bindings to expose the notebookx library
//! to Python.

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use std::collections::HashSet;
use std::path::PathBuf;

/// Supported notebook formats.
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Jupyter notebook format (.ipynb)
    Ipynb,
    /// Python percent format (.pct.py)
    Percent,
}

impl From<Format> for notebookx::NotebookFormat {
    fn from(f: Format) -> Self {
        match f {
            Format::Ipynb => notebookx::NotebookFormat::Ipynb,
            Format::Percent => notebookx::NotebookFormat::Percent,
        }
    }
}

impl From<notebookx::NotebookFormat> for Format {
    fn from(f: notebookx::NotebookFormat) -> Self {
        match f {
            notebookx::NotebookFormat::Ipynb => Format::Ipynb,
            notebookx::NotebookFormat::Percent => Format::Percent,
        }
    }
}

/// Options for cleaning a notebook.
#[pyclass]
#[derive(Clone, Default)]
pub struct CleanOptions {
    /// Remove all outputs from code cells.
    #[pyo3(get, set)]
    pub remove_outputs: bool,

    /// Remove execution counts from code cells.
    #[pyo3(get, set)]
    pub remove_execution_counts: bool,

    /// Remove cell-level metadata.
    #[pyo3(get, set)]
    pub remove_cell_metadata: bool,

    /// Remove notebook-level metadata.
    #[pyo3(get, set)]
    pub remove_notebook_metadata: bool,

    /// Remove kernel specification.
    #[pyo3(get, set)]
    pub remove_kernel_info: bool,

    /// Preserve cell IDs.
    #[pyo3(get, set)]
    pub preserve_cell_ids: bool,

    /// Remove metadata from outputs (ExecuteResult, DisplayData).
    #[pyo3(get, set)]
    pub remove_output_metadata: bool,

    /// Remove execution counts from output results.
    #[pyo3(get, set)]
    pub remove_output_execution_counts: bool,

    /// Allowed cell metadata keys (None means all allowed).
    #[pyo3(get, set)]
    pub allowed_cell_metadata_keys: Option<Vec<String>>,

    /// Allowed notebook metadata keys (None means all allowed).
    #[pyo3(get, set)]
    pub allowed_notebook_metadata_keys: Option<Vec<String>>,
}

#[pymethods]
impl CleanOptions {
    #[new]
    #[pyo3(signature = (
        remove_outputs = false,
        remove_execution_counts = false,
        remove_cell_metadata = false,
        remove_notebook_metadata = false,
        remove_kernel_info = false,
        preserve_cell_ids = false,
        remove_output_metadata = false,
        remove_output_execution_counts = false,
        allowed_cell_metadata_keys = None,
        allowed_notebook_metadata_keys = None,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        remove_outputs: bool,
        remove_execution_counts: bool,
        remove_cell_metadata: bool,
        remove_notebook_metadata: bool,
        remove_kernel_info: bool,
        preserve_cell_ids: bool,
        remove_output_metadata: bool,
        remove_output_execution_counts: bool,
        allowed_cell_metadata_keys: Option<Vec<String>>,
        allowed_notebook_metadata_keys: Option<Vec<String>>,
    ) -> Self {
        CleanOptions {
            remove_outputs,
            remove_execution_counts,
            remove_cell_metadata,
            remove_notebook_metadata,
            remove_kernel_info,
            preserve_cell_ids,
            remove_output_metadata,
            remove_output_execution_counts,
            allowed_cell_metadata_keys,
            allowed_notebook_metadata_keys,
        }
    }

    /// Create options for version control (removes cell metadata, execution counts, and output metadata/execution counts).
    #[staticmethod]
    fn for_vcs() -> Self {
        CleanOptions {
            remove_cell_metadata: true,
            remove_execution_counts: true,
            remove_output_metadata: true,
            remove_output_execution_counts: true,
            ..Default::default()
        }
    }

    /// Create options that strip all metadata and outputs.
    #[staticmethod]
    fn strip_all() -> Self {
        CleanOptions {
            remove_outputs: true,
            remove_execution_counts: true,
            remove_cell_metadata: true,
            remove_notebook_metadata: true,
            remove_kernel_info: true,
            preserve_cell_ids: false,
            remove_output_metadata: true,
            remove_output_execution_counts: true,
            allowed_cell_metadata_keys: None,
            allowed_notebook_metadata_keys: None,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "CleanOptions(remove_outputs={}, remove_execution_counts={}, remove_cell_metadata={}, remove_notebook_metadata={}, remove_kernel_info={}, preserve_cell_ids={}, remove_output_metadata={}, remove_output_execution_counts={})",
            self.remove_outputs,
            self.remove_execution_counts,
            self.remove_cell_metadata,
            self.remove_notebook_metadata,
            self.remove_kernel_info,
            self.preserve_cell_ids,
            self.remove_output_metadata,
            self.remove_output_execution_counts,
        )
    }
}

impl From<&CleanOptions> for notebookx::CleanOptions {
    fn from(opts: &CleanOptions) -> Self {
        notebookx::CleanOptions {
            remove_outputs: opts.remove_outputs,
            remove_execution_counts: opts.remove_execution_counts,
            remove_cell_metadata: opts.remove_cell_metadata,
            remove_notebook_metadata: opts.remove_notebook_metadata,
            remove_kernel_info: opts.remove_kernel_info,
            preserve_cell_ids: opts.preserve_cell_ids,
            remove_output_metadata: opts.remove_output_metadata,
            remove_output_execution_counts: opts.remove_output_execution_counts,
            allowed_cell_metadata_keys: opts
                .allowed_cell_metadata_keys
                .as_ref()
                .map(|v| v.iter().cloned().collect::<HashSet<_>>()),
            allowed_notebook_metadata_keys: opts
                .allowed_notebook_metadata_keys
                .as_ref()
                .map(|v| v.iter().cloned().collect::<HashSet<_>>()),
        }
    }
}

/// A Jupyter notebook.
#[pyclass]
pub struct Notebook {
    inner: notebookx::Notebook,
}

#[pymethods]
impl Notebook {
    /// Create a new empty notebook.
    #[new]
    fn new() -> Self {
        Notebook {
            inner: notebookx::Notebook::new(),
        }
    }

    /// Load a notebook from a file.
    ///
    /// Args:
    ///     path: Path to the notebook file.
    ///     format: Optional format. If not specified, inferred from extension.
    ///
    /// Returns:
    ///     The loaded notebook.
    ///
    /// Raises:
    ///     ValueError: If the file cannot be parsed.
    ///     IOError: If the file cannot be read.
    #[staticmethod]
    #[pyo3(signature = (path, format = None))]
    fn from_file(path: &str, format: Option<Format>) -> PyResult<Self> {
        let path_buf = PathBuf::from(path);

        let fmt = match format {
            Some(f) => f.into(),
            None => notebookx::NotebookFormat::from_path(&path_buf).ok_or_else(|| {
                PyValueError::new_err(format!(
                    "Cannot infer format from '{}'. Specify format explicitly.",
                    path
                ))
            })?,
        };

        let content = std::fs::read_to_string(&path_buf)
            .map_err(|e| PyIOError::new_err(format!("Failed to read '{}': {}", path, e)))?;

        let notebook = fmt
            .parse(&content)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse '{}': {}", path, e)))?;

        Ok(Notebook { inner: notebook })
    }

    /// Load a notebook from a string.
    ///
    /// Args:
    ///     content: The notebook content as a string.
    ///     format: The format of the content.
    ///
    /// Returns:
    ///     The loaded notebook.
    ///
    /// Raises:
    ///     ValueError: If the content cannot be parsed.
    #[staticmethod]
    fn from_string(content: &str, format: Format) -> PyResult<Self> {
        let fmt: notebookx::NotebookFormat = format.into();
        let notebook = fmt
            .parse(content)
            .map_err(|e| PyValueError::new_err(format!("Failed to parse content: {}", e)))?;

        Ok(Notebook { inner: notebook })
    }

    /// Save the notebook to a file.
    ///
    /// Args:
    ///     path: Path to save the notebook to.
    ///     format: Optional format. If not specified, inferred from extension.
    ///
    /// Raises:
    ///     ValueError: If the notebook cannot be serialized.
    ///     IOError: If the file cannot be written.
    #[pyo3(signature = (path, format = None))]
    fn to_file(&self, path: &str, format: Option<Format>) -> PyResult<()> {
        let path_buf = PathBuf::from(path);

        let fmt = match format {
            Some(f) => f.into(),
            None => notebookx::NotebookFormat::from_path(&path_buf).ok_or_else(|| {
                PyValueError::new_err(format!(
                    "Cannot infer format from '{}'. Specify format explicitly.",
                    path
                ))
            })?,
        };

        let content = fmt
            .serialize(&self.inner)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize: {}", e)))?;

        std::fs::write(&path_buf, content)
            .map_err(|e| PyIOError::new_err(format!("Failed to write '{}': {}", path, e)))?;

        Ok(())
    }

    /// Serialize the notebook to a string.
    ///
    /// Args:
    ///     format: The format to serialize to.
    ///
    /// Returns:
    ///     The notebook as a string.
    ///
    /// Raises:
    ///     ValueError: If the notebook cannot be serialized.
    fn to_string(&self, format: Format) -> PyResult<String> {
        let fmt: notebookx::NotebookFormat = format.into();
        fmt.serialize(&self.inner)
            .map_err(|e| PyValueError::new_err(format!("Failed to serialize: {}", e)))
    }

    /// Clean the notebook according to the specified options.
    ///
    /// This returns a new notebook with the requested content removed.
    /// The original notebook is not modified.
    ///
    /// Args:
    ///     options: Cleaning options. If not specified, uses default options.
    ///
    /// Returns:
    ///     A new cleaned notebook.
    #[pyo3(signature = (options = None))]
    fn clean(&self, options: Option<&CleanOptions>) -> Self {
        let rust_options = options.map(|o| o.into()).unwrap_or_default();

        Notebook {
            inner: self.inner.clean(&rust_options),
        }
    }

    /// Get the number of cells in the notebook.
    fn __len__(&self) -> usize {
        self.inner.len()
    }

    /// Check if the notebook is empty.
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the number of code cells.
    #[getter]
    fn code_cell_count(&self) -> usize {
        self.inner.cells.iter().filter(|c| c.is_code()).count()
    }

    /// Get the number of markdown cells.
    #[getter]
    fn markdown_cell_count(&self) -> usize {
        self.inner.cells.iter().filter(|c| c.is_markdown()).count()
    }

    /// Get the number of raw cells.
    #[getter]
    fn raw_cell_count(&self) -> usize {
        self.inner.cells.iter().filter(|c| c.is_raw()).count()
    }

    /// Get the nbformat version.
    #[getter]
    fn nbformat(&self) -> u8 {
        self.inner.nbformat
    }

    /// Get the nbformat minor version.
    #[getter]
    fn nbformat_minor(&self) -> u8 {
        self.inner.nbformat_minor
    }

    fn __repr__(&self) -> String {
        format!(
            "Notebook(cells={}, nbformat={}.{})",
            self.inner.len(),
            self.inner.nbformat,
            self.inner.nbformat_minor
        )
    }
}

// Custom exception for notebook errors.
pyo3::create_exception!(_notebookx, NotebookError, pyo3::exceptions::PyException);

/// Convert a notebook between formats.
///
/// Args:
///     input_path: Path to the input notebook.
///     output_path: Path to save the converted notebook.
///     from_fmt: Optional input format. If not specified, inferred from extension.
///     to_fmt: Optional output format. If not specified, inferred from extension.
///
/// Raises:
///     ValueError: If formats cannot be inferred or conversion fails.
///     IOError: If files cannot be read or written.
#[pyfunction]
#[pyo3(signature = (input_path, output_path, from_fmt = None, to_fmt = None))]
fn convert(
    input_path: &str,
    output_path: &str,
    from_fmt: Option<Format>,
    to_fmt: Option<Format>,
) -> PyResult<()> {
    let notebook = Notebook::from_file(input_path, from_fmt)?;
    notebook.to_file(output_path, to_fmt)?;
    Ok(())
}

/// Clean a notebook file.
///
/// Args:
///     input_path: Path to the input notebook.
///     output_path: Optional path to save the cleaned notebook.
///                  If not specified, cleans in place.
///     **options: Cleaning options passed to CleanOptions.
///
/// Raises:
///     ValueError: If the file cannot be parsed.
///     IOError: If files cannot be read or written.
#[pyfunction]
#[pyo3(signature = (
    input_path,
    output_path = None,
    remove_outputs = false,
    remove_execution_counts = false,
    remove_cell_metadata = false,
    remove_notebook_metadata = false,
    remove_kernel_info = false,
    preserve_cell_ids = false,
    remove_output_metadata = false,
    remove_output_execution_counts = false,
))]
#[allow(clippy::too_many_arguments)]
fn clean_notebook(
    input_path: &str,
    output_path: Option<&str>,
    remove_outputs: bool,
    remove_execution_counts: bool,
    remove_cell_metadata: bool,
    remove_notebook_metadata: bool,
    remove_kernel_info: bool,
    preserve_cell_ids: bool,
    remove_output_metadata: bool,
    remove_output_execution_counts: bool,
) -> PyResult<()> {
    let notebook = Notebook::from_file(input_path, None)?;

    let options = CleanOptions {
        remove_outputs,
        remove_execution_counts,
        remove_cell_metadata,
        remove_notebook_metadata,
        remove_kernel_info,
        preserve_cell_ids,
        remove_output_metadata,
        remove_output_execution_counts,
        allowed_cell_metadata_keys: None,
        allowed_notebook_metadata_keys: None,
    };

    let cleaned = notebook.clean(Some(&options));
    let output = output_path.unwrap_or(input_path);
    cleaned.to_file(output, None)?;

    Ok(())
}

/// Run the nbx CLI with the given arguments.
///
/// This function runs the Rust CLI directly, providing the same functionality
/// as the standalone `nbx` binary.
///
/// Args:
///     args: Command-line arguments (including the program name as first element).
///
/// Returns:
///     Exit code (0 for success, non-zero for errors).
#[pyfunction]
fn run_cli(args: Vec<String>) -> i32 {
    notebookx::cli::run(args)
}

/// Python module for notebookx.
#[pymodule]
fn _notebookx(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Format>()?;
    m.add_class::<CleanOptions>()?;
    m.add_class::<Notebook>()?;
    m.add_function(wrap_pyfunction!(convert, m)?)?;
    m.add_function(wrap_pyfunction!(clean_notebook, m)?)?;
    m.add_function(wrap_pyfunction!(run_cli, m)?)?;
    m.add("NotebookError", m.py().get_type::<NotebookError>())?;
    Ok(())
}
