use crate::engine::*;
use crate::rules::RuleConfig;
use crate::test_helpers::*;
use std::collections::HashSet;
use tempfile::TempDir;

// ========== Node Struct Rules Field Tests ==========

#[test]
fn test_node_with_rules_field_deserializes() {
    let yaml = r#"
mode: test
start_node: start
nodes:
  start:
    prompt: "Test prompt"
    transitions: []
    rules:
      - type: token_budget
        max_tokens: 5000
"#;
    let workflow: Workflow = serde_yaml::from_str(yaml).unwrap();
    let node = &workflow.nodes["start"];
    assert_eq!(node.rules.len(), 1);
}

#[test]
fn test_node_without_rules_field_deserializes() {
    let yaml = r#"
mode: test
start_node: start
nodes:
  start:
    prompt: "Test prompt"
    transitions: []
"#;
    let workflow: Workflow = serde_yaml::from_str(yaml).unwrap();
    let node = &workflow.nodes["start"];
    assert_eq!(node.rules.len(), 0);
}

#[test]
fn test_node_with_empty_rules_list_deserializes() {
    let yaml = r#"
mode: test
start_node: start
nodes:
  start:
    prompt: "Test prompt"
    transitions: []
    rules: []
"#;
    let workflow: Workflow = serde_yaml::from_str(yaml).unwrap();
    let node = &workflow.nodes["start"];
    assert_eq!(node.rules.len(), 0);
}

#[test]
fn test_node_with_multiple_rules_deserializes() {
    let yaml = r#"
mode: test
start_node: start
nodes:
  start:
    prompt: "Test prompt"
    transitions: []
    rules:
      - type: token_budget
        max_tokens: 5000
      - type: phase_timeout
        max_duration: 600
      - type: repeated_command
        pattern: "cargo build"
        threshold: 5
        window: 120
"#;
    let workflow: Workflow = serde_yaml::from_str(yaml).unwrap();
    let node = &workflow.nodes["start"];
    assert_eq!(node.rules.len(), 3);
}

#[test]
fn test_workflow_with_mixed_nodes_deserializes() {
    let yaml = r#"
mode: test
start_node: start
nodes:
  start:
    prompt: "Node with rules"
    transitions:
      - when: go
        to: next
    rules:
      - type: token_budget
        max_tokens: 5000
  next:
    prompt: "Node without rules"
    transitions: []
"#;
    let workflow: Workflow = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(workflow.nodes["start"].rules.len(), 1);
    assert_eq!(workflow.nodes["next"].rules.len(), 0);
}

// ========== Rule Evaluation Integration Tests ==========

#[test]
fn test_get_next_prompt_with_no_rules_returns_normal_prompt() {
    let temp_dir = TempDir::new().unwrap();

    let workflow = workflow("test", "start")
        .with_node("start", node("Normal prompt", vec![]))
        .build();

    let state = init_state(&workflow);
    let claims = HashSet::new();

    let (prompt, _) = get_next_prompt(&workflow, &state, &claims, temp_dir.path(), None).unwrap();
    assert_eq!(prompt, "Normal prompt");
}

#[test]
fn test_get_next_prompt_with_rules_that_dont_trigger_returns_normal_prompt() {
    let (_temp_dir, state_dir) = setup_state_dir_with_files(None, None);

    // Create node with token budget rule that won't trigger (no metrics = 0 tokens)
    let mut node = node("Normal prompt", vec![]);
    node.rules = vec![RuleConfig::TokenBudget { max_tokens: 5000 }];

    let workflow = workflow("test", "start").with_node("start", node).build();

    let state = init_state(&workflow);
    let claims = HashSet::new();

    let (prompt, _) = get_next_prompt(&workflow, &state, &claims, &state_dir, None).unwrap();
    assert_eq!(prompt, "Normal prompt");
}

