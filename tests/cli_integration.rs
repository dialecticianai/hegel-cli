//! Integration tests for Hegel CLI
//!
//! Tests the main.rs entry point by spawning the binary as a subprocess
//! and validating command behavior end-to-end.

use std::process::Command;
use tempfile::TempDir;

/// Helper to run hegel command with args
fn run_hegel(args: &[&str], state_dir: Option<&str>) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hegel"));

    if let Some(dir) = state_dir {
        cmd.arg("--state-dir").arg(dir);
    }

    cmd.args(args)
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

#[test]
fn test_no_command_shows_coming_soon() {
    let output = run_hegel(&[], None);

    let out = stdout(&output);
    assert!(out.contains("Hegel - Dialectic-Driven Development CLI"));
    assert!(out.contains("Thesis. Antithesis. Synthesis."));
    assert!(out.contains("Coming soon..."));
    assert!(out.contains("https://dialectician.ai"));
}

#[test]
fn test_start_workflow_success() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    let output = run_hegel(&["start", "discovery"], Some(state_path));

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(out.contains("Workflow started"));
    assert!(out.contains("Mode: discovery"));
    assert!(out.contains("Current node: spec"));
}

#[test]
fn test_start_workflow_missing_file() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    let output = run_hegel(&["start", "nonexistent"], Some(state_path));

    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("Failed to load workflow"));
}

#[test]
fn test_status_no_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    let output = run_hegel(&["status"], Some(state_path));

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(out.contains("No workflow loaded"));
    assert!(out.contains("hegel start <workflow>"));
}

#[test]
fn test_status_with_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    // Start workflow first
    let start_output = run_hegel(&["start", "discovery"], Some(state_path));
    assert!(start_output.status.success());

    // Then check status
    let output = run_hegel(&["status"], Some(state_path));

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(out.contains("Workflow Status"));
    assert!(out.contains("Mode: discovery"));
    assert!(out.contains("Current node: spec"));
}

#[test]
fn test_next_no_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    let output = run_hegel(&["next", r#"{"spec_complete": true}"#], Some(state_path));

    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("No workflow loaded"));
}

#[test]
fn test_next_successful_transition() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    // Start workflow
    run_hegel(&["start", "discovery"], Some(state_path));

    // Transition to next node
    let output = run_hegel(&["next", r#"{"spec_complete": true}"#], Some(state_path));

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(out.contains("Transitioned"));
    assert!(out.contains("spec"));
    assert!(out.contains("plan"));
}

#[test]
fn test_next_no_matching_transition() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    // Start workflow
    run_hegel(&["start", "discovery"], Some(state_path));

    // Try transition with wrong claim
    let output = run_hegel(&["next", r#"{"wrong_claim": true}"#], Some(state_path));

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(out.contains("Stayed at current node"));
}

#[test]
fn test_next_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    // Start workflow
    run_hegel(&["start", "discovery"], Some(state_path));

    // Try with invalid JSON
    let output = run_hegel(&["next", "not valid json"], Some(state_path));

    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("Failed to parse claims JSON"));
}

#[test]
fn test_repeat_no_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    let output = run_hegel(&["repeat"], Some(state_path));

    assert!(!output.status.success());
    let err = stderr(&output);
    assert!(err.contains("No workflow loaded"));
}

#[test]
fn test_repeat_with_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    // Start workflow
    run_hegel(&["start", "discovery"], Some(state_path));

    // Repeat
    let output = run_hegel(&["repeat"], Some(state_path));

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(out.contains("Re-displaying current prompt"));
    assert!(out.contains("Current node: spec"));
}

#[test]
fn test_reset_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    // Start workflow
    run_hegel(&["start", "discovery"], Some(state_path));

    // Reset
    let output = run_hegel(&["reset"], Some(state_path));

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(out.contains("Workflow state cleared"));

    // Verify it's cleared
    let status_output = run_hegel(&["status"], Some(state_path));
    let status_out = stdout(&status_output);
    assert!(status_out.contains("No workflow loaded"));
}

#[test]
fn test_analyze_empty_state() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    let output = run_hegel(&["analyze"], Some(state_path));

    assert!(output.status.success());
    let out = stdout(&output);
    assert!(out.contains("Hegel Metrics Analysis"));
    assert!(out.contains("Session"));
}

#[test]
fn test_hook_command_requires_stdin() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    // Hook command expects JSON on stdin, will fail without it
    let output = run_hegel(&["hook", "PostToolUse"], Some(state_path));

    // Should fail because no stdin provided
    assert!(!output.status.success());
}

#[test]
fn test_full_workflow_cycle() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().to_str().unwrap();

    // Start workflow
    let start = run_hegel(&["start", "discovery"], Some(state_path));
    assert!(start.status.success());
    assert!(stdout(&start).contains("Current node: spec"));

    // Verify status after start
    let status1 = run_hegel(&["status"], Some(state_path));
    assert!(stdout(&status1).contains("Current node: spec"));
    assert!(stdout(&status1).contains("Mode: discovery"));

    // Transition to plan
    let next1 = run_hegel(&["next", r#"{"spec_complete": true}"#], Some(state_path));
    assert!(next1.status.success());
    assert!(stdout(&next1).contains("plan"));

    // Verify status after transition
    let status2 = run_hegel(&["status"], Some(state_path));
    assert!(stdout(&status2).contains("Current node: plan"));

    // Reset workflow
    let reset = run_hegel(&["reset"], Some(state_path));
    assert!(reset.status.success());

    // Verify reset worked
    let status3 = run_hegel(&["status"], Some(state_path));
    assert!(stdout(&status3).contains("No workflow loaded"));
}
