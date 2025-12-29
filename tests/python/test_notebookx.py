"""Tests for the notebookx Python bindings."""

import json
import os
import tempfile

import pytest

from notebookx import (
    Notebook,
    Format,
    CleanOptions,
    convert,
    clean_notebook,
)

EXAMPLE_IPYNB = "nb_format_examples/World population.ipynb"
EXAMPLE_PERCENT = "nb_format_examples/World population.pct.py"


class TestNotebook:
    """Tests for the Notebook class."""

    def test_new_notebook(self):
        """Test creating a new empty notebook."""
        nb = Notebook()
        assert len(nb) == 0
        assert nb.is_empty()
        assert nb.code_cell_count == 0
        assert nb.markdown_cell_count == 0
        assert nb.nbformat == 4
        assert nb.nbformat_minor == 5

    def test_from_file_ipynb(self):
        """Test loading a notebook from an ipynb file."""
        nb = Notebook.from_file(EXAMPLE_IPYNB)
        assert len(nb) > 0
        assert not nb.is_empty()
        assert nb.code_cell_count > 0
        assert nb.markdown_cell_count > 0

    def test_from_file_percent(self):
        """Test loading a notebook from a percent file."""
        nb = Notebook.from_file(EXAMPLE_PERCENT)
        assert len(nb) > 0
        assert nb.code_cell_count > 0
        assert nb.markdown_cell_count > 0

    def test_from_file_with_explicit_format(self):
        """Test loading a notebook with explicit format."""
        nb = Notebook.from_file(EXAMPLE_IPYNB, Format.Ipynb)
        assert len(nb) > 0

    def test_from_file_not_found(self):
        """Test loading a non-existent file raises IOError."""
        with pytest.raises(IOError):
            Notebook.from_file("nonexistent.ipynb")

    def test_from_string_ipynb(self):
        """Test loading a notebook from an ipynb string."""
        with open(EXAMPLE_IPYNB) as f:
            content = f.read()
        nb = Notebook.from_string(content, Format.Ipynb)
        assert len(nb) > 0

    def test_from_string_percent(self):
        """Test loading a notebook from a percent string."""
        with open(EXAMPLE_PERCENT) as f:
            content = f.read()
        nb = Notebook.from_string(content, Format.Percent)
        assert len(nb) > 0

    def test_from_string_invalid(self):
        """Test loading invalid content raises ValueError."""
        with pytest.raises(ValueError):
            Notebook.from_string("invalid json", Format.Ipynb)

    def test_to_string_ipynb(self):
        """Test serializing to ipynb string."""
        nb = Notebook.from_file(EXAMPLE_IPYNB)
        content = nb.to_string(Format.Ipynb)
        assert '"cells"' in content
        assert '"cell_type"' in content

    def test_to_string_percent(self):
        """Test serializing to percent string."""
        nb = Notebook.from_file(EXAMPLE_IPYNB)
        content = nb.to_string(Format.Percent)
        assert "# %%" in content
        assert "# %% [markdown]" in content

    def test_to_file(self):
        """Test saving a notebook to a file."""
        nb = Notebook.from_file(EXAMPLE_IPYNB)
        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = os.path.join(tmpdir, "output.pct.py")
            nb.to_file(output_path)
            assert os.path.exists(output_path)
            with open(output_path) as f:
                content = f.read()
            assert "# %%" in content

    def test_to_file_with_explicit_format(self):
        """Test saving a notebook with explicit format."""
        nb = Notebook.from_file(EXAMPLE_IPYNB)
        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = os.path.join(tmpdir, "output.txt")
            nb.to_file(output_path, Format.Percent)
            with open(output_path) as f:
                content = f.read()
            assert "# %%" in content

    def test_repr(self):
        """Test notebook string representation."""
        nb = Notebook()
        assert "Notebook" in repr(nb)
        assert "cells=0" in repr(nb)


class TestCleanOptions:
    """Tests for the CleanOptions class."""

    def test_default_options(self):
        """Test default clean options."""
        options = CleanOptions()
        assert not options.remove_outputs
        assert not options.remove_execution_counts
        assert not options.remove_cell_metadata
        assert not options.remove_notebook_metadata
        assert not options.remove_kernel_info
        assert not options.preserve_cell_ids

    def test_custom_options(self):
        """Test custom clean options."""
        options = CleanOptions(
            remove_outputs=True,
            remove_execution_counts=True,
        )
        assert options.remove_outputs
        assert options.remove_execution_counts

    def test_for_vcs(self):
        """Test for_vcs preset."""
        options = CleanOptions.for_vcs()
        # for_vcs preserves outputs but removes metadata/execution counts
        assert not options.remove_outputs
        assert options.remove_execution_counts
        assert options.remove_cell_metadata
        assert options.remove_output_metadata
        assert options.remove_output_execution_counts

    def test_strip_all(self):
        """Test strip_all preset."""
        options = CleanOptions.strip_all()
        assert options.remove_outputs
        assert options.remove_execution_counts
        assert options.remove_cell_metadata
        assert options.remove_notebook_metadata
        assert options.remove_kernel_info

    def test_repr(self):
        """Test clean options string representation."""
        options = CleanOptions()
        assert "CleanOptions" in repr(options)


