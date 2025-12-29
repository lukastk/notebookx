//! Command-line interface for notebookx.
//!
//! This module provides the `nbx` CLI tool for converting and cleaning
//! Jupyter notebooks. It can be used both as a standalone binary and
//! called from Python via PyO3.
//!
//! # Example
//!
//! ```bash
//! # Convert notebook to percent format
//! nbx convert notebook.ipynb --output script.pct.py
//!
//! # Clean a notebook (remove outputs)
//! nbx clean notebook.ipynb --remove-outputs --in-place
//! ```

use clap::{Parser, Subcommand, ValueEnum};
use std::collections::HashSet;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::{CleanOptions, NotebookFormat};

/// Exit codes for scripting
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const PARSE_ERROR: i32 = 1;
    pub const SERIALIZE_ERROR: i32 = 2;
    pub const IO_ERROR: i32 = 3;
    pub const INVALID_ARGS: i32 = 4;
}

/// Fast, lightweight notebook conversion tool
#[derive(Parser)]
#[command(name = "nbx")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a notebook between formats
    Convert {
        /// Input file (use '-' for stdin)
        input: String,

        /// Output file (use '-' for stdout)
        #[arg(short, long)]
        output: String,

        /// Input format (inferred from extension if not specified)
        #[arg(long, value_name = "FORMAT")]
        from_fmt: Option<Format>,

        /// Output format (inferred from extension if not specified)
        #[arg(long, value_name = "FORMAT")]
        to_fmt: Option<Format>,

        /// Remove outputs during conversion
        #[arg(long)]
        strip_outputs: bool,

        /// Remove metadata during conversion
        #[arg(long)]
        strip_metadata: bool,
    },

    /// Clean a notebook (remove outputs, metadata, etc.)
    Clean {
        /// Input file
        input: PathBuf,

        /// Output file (if not specified, prints to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Modify the input file in place
        #[arg(short, long, conflicts_with = "output")]
        in_place: bool,

        /// Remove all outputs from code cells
        #[arg(short = 'O', long)]
        remove_outputs: bool,

        /// Remove execution counts from code cells
        #[arg(short = 'e', long)]
        remove_execution_counts: bool,

        /// Remove cell-level metadata
        #[arg(long)]
        remove_cell_metadata: bool,

        /// Remove notebook-level metadata
        #[arg(long)]
        remove_notebook_metadata: bool,

        /// Remove kernel specification
        #[arg(long)]
        remove_kernel_info: bool,

        /// Preserve cell IDs
        #[arg(long)]
        preserve_cell_ids: bool,

        /// Remove metadata from outputs (ExecuteResult, DisplayData)
        #[arg(long)]
        remove_output_metadata: bool,

        /// Remove execution counts from output results
        #[arg(long)]
        remove_output_execution_counts: bool,

        /// Keep only these metadata keys (comma-separated)
        #[arg(long, value_delimiter = ',')]
        keep_only: Option<Vec<String>>,
    },
}

/// Supported notebook formats for CLI
#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Format {
    /// Jupyter notebook format (.ipynb)
    Ipynb,
    /// Python percent format (.pct.py)
    Percent,
}

impl From<Format> for NotebookFormat {
    fn from(f: Format) -> Self {
        match f {
            Format::Ipynb => NotebookFormat::Ipynb,
            Format::Percent => NotebookFormat::Percent,
        }
    }
}

/// Run the CLI with the given arguments.
///
/// This is the main entry point for the CLI, used by both the native binary
/// and the Python wrapper.
///
/// # Arguments
///
/// * `args` - Command-line arguments, including the program name as the first element
///
/// # Returns
///
/// Exit code (0 for success, non-zero for errors)
///
/// # Example
///
/// ```no_run
/// use notebookx::cli;
///
/// // Run with command-line arguments
/// let exit_code = cli::run(std::env::args_os());
/// std::process::exit(exit_code);
/// ```
pub fn run<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(e) => {
            // Print help/error message
            let _ = e.print();
            return if e.kind() == clap::error::ErrorKind::DisplayHelp
                || e.kind() == clap::error::ErrorKind::DisplayVersion
            {
                exit_codes::SUCCESS
            } else {
                exit_codes::INVALID_ARGS
            };
        }
    };

    let result = match cli.command {
        Commands::Convert {
            input,
            output,
            from_fmt,
            to_fmt,
            strip_outputs,
            strip_metadata,
        } => run_convert(
            &input,
            &output,
            from_fmt,
            to_fmt,
            strip_outputs,
            strip_metadata,
        ),
        Commands::Clean {
            input,
            output,
            in_place,
            remove_outputs,
            remove_execution_counts,
            remove_cell_metadata,
            remove_notebook_metadata,
            remove_kernel_info,
            preserve_cell_ids,
            remove_output_metadata,
            remove_output_execution_counts,
            keep_only,
        } => run_clean(
            &input,
            output.as_deref(),
            in_place,
            remove_outputs,
            remove_execution_counts,
            remove_cell_metadata,
            remove_notebook_metadata,
            remove_kernel_info,
            preserve_cell_ids,
            remove_output_metadata,
            remove_output_execution_counts,
            keep_only,
        ),
    };

    match result {
        Ok(()) => exit_codes::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            e.exit_code()
        }
    }
}

