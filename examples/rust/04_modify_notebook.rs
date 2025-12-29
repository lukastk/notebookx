//! Example 04: Modifying notebook contents
//!
//! This example demonstrates how to modify notebooks: adding, removing,
//! and reordering cells, as well as modifying metadata.
//!
//! Run with: cargo run --example 04_modify_notebook

use notebookx::{Cell, KernelSpec, Notebook, NotebookFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start with a new notebook
    let mut notebook = Notebook::new();

    println!("=== Building a Notebook ===");

    // Add cells programmatically
    notebook.cells.push(Cell::markdown(
        "# Data Analysis Notebook\n\nThis notebook demonstrates data analysis techniques.",
    ));
    notebook
        .cells
        .push(Cell::code("import pandas as pd\nimport numpy as np"));
    notebook.cells.push(Cell::markdown("## Loading Data"));
    notebook
        .cells
        .push(Cell::code("df = pd.read_csv('data.csv')\ndf.head()"));
    notebook.cells.push(Cell::markdown("## Analysis"));
    notebook.cells.push(Cell::code("df.describe()"));

    println!("Created notebook with {} cells", notebook.len());

    // Modify metadata
    notebook.metadata.kernelspec = Some(KernelSpec {
        name: "python3".to_string(),
        display_name: "Python 3".to_string(),
        language: "python".to_string(),
    });

    println!(
        "Set kernel to: {}",
        notebook.metadata.kernelspec.as_ref().unwrap().display_name
    );

    // Insert a cell at a specific position
    println!("\n=== Inserting a Cell ===");
    let setup_cell = Cell::code("# Setup\nimport warnings\nwarnings.filterwarnings('ignore')");
    notebook.cells.insert(1, setup_cell);
    println!("Inserted setup cell at position 1");
    println!("New cell count: {}", notebook.len());

    // Remove a cell
    println!("\n=== Removing a Cell ===");
    let removed = notebook.cells.remove(5); // Remove "## Analysis" markdown
    println!(
        "Removed cell with content: {}...",
        removed.source_string().chars().take(30).collect::<String>()
    );
    println!("New cell count: {}", notebook.len());

    // Modify an existing cell's source
    println!("\n=== Modifying Cell Content ===");
    if let Some(cell) = notebook.cells.get_mut(0) {
        *cell =
            Cell::markdown("# Advanced Data Analysis\n\nUpdated title for our analysis notebook.");
        println!("Updated first cell content");
    }

    // Filter cells by type
    println!("\n=== Filtering Cells ===");
    let code_cells: Vec<_> = notebook.cells.iter().filter(|c| c.is_code()).collect();
    let markdown_cells: Vec<_> = notebook.cells.iter().filter(|c| c.is_markdown()).collect();
    println!("Code cells: {}", code_cells.len());
    println!("Markdown cells: {}", markdown_cells.len());

    // Create a code-only version
    println!("\n=== Creating Code-Only Version ===");
    let mut code_only = Notebook::new();
    code_only.metadata = notebook.metadata.clone();
    code_only.cells = notebook
        .cells
        .iter()
        .filter(|c| c.is_code())
        .cloned()
        .collect();
    println!("Code-only notebook has {} cells", code_only.len());

    // Serialize both versions
    println!("\n=== Serialization ===");
    let full_ipynb = NotebookFormat::Ipynb.serialize(&notebook)?;
    let code_only_percent = NotebookFormat::Percent.serialize(&code_only)?;

    println!("Full notebook (ipynb): {} bytes", full_ipynb.len());
    println!("Code-only (percent): {} bytes", code_only_percent.len());

    // Show the percent format output
    println!("\n=== Code-Only Percent Format ===");
    println!("{}", code_only_percent);

    Ok(())
}