class TestClean:
    """Tests for the clean functionality."""

    def test_clean_default(self):
        """Test cleaning with default options."""
        nb = Notebook.from_file(EXAMPLE_IPYNB)
        cleaned = nb.clean()
        assert len(cleaned) == len(nb)

    def test_clean_removes_outputs(self):
        """Test cleaning removes outputs."""
        nb = Notebook.from_file(EXAMPLE_IPYNB)
        options = CleanOptions(remove_outputs=True)
        cleaned = nb.clean(options)

        # Verify outputs are removed by serializing and checking
        content = cleaned.to_string(Format.Ipynb)
        data = json.loads(content)
        for cell in data["cells"]:
            if cell["cell_type"] == "code":
                assert cell["outputs"] == []

    def test_clean_removes_execution_counts(self):
        """Test cleaning removes execution counts."""
        nb = Notebook.from_file(EXAMPLE_IPYNB)
        options = CleanOptions(remove_execution_counts=True)
        cleaned = nb.clean(options)

        content = cleaned.to_string(Format.Ipynb)
        data = json.loads(content)
        for cell in data["cells"]:
            if cell["cell_type"] == "code":
                assert cell["execution_count"] is None

    def test_clean_immutable(self):
        """Test that cleaning doesn't modify the original notebook."""
        nb = Notebook.from_file(EXAMPLE_IPYNB)
        original_len = len(nb)

        options = CleanOptions.strip_all()
        cleaned = nb.clean(options)

        assert len(nb) == original_len
        assert len(cleaned) == original_len


class TestConvert:
    """Tests for the convert convenience function."""

    def test_convert_ipynb_to_percent(self):
        """Test converting ipynb to percent."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = os.path.join(tmpdir, "output.pct.py")
            convert(EXAMPLE_IPYNB, output_path)

            assert os.path.exists(output_path)
            with open(output_path) as f:
                content = f.read()
            assert "# %%" in content

    def test_convert_percent_to_ipynb(self):
        """Test converting percent to ipynb."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = os.path.join(tmpdir, "output.ipynb")
            convert(EXAMPLE_PERCENT, output_path)

            assert os.path.exists(output_path)
            with open(output_path) as f:
                content = f.read()
            assert '"cells"' in content

    def test_convert_with_explicit_formats(self):
        """Test converting with explicit formats."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = os.path.join(tmpdir, "output.txt")
            convert(EXAMPLE_IPYNB, output_path,
                   from_fmt=Format.Ipynb, to_fmt=Format.Percent)

            with open(output_path) as f:
                content = f.read()
            assert "# %%" in content


class TestCleanNotebook:
    """Tests for the clean_notebook convenience function."""

    def test_clean_notebook_to_new_file(self):
        """Test cleaning a notebook to a new file."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = os.path.join(tmpdir, "clean.ipynb")
            clean_notebook(EXAMPLE_IPYNB, output_path,
                          remove_outputs=True)

            assert os.path.exists(output_path)
            nb = Notebook.from_file(output_path)

            content = nb.to_string(Format.Ipynb)
            data = json.loads(content)
            for cell in data["cells"]:
                if cell["cell_type"] == "code":
                    assert cell["outputs"] == []

    def test_clean_notebook_in_place(self):
        """Test cleaning a notebook in place."""
        with tempfile.TemporaryDirectory() as tmpdir:
            # First copy the file
            input_path = os.path.join(tmpdir, "notebook.ipynb")
            with open(EXAMPLE_IPYNB) as f:
                content = f.read()
            with open(input_path, "w") as f:
                f.write(content)

            # Clean in place
            clean_notebook(input_path, remove_outputs=True)

            # Verify
            nb = Notebook.from_file(input_path)
            content = nb.to_string(Format.Ipynb)
            data = json.loads(content)
            for cell in data["cells"]:
                if cell["cell_type"] == "code":
                    assert cell["outputs"] == []


class TestFormat:
    """Tests for the Format enum."""

    def test_format_values(self):
        """Test format enum values exist."""
        assert Format.Ipynb is not None
        assert Format.Percent is not None

    def test_format_equality(self):
        """Test format enum equality."""
        assert Format.Ipynb == Format.Ipynb
        assert Format.Ipynb != Format.Percent


class TestRoundTrip:
    """Tests for round-trip conversions."""

    def test_ipynb_percent_ipynb(self):
        """Test round-trip: ipynb -> percent -> ipynb."""
        nb1 = Notebook.from_file(EXAMPLE_IPYNB)

        # Convert to percent
        percent_content = nb1.to_string(Format.Percent)

        # Parse percent
        nb2 = Notebook.from_string(percent_content, Format.Percent)

        # Convert back to ipynb
        ipynb_content = nb2.to_string(Format.Ipynb)

        # Parse ipynb
        nb3 = Notebook.from_string(ipynb_content, Format.Ipynb)

        # Verify cell count is preserved
        assert len(nb1) == len(nb3)
