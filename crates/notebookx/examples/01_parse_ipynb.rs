//! Example 01: Parsing an existing ipynb file
//!
//! This example demonstrates how to parse a Jupyter notebook file
//! and access its contents.
//!
//! Run with: cargo run --example 01_parse_ipynb

use notebookx::NotebookFormat;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to the example notebook
    let notebook_path = Path::new("nb_format_examples/World population.ipynb");

    // Check if the file exists
    if !notebook_path.exists() {
        eprintln!("Error: Notebook file not found at {:?}", notebook_path);
        eprintln!("Make sure you're running from the repository root directory.");
        std::process::exit(1);
    }

    // Read the file content
    let content = std::fs::read_to_string(notebook_path)?;
    println!("Read {} bytes from {:?}", content.len(), notebook_path);

    // Parse the notebook
    let notebook = NotebookFormat::Ipynb.parse(&content)?;

    // Print basic info
    println!("\n=== Notebook Info ===");
    println!("Format: nbformat {}.{}", notebook.nbformat, notebook.nbformat_minor);
    println!("Total cells: {}", notebook.len());

    // Print kernel info
    if let Some(ref kernelspec) = notebook.metadata.kernelspec {
        println!("\n=== Kernel Info ===");
        println!("Name: {}", kernelspec.name);
        println!("Display name: {}", kernelspec.display_name);
        println!("Language: {}", kernelspec.language);
    }

    // Count cell types
    let code_cells = notebook.cells.iter().filter(|c| c.is_code()).count();
    let markdown_cells = notebook.cells.iter().filter(|c| c.is_markdown()).count();
    let raw_cells = notebook.cells.iter().filter(|c| c.is_raw()).count();

    println!("\n=== Cell Statistics ===");
    println!("Code cells: {}", code_cells);
    println!("Markdown cells: {}", markdown_cells);
    println!("Raw cells: {}", raw_cells);

    // Print first few cells
    println!("\n=== First 3 Cells ===");
    for (i, cell) in notebook.cells.iter().take(3).enumerate() {
        let cell_type = if cell.is_code() {
            "CODE"
        } else if cell.is_markdown() {
            "MARKDOWN"
        } else {
            "RAW"
        };

        let source = cell.source_string();
        let preview = if source.len() > 100 {
            format!("{}...", &source[..100])
        } else {
            source
        };

        println!("\n[Cell {} - {}]", i, cell_type);
        println!("{}", preview.replace('\n', "\n  "));

        // Show execution count for code cells
        if let Some(exec_count) = cell.execution_count() {
            println!("  (execution_count: {})", exec_count);
        }

        // Show output count for code cells
        if let Some(outputs) = cell.outputs() {
            if !outputs.is_empty() {
                println!("  ({} output(s))", outputs.len());
            }
        }
    }

    Ok(())
}
