mod template;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::storage::WorkflowState;
pub use template::render_template;

/// Workflow transition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub when: String,
    pub to: String,
}

/// Workflow node definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub prompt: String,
    pub transitions: Vec<Transition>,
}

/// Complete workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub mode: String,
    pub start_node: String,
    pub nodes: HashMap<String, Node>,
}

/// Load workflow definition from YAML file
pub fn load_workflow<P: AsRef<Path>>(yaml_path: P) -> Result<Workflow> {
    let content = fs::read_to_string(yaml_path.as_ref())
        .with_context(|| format!("Failed to read workflow file: {:?}", yaml_path.as_ref()))?;

    let workflow: Workflow =
        serde_yaml::from_str(&content).with_context(|| "Failed to parse workflow YAML")?;

    Ok(workflow)
}

/// Initialize workflow state from workflow definition
pub fn init_state(workflow: &Workflow) -> WorkflowState {
    let start = workflow.start_node.clone();
    WorkflowState {
        current_node: start.clone(),
        mode: workflow.mode.clone(),
        history: vec![start],
        workflow_id: None,
    }
}

/// Get next prompt based on current state and claims
pub fn get_next_prompt(
    workflow: &Workflow,
    state: &WorkflowState,
    claims: &HashMap<String, bool>,
) -> Result<(String, WorkflowState)> {
    let current = &state.current_node;
    let node = workflow
        .nodes
        .get(current)
        .with_context(|| format!("Node not found in workflow: {}", current))?;

    // Evaluate transitions - find first matching claim
    let mut next_node = current.clone();
    for transition in &node.transitions {
        if claims.get(&transition.when) == Some(&true) {
            next_node = transition.to.clone();
            break;
        }
    }

    // Build new state
    let mut new_state = state.clone();
    if next_node != *current {
        new_state.current_node = next_node.clone();
        new_state.history.push(next_node.clone());
    }

    // Get prompt for resulting node
    let next_node_obj = workflow
        .nodes
        .get(&new_state.current_node)
        .with_context(|| {
            format!(
                "Next node not found in workflow: {}",
                new_state.current_node
            )
        })?;

    Ok((next_node_obj.prompt.clone(), new_state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Helper to create test workflow YAML
    fn create_test_workflow_file(
        temp_dir: &TempDir,
        name: &str,
        content: &str,
    ) -> std::path::PathBuf {
        let workflow_path = temp_dir.path().join(format!("{}.yaml", name));
        std::fs::write(&workflow_path, content).unwrap();
        workflow_path
    }

    // ========== load_workflow Tests ==========

    #[test]
    fn test_load_workflow_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_content = r#"
mode: discovery
start_node: spec
nodes:
  spec:
    prompt: "Write SPEC.md"
    transitions:
      - when: spec_complete
        to: plan
  plan:
    prompt: "Write PLAN.md"
    transitions:
      - when: plan_complete
        to: done
  done:
    prompt: "Complete!"
    transitions: []
"#;
        let workflow_path = create_test_workflow_file(&temp_dir, "discovery", yaml_content);

        let workflow = load_workflow(&workflow_path).unwrap();
        assert_eq!(workflow.mode, "discovery");
        assert_eq!(workflow.start_node, "spec");
        assert_eq!(workflow.nodes.len(), 3);
        assert!(workflow.nodes.contains_key("spec"));
        assert!(workflow.nodes.contains_key("plan"));
        assert!(workflow.nodes.contains_key("done"));
    }

    #[test]
    fn test_load_workflow_execution() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_content = r#"
mode: execution
start_node: spec
nodes:
  spec:
    prompt: "Write SPEC.md"
    transitions:
      - when: spec_complete
        to: code
  code:
    prompt: "Write code"
    transitions:
      - when: code_complete
        to: review
  review:
    prompt: "Review code"
    transitions:
      - when: review_passed
        to: done
      - when: review_failed
        to: refactor
  refactor:
    prompt: "Refactor code"
    transitions:
      - when: refactor_complete
        to: code
  done:
    prompt: "Complete!"
    transitions: []
"#;
        let workflow_path = create_test_workflow_file(&temp_dir, "execution", yaml_content);

        let workflow = load_workflow(&workflow_path).unwrap();
        assert_eq!(workflow.mode, "execution");
        assert_eq!(workflow.start_node, "spec");
        assert_eq!(workflow.nodes.len(), 5);

        // Verify review node has multiple transitions
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
            .contains("Failed to read workflow file"));
    }

    #[test]
    fn test_load_workflow_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_yaml = "this is not: valid: yaml: content:::\n  - broken";
        let workflow_path = create_test_workflow_file(&temp_dir, "invalid", invalid_yaml);

        let result = load_workflow(&workflow_path);
        assert!(result.is_err());
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

    // ========== get_next_prompt Tests ==========

    #[test]
    fn test_get_next_prompt_successful_transition() {
        use crate::test_helpers::*;

        let workflow = workflow("discovery", "spec")
            .with_node(
                "spec",
                node("Write SPEC.md", vec![transition("spec_complete", "plan")]),
            )
            .with_node("plan", node("Write PLAN.md", vec![]))
            .build();

        let state = init_state(&workflow);
        let mut claims = HashMap::new();
        claims.insert("spec_complete".to_string(), true);

        let (prompt, new_state) = get_next_prompt(&workflow, &state, &claims).unwrap();
        assert_eq!(new_state.current_node, "plan");
        assert_eq!(new_state.history, vec!["spec", "plan"]);
        assert_eq!(prompt, "Write PLAN.md");
    }

    #[test]
    fn test_get_next_prompt_no_matching_transition() {
        use crate::test_helpers::*;

        let workflow = workflow("discovery", "spec")
            .with_node(
                "spec",
                node("Write SPEC.md", vec![transition("spec_complete", "plan")]),
            )
            .build();

        let state = init_state(&workflow);
        let mut claims = HashMap::new();
        claims.insert("wrong_claim".to_string(), true);

        let (prompt, new_state) = get_next_prompt(&workflow, &state, &claims).unwrap();
        assert_eq!(new_state.current_node, "spec");
        assert_eq!(new_state.history, vec!["spec"]);
        assert_eq!(prompt, "Write SPEC.md");
    }

    #[test]
    fn test_get_next_prompt_full_workflow_cycle() {
        use crate::test_helpers::*;

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
            .with_node("done", node("Complete!", vec![]))
            .build();

        let mut state = init_state(&workflow);
        let mut claims = HashMap::new();

        // SPEC -> PLAN -> CODE -> LEARNINGS -> DONE
        for (claim, expected_node) in [
            ("spec_complete", "plan"),
            ("plan_complete", "code"),
            ("code_complete", "learnings"),
            ("learnings_complete", "done"),
        ] {
            claims.clear();
            claims.insert(claim.to_string(), true);
            let (_, new_state) = get_next_prompt(&workflow, &state, &claims).unwrap();
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
        use crate::test_helpers::*;

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
            .with_node("done", node("Complete!", vec![]))
            .build();

        let mut state = WorkflowState {
            current_node: "code".to_string(),
            mode: "execution".to_string(),
            history: vec!["code".to_string()],
            workflow_id: None,
        };

        let mut claims = HashMap::new();

        // CODE -> REVIEW -> REFACTOR -> CODE (loop)
        for (claim, expected_node) in [
            ("code_complete", "review"),
            ("review_failed", "refactor"),
            ("refactor_complete", "code"),
        ] {
            claims.clear();
            claims.insert(claim.to_string(), true);
            let (_, new_state) = get_next_prompt(&workflow, &state, &claims).unwrap();
            state = new_state;
            assert_eq!(state.current_node, expected_node);
        }

        assert_eq!(state.history, vec!["code", "review", "refactor", "code"]);
    }

    #[test]
    fn test_get_next_prompt_multiple_transitions_first_match_wins() {
        use crate::test_helpers::*;

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
        let mut claims = HashMap::new();
        claims.insert("option_b".to_string(), true);
        claims.insert("option_c".to_string(), true);

        let (_, new_state) = get_next_prompt(&workflow, &state, &claims).unwrap();
        assert_eq!(new_state.current_node, "path_b");
    }

    #[test]
    fn test_get_next_prompt_invalid_next_node() {
        use crate::test_helpers::*;

        let workflow = workflow("test", "start")
            .with_node(
                "start",
                node("Start", vec![transition("go", "nonexistent")]),
            )
            .build();

        let state = init_state(&workflow);
        let mut claims = HashMap::new();
        claims.insert("go".to_string(), true);

        let result = get_next_prompt(&workflow, &state, &claims);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Next node not found"));
        assert!(err_msg.contains("nonexistent"));
    }
}
