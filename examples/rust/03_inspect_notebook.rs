//! Example 03: Inspecting notebook contents in detail
//!
//! This example demonstrates how to deeply inspect notebook contents,
//! including outputs, metadata, and MIME data.
//!
//! Run with: cargo run --example 03_inspect_notebook

use notebookx::NotebookFormat;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let notebook_path = Path::new("nb_format_examples/World population.ipynb");

    if !notebook_path.exists() {
        eprintln!("Error: Notebook file not found at {:?}", notebook_path);
        std::process::exit(1);
    }

    let content = std::fs::read_to_string(notebook_path)?;
    let notebook = NotebookFormat::Ipynb.parse(&content)?;

    // Inspect notebook metadata
    println!("=== Notebook Metadata ===");
    println!(
        "nbformat: {}.{}",
        notebook.nbformat, notebook.nbformat_minor
    );

    if let Some(ref kernelspec) = notebook.metadata.kernelspec {
        println!("\nKernel Specification:");
        println!("  Name: {}", kernelspec.name);
        println!("  Display name: {}", kernelspec.display_name);
        println!("  Language: {}", kernelspec.language);
    }

    if let Some(ref lang_info) = notebook.metadata.language_info {
        println!("\nLanguage Info:");
        println!("  Name: {}", lang_info.name);
        if let Some(ref version) = lang_info.version {
            println!("  Version: {}", version);
        }
        if let Some(ref mimetype) = lang_info.mimetype {
            println!("  MIME type: {}", mimetype);
        }
        if let Some(ref ext) = lang_info.file_extension {
            println!("  File extension: {}", ext);
        }
    }

    // Inspect cells with outputs
    println!("\n=== Cells with Outputs ===");
    for (i, cell) in notebook.cells.iter().enumerate() {
        if let Some(outputs) = cell.outputs() {
            if outputs.is_empty() {
                continue;
            }

            println!(
                "\nCell {} (execution_count: {:?}):",
                i,
                cell.execution_count()
            );

            // Show a preview of the source
            let source = cell.source_string();
            let preview: String = source.lines().take(2).collect::<Vec<_>>().join("\n");
            println!("  Source preview: {}", preview.replace('\n', " | "));

            // Inspect each output
            for (j, output) in outputs.iter().enumerate() {
                println!("  Output {}:", j);
                match output {
                    notebookx::Output::ExecuteResult {
                        data,
                        execution_count,
                        ..
                    } => {
                        println!(
                            "    Type: ExecuteResult (execution_count: {:?})",
                            execution_count
                        );
                        print_mime_bundle(data);
                    }
                    notebookx::Output::DisplayData { data, .. } => {
                        println!("    Type: DisplayData");
                        print_mime_bundle(data);
                    }
                    notebookx::Output::Stream { name, text } => {
                        let text_str = text.as_string();
                        let preview = if text_str.len() > 50 {
                            format!("{}...", &text_str[..50])
                        } else {
                            text_str
                        };
                        println!("    Type: Stream ({:?})", name);
                        println!("    Content: {}", preview.replace('\n', "\\n"));
                    }
                    notebookx::Output::Error { ename, evalue, .. } => {
                        println!("    Type: Error");
                        println!("    Name: {}", ename);
                        println!("    Value: {}", evalue);
                    }
                }
            }
        }
    }

    // Summary statistics
    println!("\n=== Summary Statistics ===");
    let total_outputs: usize = notebook
        .cells
        .iter()
        .filter_map(|c| c.outputs())
        .map(|o| o.len())
        .sum();

    let cells_with_outputs = notebook
        .cells
        .iter()
        .filter(|c| c.outputs().map(|o| !o.is_empty()).unwrap_or(false))
        .count();

    println!("Total cells: {}", notebook.len());
    println!("Cells with outputs: {}", cells_with_outputs);
    println!("Total outputs: {}", total_outputs);

    Ok(())
}

fn print_mime_bundle(data: &notebookx::MimeBundle) {
    println!("    MIME types present:");
    for (mime_type, mime_data) in data {
        let size_info = match mime_data {
            notebookx::MimeData::String(s) => format!("{} chars", s.len()),
            notebookx::MimeData::Lines(lines) => format!("{} lines", lines.len()),
            notebookx::MimeData::Json(_) => "JSON object".to_string(),
        };
        println!("      - {}: {}", mime_type, size_info);
    }
}
