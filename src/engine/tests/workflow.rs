use crate::engine::*;
use crate::test_helpers::*;
use std::collections::HashMap;
use tempfile::TempDir;

// Helper to create test workflow YAML
fn create_test_workflow_file(temp_dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let workflow_path = temp_dir.path().join(format!("{}.yaml", name));
    std::fs::write(&workflow_path, content).unwrap();
    workflow_path
}

// ========== load_workflow Tests ==========

#[test]
fn test_load_workflow_discovery() {
    let temp_dir = TempDir::new().unwrap();

    let wf = workflow("discovery", "spec")
        .with_node(
            "spec",
            node("Write SPEC.md", vec![transition("spec_complete", "plan")]),
        )
        .with_node(
            "plan",
            node("Write PLAN.md", vec![transition("plan_complete", "done")]),
        )
        .build();

    let workflow_path = create_test_workflow_file(
        &temp_dir,
        "test_discovery",
        &serde_yaml::to_string(&wf).unwrap(),
    );
    let workflow = load_workflow(&workflow_path).unwrap();

    assert_eq!(workflow.mode, "discovery");
    assert_eq!(workflow.start_node, "spec");
    assert_eq!(
        workflow.nodes.len(),
        3,
        "Should have 2 explicit nodes + 1 implicit done node"
    );
    assert!(workflow.nodes.contains_key("spec"));
    assert!(workflow.nodes.contains_key("plan"));
    assert!(
        workflow.nodes.contains_key("done"),
        "Done node should be auto-injected"
    );
}

#[test]
fn test_load_workflow_execution() {
    let temp_dir = TempDir::new().unwrap();

    let wf = workflow("execution", "spec")
        .with_node(
            "spec",
            node("Write SPEC.md", vec![transition("spec_complete", "code")]),
        )
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
        .build();

    let workflow_path = create_test_workflow_file(
        &temp_dir,
        "test_execution",
        &serde_yaml::to_string(&wf).unwrap(),
    );
    let workflow = load_workflow(&workflow_path).unwrap();

    assert_eq!(workflow.mode, "execution");
    assert_eq!(workflow.start_node, "spec");
    assert_eq!(
        workflow.nodes.len(),
        5,
        "Should have 4 explicit nodes + 1 implicit done node"
    );

    let review_node = &workflow.nodes["review"];
    assert_eq!(review_node.transitions.len(), 2);
}

#[test]
fn test_load_workflow_missing_file() {
    let result = load_workflow("/nonexistent/workflow.yaml");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Workflow not found"));
}

#[test]
fn test_load_workflow_invalid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let invalid_yaml = "this is not: valid: yaml: content:::\n  - broken";
    let workflow_path = create_test_workflow_file(&temp_dir, "invalid", invalid_yaml);

    let result = load_workflow(&workflow_path);
    assert!(result.is_err());
}

#[test]
fn test_load_workflow_invalid_repeated_command_regex() {
    let yaml = r#"
mode: discovery
start_node: spec
nodes:
  spec:
    prompt: "Test"
    transitions:
      - when: done
        to: end
    rules:
      - type: repeated_command
        pattern: "[invalid"
        threshold: 5
        window: 120
  end:
    transitions: []
"#;
    let result = load_workflow_from_str(yaml);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Invalid rule in node 'spec'"));
}

#[test]
fn test_load_workflow_invalid_repeated_file_edit_regex() {
    let yaml = r#"
mode: execution
start_node: code
nodes:
  code:
    prompt: "Write code"
    transitions:
      - when: done
        to: end
    rules:
      - type: repeated_file_edit
        path_pattern: "(unclosed"
        threshold: 8
        window: 180
  end:
    transitions: []
"#;
    let result = load_workflow_from_str(yaml);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Invalid rule in node 'code'"));
}

// ========== init_state Tests ==========

#[test]
fn test_init_state_creates_correct_state() {
    let workflow = Workflow {
        mode: "discovery".to_string(),
        start_node: "spec".to_string(),
        nodes: HashMap::new(),
    };

    let state = init_state(&workflow);
    assert_eq!(state.current_node, "spec");
    assert_eq!(state.mode, "discovery");
    assert_eq!(state.history, vec!["spec"]);
}

#[test]
fn test_init_state_different_start_nodes() {
    let workflow1 = Workflow {
        mode: "execution".to_string(),
        start_node: "kickoff".to_string(),
        nodes: HashMap::new(),
    };

    let state1 = init_state(&workflow1);
    assert_eq!(state1.current_node, "kickoff");
    assert_eq!(state1.history, vec!["kickoff"]);

    let workflow2 = Workflow {
        mode: "minimal".to_string(),
        start_node: "begin".to_string(),
        nodes: HashMap::new(),
    };

    let state2 = init_state(&workflow2);
    assert_eq!(state2.current_node, "begin");
    assert_eq!(state2.history, vec!["begin"]);
}