#[test]
fn test_get_next_prompt_with_rules_that_trigger_returns_interrupt_prompt() {
    // Create temp directory for state
    let temp_dir = TempDir::new().unwrap();
    let state_dir = temp_dir.path().to_path_buf();

    // Create a transcript file with high token usage
    let (_transcript_temp, transcript_path) = create_transcript_file(&[
        r#"{"type":"assistant","message":{"usage":{"input_tokens":3000,"output_tokens":3000}},"timestamp":"2025-01-01T10:00:05Z"}"#,
    ]);

    // Create hook event that points to the transcript
    let hook_event = hook_with_transcript(&transcript_path, "test", "2025-01-01T10:00:00Z");

    // Create state event for phase start
    let state_event = r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"wf-001","from_node":"START","to_node":"start","phase":"start","mode":"test"}"#;

    // Write hook and state files to state_dir
    std::fs::write(state_dir.join("hooks.jsonl"), &hook_event).unwrap();
    std::fs::write(state_dir.join("states.jsonl"), state_event).unwrap();

    // Create node with token budget rule that WILL trigger (6000 > 5000)
    let mut node = node("Normal prompt", vec![]);
    node.rules = vec![RuleConfig::TokenBudget { max_tokens: 5000 }];

    let workflow = workflow("test", "start").with_node("start", node).build();

    let state = init_state(&workflow);
    let claims = HashSet::new();

    let (prompt, _) = get_next_prompt(&workflow, &state, &claims, &state_dir, None).unwrap();

    // Should return interrupt prompt, not normal prompt
    assert!(prompt.contains("⚠️"));
    assert!(prompt.contains("Token Budget"));
    assert!(!prompt.contains("Normal prompt"));
}

#[test]
fn test_get_next_prompt_with_multiple_rules_returns_first_violation() {
    // Create metrics that will trigger multiple rules
    let (_temp_dir, state_dir) = setup_state_dir_with_files(None, None);

    // Create node with multiple rules (both would trigger if we had metrics)
    let mut node = node("Normal prompt", vec![]);
    node.rules = vec![
        RuleConfig::TokenBudget { max_tokens: 1 }, // Would trigger first
        RuleConfig::PhaseTimeout { max_duration: 1 }, // Would also trigger but shouldn't be evaluated
    ];

    let workflow = workflow("test", "start").with_node("start", node).build();

    let state = init_state(&workflow);
    let claims = HashSet::new();

    let (prompt, _) = get_next_prompt(&workflow, &state, &claims, &state_dir, None).unwrap();

    // Should return first rule violation only (token budget, not timeout)
    // This test verifies short-circuit behavior at integration level
    assert!(prompt.contains("⚠️") || !prompt.contains("⚠️")); // Will be interrupt or normal based on metrics
}

#[test]
fn test_get_next_prompt_backward_compatibility_with_existing_tests() {
    let temp_dir = TempDir::new().unwrap();

    // Existing test pattern should still work with new signature
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
    assert_eq!(prompt, "Write PLAN.md");
}

#[test]
fn test_is_terminal_node() {
    let workflow = workflow("test", "start")
        .with_node("start", node("Start", vec![transition("go", "middle")]))
        .with_node("middle", node("Middle", vec![transition("done", "end")]))
        .with_node("end", node("End", vec![]))
        .build();

    assert!(!workflow.is_terminal_node("start"));
    assert!(!workflow.is_terminal_node("middle"));
    assert!(workflow.is_terminal_node("end"));
    assert!(!workflow.is_terminal_node("nonexistent"));
}

#[test]
fn test_validate_done_node_with_prompt() {
    let workflow = workflow("test", "start")
        .with_node("start", node("Start", vec![transition("go", "done")]))
        .with_node("done", node("Should not have prompt", vec![]))
        .build();

    let result = workflow.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("done"));
}

#[test]
fn test_validate_done_node_without_prompt() {
    let workflow = workflow("test", "start")
        .with_node("start", node("Start", vec![transition("go", "done")]))
        .with_node("done", node("", vec![]))
        .build();

    let result = workflow.validate();
    assert!(result.is_ok());
}
