//! Example 02: Converting between notebook formats
//!
//! This example demonstrates how to convert notebooks between
//! ipynb and percent formats.
//!
//! Run with: cargo run --example 02_convert_formats

use notebookx::NotebookFormat;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to the example notebook
    let notebook_path = Path::new("nb_format_examples/World population.ipynb");

    if !notebook_path.exists() {
        eprintln!("Error: Notebook file not found at {:?}", notebook_path);
        std::process::exit(1);
    }

    // Read and parse the ipynb file
    let content = std::fs::read_to_string(notebook_path)?;
    let notebook = NotebookFormat::Ipynb.parse(&content)?;

    println!("Loaded notebook with {} cells", notebook.len());

    // Convert to percent format
    let percent_output = NotebookFormat::Percent.serialize(&notebook)?;
    println!("\n=== Converted to Percent Format ===");
    println!("{}", &percent_output[..percent_output.len().min(1000)]);
    if percent_output.len() > 1000 {
        println!("... (truncated)");
    }

    // Parse the percent format back
    let reparsed = NotebookFormat::Percent.parse(&percent_output)?;
    println!("\n=== Round-trip Verification ===");
    println!("Original cells: {}", notebook.len());
    println!("Reparsed cells: {}", reparsed.len());

    // Verify cell content is preserved
    let mut all_match = true;
    for (i, (orig, new)) in notebook.cells.iter().zip(reparsed.cells.iter()).enumerate() {
        let orig_src = orig.source_string();
        let new_src = new.source_string();
        if orig_src.trim() != new_src.trim() {
            println!("Cell {} content differs!", i);
            all_match = false;
        }
    }

    if all_match {
        println!("All cell contents preserved correctly!");
    }

    // Demonstrate format detection from path
    println!("\n=== Format Detection ===");
    let paths = [
        "notebook.ipynb",
        "script.pct.py",
        "unknown.txt",
    ];

    for path_str in paths {
        let path = Path::new(path_str);
        match NotebookFormat::from_path(path) {
            Some(format) => println!("{} -> {} format", path_str, format),
            None => println!("{} -> unknown format", path_str),
        }
    }

    Ok(())
}
