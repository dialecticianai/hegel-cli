use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Helper to run hegel command with args
fn run_hegel(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_hegel"))
        .args(args)
        .output()
        .expect("Failed to execute hegel command")
}

/// Helper to get stdout as string
fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Helper to create a test Hegel project with a file
fn setup_test_project() -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let hegel_dir = temp_dir.path().join(".hegel");
    fs::create_dir(&hegel_dir).unwrap();

    let test_file = temp_dir.path().join("test.md");
    fs::write(&test_file, "# Test File").unwrap();

    (temp_dir, test_file)
}

#[test]
fn test_review_command_exists() {
    let output = run_hegel(&["--help"]);
    let out = stdout(&output);

    assert!(
        out.contains("review"),
        "Review command should appear in help"
    );
}

#[test]
fn test_review_requires_file_argument() {
    let output = run_hegel(&["review"]);

    assert!(
        !output.status.success(),
        "Should fail without file argument"
    );
}

#[test]
fn test_review_command_basic_call() {
    let (_temp_dir, test_file) = setup_test_project();

    let output = run_hegel(&["review", test_file.to_str().unwrap()]);

    // Should succeed with our stub handler
    assert!(output.status.success(), "Review command should succeed");
}
