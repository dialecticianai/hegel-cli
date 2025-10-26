mod claims;
mod context;
mod transitions;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::engine::{init_state, load_workflow};
use crate::storage::{FileStorage, State};
use crate::theme::Theme;

pub use claims::ClaimAlias;
pub use context::{display_workflow_prompt, load_workflow_context, render_node_prompt};
pub use transitions::{evaluate_transition, execute_transition};

// Re-export for tests
#[cfg(test)]
pub use transitions::{TransitionOption, TransitionOutcome};

pub fn start_workflow(
    workflow_name: &str,
    start_node: Option<&str>,
    storage: &FileStorage,
) -> Result<()> {
    use chrono::Utc;

    // Load current state to check for existing meta-mode
    let existing_state = storage.load()?;

    // Check if there's already an active workflow
    if let Some(existing_ws) = &existing_state.workflow_state {
        if existing_state.workflow.is_some() {
            anyhow::bail!(
                "Cannot start new workflow: already in workflow '{}' at node '{}'.\n\
                 Run 'hegel abort' to abandon the current workflow first.",
                existing_ws.mode,
                existing_ws.current_node
            );
        }
    }

    let existing_meta_mode = existing_state
        .workflow_state
        .as_ref()
        .and_then(|ws| ws.meta_mode.clone())
        .or_else(|| {
            // Default to "standard" meta-mode for backward compatibility with tests
            // In production, users should run 'hegel meta <name>' first
            Some(crate::storage::MetaMode {
                name: "standard".to_string(),
            })
        });

    // Load workflow from YAML file
    let workflows_dir = storage.workflows_dir();
    let workflow_path = format!("{}/{}.yaml", workflows_dir, workflow_name);
    let workflow = load_workflow(&workflow_path)
        .with_context(|| format!("Failed to load workflow: {}", workflow_name))?;

    // Initialize workflow state with workflow_id (ISO timestamp)
    let mut workflow_state = init_state(&workflow);
    workflow_state.workflow_id = Some(Utc::now().to_rfc3339());

    // Override starting node if provided
    if let Some(node_name) = start_node {
        // Validate node exists
        if !workflow.nodes.contains_key(node_name) {
            anyhow::bail!(
                "Invalid starting node '{}'. Available nodes: {}",
                node_name,
                workflow
                    .nodes
                    .keys()
                    .map(|k| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        workflow_state.current_node = node_name.to_string();
        workflow_state.history = vec![node_name.to_string()];
    }

    // Preserve meta-mode from existing state
    workflow_state.meta_mode = existing_meta_mode;

    // Get current node and prompt
    let current_node = &workflow_state.current_node;
    let node = workflow
        .nodes
        .get(current_node)
        .with_context(|| format!("Node not found: {}", current_node))?;

    // Store state (preserve session_metadata from existing state)
    let state = State {
        workflow: Some(serde_yaml::to_value(&workflow)?),
        workflow_state: Some(workflow_state.clone()),
        session_metadata: existing_state.session_metadata,
    };
    storage.save(&state)?;

    // Display output
    println!("{}", Theme::success("Workflow started").bold());
    println!();
    display_workflow_prompt(current_node, &workflow_state.mode, &node.prompt, storage)?;

    Ok(())
}

/// Core workflow advancement logic used by next, repeat, and restart
fn advance_workflow(claim_alias: ClaimAlias, storage: &FileStorage) -> Result<()> {
    // Load workflow context
    let mut context = load_workflow_context(storage)?;

    // Convert alias to claims
    let claims = claim_alias.to_claims(&context.workflow_state.current_node)?;

    // Evaluate transition
    let outcome = evaluate_transition(&context, &claims, storage)?;

    // Execute transition
    execute_transition(outcome, &mut context, storage)?;

    Ok(())
}

pub fn next_prompt(claims_str: Option<&str>, storage: &FileStorage) -> Result<()> {
    let claim_alias = match claims_str {
        Some(json) => ClaimAlias::Custom(json.to_string()),
        None => ClaimAlias::Next,
    };
    advance_workflow(claim_alias, storage)
}

pub fn show_status(storage: &FileStorage) -> Result<()> {
    let state = storage.load()?;

    if state.workflow.is_none() || state.workflow_state.is_none() {
        println!("{}", Theme::warning("No workflow loaded"));
        println!();
        println!(
            "Start a workflow with: {}",
            Theme::highlight("hegel start <workflow>")
        );
        return Ok(());
    }

    let workflow_state = state.workflow_state.as_ref().unwrap();

    println!("{}", Theme::header("Workflow Status"));
    println!();
    println!("{}: {}", Theme::label("Mode"), workflow_state.mode);
    println!(
        "{}: {}",
        Theme::label("Current node"),
        workflow_state.current_node
    );
    println!();
    println!("{}", Theme::label("History:"));
    for (i, node) in workflow_state.history.iter().enumerate() {
        if i == workflow_state.history.len() - 1 {
            println!("  {} {}", Theme::highlight("→"), Theme::highlight(node));
        } else {
            println!("    {}", Theme::secondary(node));
        }
    }

    Ok(())
}

pub fn reset_workflow(storage: &FileStorage) -> Result<()> {
    // Load current state to preserve session_metadata
    let state = storage.load()?;

    // Clear workflow fields but keep session_metadata
    let cleared_state = State {
        workflow: None,
        workflow_state: None,
        session_metadata: state.session_metadata,
    };

    storage.save(&cleared_state)?;
    println!("{}", Theme::success("Workflow state cleared"));
    Ok(())
}

pub fn abort_workflow(storage: &FileStorage) -> Result<()> {
    // Load current state
    let state = storage.load()?;

    // Check if there's an active workflow
    if state.workflow.is_none() {
        println!("No active workflow to abort.");
        return Ok(());
    }

    let workflow_state = state
        .workflow_state
        .as_ref()
        .context("No workflow state found")?;

    println!(
        "{}",
        Theme::warning(&format!(
            "Aborting workflow '{}' at node '{}'",
            workflow_state.mode, workflow_state.current_node
        ))
    );

    // Clear workflow fields but keep session_metadata
    let cleared_state = State {
        workflow: None,
        workflow_state: None,
        session_metadata: state.session_metadata,
    };

    storage.save(&cleared_state)?;
    println!("{}", Theme::success("Workflow aborted"));
    Ok(())
}

pub fn repeat_prompt(storage: &FileStorage) -> Result<()> {
    // Load current state
    let state = storage.load()?;

    let workflow_yaml = state
        .workflow
        .as_ref()
        .context("No workflow loaded. Run 'hegel start <workflow>' first.")?;

    let workflow_state = state
        .workflow_state
        .as_ref()
        .context("No workflow state found")?;

    // Parse workflow from stored YAML value
    let workflow: crate::engine::Workflow =
        serde_yaml::from_value(workflow_yaml.clone()).context("Failed to parse stored workflow")?;

    // Get current node prompt
    let current_node = &workflow_state.current_node;
    let node = workflow
        .nodes
        .get(current_node)
        .with_context(|| format!("Current node not found: {}", current_node))?;

    // Render prompt with guides
    let rendered_prompt = render_node_prompt(&node.prompt, storage)?;

    // Display output
    println!("{}", Theme::warning("Re-displaying current prompt"));
    println!("{}: {}", Theme::label("Current node"), current_node);
    println!();
    println!("{}", Theme::header("Prompt:"));
    println!("{}", rendered_prompt);

    Ok(())
}

pub fn restart_workflow(storage: &FileStorage) -> Result<()> {
    advance_workflow(ClaimAlias::Restart, storage)
}

/// List all available workflows
pub fn list_workflows(storage: &FileStorage) -> Result<()> {
    use std::collections::HashSet;
    use std::fs;

    // Get embedded workflows
    let embedded: HashSet<String> = crate::embedded::list_workflows()
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Get filesystem workflows
    let workflows_dir = storage.workflows_dir();
    let mut filesystem = HashSet::new();

    if let Ok(entries) = fs::read_dir(&workflows_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    filesystem.insert(name.to_string());
                }
            }
        }
    }

    // Combine and sort
    let mut all_workflows: Vec<String> = embedded.union(&filesystem).cloned().collect();
    all_workflows.sort();

    println!("Available workflows:\n");
    for workflow in &all_workflows {
        let mut markers = Vec::new();
        if embedded.contains(workflow) {
            markers.push("embedded");
        }
        if filesystem.contains(workflow) {
            markers.push("local");
        }

        // Load workflow to extract node flow
        let workflow_path = format!("{}/{}.yaml", storage.workflows_dir(), workflow);
        let flow = match load_workflow(&workflow_path) {
            Ok(wf) => extract_node_flow(&wf),
            Err(_) => "".to_string(),
        };

        if markers.is_empty() {
            if flow.is_empty() {
                println!("  {}", workflow);
            } else {
                println!("  {}\n    {}", workflow, flow.dimmed());
            }
        } else {
            if flow.is_empty() {
                println!("  {} ({})", workflow, markers.join(", "));
            } else {
                println!(
                    "  {} ({})\n    {}",
                    workflow,
                    markers.join(", "),
                    flow.dimmed()
                );
            }
        }
    }

    Ok(())
}

