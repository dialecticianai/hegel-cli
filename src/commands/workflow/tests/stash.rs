use super::*;
use crate::test_helpers::*;

// ========== Test Helpers ==========

fn stash_active_workflow(storage: &FileStorage, message: Option<&str>) {
    let msg = message.map(|s| s.to_string());
    crate::commands::workflow::stash_workflow(msg, storage).unwrap();
}

fn list_stashes_count(storage: &FileStorage) -> usize {
    storage.list_stashes().unwrap().len()
}

fn assert_workflow_cleared(storage: &FileStorage) {
    let state = storage.load().unwrap();
    assert!(state.workflow.is_none());
    assert!(state.workflow_state.is_none());
}

// ========== Stash Command Tests ==========

#[test]
fn test_stash_creates_stash_and_clears_workflow() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);

    stash_active_workflow(&storage, Some("test message"));

    assert_eq!(list_stashes_count(&storage), 1);
    assert_workflow_cleared(&storage);
}

#[test]
fn test_stash_with_message() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);

    stash_active_workflow(&storage, Some("fixing bug"));

    let stashes = storage.list_stashes().unwrap();
    assert_eq!(stashes[0].message.as_deref(), Some("fixing bug"));
}

#[test]
fn test_stash_without_message() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);

    stash_active_workflow(&storage, None);

    let stashes = storage.list_stashes().unwrap();
    assert!(stashes[0].message.is_none());
}

#[test]
fn test_stash_no_active_workflow_fails() {
    let (_tmp, storage) = setup_workflow_env();

    let result = crate::commands::workflow::stash_workflow(None, &storage);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No active workflow"));
}

#[test]
fn test_stash_multiple_workflows() {
    let (_tmp, storage) = setup_workflow_env();

    start(&storage);
    stash_active_workflow(&storage, Some("first"));

    start(&storage);
    next(&storage);
    stash_active_workflow(&storage, Some("second"));

    assert_eq!(list_stashes_count(&storage), 2);
}

// ========== List Command Tests ==========

#[test]
fn test_list_stashes_empty() {
    let (_tmp, storage) = setup_workflow_env();

    let result = crate::commands::workflow::list_stashes(&storage);
    assert!(result.is_ok());
}

#[test]
fn test_list_stashes_shows_newest_first() {
    let (_tmp, storage) = setup_workflow_env();

    start(&storage);
    stash_active_workflow(&storage, Some("first"));
    std::thread::sleep(std::time::Duration::from_millis(10));

    start(&storage);
    stash_active_workflow(&storage, Some("second"));

    let stashes = storage.list_stashes().unwrap();
    assert_eq!(stashes[0].message.as_deref(), Some("second"));
    assert_eq!(stashes[1].message.as_deref(), Some("first"));
}
