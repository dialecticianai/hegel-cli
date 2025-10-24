use super::super::*;
use super::*;
use crate::test_helpers::*;

// ========== State Transition Logging Tests ==========

#[test]
fn test_next_prompt_logs_state_transition() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next_with(r#"{"spec_complete": true}"#, &storage);
    let event = first_transition(&storage);
    assert_eq!(event["from_node"], "spec");
    assert_eq!(event["to_node"], "plan");
    assert_eq!(event["phase"], "plan");
    assert_eq!(event["mode"], "test_mode");
}

#[test]
fn test_next_prompt_logs_multiple_transitions() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next(&storage);
    next(&storage);
    assert_eq!(transition_count(&storage), 2);
}

#[test]
fn test_next_prompt_no_log_when_no_transition() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    next_with(r#"{"wrong_claim": true}"#, &storage);
    assert_eq!(transition_count(&storage), 0);
}

#[test]
fn test_state_transition_includes_workflow_id() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    let workflow_id = get_state(&storage)
        .workflow_state
        .as_ref()
        .unwrap()
        .workflow_id
        .as_ref()
        .unwrap()
        .clone();
    next_with(r#"{"spec_complete": true}"#, &storage);
    let event = first_transition(&storage);
    assert_eq!(event["workflow_id"], workflow_id.as_str());
}

// ========== evaluate_transition Tests ==========

#[test]
fn test_evaluate_intra_workflow_transition() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);

    let context = load_workflow_context(&storage).unwrap();
    let claims = claim("spec_complete", true);
    let outcome = evaluate_transition(&context, &claims, &storage).unwrap();

    match outcome {
        TransitionOutcome::IntraWorkflow {
            from_node, to_node, ..
        } => {
            assert_eq!(from_node, "spec");
            assert_eq!(to_node, "plan");
        }
        _ => panic!("Expected IntraWorkflow outcome"),
    }
}

#[test]
fn test_evaluate_stay_at_current_node() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);

    let context = load_workflow_context(&storage).unwrap();
    let claims = claim("wrong_claim", true);
    let outcome = evaluate_transition(&context, &claims, &storage).unwrap();

    match outcome {
        TransitionOutcome::Stay { current_node, .. } => {
            assert_eq!(current_node, "spec");
        }
        _ => panic!("Expected Stay outcome"),
    }
}

#[test]
fn test_evaluate_inter_workflow_transition_research_to_discovery() {
    let (_tmp, storage) = setup_meta_mode_workflows();

    start_workflow("research", &storage).unwrap();
    set_node(&storage, "done");
    set_meta_mode(&storage, "learning");

    let context = load_workflow_context(&storage).unwrap();
    let claims = claim("done_complete", true);
    let outcome = evaluate_transition(&context, &claims, &storage).unwrap();

    match outcome {
        TransitionOutcome::InterWorkflow {
            from_workflow,
            from_node,
            to_workflow,
            to_node,
            ..
        } => {
            assert_eq!(from_workflow, "research");
            assert_eq!(from_node, "done");
            assert_eq!(to_workflow, "discovery");
            assert_eq!(to_node, "spec");
        }
        _ => panic!("Expected InterWorkflow outcome, got: {:?}", outcome),
    }
}

#[test]
fn test_evaluate_ambiguous_discovery_done_in_learning_mode() {
    let (_tmp, storage) = setup_meta_mode_workflows();

    start_workflow("discovery", &storage).unwrap();
    set_node(&storage, "done");
    set_meta_mode(&storage, "learning");

    let context = load_workflow_context(&storage).unwrap();
    let claims = claim("done_complete", true);
    let outcome = evaluate_transition(&context, &claims, &storage).unwrap();

    match outcome {
        TransitionOutcome::Ambiguous { options } => {
            assert_eq!(options.len(), 2);
            assert_eq!(options[0].target_workflow, "research");
            assert_eq!(options[1].target_workflow, "execution");
        }
        _ => panic!("Expected Ambiguous outcome, got: {:?}", outcome),
    }
}

#[test]
fn test_evaluate_stay_at_done_no_meta_mode() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);
    set_node(&storage, "done");

    let context = load_workflow_context(&storage).unwrap();
    let claims = claim("done_complete", true);
    let outcome = evaluate_transition(&context, &claims, &storage).unwrap();

    match outcome {
        TransitionOutcome::Stay { current_node, .. } => {
            assert_eq!(current_node, "done");
        }
        _ => panic!("Expected Stay outcome at done without meta-mode"),
    }
}

// ========== execute_transition Tests ==========

#[test]
fn test_execute_intra_workflow_transition() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);

    let mut context = load_workflow_context(&storage).unwrap();
    let outcome = TransitionOutcome::IntraWorkflow {
        from_node: "spec".to_string(),
        to_node: "plan".to_string(),
        prompt: "Plan prompt".to_string(),
    };

    execute_transition(outcome, &mut context, &storage).unwrap();

    assert_at(&storage, "plan", "test_mode", &["spec", "plan"]);

    let event = first_transition(&storage);
    assert_eq!(event["from_node"], "spec");
    assert_eq!(event["to_node"], "plan");
}

#[test]
fn test_execute_stay_no_state_change() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);

    let mut context = load_workflow_context(&storage).unwrap();
    let outcome = TransitionOutcome::Stay {
        current_node: "spec".to_string(),
        prompt: "Spec prompt".to_string(),
    };

    execute_transition(outcome, &mut context, &storage).unwrap();

    assert_at(&storage, "spec", "test_mode", &["spec"]);
    assert_eq!(transition_count(&storage), 0);
}

#[test]
fn test_execute_inter_workflow_transition() {
    let (_tmp, storage) = setup_meta_mode_workflows();

    start_workflow("research", &storage).unwrap();
    set_meta_mode(&storage, "learning");

    let mut context = load_workflow_context(&storage).unwrap();
    let outcome = TransitionOutcome::InterWorkflow {
        from_workflow: "research".to_string(),
        from_node: "done".to_string(),
        to_workflow: "discovery".to_string(),
        to_node: "spec".to_string(),
        prompt: "Transition description".to_string(),
    };

    execute_transition(outcome, &mut context, &storage).unwrap();

    let state = get_state(&storage);
    assert_eq!(state.workflow_state.as_ref().unwrap().mode, "discovery");
    assert_eq!(state.workflow_state.as_ref().unwrap().current_node, "spec");
    assert_eq!(
        state
            .workflow_state
            .as_ref()
            .unwrap()
            .meta_mode
            .as_ref()
            .unwrap()
            .name,
        "learning"
    );

    let event = first_transition(&storage);
    assert_eq!(event["from_node"], "done");
    assert_eq!(event["to_node"], "spec");
    assert_eq!(event["mode"], "discovery");
}

#[test]
fn test_execute_ambiguous_no_state_change() {
    let (_tmp, storage) = setup_workflow_env();
    start(&storage);

    let mut context = load_workflow_context(&storage).unwrap();
    let outcome = TransitionOutcome::Ambiguous {
        options: vec![
            TransitionOption {
                description: "Option 1".to_string(),
                target_workflow: "workflow1".to_string(),
                target_node: "spec".to_string(),
            },
            TransitionOption {
                description: "Option 2".to_string(),
                target_workflow: "workflow2".to_string(),
                target_node: "spec".to_string(),
            },
        ],
    };

    execute_transition(outcome, &mut context, &storage).unwrap();

    assert_at(&storage, "spec", "test_mode", &["spec"]);
    assert_eq!(transition_count(&storage), 0);
}
