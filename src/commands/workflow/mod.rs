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
use transitions::detect_and_archive_cowboy_activity;
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
        if let Some(existing_workflow_value) = &existing_state.workflow {
            // Deserialize workflow to check if at terminal node
            let existing_workflow: crate::engine::Workflow =
                serde_yaml::from_value(existing_workflow_value.clone())
                    .context("Failed to deserialize existing workflow")?;

            // Allow starting new workflow if current is at a terminal node
            if !existing_workflow.is_terminal_node(&existing_ws.current_node) {
                anyhow::bail!(
                    "Cannot start new workflow: already in workflow '{}' at node '{}'.\n\
                     Run 'hegel abort' to abandon the current workflow first.",
                    existing_ws.mode,
                    existing_ws.current_node
                );
            }
        }
    }

    // Preserve existing meta-mode if set (optional - only needed for inter-workflow transitions)
    let existing_meta_mode = existing_state
        .workflow_state
        .as_ref()
        .and_then(|ws| ws.meta_mode.clone());

    // Detect and archive any cowboy activity between last workflow and now
    let now_timestamp = chrono::Utc::now().to_rfc3339();
    detect_and_archive_cowboy_activity(storage.state_dir(), &now_timestamp)?;

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

    // Store state (preserve session_metadata and cumulative_totals from existing state)
    let state = State {
        workflow: Some(serde_yaml::to_value(&workflow)?),
        workflow_state: Some(workflow_state.clone()),
        session_metadata: existing_state.session_metadata,
        cumulative_totals: existing_state.cumulative_totals,
        git_info: existing_state.git_info,
    };
    storage.save(&state)?;

    // Display output
    println!("{}", Theme::success("Workflow started").bold());
    println!();
    display_workflow_prompt(
        current_node,
        &workflow_state.mode,
        &node.prompt,
        workflow_state.is_handlebars,
        storage,
    )?;

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

pub fn next_prompt(claim_str: Option<&str>, storage: &FileStorage) -> Result<()> {
    let claim_alias = match claim_str {
        Some(name) => ClaimAlias::Custom(name.to_string()),
        None => ClaimAlias::Next,
    };
    advance_workflow(claim_alias, storage)
}