/// CLI error types
#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error("Failed to parse '{path}': {message}")]
    Parse { path: String, message: String },

    #[error("Failed to serialize notebook: {0}")]
    Serialize(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    InvalidArgs(String),
}

impl CliError {
    fn exit_code(&self) -> i32 {
        match self {
            CliError::Parse { .. } => exit_codes::PARSE_ERROR,
            CliError::Serialize(_) => exit_codes::SERIALIZE_ERROR,
            CliError::Io(_) => exit_codes::IO_ERROR,
            CliError::InvalidArgs(_) => exit_codes::INVALID_ARGS,
        }
    }
}

/// Infer format from path, handling stdin/stdout special case
fn infer_format(path: &str, explicit_format: Option<Format>) -> Result<NotebookFormat, CliError> {
    if let Some(fmt) = explicit_format {
        return Ok(fmt.into());
    }

    if path == "-" {
        return Err(CliError::InvalidArgs(
            "Cannot infer format for stdin/stdout. Use --from-fmt or --to-fmt.".to_string(),
        ));
    }

    let path_buf = PathBuf::from(path);
    NotebookFormat::from_path(&path_buf).ok_or_else(|| {
        CliError::InvalidArgs(format!(
            "Cannot infer format from '{}'. Use --from-fmt or --to-fmt.",
            path
        ))
    })
}

/// Read content from file or stdin
fn read_content(path: &str) -> Result<String, CliError> {
    if path == "-" {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content)?;
        Ok(content)
    } else {
        std::fs::read_to_string(path).map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                CliError::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("File not found: {}", path),
                ))
            } else {
                CliError::Io(e)
            }
        })
    }
}

/// Write content to file or stdout
fn write_content(path: &str, content: &str) -> Result<(), CliError> {
    if path == "-" {
        io::stdout().write_all(content.as_bytes())?;
        Ok(())
    } else {
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Run the convert command
fn run_convert(
    input: &str,
    output: &str,
    from_fmt: Option<Format>,
    to_fmt: Option<Format>,
    strip_outputs: bool,
    strip_metadata: bool,
) -> Result<(), CliError> {
    let input_format = infer_format(input, from_fmt)?;
    let output_format = infer_format(output, to_fmt)?;

    let content = read_content(input)?;

    let notebook = input_format.parse(&content).map_err(|e| CliError::Parse {
        path: input.to_string(),
        message: e.to_string(),
    })?;

    // Apply cleaning if requested
    let notebook = if strip_outputs || strip_metadata {
        let options = CleanOptions {
            remove_outputs: strip_outputs,
            remove_execution_counts: strip_outputs,
            remove_cell_metadata: strip_metadata,
            remove_notebook_metadata: strip_metadata,
            ..Default::default()
        };
        notebook.clean(&options)
    } else {
        notebook
    };

    let output_content = output_format
        .serialize(&notebook)
        .map_err(|e| CliError::Serialize(e.to_string()))?;

    write_content(output, &output_content)?;

    Ok(())
}

/// Run the clean command
#[allow(clippy::too_many_arguments)]
fn run_clean(
    input: &Path,
    output: Option<&Path>,
    in_place: bool,
    remove_outputs: bool,
    remove_execution_counts: bool,
    remove_cell_metadata: bool,
    remove_notebook_metadata: bool,
    remove_kernel_info: bool,
    preserve_cell_ids: bool,
    remove_output_metadata: bool,
    remove_output_execution_counts: bool,
    keep_only: Option<Vec<String>>,
) -> Result<(), CliError> {
    let format = NotebookFormat::from_path(input).ok_or_else(|| {
        CliError::InvalidArgs(format!(
            "Cannot infer format from '{}'. Supported extensions: .ipynb, .pct.py",
            input.display()
        ))
    })?;

    let content = std::fs::read_to_string(input).map_err(|e| {
        if e.kind() == io::ErrorKind::NotFound {
            CliError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("File not found: {}", input.display()),
            ))
        } else {
            CliError::Io(e)
        }
    })?;

    let notebook = format.parse(&content).map_err(|e| CliError::Parse {
        path: input.display().to_string(),
        message: e.to_string(),
    })?;

    let allowed_keys = keep_only.map(|keys| keys.into_iter().collect::<HashSet<_>>());

    let options = CleanOptions {
        remove_outputs,
        remove_execution_counts,
        remove_cell_metadata,
        remove_notebook_metadata,
        remove_kernel_info,
        preserve_cell_ids,
        remove_output_metadata,
        remove_output_execution_counts,
        allowed_cell_metadata_keys: allowed_keys.clone(),
        allowed_notebook_metadata_keys: allowed_keys,
    };

    let cleaned = notebook.clean(&options);

    let output_content = format
        .serialize(&cleaned)
        .map_err(|e| CliError::Serialize(e.to_string()))?;

    if in_place {
        std::fs::write(input, &output_content)?;
    } else if let Some(output_path) = output {
        std::fs::write(output_path, &output_content)?;
    } else {
        io::stdout().write_all(output_content.as_bytes())?;
    }

    Ok(())
}
