"""notebookx - Fast, lightweight notebook conversion library.

This library provides a unified representation for Jupyter notebooks
and supports conversion between different notebook formats.

Example:
    >>> from notebookx import Notebook, Format
    >>>
    >>> # Load a notebook
    >>> nb = Notebook.from_file("example.ipynb")
    >>>
    >>> # Convert to percent format
    >>> nb.to_file("example.pct.py")
    >>>
    >>> # Clean a notebook
    >>> clean_nb = nb.clean(remove_outputs=True)
"""

from notebookx._notebookx import (
    Notebook,
    Format,
    CleanOptions,
    convert,
    clean_notebook,
    run_cli,
    NotebookError,
    format_from_extension,
    format_from_path,
    format_extension,
)

__all__ = [
    "Notebook",
    "Format",
    "CleanOptions",
    "convert",
    "clean_notebook",
    "run_cli",
    "NotebookError",
    "format_from_extension",
    "format_from_path",
    "format_extension",
]

__version__ = "0.1.0"
