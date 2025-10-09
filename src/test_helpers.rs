//! Test utilities and fixtures for Hegel tests
//!
//! This module provides common test helpers to reduce boilerplate and improve
//! test readability across the codebase.

use crate::engine::{Node, Transition, Workflow};
use crate::storage::{FileStorage, WorkflowState};
use std::collections::HashMap;
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

// ========== Workflow Builder Helpers ==========

/// Create a Transition for workflow tests
///
/// # Example
/// ```ignore
/// let t = transition("spec_complete", "plan");
/// ```
pub fn transition(when: &str, to: &str) -> Transition {
    Transition {
        when: when.to_string(),
        to: to.to_string(),
    }
}

/// Create a Node for workflow tests
///
/// # Example
/// ```ignore
/// let node = node("Write SPEC.md", vec![transition("spec_complete", "plan")]);
/// ```
pub fn node(prompt: &str, transitions: Vec<Transition>) -> Node {
    Node {
        prompt: prompt.to_string(),
        transitions,
    }
}

/// Create a Workflow for testing with a fluent builder pattern
///
/// # Example
/// ```ignore
/// let wf = workflow("discovery", "spec")
///     .with_node("spec", node("Write SPEC", vec![transition("done", "plan")]))
///     .with_node("plan", node("Write PLAN", vec![]))
///     .build();
/// ```
pub struct WorkflowBuilder {
    mode: String,
    start_node: String,
    nodes: HashMap<String, Node>,
}

impl WorkflowBuilder {
    pub fn new(mode: &str, start_node: &str) -> Self {
        Self {
            mode: mode.to_string(),
            start_node: start_node.to_string(),
            nodes: HashMap::new(),
        }
    }

    pub fn with_node(mut self, name: &str, node: Node) -> Self {
        self.nodes.insert(name.to_string(), node);
        self
    }

    pub fn build(self) -> Workflow {
        Workflow {
            mode: self.mode,
            start_node: self.start_node,
            nodes: self.nodes,
        }
    }
}

/// Create a workflow builder for testing
///
/// # Example
/// ```ignore
/// let wf = workflow("discovery", "spec")
///     .with_node("spec", node("Write SPEC", vec![]))
///     .build();
/// ```
pub fn workflow(mode: &str, start_node: &str) -> WorkflowBuilder {
    WorkflowBuilder::new(mode, start_node)
}

// ========== Template Test Helpers ==========

/// Create a context HashMap builder for template tests
///
/// # Example
/// ```ignore
/// let context = ctx().add("name", "World").add("task", "test").build();
/// ```
pub struct CtxBuilder {
    map: HashMap<String, String>,
}

impl CtxBuilder {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add(mut self, key: &str, value: &str) -> Self {
        self.map.insert(key.to_string(), value.to_string());
        self
    }

    pub fn build(self) -> HashMap<String, String> {
        self.map
    }
}

/// Create a context builder
pub fn ctx() -> CtxBuilder {
    CtxBuilder::new()
}

/// Create a test guides directory with common guide files
///
/// Returns (TempDir, guides_path)
pub fn test_guides() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let guides_path = temp_dir.path().join("guides");
    std::fs::create_dir(&guides_path).unwrap();

    std::fs::write(
        guides_path.join("SPEC_WRITING.md"),
        "# SPEC Writing Guide\n\nWrite a specification document.",
    )
    .unwrap();

    std::fs::write(
        guides_path.join("PLAN_WRITING.md"),
        "# PLAN Writing Guide\n\nWrite an implementation plan.",
    )
    .unwrap();

    (temp_dir, guides_path)
}

// ========== Storage Test Helpers ==========

use crate::storage::State;

/// Create a State for testing with optional workflow_state
///
/// # Example
/// ```ignore
/// let state = test_state("spec", "discovery", &["spec"]);
/// ```
pub fn test_state(node: &str, mode: &str, history: &[&str]) -> State {
    State {
        workflow: None,
        workflow_state: Some(test_workflow_state(node, mode, history)),
    }
}

/// Assert that a loaded workflow state matches expected values
pub fn assert_state_eq(state: &State, node: &str, mode: &str, history: &[&str]) {
    let ws = state.workflow_state.as_ref().unwrap();
    assert_eq!(ws.current_node, node);
    assert_eq!(ws.mode, mode);
    assert_eq!(ws.history, history);
}

// ========== Workflow Command Test Helpers ==========

/// Helper to save and restore working directory
pub struct WorkingDirGuard {
    original_dir: Option<PathBuf>,
}

impl WorkingDirGuard {
    pub fn new() -> Self {
        Self {
            // Handle case where current dir doesn't exist (parallel test race condition)
            original_dir: std::env::current_dir().ok(),
        }
    }
}

impl Drop for WorkingDirGuard {
    fn drop(&mut self) {
        // Only restore if we successfully captured the original dir
        if let Some(dir) = &self.original_dir {
            let _ = std::env::set_current_dir(dir);
        }
    }
}

/// Setup test environment for workflow commands
///
/// Creates temp directory with workflows/, guides/, and test storage
/// Returns (TempDir, FileStorage, WorkingDirGuard) and sets cwd to temp_dir
pub fn setup_workflow_env() -> (TempDir, FileStorage, WorkingDirGuard) {
    let guard = WorkingDirGuard::new();
    let temp_dir = TempDir::new().unwrap();

    // Create workflows and guides directories
    std::fs::create_dir(temp_dir.path().join("workflows")).unwrap();
    std::fs::create_dir(temp_dir.path().join("guides")).unwrap();

    // Write test workflow
    std::fs::write(
        temp_dir.path().join("workflows/discovery.yaml"),
        TEST_WORKFLOW_YAML,
    )
    .unwrap();

    // Create storage
    let storage = FileStorage::new(temp_dir.path().join("state")).unwrap();

    // Change to temp dir
    std::env::set_current_dir(temp_dir.path()).unwrap();

    (temp_dir, storage, guard)
}
