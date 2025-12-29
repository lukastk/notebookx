//! Example 00: Creating a basic notebook programmatically
//!
//! This example demonstrates how to create a simple Jupyter notebook
//! from scratch using the notebookx library.
//!
//! Run with: cargo run --example 00_basic_notebook

use notebookx::{Cell, Notebook, NotebookFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new empty notebook
    let mut notebook = Notebook::new();

    // Add a markdown cell with a title
    notebook.cells.push(Cell::markdown(
        "# My First Notebook\n\nThis notebook was created with notebookx!",
    ));

    // Add a code cell
    notebook.cells.push(Cell::code("print('Hello, World!')"));

    // Add another markdown cell
    notebook.cells.push(Cell::markdown(
        "## Data Analysis\n\nLet's do some calculations:",
    ));

    // Add more code cells
    notebook.cells.push(Cell::code(
        "x = 10\ny = 20\nresult = x + y\nprint(f'The sum is: {result}')",
    ));

    // Add a final markdown cell
    notebook.cells.push(Cell::markdown(
        "### Conclusion\n\nWe successfully created a notebook programmatically!",
    ));

    // Print notebook info
    println!("Created notebook with {} cells:", notebook.len());
    for (i, cell) in notebook.iter().enumerate() {
        let cell_type = if cell.is_code() {
            "code"
        } else if cell.is_markdown() {
            "markdown"
        } else {
            "raw"
        };
        println!(
            "  Cell {}: {} ({} chars)",
            i,
            cell_type,
            cell.source_string().len()
        );
    }

    // Serialize to ipynb format
    let ipynb_output = NotebookFormat::Ipynb.serialize(&notebook)?;
    println!("\n--- ipynb output (first 500 chars) ---");
    println!("{}", &ipynb_output[..ipynb_output.len().min(500)]);

    // Serialize to percent format
    let percent_output = NotebookFormat::Percent.serialize(&notebook)?;
    println!("\n--- percent format output ---");
    println!("{}", percent_output);

    Ok(())
}