/// Extract a simple node flow visualization from a workflow
fn extract_node_flow(workflow: &crate::engine::Workflow) -> String {
    use std::collections::{HashMap, HashSet};

    let start = &workflow.start_node;
    let mut visited = HashSet::new();
    let mut flow = Vec::new();
    let mut current = start.clone();

    // Build a map of node -> next node (taking first transition)
    let mut next_map: HashMap<String, Option<String>> = HashMap::new();
    for (node_name, node) in &workflow.nodes {
        let next = node
            .transitions
            .first()
            .map(|t| t.to.clone())
            .filter(|to| to != node_name); // Skip self-loops
        next_map.insert(node_name.clone(), next);
    }

    // Follow the chain from start node
    flow.push(current.clone());
    visited.insert(current.clone());

    while let Some(Some(next)) = next_map.get(&current) {
        if visited.contains(next) {
            break; // Prevent infinite loops
        }
        flow.push(next.clone());
        visited.insert(next.clone());
        current = next.clone();
    }

    flow.join(" → ")
}

/// List all available guides
pub fn list_guides(storage: &FileStorage) -> Result<()> {
    use std::collections::HashSet;
    use std::fs;

    // Get embedded guides
    let embedded: HashSet<String> = crate::embedded::list_guides()
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Get filesystem guides
    let guides_dir = storage.guides_dir();
    let mut filesystem = HashSet::new();

    if let Ok(entries) = fs::read_dir(&guides_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    filesystem.insert(name.to_string());
                }
            }
        }
    }

    // Combine and sort
    let mut all_guides: Vec<String> = embedded.union(&filesystem).cloned().collect();
    all_guides.sort();

    println!("Available guides:");
    for guide in &all_guides {
        let mut markers = Vec::new();
        if embedded.contains(guide) {
            markers.push("embedded");
        }
        if filesystem.contains(guide) {
            markers.push("local");
        }

        if markers.is_empty() {
            println!("  {}", guide);
        } else {
            println!("  {} ({})", guide, markers.join(", "));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
