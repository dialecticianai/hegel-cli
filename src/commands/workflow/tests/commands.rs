use super::super::*;
use super::*;
use crate::commands::show_status;

// ========== start_workflow Tests ==========

#[test]
fn test_start_workflow_success() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    let state = get_state(&storage);
    assert_at(&storage, "spec", "test_workflow", &["spec"]);

    let wf_id = state.workflow.unwrap().workflow_id.unwrap();
    use chrono::DateTime;
    assert!(DateTime::parse_from_rfc3339(&wf_id).is_ok());
}

#[test]
fn test_start_workflow_missing_file() {
    let (_tmp, storage) = setup_workflow_env();
    let result = start_workflow("nonexistent", None, &storage);
    assert!(
        result.is_err()
            && result
                .unwrap_err()
                .to_string()
                .contains("Failed to load workflow")
    );
}

#[test]
fn test_start_workflow_with_custom_start_node() {
    let (_tmp, storage) = setup_workflow_env();
    start_workflow("test_workflow", Some("plan"), &storage).unwrap();
    assert_at(&storage, "plan", "test_workflow", &["plan"]);
}

#[test]
fn test_start_workflow_with_invalid_start_node() {
    let (_tmp, storage) = setup_workflow_env();
    let result = start_workflow("test_workflow", Some("nonexistent"), &storage);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Invalid starting node"));
    assert!(err_msg.contains("nonexistent"));
    assert!(err_msg.contains("Available nodes"));
}

// ========== next_prompt Tests ==========

#[test]
fn test_next_prompt_successful_transition() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next_with("spec_complete", &storage);
    assert_at(&storage, "plan", "test_workflow", &["spec", "plan"]);
}

#[test]
fn test_next_prompt_no_matching_transition() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next_with("wrong_claim", &storage);
    assert_at(&storage, "spec", "test_workflow", &["spec"]);
}

#[test]
fn test_next_prompt_no_workflow_loaded() {
    let (_tmp, storage) = setup_workflow_env();
    let result = next_prompt(Some(r#"{"spec_complete": true}"#), None, &storage);
    assert!(
        result.is_err()
            && result
                .unwrap_err()
                .to_string()
                .contains("No workflow loaded")
    );
}

#[test]
fn test_next_prompt_multiple_transitions() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);

    next(&storage);
    assert_eq!(
        get_state(&storage).workflow.as_ref().unwrap().history.len(),
        2
    );

    next(&storage);
    assert_eq!(
        get_state(&storage).workflow.as_ref().unwrap().history.len(),
        3
    );
}

// ========== prev_prompt Tests ==========

#[test]
fn test_prev_prompt_successful_transition() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next(&storage); // spec -> plan
    assert_at(&storage, "plan", "test_workflow", &["spec", "plan"]);

    prev_prompt(&storage).unwrap();
    assert_at(&storage, "spec", "test_workflow", &["spec"]);
}

#[test]
fn test_prev_prompt_at_start_fails() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    assert_at(&storage, "spec", "test_workflow", &["spec"]);

    let result = prev_prompt(&storage);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Cannot go back"));
    assert!(err_msg.contains("already at the start"));
}

#[test]
fn test_prev_prompt_no_workflow_loaded() {
    let (_tmp, storage) = setup_workflow_env();
    let result = prev_prompt(&storage);
    assert!(result.is_err() && result.unwrap_err().to_string().contains("No workflow"));
}

#[test]
fn test_prev_prompt_multiple_transitions() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next(&storage); // spec -> plan
    assert_at(&storage, "plan", "test_workflow", &["spec", "plan"]);

    prev_prompt(&storage).unwrap(); // plan -> spec
    assert_at(&storage, "spec", "test_workflow", &["spec"]);

    // Go forward again and back again to test it multiple times
    next(&storage); // spec -> plan
    assert_at(&storage, "plan", "test_workflow", &["spec", "plan"]);

    prev_prompt(&storage).unwrap(); // plan -> spec
    assert_at(&storage, "spec", "test_workflow", &["spec"]);
}

#[test]
fn test_prev_then_next() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next(&storage); // spec -> plan
    assert_at(&storage, "plan", "test_workflow", &["spec", "plan"]);

    prev_prompt(&storage).unwrap(); // plan -> spec
    assert_at(&storage, "spec", "test_workflow", &["spec"]);

    next(&storage); // spec -> plan (should work again)
    assert_at(&storage, "plan", "test_workflow", &["spec", "plan"]);
}

#[test]
fn test_prev_prompt_logs_transition() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next(&storage); // spec -> plan

    let count_before = transition_count(&storage);
    prev_prompt(&storage).unwrap(); // plan -> spec
    let count_after = transition_count(&storage);

    assert_eq!(count_after, count_before + 1);
}

// ========== show_status Tests ==========

#[test]
fn test_show_status_with_workflow() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    assert!(show_status(&storage).is_ok());
}

#[test]
fn test_show_status_no_workflow() {
    let (_tmp, storage) = test_storage();
    assert!(show_status(&storage).is_ok());
}

#[test]
fn test_show_status_after_transitions() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next_with("spec_complete", &storage);
    assert!(show_status(&storage).is_ok());
    assert_at(&storage, "plan", "test_workflow", &["spec", "plan"]);
}

// ========== reset_workflow Tests ==========

