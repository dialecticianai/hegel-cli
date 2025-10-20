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

pub fn start_workflow(workflow_name: &str, storage: &FileStorage) -> Result<()> {
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
    use std::fs;

    let workflows_dir = storage.workflows_dir();
    let entries = fs::read_dir(&workflows_dir)
        .with_context(|| format!("Failed to read workflows directory: {}", workflows_dir))?;

    let mut workflows = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                workflows.push(name.to_string());
            }
        }
    }

    workflows.sort();

    println!("Available workflows:");
    for workflow in workflows {
        println!("  {}", workflow);
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
