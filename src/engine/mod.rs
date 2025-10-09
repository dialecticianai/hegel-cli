use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::storage::WorkflowState;

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

    let workflow: Workflow = serde_yaml::from_str(&content)
        .with_context(|| "Failed to parse workflow YAML")?;

    Ok(workflow)
}

/// Initialize workflow state from workflow definition
pub fn init_state(workflow: &Workflow) -> WorkflowState {
    let start = workflow.start_node.clone();
    WorkflowState {
        current_node: start.clone(),
        mode: workflow.mode.clone(),
        history: vec![start],
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
        .with_context(|| format!("Next node not found in workflow: {}", new_state.current_node))?;

    Ok((next_node_obj.prompt.clone(), new_state))
}
