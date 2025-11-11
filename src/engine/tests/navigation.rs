use crate::engine::*;
use crate::storage::WorkflowState;
use crate::test_helpers::*;
use std::collections::HashSet;
use tempfile::TempDir;

// ========== get_next_prompt Tests ==========

#[test]
fn test_get_next_prompt_successful_transition() {
    let temp_dir = TempDir::new().unwrap();

    let workflow = workflow("discovery", "spec")
        .with_node(
            "spec",
            node("Write SPEC.md", vec![transition("spec_complete", "plan")]),
        )
        .with_node("plan", node("Write PLAN.md", vec![]))
        .build();

    let state = init_state(&workflow);
    let claims = HashSet::from(["spec_complete".to_string()]);

    let (prompt, new_state) =
        get_next_prompt(&workflow, &state, &claims, temp_dir.path(), None).unwrap();
    assert_eq!(new_state.current_node, "plan");
    assert_eq!(new_state.history, vec!["spec", "plan"]);
    assert_eq!(prompt, "Write PLAN.md");
}

#[test]
fn test_get_next_prompt_no_matching_transition() {
    let temp_dir = TempDir::new().unwrap();

    let workflow = workflow("discovery", "spec")
        .with_node(
            "spec",
            node("Write SPEC.md", vec![transition("spec_complete", "plan")]),
        )
        .build();

    let state = init_state(&workflow);
    let claims = HashSet::from(["wrong_claim".to_string()]);

    let (prompt, new_state) =
        get_next_prompt(&workflow, &state, &claims, temp_dir.path(), None).unwrap();
    assert_eq!(new_state.current_node, "spec");
    assert_eq!(new_state.history, vec!["spec"]);
    assert_eq!(prompt, "Write SPEC.md");
}

#[test]
fn test_get_next_prompt_full_workflow_cycle() {
    let temp_dir = TempDir::new().unwrap();

    let workflow = workflow("discovery", "spec")
        .with_node(
            "spec",
            node("Write SPEC.md", vec![transition("spec_complete", "plan")]),
        )
        .with_node(
            "plan",
            node("Write PLAN.md", vec![transition("plan_complete", "code")]),
        )
        .with_node(
            "code",
            node("Write code", vec![transition("code_complete", "learnings")]),
        )
        .with_node(
            "learnings",
            node(
                "Write LEARNINGS.md",
                vec![transition("learnings_complete", "done")],
            ),
        )
        .with_node("done", node("", vec![]))
        .build();

    let mut state = init_state(&workflow);

    // SPEC -> PLAN -> CODE -> LEARNINGS -> DONE
    for (claim, expected_node) in [
        ("spec_complete", "plan"),
        ("plan_complete", "code"),
        ("code_complete", "learnings"),
        ("learnings_complete", "done"),
    ] {
        let claims = HashSet::from([claim.to_string()]);
        let (_, new_state) =
            get_next_prompt(&workflow, &state, &claims, temp_dir.path(), None).unwrap();
        state = new_state;
        assert_eq!(state.current_node, expected_node);
    }

    assert_eq!(
        state.history,
        vec!["spec", "plan", "code", "learnings", "done"]
    );
}

#[test]
fn test_get_next_prompt_review_loop() {
    let temp_dir = TempDir::new().unwrap();

    let workflow = workflow("execution", "code")
        .with_node(
            "code",
            node("Write code", vec![transition("code_complete", "review")]),
        )
        .with_node(
            "review",
            node(
                "Review code",
                vec![
                    transition("review_passed", "done"),
                    transition("review_failed", "refactor"),
                ],
            ),
        )
        .with_node(
            "refactor",
            node(
                "Refactor code",
                vec![transition("refactor_complete", "code")],
            ),
        )
        .with_node("done", node("", vec![]))
        .build();

    let mut state = WorkflowState {
        current_node: "code".to_string(),
        mode: "execution".to_string(),
        history: vec!["code".to_string()],
        workflow_id: None,
        meta_mode: None,
        phase_start_time: Some(chrono::Utc::now().to_rfc3339()),
        is_handlebars: false,
    };

    // CODE -> REVIEW -> REFACTOR -> CODE (loop)
    for (claim, expected_node) in [
        ("code_complete", "review"),
        ("review_failed", "refactor"),
        ("refactor_complete", "code"),
    ] {
        let claims = HashSet::from([claim.to_string()]);
        let (_, new_state) =
            get_next_prompt(&workflow, &state, &claims, temp_dir.path(), None).unwrap();
        state = new_state;
        assert_eq!(state.current_node, expected_node);
    }

    assert_eq!(state.history, vec!["code", "review", "refactor", "code"]);
}

#[test]
fn test_get_next_prompt_multiple_transitions_first_match_wins() {
    let temp_dir = TempDir::new().unwrap();

    let workflow = workflow("test", "start")
        .with_node(
            "start",
            node(
                "Start",
                vec![
                    transition("option_a", "path_a"),
                    transition("option_b", "path_b"),
                    transition("option_c", "path_c"),
                ],
            ),
        )
        .with_node("path_a", node("Path A", vec![]))
        .with_node("path_b", node("Path B", vec![]))
        .with_node("path_c", node("Path C", vec![]))
        .build();

    let state = init_state(&workflow);
    let claims = HashSet::from(["option_b".to_string(), "option_c".to_string()]);

    let (_, new_state) =
        get_next_prompt(&workflow, &state, &claims, temp_dir.path(), None).unwrap();
    assert_eq!(new_state.current_node, "path_b");
}

#[test]
fn test_get_next_prompt_invalid_next_node() {
    let temp_dir = TempDir::new().unwrap();

    let workflow = workflow("test", "start")
        .with_node(
            "start",
            node("Start", vec![transition("go", "nonexistent")]),
        )
        .build();

    let state = init_state(&workflow);
    let claims = HashSet::from(["go".to_string()]);

    let result = get_next_prompt(&workflow, &state, &claims, temp_dir.path(), None);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Next node not found"));
    assert!(err_msg.contains("nonexistent"));
}
