"""Type stubs for notebookx."""

from enum import Enum
from typing import Optional, List

class Format(Enum):
    """Supported notebook formats."""
    Ipynb = ...
    Percent = ...

class CleanOptions:
    """Options for cleaning a notebook."""

    remove_outputs: bool
    remove_execution_counts: bool
    remove_cell_metadata: bool
    remove_notebook_metadata: bool
    remove_kernel_info: bool
    preserve_cell_ids: bool
    remove_output_metadata: bool
    remove_output_execution_counts: bool
    allowed_cell_metadata_keys: Optional[List[str]]
    allowed_notebook_metadata_keys: Optional[List[str]]

    def __init__(
        self,
        remove_outputs: bool = False,
        remove_execution_counts: bool = False,
        remove_cell_metadata: bool = False,
        remove_notebook_metadata: bool = False,
        remove_kernel_info: bool = False,
        preserve_cell_ids: bool = False,
        remove_output_metadata: bool = False,
        remove_output_execution_counts: bool = False,
        allowed_cell_metadata_keys: Optional[List[str]] = None,
        allowed_notebook_metadata_keys: Optional[List[str]] = None,
    ) -> None: ...

    @staticmethod
    def for_vcs() -> CleanOptions:
        """Create options for version control (removes cell metadata, execution counts, and output metadata/execution counts)."""
        ...

    @staticmethod
    def strip_all() -> CleanOptions:
        """Create options that strip all metadata and outputs."""
        ...

class Notebook:
    """A Jupyter notebook."""

    @property
    def code_cell_count(self) -> int:
        """Get the number of code cells."""
        ...

    @property
    def markdown_cell_count(self) -> int:
        """Get the number of markdown cells."""
        ...

    @property
    def raw_cell_count(self) -> int:
        """Get the number of raw cells."""
        ...

    @property
    def nbformat(self) -> int:
        """Get the nbformat version."""
        ...

    @property
    def nbformat_minor(self) -> int:
        """Get the nbformat minor version."""
        ...

    def __init__(self) -> None:
        """Create a new empty notebook."""
        ...

    @staticmethod
    def from_file(path: str, format: Optional[Format] = None) -> Notebook:
        """Load a notebook from a file.

        Args:
            path: Path to the notebook file.
            format: Optional format. If not specified, inferred from extension.

        Returns:
            The loaded notebook.

        Raises:
            ValueError: If the file cannot be parsed.
            IOError: If the file cannot be read.
        """
        ...

    @staticmethod
    def from_string(content: str, format: Format) -> Notebook:
        """Load a notebook from a string.

        Args:
            content: The notebook content as a string.
            format: The format of the content.

        Returns:
            The loaded notebook.

        Raises:
            ValueError: If the content cannot be parsed.
        """
        ...

    def to_file(self, path: str, format: Optional[Format] = None) -> None:
        """Save the notebook to a file.

        Args:
            path: Path to save the notebook to.
            format: Optional format. If not specified, inferred from extension.

        Raises:
            ValueError: If the notebook cannot be serialized.
            IOError: If the file cannot be written.
        """
        ...

    def to_string(self, format: Format) -> str:
        """Serialize the notebook to a string.

        Args:
            format: The format to serialize to.

        Returns:
            The notebook as a string.

        Raises:
            ValueError: If the notebook cannot be serialized.
        """
        ...

    def clean(self, options: Optional[CleanOptions] = None) -> Notebook:
        """Clean the notebook according to the specified options.

        This returns a new notebook with the requested content removed.
        The original notebook is not modified.

        Args:
            options: Cleaning options. If not specified, uses default options.

        Returns:
            A new cleaned notebook.
        """
        ...

    def is_empty(self) -> bool:
        """Check if the notebook is empty."""
        ...

    def __len__(self) -> int:
        """Get the number of cells in the notebook."""
        ...

    def __repr__(self) -> str: ...

class NotebookError(Exception):
    """Custom exception for notebook errors."""
    ...

def convert(
    input_path: str,
    output_path: str,
    from_fmt: Optional[Format] = None,
    to_fmt: Optional[Format] = None,
) -> None:
    """Convert a notebook between formats.

    Args:
        input_path: Path to the input notebook.
        output_path: Path to save the converted notebook.
        from_fmt: Optional input format. If not specified, inferred from extension.
        to_fmt: Optional output format. If not specified, inferred from extension.

    Raises:
        ValueError: If formats cannot be inferred or conversion fails.
        IOError: If files cannot be read or written.
    """
    ...

def clean_notebook(
    input_path: str,
    output_path: Optional[str] = None,
    remove_outputs: bool = False,
    remove_execution_counts: bool = False,
    remove_cell_metadata: bool = False,
    remove_notebook_metadata: bool = False,
    remove_kernel_info: bool = False,
    preserve_cell_ids: bool = False,
    remove_output_metadata: bool = False,
    remove_output_execution_counts: bool = False,
) -> None:
    """Clean a notebook file.

    Args:
        input_path: Path to the input notebook.
        output_path: Optional path to save the cleaned notebook.
                     If not specified, cleans in place.
        remove_outputs: Remove all outputs from code cells.
        remove_execution_counts: Remove execution counts from code cells.
        remove_cell_metadata: Remove cell-level metadata.
        remove_notebook_metadata: Remove notebook-level metadata.
        remove_kernel_info: Remove kernel specification.
        preserve_cell_ids: Preserve cell IDs.
        remove_output_metadata: Remove metadata from outputs (ExecuteResult, DisplayData).
        remove_output_execution_counts: Remove execution counts from output results.

    Raises:
        ValueError: If the file cannot be parsed.
        IOError: If files cannot be read or written.
    """
    ...

def run_cli(args: List[str]) -> int:
    """Run the nbx CLI with the given arguments.

    This function runs the Rust CLI directly, providing the same functionality
    as the standalone `nbx` binary.

    Args:
        args: Command-line arguments (including the program name as first element).

    Returns:
        Exit code (0 for success, non-zero for errors).
    """
    ...

__version__: str
__all__: List[str]
