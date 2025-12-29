"""Command-line interface for notebookx.

This module provides the `nbx` command when notebookx is installed via pip.
It's a thin wrapper that calls the Rust CLI implementation directly.
"""

import sys

from notebookx._notebookx import run_cli


def cli_main():
    """Entry point that runs the Rust CLI and exits with its return code."""
    sys.exit(run_cli(sys.argv))


if __name__ == "__main__":
    cli_main()
