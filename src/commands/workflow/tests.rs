use super::*;
use crate::test_helpers::*;

    // ========== Test Helpers ==========

    /// Start workflow and return storage (ergonomic wrapper)
    fn start(storage: &FileStorage) {
        start_workflow("test_workflow", storage).unwrap();
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

    // ========== start_workflow Tests ==========

    #[test]
    fn test_start_workflow_success() {
        let (_tmp, storage) = setup_workflow_env();
        start(&storage);
        let state = get_state(&storage);
        assert!(state.workflow.is_some());
        assert_at(&storage, "spec", "test_mode", &["spec"]);

        let wf_id = state.workflow_state.unwrap().workflow_id.unwrap();
        use chrono::DateTime;
        assert!(DateTime::parse_from_rfc3339(&wf_id).is_ok());
    }

    #[test]
    fn test_start_workflow_missing_file() {
        let (_tmp, storage) = setup_workflow_env();
        let result = start_workflow("nonexistent", &storage);
        assert!(
            result.is_err()
                && result
                    .unwrap_err()
                    .to_string()
                    .contains("Failed to load workflow")
        );
    }

    // ========== next_prompt Tests ==========

    #[test]
    fn test_next_prompt_successful_transition() {
        let (_tmp, storage) = setup_workflow_env();
        start(&storage);
        next_with(r#"{"spec_complete": true}"#, &storage);
        assert_at(&storage, "plan", "test_mode", &["spec", "plan"]);
    }

    #[test]
    fn test_next_prompt_no_matching_transition() {
        let (_tmp, storage) = setup_workflow_env();
        start(&storage);
        next_with(r#"{"wrong_claim": true}"#, &storage);
        assert_at(&storage, "spec", "test_mode", &["spec"]);
    }

    #[test]
    fn test_next_prompt_no_workflow_loaded() {
        let (_tmp, storage) = setup_workflow_env();
        let result = next_prompt(Some(r#"{"spec_complete": true}"#), &storage);
        assert!(
            result.is_err()
                && result
                    .unwrap_err()
                    .to_string()
                    .contains("No workflow loaded")
        );
    }

    #[test]
    fn test_next_prompt_invalid_json() {
        let (_tmp, storage) = setup_workflow_env();
        start(&storage);
        let result = next_prompt(Some("not valid json"), &storage);
        assert!(
            result.is_err()
                && result
                    .unwrap_err()
                    .to_string()
                    .contains("Failed to parse claims JSON")
        );
    }

    #[test]
    fn test_next_prompt_multiple_transitions() {
        let (_tmp, storage) = setup_workflow_env();
        start(&storage);

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

        next(&storage);
        assert_eq!(
            get_state(&storage)
                .workflow_state
                .as_ref()
                .unwrap()
                .history
                .len(),
            3
        );
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
        next_with(r#"{"spec_complete": true}"#, &storage);
        assert!(show_status(&storage).is_ok());
        assert_at(&storage, "plan", "test_mode", &["spec", "plan"]);
    }

    // ========== reset_workflow Tests ==========

    #[test]
    fn test_reset_workflow_clears_state() {
        let (_tmp, storage) = setup_workflow_env();
        start(&storage);
        assert!(get_state(&storage).workflow.is_some());
        reset_workflow(&storage).unwrap();
        let state = get_state(&storage);
        assert!(state.workflow.is_none() && state.workflow_state.is_none());
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
        assert!(state.workflow.is_none());
        assert!(state.workflow_state.is_none());
        assert!(state.session_metadata.is_some());
        assert_eq!(state.session_metadata.unwrap().session_id, "test-session");
    }

    #[test]
    fn test_reset_then_start_new_workflow() {
        let (_tmp, storage) = setup_workflow_env();
        start(&storage);
        next_with(r#"{"spec_complete": true}"#, &storage);
        reset_workflow(&storage).unwrap();
        start(&storage);
        assert_at(&storage, "spec", "test_mode", &["spec"]);
    }

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
        assert!(
            result.is_err()
                && result
                    .unwrap_err()
                    .to_string()
                    .contains("No workflow loaded")
        );
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

        assert_state_eq(&state_before, "spec", "test_mode", &["spec"]);
        assert_state_eq(&state_after, "spec", "test_mode", &["spec"]);
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
        assert_at(&storage, "plan", "test_mode", &["spec", "plan"]);
    }

    #[test]
    fn test_next_prompt_implicit_multiple_transitions() {
        let (_tmp, storage) = setup_workflow_env();
        start(&storage);

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

        next(&storage);
        assert_eq!(
            get_state(&storage)
                .workflow_state
                .as_ref()
                .unwrap()
                .history
                .len(),
            3
        );
    }

    // ========== Restart Workflow Tests ==========

    #[test]
    fn test_restart_workflow_returns_to_spec() {
        let (_tmp, storage) = setup_workflow_env();
        start(&storage);
        next(&storage);
        assert_at(&storage, "plan", "test_mode", &["spec", "plan"]);

        restart_workflow(&storage).unwrap();
        assert_at(&storage, "spec", "test_mode", &["spec", "plan", "spec"]);
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

    // ========== Production Workflow Tests ==========

    #[test]
    #[cfg_attr(coverage, ignore)] // Skip during coverage runs (different working directory)
    fn test_discovery_workflow_loads_with_rules() {
        use crate::engine::load_workflow;

        let temp_dir = setup_production_workflows();
        let workflow_path = temp_dir.path().join("workflows/discovery.yaml");
        let workflow = load_workflow(&workflow_path).unwrap();
        assert_eq!(workflow.mode, "discovery");

        // Verify code node has rules
        let code_node = &workflow.nodes["code"];
        assert_eq!(code_node.rules.len(), 3);
    }

    #[test]
    #[cfg_attr(coverage, ignore)] // Skip during coverage runs (different working directory)
    fn test_execution_workflow_loads_with_rules() {
        use crate::engine::load_workflow;

        let temp_dir = setup_production_workflows();
        let workflow_path = temp_dir.path().join("workflows/execution.yaml");
        let workflow = load_workflow(&workflow_path).unwrap();
        assert_eq!(workflow.mode, "execution");

        // Verify code node has rules (4 rules including phase_timeout)
        let code_node = &workflow.nodes["code"];
        assert_eq!(code_node.rules.len(), 4);
    }

    #[test]
    #[cfg_attr(coverage, ignore)] // Skip during coverage runs (different working directory)
    fn test_discovery_workflow_rules_are_valid() {
        use crate::engine::load_workflow;

        let temp_dir = setup_production_workflows();
        let workflow_path = temp_dir.path().join("workflows/discovery.yaml");
        let workflow = load_workflow(&workflow_path).unwrap();
        let code_node = &workflow.nodes["code"];

        // Validate all rules (errors would panic)
        for rule in &code_node.rules {
            rule.validate().unwrap();
        }
    }

    #[test]
    #[cfg_attr(coverage, ignore)] // Skip during coverage runs (different working directory)
    fn test_execution_workflow_rules_are_valid() {
        use crate::engine::load_workflow;

        let temp_dir = setup_production_workflows();
        let workflow_path = temp_dir.path().join("workflows/execution.yaml");
        let workflow = load_workflow(&workflow_path).unwrap();
        let code_node = &workflow.nodes["code"];

        // Validate all rules (errors would panic)
        for rule in &code_node.rules {
            rule.validate().unwrap();
        }
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

    // ========== Integration Tests (End-to-End) ==========

    #[test]
    fn test_next_at_research_done_auto_transitions_to_discovery() {
        let (_tmp, storage) = setup_meta_mode_workflows();

        start_workflow("research", &storage).unwrap();
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

        start_workflow("discovery", &storage).unwrap();
        set_meta_mode(&storage, "learning");
        set_node(&storage, "done");

        next(&storage);

        let ws = get_state(&storage).workflow_state.unwrap();
        assert_eq!(ws.mode, "discovery");
        assert_eq!(ws.current_node, "done");
        assert_eq!(transition_count(&storage), 0);
    }
