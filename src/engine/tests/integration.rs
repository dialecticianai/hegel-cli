use crate::engine::*;
use crate::storage::{GitInfo, State};
use crate::test_helpers::*;
use std::collections::HashSet;
use std::fs;
use tempfile::TempDir;

// ========== Step 7: End-to-End Integration Tests ==========

#[test]
fn test_workflow_with_prompt_hbs_field() {
    // Test that workflow nodes can use prompt_hbs field
    let yaml = r#"
mode: test
start_node: spec
nodes:
  spec:
    prompt_hbs: "Write {{> code_map}}"
    transitions:
      - when: done
        to: done
  done:
    transitions: []
"#;
    let workflow: Workflow = serde_yaml::from_str(yaml).unwrap();

    // Verify prompt_hbs field is set
    assert!(workflow.nodes["spec"].prompt.is_empty());
    assert!(!workflow.nodes["spec"].prompt_hbs.is_empty());
    assert_eq!(workflow.nodes["spec"].prompt_hbs, "Write {{> code_map}}");
}

#[test]
fn test_workflow_rejects_both_prompt_and_prompt_hbs() {
    let yaml = r#"
mode: test
start_node: spec
nodes:
  spec:
    prompt: "Old style"
    prompt_hbs: "New style"
    transitions:
      - when: done
        to: done
"#;
    let result: Result<Workflow, _> = serde_yaml::from_str(yaml);
    assert!(result.is_ok()); // YAML parses fine

    // But validation should fail
    let workflow = result.unwrap();
    let validation = workflow.validate();
    assert!(validation.is_err());
    assert!(validation
        .unwrap_err()
        .to_string()
        .contains("cannot have both"));
}

#[test]
fn test_init_state_sets_is_handlebars_for_hbs_node() {
    // Create workflow with prompt_hbs start node
    let yaml = r#"
mode: test
start_node: spec
nodes:
  spec:
    prompt_hbs: "{{> code_map}}"
    transitions: []
"#;
    let workflow: Workflow = serde_yaml::from_str(yaml).unwrap();
    let state = init_state(&workflow);

    // Verify is_handlebars is set to true
    assert!(state.is_handlebars);
}

#[test]
fn test_init_state_sets_is_handlebars_false_for_md_node() {
    // Create workflow with regular prompt start node
    let workflow = workflow("test", "spec")
        .with_node("spec", node("Write SPEC.md", vec![]))
        .build();

    let state = init_state(&workflow);

    // Verify is_handlebars is set to false
    assert!(!state.is_handlebars);
}

#[test]
fn test_get_next_prompt_updates_is_handlebars_on_transition() {
    let temp_dir = TempDir::new().unwrap();

    // Create workflow with mixed nodes
    let node_md = node("Old style prompt", vec![transition("next", "hbs_node")]);
    let node_hbs = Node {
        prompt: String::new(),
        prompt_hbs: "{{> code_map}}".to_string(),
        summary: String::new(),
        transitions: vec![],
        rules: vec![],
    };

    let workflow = workflow("test", "md_node")
        .with_node("md_node", node_md)
        .with_node("hbs_node", node_hbs)
        .build();

    let state = init_state(&workflow);
    assert!(!state.is_handlebars); // Start with MD

    // Transition to HBS node
    let mut claims = HashSet::new();
    claims.insert("next".to_string());

    let (prompt, new_state) =
        get_next_prompt(&workflow, &state, &claims, temp_dir.path(), None).unwrap();

    // Verify transition happened
    assert_eq!(new_state.current_node, "hbs_node");

    // Verify is_handlebars was updated
    assert!(new_state.is_handlebars);

    // Verify correct prompt was returned
    assert_eq!(prompt, "{{> code_map}}");
}

#[test]
fn test_end_to_end_workflow_with_handlebars_partial() {
    // Create workflow that references actual code_map partial
    let yaml = r#"
mode: test
start_node: spec
nodes:
  spec:
    prompt_hbs: "{{> code_map}}"
    transitions:
      - when: done
        to: done
  done:
    transitions: []
"#;
    let workflow: Workflow = serde_yaml::from_str(yaml).unwrap();

    // Validate workflow
    assert!(workflow.validate().is_ok());

    // Initialize state
    let state = init_state(&workflow);
    assert_eq!(state.current_node, "spec");
    assert!(state.is_handlebars);

    // Get next prompt
    let temp_dir = TempDir::new().unwrap();
    let claims = HashSet::new();
    let (prompt, _) = get_next_prompt(&workflow, &state, &claims, temp_dir.path(), None).unwrap();

    // Verify we got the Handlebars template (not rendered yet - that happens in render_node_prompt)
    assert_eq!(prompt, "{{> code_map}}");
}

#[test]
fn test_require_commits_rule_validation_rejects_zero() {
    use crate::rules::RuleConfig;

    // Test validation directly on RuleConfig
    let rule = RuleConfig::RequireCommits { lookback_phases: 0 };

    let result = rule.validate();
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("require_commits.lookback_phases must be >= 1"));
}

