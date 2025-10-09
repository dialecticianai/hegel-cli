//! Test utilities and fixtures for Hegel tests
//!
//! This module provides common test helpers to reduce boilerplate and improve
//! test readability across the codebase.

use crate::storage::{FileStorage, WorkflowState};
use std::path::PathBuf;
use tempfile::TempDir;

/// Test workflow YAML - simple 3-node workflow for testing
pub const TEST_WORKFLOW_YAML: &str = r#"
mode: discovery
start_node: spec
nodes:
  spec:
    prompt: "Write SPEC.md"
    transitions:
      - when: spec_complete
        to: plan
  plan:
    prompt: "Write PLAN.md"
    transitions:
      - when: plan_complete
        to: done
  done:
    prompt: "Complete!"
    transitions: []
"#;

/// Create a temporary directory and FileStorage instance for testing
///
/// # Returns
/// A tuple of (TempDir, FileStorage) where the storage uses the temp directory's path
///
/// # Example
/// ```ignore
/// let (temp_dir, storage) = test_storage();
/// // Use storage for testing...
/// // temp_dir is automatically cleaned up when dropped
/// ```
pub fn test_storage() -> (TempDir, FileStorage) {
    let temp_dir = TempDir::new().unwrap();
    let storage = FileStorage::new(temp_dir.path()).unwrap();
    (temp_dir, storage)
}

/// Create a WorkflowState for testing with common defaults
///
/// # Arguments
/// * `node` - Current node name
/// * `mode` - Workflow mode (e.g., "discovery", "execution")
/// * `history` - History of nodes visited (as strings)
///
/// # Example
/// ```ignore
/// let state = test_workflow_state("spec", "discovery", &["spec"]);
/// ```
pub fn test_workflow_state(node: &str, mode: &str, history: &[&str]) -> WorkflowState {
    WorkflowState {
        current_node: node.to_string(),
        mode: mode.to_string(),
        history: history.iter().map(|s| s.to_string()).collect(),
        workflow_id: None,
    }
}

/// Read a specific line from a JSONL file and parse as JSON
///
/// # Arguments
/// * `path` - Path to the JSONL file
/// * `line_num` - Zero-indexed line number to read
///
/// # Returns
/// Parsed JSON value from the specified line
///
/// # Panics
/// Panics if file doesn't exist, line doesn't exist, or JSON is invalid
///
/// # Example
/// ```ignore
/// let event = read_jsonl_line(&hooks_file, 0);
/// assert_eq!(event["session_id"], "test");
/// ```
pub fn read_jsonl_line(path: &PathBuf, line_num: usize) -> serde_json::Value {
    let content = std::fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    serde_json::from_str(lines[line_num]).unwrap()
}

/// Read all lines from a JSONL file and parse as JSON array
///
/// # Arguments
/// * `path` - Path to the JSONL file
///
/// # Returns
/// Vector of parsed JSON values, one per line
///
/// # Example
/// ```ignore
/// let events = read_jsonl_all(&hooks_file);
/// assert_eq!(events.len(), 3);
/// ```
pub fn read_jsonl_all(path: &PathBuf) -> Vec<serde_json::Value> {
    let content = std::fs::read_to_string(path).unwrap();
    content
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

/// Count the number of lines in a JSONL file
///
/// # Arguments
/// * `path` - Path to the JSONL file
///
/// # Returns
/// Number of lines in the file, or 0 if file doesn't exist
///
/// # Example
/// ```ignore
/// let count = count_jsonl_lines(&states_file);
/// assert_eq!(count, 2);
/// ```
pub fn count_jsonl_lines(path: &PathBuf) -> usize {
    if !path.exists() {
        return 0;
    }
    let content = std::fs::read_to_string(path).unwrap();
    content.lines().count()
}