#[test]
fn test_reset_workflow_clears_state() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    assert!(get_state(&storage).workflow.is_some());
    reset_workflow(&storage).unwrap();
    let state = get_state(&storage);
    assert!(state.workflow.is_none());
}

#[test]
fn test_reset_workflow_when_no_state() {
    let (_tmp, storage) = setup_workflow_env();
    assert!(reset_workflow(&storage).is_ok());
}

#[test]
fn test_reset_workflow_preserves_session_metadata() {
    use crate::storage::SessionMetadata;

    let (_tmp, storage) = setup_workflow_env();
    start(&storage);

    let mut state = get_state(&storage);
    state.session_metadata = Some(SessionMetadata {
        session_id: "test-session".to_string(),
        transcript_path: "/tmp/transcript.jsonl".to_string(),
        started_at: "2025-01-01T10:00:00Z".to_string(),
    });
    storage.save(&state).unwrap();

    reset_workflow(&storage).unwrap();

    let state = get_state(&storage);
    assert!(state.session_metadata.is_some());
    assert_eq!(state.session_metadata.unwrap().session_id, "test-session");
}

#[test]
fn test_reset_then_start_new_workflow() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next_with("spec_complete", &storage);
    reset_workflow(&storage).unwrap();
    start(&storage);
    assert_at(&storage, "spec", "test_workflow", &["spec"]);
}

// ========== Integration Tests ==========

#[test]
fn test_full_workflow_cycle() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    let state = get_state(&storage);

    assert_eq!(state.workflow.as_ref().unwrap().history.len(), 1);

    next(&storage);
    assert_eq!(
        get_state(&storage).workflow.as_ref().unwrap().history.len(),
        2
    );
}

// ========== State Transition Logging Tests ==========

#[test]
fn test_next_prompt_logs_state_transition() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next_with("spec_complete", &storage);
    let event = first_transition(&storage);
    assert_eq!(event["from_node"], "spec");
    assert_eq!(event["to_node"], "plan");
    assert_eq!(event["phase"], "plan");
    assert_eq!(event["mode"], "test_workflow");
}

#[test]
fn test_next_prompt_logs_multiple_transitions() {
    use crate::storage::archive::read_archives;

    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next(&storage); // spec -> plan (transition 1)

    // Before archiving, check 1 transition
    assert_eq!(transition_count(&storage), 1);

    next(&storage); // plan -> done (transition 2, triggers archiving)

    // After archiving, transitions should be in archive, not live log
    let archives = read_archives(storage.state_dir()).unwrap();
    assert_eq!(archives.len(), 1);
    assert_eq!(archives[0].transitions.len(), 2); // Both transitions archived
}

#[test]
fn test_next_prompt_no_log_when_no_transition() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next_with("wrong_claim", &storage);
    assert_eq!(transition_count(&storage), 0);
}

#[test]
fn test_state_transition_includes_workflow_id() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    let workflow_id = get_state(&storage)
        .workflow
        .as_ref()
        .unwrap()
        .workflow_id
        .as_ref()
        .unwrap()
        .clone();
    next_with("spec_complete", &storage);
    let event = first_transition(&storage);
    assert_eq!(event["workflow_id"], workflow_id.as_str());
}

// ========== repeat_prompt Tests ==========

#[test]
fn test_continue_with_active_workflow_returns_current_node_prompt() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    assert!(repeat_prompt(&storage).is_ok());
}

#[test]
fn test_continue_with_no_workflow_loaded_returns_error() {
    let (_tmp, storage) = setup_workflow_env();
    let result = repeat_prompt(&storage);
    assert!(result.is_err() && result.unwrap_err().to_string().contains("No workflow"));
}

#[test]
fn test_continue_renders_template_with_guides() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    assert!(repeat_prompt(&storage).is_ok());
}

#[test]
fn test_continue_does_not_change_workflow_state() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    let state_before = get_state(&storage);
    repeat_prompt(&storage).unwrap();
    let state_after = get_state(&storage);

    assert_state_eq(&state_before, "spec", "test_workflow", &["spec"]);
    assert_state_eq(&state_after, "spec", "test_workflow", &["spec"]);
}

#[test]
fn test_continue_does_not_log_state_transition() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    repeat_prompt(&storage).unwrap();
    assert_eq!(transition_count(&storage), 0);
}

// ========== Implicit Next Tests ==========

#[test]
fn test_next_prompt_implicit_happy_path() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next(&storage);
    assert_at(&storage, "plan", "test_workflow", &["spec", "plan"]);
}

#[test]
fn test_next_prompt_implicit_multiple_transitions() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);

    next(&storage);
    assert_eq!(
        get_state(&storage).workflow.as_ref().unwrap().history.len(),
        2
    );

    next(&storage);
    assert_eq!(
        get_state(&storage).workflow.as_ref().unwrap().history.len(),
        3
    );
}

#[test]
fn test_restart_workflow_returns_to_spec() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next(&storage);
    assert_at(&storage, "plan", "test_workflow", &["spec", "plan"]);

    restart_workflow(&storage).unwrap();
    assert_at(&storage, "spec", "test_workflow", &["spec", "plan", "spec"]);
}

#[test]
fn test_restart_workflow_no_workflow_loaded() {
    let (_tmp, storage) = setup_workflow_env();
    let result = restart_workflow(&storage);
    assert!(
        result.is_err()
            && result
                .unwrap_err()
                .to_string()
                .contains("No workflow loaded")
    );
}