// Note: Full end-to-end integration test for require_commits rule blocking
// is covered by manual testing. The rule evaluation logic is thoroughly
// tested in src/rules/tests/evaluator.rs. This engine-level test focuses
// on the positive case (with commits) and force bypass functionality.

#[test]
fn test_require_commits_rule_allows_with_commits() {
    let temp_dir = TempDir::new().unwrap();

    // Create hooks and states files
    create_hooks_file(&[]);
    create_states_file(&[]);

    // Manually create phase metrics file with git commits
    let metrics_path = temp_dir.path().join("phase_metrics.jsonl");
    let phase_json = serde_json::json!({
        "phase_name": "code",
        "start_time": "2025-01-01T10:00:00Z".to_string(),
        "end_time": "2025-01-01T10:00:00Z".to_string(),
        "duration_seconds": 100,
        "token_metrics": {
            "total_input_tokens": 0,
            "total_output_tokens": 0,
            "total_cache_creation_tokens": 0,
            "total_cache_read_tokens": 0,
            "assistant_turns": 0
        },
        "git_commits": [{
            "hash": "abc123",
            "author": "test@example.com",
            "timestamp": "2025-01-01T10:00:00Z".to_string(),
            "message": "test commit",
            "files_changed": 1,
            "insertions": 10,
            "deletions": 5
        }]
    });
    fs::write(&metrics_path, serde_json::to_string(&phase_json).unwrap()).unwrap();

    let yaml = r#"
name: test
mode: test
start_node: code
nodes:
  code:
    prompt: "Write code"
    rules:
      - type: require_commits
        lookback_phases: 1
    transitions:
      - when: code_complete
        to: review
  review:
    prompt: "Review"
    transitions: []
"#;
    let workflow = load_workflow_from_str(yaml).unwrap();
    let mut state = init_state(&workflow);
    state.phase_start_time = Some("2025-01-01T10:00:00Z".to_string());

    let claims = HashSet::from(["code_complete".to_string()]);
    let (prompt, new_state) =
        get_next_prompt(&workflow, &state, &claims, temp_dir.path(), None).unwrap();

    // Should transition to review node
    assert_eq!(new_state.current_node, "review");
    assert_eq!(prompt, "Review");
}

#[test]
fn test_require_commits_force_bypass_all() {
    let temp_dir = TempDir::new().unwrap();

    // Create empty hooks and states files
    create_hooks_file(&[]);
    create_states_file(&[]);

    // Create git_info indicating we have a git repo
    let state_obj = State {
        workflow: None,
        session_metadata: None,
        cumulative_totals: None,
        git_info: Some(GitInfo {
            has_repo: true,
            current_branch: Some("main".to_string()),
            remote_url: None,
        }),
    };
    fs::write(
        temp_dir.path().join("state.json"),
        serde_json::to_string(&state_obj).unwrap(),
    )
    .unwrap();

    let yaml = r#"
name: test
mode: test
start_node: code
nodes:
  code:
    prompt: "Write code"
    rules:
      - type: require_commits
        lookback_phases: 1
    transitions:
      - when: code_complete
        to: review
  review:
    prompt: "Review"
    transitions: []
"#;
    let workflow = load_workflow_from_str(yaml).unwrap();
    let state = init_state(&workflow);

    let claims = HashSet::from(["code_complete".to_string()]);
    let force_bypass: Option<String> = None; // --force with no argument
    let (prompt, new_state) = get_next_prompt(
        &workflow,
        &state,
        &claims,
        temp_dir.path(),
        Some(&force_bypass),
    )
    .unwrap();

    // Should transition despite no commits (force bypass)
    assert_eq!(new_state.current_node, "review");
    assert_eq!(prompt, "Review");
}

#[test]
fn test_require_commits_force_bypass_specific() {
    let temp_dir = TempDir::new().unwrap();

    // Create empty hooks and states files
    create_hooks_file(&[]);
    create_states_file(&[]);

    // Create git_info indicating we have a git repo
    let state_obj = State {
        workflow: None,
        session_metadata: None,
        cumulative_totals: None,
        git_info: Some(GitInfo {
            has_repo: true,
            current_branch: Some("main".to_string()),
            remote_url: None,
        }),
    };
    fs::write(
        temp_dir.path().join("state.json"),
        serde_json::to_string(&state_obj).unwrap(),
    )
    .unwrap();

    let yaml = r#"
name: test
mode: test
start_node: code
nodes:
  code:
    prompt: "Write code"
    rules:
      - type: require_commits
        lookback_phases: 1
    transitions:
      - when: code_complete
        to: review
  review:
    prompt: "Review"
    transitions: []
"#;
    let workflow = load_workflow_from_str(yaml).unwrap();
    let state = init_state(&workflow);

    let claims = HashSet::from(["code_complete".to_string()]);
    let force_bypass: Option<String> = Some("require_commits".to_string()); // --force require_commits
    let (prompt, new_state) = get_next_prompt(
        &workflow,
        &state,
        &claims,
        temp_dir.path(),
        Some(&force_bypass),
    )
    .unwrap();

    // Should transition despite no commits (force bypass specific rule)
    assert_eq!(new_state.current_node, "review");
    assert_eq!(prompt, "Review");
}
