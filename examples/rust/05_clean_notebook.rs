//! Example: Cleaning notebooks
//!
//! This example demonstrates how to clean notebooks by removing outputs,
//! execution counts, and metadata.
//!
//! Run with:
//! ```
//! cargo run --example 05_clean_notebook
//! ```

use notebookx::{Cell, CleanOptions, Notebook, NotebookFormat};

fn main() {
    // Create a notebook with outputs and metadata
    let mut notebook = Notebook::new();

    // Add a code cell with simulated output
    notebook.cells.push(Cell::Code {
        source: notebookx::MultilineString::from_string("print('Hello, World!')"),
        execution_count: Some(1),
        outputs: vec![notebookx::Output::Stream {
            name: notebookx::StreamName::Stdout,
            text: notebookx::MultilineString::from_string("Hello, World!\n"),
        }],
        metadata: notebookx::CellMetadata {
            tags: Some(vec!["example".to_string()]),
            ..Default::default()
        },
        id: Some("cell-001".to_string()),
    });

    notebook.cells.push(Cell::markdown("# Results"));

    notebook.cells.push(Cell::Code {
        source: notebookx::MultilineString::from_string("2 + 2"),
        execution_count: Some(2),
        outputs: vec![notebookx::Output::ExecuteResult {
            execution_count: 2,
            data: {
                let mut data = notebookx::MimeBundle::new();
                data.insert(
                    "text/plain".to_string(),
                    notebookx::MimeData::String("4".to_string()),
                );
                data
            },
            metadata: Default::default(),
        }],
        metadata: Default::default(),
        id: Some("cell-002".to_string()),
    });

    println!("Original notebook:");
    print_notebook_info(&notebook);

    // Clean for version control (removes outputs and execution counts)
    let vcs_options = CleanOptions::for_vcs();
    let vcs_clean = notebook.clean(&vcs_options);

    println!("\nAfter CleanOptions::for_vcs():");
    print_notebook_info(&vcs_clean);

    // Strip all metadata and outputs
    let strip_all_options = CleanOptions::strip_all();
    let stripped = notebook.clean(&strip_all_options);

    println!("\nAfter CleanOptions::strip_all():");
    print_notebook_info(&stripped);

    // Custom cleaning options
    let custom_options = CleanOptions {
        remove_outputs: true,
        remove_execution_counts: true,
        preserve_cell_ids: true, // Keep cell IDs
        ..Default::default()
    };
    let custom_clean = notebook.clean(&custom_options);

    println!("\nAfter custom options (preserve cell IDs):");
    print_notebook_info(&custom_clean);

    // Demonstrate that original is unchanged (immutability)
    println!("\nOriginal notebook (unchanged):");
    print_notebook_info(&notebook);

    // Show serialized output
    println!("\n--- Cleaned ipynb (for_vcs) ---");
    let json = NotebookFormat::Ipynb.serialize(&vcs_clean).unwrap();
    println!("{}", &json[..json.len().min(800)]);
}

fn print_notebook_info(notebook: &Notebook) {
    println!("  Cells: {}", notebook.len());

    for (i, cell) in notebook.cells.iter().enumerate() {
        match cell {
            Cell::Code {
                execution_count,
                outputs,
                metadata,
                id,
                ..
            } => {
                println!(
                    "  Cell {}: code, exec_count={:?}, outputs={}, tags={:?}, id={:?}",
                    i,
                    execution_count,
                    outputs.len(),
                    metadata.tags,
                    id
                );
            }
            Cell::Markdown { id, .. } => {
                println!("  Cell {}: markdown, id={:?}", i, id);
            }
            Cell::Raw { id, .. } => {
                println!("  Cell {}: raw, id={:?}", i, id);
            }
        }
    }
}