pub fn reset_workflow(storage: &FileStorage) -> Result<()> {
    // Load current state to preserve session_metadata
    let state = storage.load()?;

    // Clear workflow fields but keep session_metadata and cumulative_totals
    let cleared_state = State {
        workflow: None,
        workflow_state: None,
        session_metadata: state.session_metadata,
        cumulative_totals: state.cumulative_totals,
        git_info: state.git_info,
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

    // Log synthetic transition to aborted terminal node
    storage.log_state_transition(
        &workflow_state.current_node,
        "aborted",
        &workflow_state.mode,
        workflow_state.workflow_id.as_deref(),
    )?;

    // Archive the aborted workflow (now that it has a terminal ABORTED node)
    transitions::archive_and_cleanup(storage)?;

    println!("{}", Theme::success("Workflow aborted and archived"));
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

    // Display output
    println!("{}", Theme::warning("Re-displaying current prompt"));
    println!("{}: {}", Theme::label("Current node"), current_node);
    println!();

    // Check if node has a prompt
    let prompt_text = if !node.prompt_hbs.is_empty() {
        &node.prompt_hbs
    } else {
        &node.prompt
    };

    if prompt_text.is_empty() {
        println!("{}", Theme::secondary("(No prompt at this node)"));
    } else {
        // Render prompt with guides
        let rendered_prompt =
            render_node_prompt(prompt_text, workflow_state.is_handlebars, storage)?;
        println!("{}", Theme::header("Prompt:"));
        println!("{}", rendered_prompt);
    }

    Ok(())
}

pub fn restart_workflow(storage: &FileStorage) -> Result<()> {
    advance_workflow(ClaimAlias::Restart, storage)
}

pub fn prev_prompt(storage: &FileStorage) -> Result<()> {
    use chrono::Utc;

    // Load current state
    let state = storage.load()?;

    let workflow_yaml = state
        .workflow
        .as_ref()
        .context("No workflow loaded. Run 'hegel start <workflow>' first.")?;

    let mut workflow_state = state
        .workflow_state
        .clone()
        .context("No workflow state found")?;

    // Parse workflow from stored YAML value
    let workflow: crate::engine::Workflow =
        serde_yaml::from_value(workflow_yaml.clone()).context("Failed to parse stored workflow")?;

    // Validate we can go back
    if workflow_state.history.len() <= 1 {
        anyhow::bail!(
            "Cannot go back: already at the start of the workflow (node: {})",
            workflow_state.current_node
        );
    }

    // Get previous node from history
    let from_node = workflow_state.current_node.clone();

    // Pop current node from history
    workflow_state.history.pop();

    // Get the new current node (last item in history)
    let to_node = workflow_state
        .history
        .last()
        .context("History should not be empty after pop")?
        .clone();

    // Update current node
    workflow_state.current_node = to_node.clone();
    workflow_state.phase_start_time = Some(Utc::now().to_rfc3339());

    // Get node for display
    let node = workflow
        .nodes
        .get(&to_node)
        .with_context(|| format!("Node not found: {}", to_node))?;

    // Persist state
    let updated_state = State {
        workflow: Some(serde_yaml::to_value(&workflow)?),
        workflow_state: Some(workflow_state.clone()),
        session_metadata: state.session_metadata,
        cumulative_totals: state.cumulative_totals,
        git_info: state.git_info,
    };
    storage.save(&updated_state)?;

    // Log transition (backward)
    storage.log_state_transition(
        &from_node,
        &to_node,
        &workflow_state.mode,
        workflow_state.workflow_id.as_deref(),
    )?;

    // Display transition
    println!(
        "{} {} {} {}",
        Theme::success("Went back:").bold(),
        Theme::secondary(&from_node),
        Theme::secondary("←"),
        Theme::highlight(&to_node)
    );
    println!();

    // Select prompt based on which field is present
    let prompt_text = if !node.prompt_hbs.is_empty() {
        &node.prompt_hbs
    } else {
        &node.prompt
    };

    display_workflow_prompt(
        &to_node,
        &workflow_state.mode,
        prompt_text,
        workflow_state.is_handlebars,
        storage,
    )?;

    Ok(())
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

    // Separate into local-only and embedded (or both)
    let mut local_only: Vec<String> = filesystem.difference(&embedded).cloned().collect();
    local_only.sort();

    let mut embedded_workflows: Vec<String> = embedded.iter().cloned().collect();
    embedded_workflows.sort();

    // Display local-only workflows first
    if !local_only.is_empty() {
        println!("Local-only:\n");
        for workflow in &local_only {
            let workflow_path = format!("{}/{}.yaml", storage.workflows_dir(), workflow);
            let flow = match load_workflow(&workflow_path) {
                Ok(wf) => extract_node_flow(&wf),
                Err(_) => "".to_string(),
            };

            if flow.is_empty() {
                println!("  {}", workflow);
            } else {
                println!("  {}\n    {}", workflow, flow.dimmed());
            }
        }
        println!();
    }

    // Display embedded workflows
    if !embedded_workflows.is_empty() {
        println!("Embedded:\n");
        for workflow in &embedded_workflows {
            // Load workflow to extract node flow
            let workflow_path = format!("{}/{}.yaml", storage.workflows_dir(), workflow);
            let flow = match load_workflow(&workflow_path) {
                Ok(wf) => extract_node_flow(&wf),
                Err(_) => "".to_string(),
            };

            if flow.is_empty() {
                println!("  {}", workflow);
            } else {
                println!("  {}\n    {}", workflow, flow.dimmed());
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

/// List all available guides or show embedded guide content
pub fn list_guides(show_embedded: Option<&str>, storage: &FileStorage) -> Result<()> {
    // If --show-embedded flag is provided, display that guide's content
    if let Some(guide_name) = show_embedded {
        return show_embedded_guide(guide_name);
    }

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

/// Display the content of an embedded guide
fn show_embedded_guide(guide_name: &str) -> Result<()> {
    // Auto-add .md extension if missing
    let normalized_name = if guide_name.ends_with(".md") {
        guide_name.to_string()
    } else {
        format!("{}.md", guide_name)
    };

    // Try to get the embedded guide
    match crate::embedded::get_guide(&normalized_name) {
        Some(content) => {
            println!("{}", content);
            Ok(())
        }
        None => {
            anyhow::bail!(
                "Embedded guide '{}' not found. Run 'hegel guides' to see available guides.",
                normalized_name
            );
        }
    }
}

#[cfg(test)]
mod tests;
