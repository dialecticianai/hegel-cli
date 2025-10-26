use super::*;
use crate::test_helpers::*;

// Submodules
mod commands;
mod integration;
mod node_flow;
mod production;
mod transitions;

// ========== Shared Test Helpers ==========

/// Start workflow and return storage (ergonomic wrapper)
fn start(storage: &FileStorage) {
    start_workflow("test_workflow", None, storage).unwrap();
}

/// Advance workflow with next (None = implicit happy path)
fn next(storage: &FileStorage) {
    next_prompt(None, storage).unwrap();
}

/// Advance workflow with custom claims JSON
fn next_with(claims: &str, storage: &FileStorage) {
    next_prompt(Some(claims), storage).unwrap();
}

/// Load and assert current state
fn assert_at(storage: &FileStorage, node: &str, mode: &str, history: &[&str]) {
    assert_state_eq(&storage.load().unwrap(), node, mode, history);
}

/// Get current workflow state from storage
fn get_state(storage: &FileStorage) -> State {
    storage.load().unwrap()
}

/// Manually set current node (for testing completion scenarios)
fn set_node(storage: &FileStorage, node: &str) {
    let mut state = storage.load().unwrap();
    let mut ws = state.workflow_state.clone().unwrap();
    ws.current_node = node.to_string();
    if !ws.history.contains(&node.to_string()) {
        ws.history.push(node.to_string());
    }
    state.workflow_state = Some(ws);
    storage.save(&state).unwrap();
}

/// Set meta-mode on current state
fn set_meta_mode(storage: &FileStorage, meta_mode_name: &str) {
    let mut state = storage.load().unwrap();
    let mut ws = state.workflow_state.clone().unwrap();
    ws.meta_mode = Some(crate::storage::MetaMode {
        name: meta_mode_name.to_string(),
    });
    state.workflow_state = Some(ws);
    storage.save(&state).unwrap();
}

/// Count transitions logged in states.jsonl
fn transition_count(storage: &FileStorage) -> usize {
    count_jsonl_lines(&storage.state_dir().join("states.jsonl"))
}

/// Get first transition from states.jsonl
fn first_transition(storage: &FileStorage) -> serde_json::Value {
    read_jsonl_line(&storage.state_dir().join("states.jsonl"), 0)
}
