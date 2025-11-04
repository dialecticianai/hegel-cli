//! Storage and state test helpers

use crate::storage::{FileStorage, State, WorkflowState};
use tempfile::TempDir;

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
        meta_mode: None,
        phase_start_time: Some(chrono::Utc::now().to_rfc3339()),
    }
}

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
        session_metadata: None,
        cumulative_totals: None,
        git_info: None,
    }
}

/// Assert that a loaded workflow state matches expected values
pub fn assert_state_eq(state: &State, node: &str, mode: &str, history: &[&str]) {
    let ws = state.workflow_state.as_ref().unwrap();
    assert_eq!(ws.current_node, node);
    assert_eq!(ws.mode, mode);
    assert_eq!(ws.history, history);
}
