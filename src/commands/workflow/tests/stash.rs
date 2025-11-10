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

// ========== Pop Command Tests ==========

#[test]
fn test_pop_restores_workflow_and_deletes_stash() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next(&storage);
    stash_active_workflow(&storage, Some("test pop"));

    crate::commands::workflow::pop_stash(None, &storage).unwrap();

    assert_eq!(list_stashes_count(&storage), 0);
    let state = storage.load().unwrap();
    assert!(state.workflow.is_some());
}

#[test]
fn test_pop_with_index() {
    let (_tmp, storage) = setup_workflow_env();

    start(&storage);
    stash_active_workflow(&storage, Some("first"));

    start(&storage);
    next(&storage);
    stash_active_workflow(&storage, Some("second"));

    crate::commands::workflow::pop_stash(Some(1), &storage).unwrap();

    assert_eq!(list_stashes_count(&storage), 1);
    assert_at(&storage, "spec", "test_workflow", &["spec"]);
}

#[test]
fn test_pop_with_active_workflow_fails() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    stash_active_workflow(&storage, Some("test"));

    start(&storage);

    let result = crate::commands::workflow::pop_stash(None, &storage);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("active workflow"));
}

#[test]
fn test_pop_with_terminal_node_succeeds() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    stash_active_workflow(&storage, Some("stashed"));

    // Start new workflow and advance to terminal "done" node
    start(&storage);
    next(&storage); // spec -> plan
    next(&storage); // plan -> done

    // Verify we're at done node
    let state = storage.load().unwrap();
    assert_eq!(state.workflow.as_ref().unwrap().current_node, "done");

    // Should be able to pop stash even though workflow is Some
    let result = crate::commands::workflow::pop_stash(None, &storage);
    assert!(result.is_ok());

    // Verify stash was restored
    let state = storage.load().unwrap();
    assert_eq!(state.workflow.as_ref().unwrap().current_node, "spec");
    assert_eq!(list_stashes_count(&storage), 0);
}

#[test]
fn test_pop_invalid_index_fails() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    stash_active_workflow(&storage, Some("only one"));

    let result = crate::commands::workflow::pop_stash(Some(5), &storage);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_pop_empty_fails() {
    let (_tmp, storage) = setup_workflow_env();

    let result = crate::commands::workflow::pop_stash(None, &storage);
    assert!(result.is_err());
}

// ========== Drop Command Tests ==========

#[test]
fn test_drop_removes_stash() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    stash_active_workflow(&storage, Some("test drop"));

    crate::commands::workflow::drop_stash(None, &storage).unwrap();

    assert_eq!(list_stashes_count(&storage), 0);
}

#[test]
fn test_drop_with_index() {
    let (_tmp, storage) = setup_workflow_env();

    start(&storage);
    stash_active_workflow(&storage, Some("first"));

    start(&storage);
    stash_active_workflow(&storage, Some("second"));

    crate::commands::workflow::drop_stash(Some(1), &storage).unwrap();

    assert_eq!(list_stashes_count(&storage), 1);
    let stashes = storage.list_stashes().unwrap();
    assert_eq!(stashes[0].message.as_deref(), Some("second"));
}

#[test]
fn test_drop_invalid_index_fails() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    stash_active_workflow(&storage, Some("only one"));

    let result = crate::commands::workflow::drop_stash(Some(5), &storage);
    assert!(result.is_err());
}
