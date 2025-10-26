use super::super::*;
use super::*;
use crate::test_helpers::*;

// ========== Integration Tests ==========

#[test]
fn test_full_workflow_cycle() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    let state = get_state(&storage);

    assert!(state.workflow.is_some());
    assert_eq!(state.workflow_state.as_ref().unwrap().history.len(), 1);

    next(&storage);
    assert_eq!(
        get_state(&storage)
            .workflow_state
            .as_ref()
            .unwrap()
            .history
            .len(),
        2
    );
}

// ========== Integration Tests (End-to-End) ==========

#[test]
fn test_next_at_research_done_auto_transitions_to_discovery() {
    let (_tmp, storage) = setup_meta_mode_workflows();

    start_workflow("research", None, &storage).unwrap();
    set_meta_mode(&storage, "learning");
    set_node(&storage, "done");

    next(&storage);

    let ws = get_state(&storage).workflow_state.unwrap();
    assert_eq!(ws.mode, "discovery");
    assert_eq!(ws.current_node, "spec");
    assert_eq!(ws.meta_mode.unwrap().name, "learning");

    let event = first_transition(&storage);
    assert_eq!(event["from_node"], "done");
    assert_eq!(event["to_node"], "spec");
    assert_eq!(event["mode"], "discovery");
}

#[test]
fn test_next_at_discovery_done_shows_ambiguous_options() {
    let (_tmp, storage) = setup_meta_mode_workflows();

    start_workflow("discovery", None, &storage).unwrap();
    set_meta_mode(&storage, "learning");
    set_node(&storage, "done");

    next(&storage);

    let ws = get_state(&storage).workflow_state.unwrap();
    assert_eq!(ws.mode, "discovery");
    assert_eq!(ws.current_node, "done");
    assert_eq!(transition_count(&storage), 0);
}
