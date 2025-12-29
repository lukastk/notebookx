//! CLI integration tests for nbx

use std::process::Command;

const NBX_BIN: &str = env!("CARGO_BIN_EXE_nbx");
const EXAMPLE_IPYNB: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../nb_format_examples/World population.ipynb"
);
const EXAMPLE_PERCENT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../nb_format_examples/World population.pct.py"
);

#[test]
fn test_help() {
    let output = Command::new(NBX_BIN)
        .arg("--help")
        .output()
        .expect("Failed to execute nbx");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("convert"));
    assert!(stdout.contains("clean"));
}

#[test]
fn test_convert_help() {
    let output = Command::new(NBX_BIN)
        .args(["convert", "--help"])
        .output()
        .expect("Failed to execute nbx");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--from-fmt"));
    assert!(stdout.contains("--to-fmt"));
    assert!(stdout.contains("--strip-outputs"));
}

#[test]
fn test_clean_help() {
    let output = Command::new(NBX_BIN)
        .args(["clean", "--help"])
        .output()
        .expect("Failed to execute nbx");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--remove-outputs"));
    assert!(stdout.contains("--remove-execution-counts"));
    assert!(stdout.contains("--in-place"));
}

#[test]
fn test_convert_ipynb_to_percent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("output.pct.py");

    let output = Command::new(NBX_BIN)
        .args([
            "convert",
            EXAMPLE_IPYNB,
            "--output",
            output_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute nbx");

    assert!(
        output.status.success(),
        "stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# %%"));
    assert!(content.contains("# %% [markdown]"));
}

#[test]
fn test_convert_percent_to_ipynb() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("output.ipynb");

    let output = Command::new(NBX_BIN)
        .args([
            "convert",
            EXAMPLE_PERCENT,
            "--output",
            output_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute nbx");

    assert!(
        output.status.success(),
        "stdout: {}, stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("\"cells\""));
    assert!(content.contains("\"cell_type\""));
}

#[test]
fn test_convert_with_strip_outputs() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("output.ipynb");

    let output = Command::new(NBX_BIN)
        .args([
            "convert",
            EXAMPLE_IPYNB,
            "--output",
            output_path.to_str().unwrap(),
            "--strip-outputs",
        ])
        .output()
        .expect("Failed to execute nbx");

    assert!(output.status.success());

    let content = std::fs::read_to_string(&output_path).unwrap();
    // Parse and verify outputs are empty
    let notebook: serde_json::Value = serde_json::from_str(&content).unwrap();
    let cells = notebook["cells"].as_array().unwrap();
    for cell in cells {
        if cell["cell_type"] == "code" {
            let outputs = cell["outputs"].as_array().unwrap();
            assert!(outputs.is_empty(), "Outputs should be stripped");
        }
    }
}

#[test]
fn test_convert_explicit_format() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("output.txt");

    let output = Command::new(NBX_BIN)
        .args([
            "convert",
            EXAMPLE_IPYNB,
            "--output",
            output_path.to_str().unwrap(),
            "--to-fmt",
            "percent",
        ])
        .output()
        .expect("Failed to execute nbx");

    assert!(output.status.success());

    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# %%"));
}

#[test]
fn test_clean_remove_outputs() {
    let output = Command::new(NBX_BIN)
        .args(["clean", EXAMPLE_IPYNB, "--remove-outputs"])
        .output()
        .expect("Failed to execute nbx");

    assert!(output.status.success());

    let content = String::from_utf8_lossy(&output.stdout);
    let notebook: serde_json::Value = serde_json::from_str(&content).unwrap();
    let cells = notebook["cells"].as_array().unwrap();
    for cell in cells {
        if cell["cell_type"] == "code" {
            let outputs = cell["outputs"].as_array().unwrap();
            assert!(outputs.is_empty(), "Outputs should be removed");
        }
    }
}

#[test]
fn test_clean_remove_execution_counts() {
    let output = Command::new(NBX_BIN)
        .args(["clean", EXAMPLE_IPYNB, "--remove-execution-counts"])
        .output()
        .expect("Failed to execute nbx");

    assert!(output.status.success());

    let content = String::from_utf8_lossy(&output.stdout);
    let notebook: serde_json::Value = serde_json::from_str(&content).unwrap();
    let cells = notebook["cells"].as_array().unwrap();
    for cell in cells {
        if cell["cell_type"] == "code" {
            assert!(
                cell["execution_count"].is_null(),
                "Execution count should be null"
            );
        }
    }
}

#[test]
fn test_clean_to_output_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("cleaned.ipynb");

    let output = Command::new(NBX_BIN)
        .args([
            "clean",
            EXAMPLE_IPYNB,
            "--output",
            output_path.to_str().unwrap(),
            "--remove-outputs",
        ])
        .output()
        .expect("Failed to execute nbx");

    assert!(output.status.success());
    assert!(output_path.exists());

    let content = std::fs::read_to_string(&output_path).unwrap();
    let notebook: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(notebook["cells"].is_array());
}

#[test]
fn test_error_missing_file() {
    let output = Command::new(NBX_BIN)
        .args([
            "convert",
            "nonexistent.ipynb",
            "--output",
            "/tmp/test.pct.py",
        ])
        .output()
        .expect("Failed to execute nbx");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(3)); // IO_ERROR

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("File not found") || stderr.contains("not found"));
}

#[test]
fn test_error_format_inference_stdin() {
    let output = Command::new(NBX_BIN)
        .args(["convert", "-", "--output", "/tmp/test.ipynb"])
        .output()
        .expect("Failed to execute nbx");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(4)); // INVALID_ARGS

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cannot infer format"));
}

#[test]
fn test_error_unknown_extension() {
    let output = Command::new(NBX_BIN)
        .args(["convert", "test.unknown", "--output", "/tmp/test.ipynb"])
        .output()
        .expect("Failed to execute nbx");

    assert!(!output.status.success());
    // Either format inference error or file not found
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cannot infer format") || stderr.contains("not found"));
}

#[test]
fn test_stdin_stdout_conversion() {
    let input = std::fs::read_to_string(EXAMPLE_IPYNB).unwrap();

    let mut child = Command::new(NBX_BIN)
        .args([
            "convert",
            "-",
            "--output",
            "-",
            "--from-fmt",
            "ipynb",
            "--to-fmt",
            "percent",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn nbx");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# %%"));
    assert!(stdout.contains("# %% [markdown]"));
}
