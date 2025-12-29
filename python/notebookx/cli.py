"""Command-line interface for notebookx.

This module provides the `nbx` command when notebookx is installed via pip.
"""

import argparse
import sys
from typing import Optional

from notebookx import Notebook, Format, CleanOptions


def infer_format(path: str) -> Optional[Format]:
    """Infer format from file path."""
    if path.endswith(".ipynb"):
        return Format.Ipynb
    elif path.endswith(".pct.py"):
        return Format.Percent
    return None


def cmd_convert(args: argparse.Namespace) -> int:
    """Handle the convert command."""
    # Determine input format
    from_fmt = None
    if args.from_fmt:
        from_fmt = Format.Ipynb if args.from_fmt == "ipynb" else Format.Percent

    # Determine output format
    to_fmt = None
    if args.to_fmt:
        to_fmt = Format.Ipynb if args.to_fmt == "ipynb" else Format.Percent

    try:
        # Handle stdin
        if args.input == "-":
            if not from_fmt:
                print("Error: --from-fmt is required when reading from stdin", file=sys.stderr)
                return 1
            content = sys.stdin.read()
            nb = Notebook.from_string(content, from_fmt)
        else:
            nb = Notebook.from_file(args.input, from_fmt)

        # Determine output format from path if not specified
        if not to_fmt and args.to != "-":
            to_fmt = infer_format(args.to)

        if not to_fmt:
            print("Error: --to-fmt is required when writing to stdout or format cannot be inferred", file=sys.stderr)
            return 1

        # Handle stdout
        if args.to == "-":
            print(nb.to_string(to_fmt))
        else:
            nb.to_file(args.to, to_fmt)

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1

    return 0


def cmd_clean(args: argparse.Namespace) -> int:
    """Handle the clean command."""
    try:
        nb = Notebook.from_file(args.input)

        options = CleanOptions(
            remove_outputs=args.remove_outputs,
            remove_execution_counts=args.remove_execution_counts,
            remove_cell_metadata=args.remove_cell_metadata,
            remove_notebook_metadata=args.remove_notebook_metadata,
            remove_kernel_info=args.remove_kernel_info,
            remove_output_metadata=args.remove_output_metadata,
            remove_output_execution_counts=args.remove_output_execution_counts,
        )

        cleaned = nb.clean(options)

        # Determine output path
        if args.output:
            output_path = args.output
        elif args.in_place:
            output_path = args.input
        else:
            # Write to stdout
            print(cleaned.to_string(Format.Ipynb))
            return 0

        cleaned.to_file(output_path)

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1

    return 0


def main() -> int:
    """Main entry point for the nbx CLI."""
    parser = argparse.ArgumentParser(
        prog="nbx",
        description="Fast, lightweight notebook conversion tool",
    )
    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # Convert command
    convert_parser = subparsers.add_parser("convert", help="Convert between notebook formats")
    convert_parser.add_argument("input", help="Input file (use - for stdin)")
    convert_parser.add_argument("--to", required=True, help="Output file (use - for stdout)")
    convert_parser.add_argument("--from-fmt", choices=["ipynb", "percent"], help="Input format")
    convert_parser.add_argument("--to-fmt", choices=["ipynb", "percent"], help="Output format")

    # Clean command
    clean_parser = subparsers.add_parser("clean", help="Clean notebook by removing outputs and metadata")
    clean_parser.add_argument("input", help="Input notebook file")
    clean_parser.add_argument("-o", "--output", help="Output file")
    clean_parser.add_argument("-i", "--in-place", action="store_true", help="Modify file in place")
    clean_parser.add_argument("-O", "--remove-outputs", action="store_true", help="Remove all outputs")
    clean_parser.add_argument("-e", "--remove-execution-counts", action="store_true", help="Remove execution counts")
    clean_parser.add_argument("--remove-cell-metadata", action="store_true", help="Remove cell metadata")
    clean_parser.add_argument("--remove-notebook-metadata", action="store_true", help="Remove notebook metadata")
    clean_parser.add_argument("--remove-kernel-info", action="store_true", help="Remove kernel info")
    clean_parser.add_argument("--remove-output-metadata", action="store_true", help="Remove metadata from outputs")
    clean_parser.add_argument("--remove-output-execution-counts", action="store_true", help="Remove execution counts from output results")

    args = parser.parse_args()

    if args.command == "convert":
        return cmd_convert(args)
    elif args.command == "clean":
        return cmd_clean(args)
    else:
        parser.print_help()
        return 0


def cli_main():
    """Entry point that exits with the return code."""
    sys.exit(main())


if __name__ == "__main__":
    cli_main()
