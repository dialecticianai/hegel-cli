use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
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

/// Helper to get stderr as string
fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
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
    let (temp_dir, test_file) = setup_test_project();
    let hegel_dir = temp_dir.path().join(".hegel");

    let output = Command::new(env!("CARGO_BIN_EXE_hegel"))
        .args(&[
            "--state-dir",
            hegel_dir.to_str().unwrap(),
            "review",
            test_file.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    // Should succeed (read mode with no reviews)
    assert!(output.status.success(), "Review command should succeed");
}

// Write mode tests

#[test]
fn test_write_mode_saves_review() {
    let (temp_dir, test_file) = setup_test_project();
    let hegel_dir = temp_dir.path().join(".hegel");

    let jsonl = r###"{"timestamp":"2025-01-10T10:00:00Z","file":"test.md","selection":{"start":{"line":1,"col":0},"end":{"line":1,"col":10}},"text":"# Test File","comment":"Test comment"}"###;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hegel"))
        .args(&[
            "--state-dir",
            hegel_dir.to_str().unwrap(),
            "review",
            test_file.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    cmd.stdin
        .as_mut()
        .unwrap()
        .write_all(jsonl.as_bytes())
        .unwrap();

    let output = cmd.wait_with_output().unwrap();
    assert!(output.status.success(), "Write mode should succeed");

    let out = stdout(&output);
    assert!(out.contains("\"file\""), "Should output JSON result");
    assert!(out.contains("\"comments\""), "Should include comment count");

    // Verify reviews.json was created
    let reviews_file = temp_dir.path().join(".hegel").join("reviews.json");
    assert!(reviews_file.exists(), "reviews.json should exist");

    let content = fs::read_to_string(&reviews_file).unwrap();
    assert!(content.contains("Test comment"), "Should contain comment");
}

#[test]
fn test_write_mode_appends_to_existing() {
    let (temp_dir, test_file) = setup_test_project();
    let hegel_dir = temp_dir.path().join(".hegel");

    // First write
    let jsonl1 = r###"{"timestamp":"2025-01-10T10:00:00Z","file":"test.md","selection":{"start":{"line":1,"col":0},"end":{"line":1,"col":10}},"text":"# Test","comment":"First"}"###;
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hegel"))
        .args(&[
            "--state-dir",
            hegel_dir.to_str().unwrap(),
            "review",
            test_file.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    cmd.stdin
        .as_mut()
        .unwrap()
        .write_all(jsonl1.as_bytes())
        .unwrap();
    cmd.wait_with_output().unwrap();

    // Second write
    let jsonl2 = r###"{"timestamp":"2025-01-10T10:01:00Z","file":"test.md","selection":{"start":{"line":2,"col":0},"end":{"line":2,"col":10}},"text":"# Test2","comment":"Second"}"###;
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hegel"))
        .args(&[
            "--state-dir",
            hegel_dir.to_str().unwrap(),
            "review",
            test_file.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    cmd.stdin
        .as_mut()
        .unwrap()
        .write_all(jsonl2.as_bytes())
        .unwrap();
    cmd.wait_with_output().unwrap();

    // Verify both reviews exist
    let reviews_file = temp_dir.path().join(".hegel").join("reviews.json");
    let content = fs::read_to_string(&reviews_file).unwrap();
    assert!(content.contains("First"), "Should contain first comment");
    assert!(content.contains("Second"), "Should contain second comment");
}

#[test]
fn test_write_mode_multiple_comments() {
    let (temp_dir, test_file) = setup_test_project();
    let hegel_dir = temp_dir.path().join(".hegel");

    let jsonl = r###"{"timestamp":"2025-01-10T10:00:00Z","file":"test.md","selection":{"start":{"line":1,"col":0},"end":{"line":1,"col":10}},"text":"# Test","comment":"Comment 1"}
{"timestamp":"2025-01-10T10:01:00Z","file":"test.md","selection":{"start":{"line":2,"col":0},"end":{"line":2,"col":10}},"text":"# Test2","comment":"Comment 2"}"###;

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hegel"))
        .args(&[
            "--state-dir",
            hegel_dir.to_str().unwrap(),
            "review",
            test_file.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    cmd.stdin
        .as_mut()
        .unwrap()
        .write_all(jsonl.as_bytes())
        .unwrap();

    let output = cmd.wait_with_output().unwrap();
    assert!(output.status.success());

    let out = stdout(&output);
    assert!(out.contains("\"comments\":2") || out.contains("\"comments\": 2"));

    // Verify both comments in same entry
    let reviews_file = temp_dir.path().join(".hegel").join("reviews.json");
    let content = fs::read_to_string(&reviews_file).unwrap();
    assert!(content.contains("Comment 1"));
    assert!(content.contains("Comment 2"));
}

// Read mode tests

#[test]
fn test_read_mode_displays_reviews() {
    let (temp_dir, test_file) = setup_test_project();
    let hegel_dir = temp_dir.path().join(".hegel");

    // First write some reviews
    let jsonl = r###"{"timestamp":"2025-01-10T10:00:00Z","file":"test.md","selection":{"start":{"line":1,"col":0},"end":{"line":1,"col":10}},"text":"# Test","comment":"Test comment"}"###;
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hegel"))
        .args(&[
            "--state-dir",
            hegel_dir.to_str().unwrap(),
            "review",
            test_file.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    cmd.stdin
        .as_mut()
        .unwrap()
        .write_all(jsonl.as_bytes())
        .unwrap();
    cmd.wait_with_output().unwrap();

    // Now read reviews
    let output = Command::new(env!("CARGO_BIN_EXE_hegel"))
        .args(&[
            "--state-dir",
            hegel_dir.to_str().unwrap(),
            "review",
            test_file.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(output.status.success(), "Read mode should succeed");
    let out = stdout(&output);
    assert!(!out.is_empty(), "Should output reviews");
    assert!(out.contains("Test comment"), "Should contain comment text");
}

#[test]
fn test_read_mode_empty_output() {
    let (temp_dir, test_file) = setup_test_project();
    let hegel_dir = temp_dir.path().join(".hegel");

    let output = Command::new(env!("CARGO_BIN_EXE_hegel"))
        .args(&[
            "--state-dir",
            hegel_dir.to_str().unwrap(),
            "review",
            test_file.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        eprintln!("stderr: {}", stderr(&output));
    }
    assert!(output.status.success(), "Should succeed with no reviews");
    let out = stdout(&output);
    assert!(
        out.is_empty() || out.trim().is_empty(),
        "Should have empty output"
    );
}
