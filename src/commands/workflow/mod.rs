mod claims;
mod context;
mod listing;
mod stash;
mod transitions;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::engine::{init_state, load_workflow};
use crate::storage::FileStorage;
use crate::theme::Theme;

pub use claims::ClaimAlias;
pub use context::{display_workflow_prompt, load_workflow_context, render_node_prompt};
use transitions::detect_and_archive_cowboy_activity;
pub use transitions::{evaluate_transition, execute_transition};

// Re-export for tests
#[cfg(test)]
pub use transitions::{TransitionOption, TransitionOutcome};

pub use listing::{list_guides, list_workflows};
pub use stash::{drop_stash, list_stashes, pop_stash, stash_workflow};

#[cfg(test)]
pub(crate) use listing::extract_node_flow;

pub fn start_workflow(
    workflow_name: &str,
    start_node: Option<&str>,
    storage: &FileStorage,
) -> Result<()> {
    use chrono::Utc;

    // Load current state to check for existing meta-mode
    let existing_state = storage.load()?;

    // Check if there's already an active workflow
    if let Some(existing_ws) = &existing_state.workflow {
        // Load existing workflow from YAML file
        let existing_workflow_path = storage.workflow_path(&existing_ws.mode);
        if let Ok(existing_workflow) = load_workflow(&existing_workflow_path) {
            // Allow starting new workflow if current is at a terminal node
            if !existing_workflow.is_terminal_node(&existing_ws.current_node) {
                anyhow::bail!(
                    "Cannot start new workflow: already in workflow '{}' at node '{}'.\n\
                     Report to the user that there is an active workflow and ask how they want to proceed.",
                    existing_ws.mode,
                    existing_ws.current_node
                );
            }
        }
    }

    // Preserve existing meta-mode if set (optional - only needed for inter-workflow transitions)
    let existing_meta_mode = existing_state
        .workflow
        .as_ref()
        .and_then(|ws| ws.meta_mode.clone());

    // Detect and archive any cowboy activity between last workflow and now
    let now_timestamp = chrono::Utc::now().to_rfc3339();
    detect_and_archive_cowboy_activity(storage.state_dir(), &now_timestamp)?;

    // Load workflow from YAML file
    let workflow_path = storage.workflow_path(workflow_name);
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
    let state = existing_state.with_workflow(Some(workflow_state.clone()));
    storage.save(&state)?;

    // Log initial state transition (START -> first node)
    // This ensures the first phase gets captured in archives
    storage.log_state_transition(
        "START",
        current_node,
        &workflow_state.mode,
        workflow_state.workflow_id.as_deref(),
    )?;

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
/// Returns the current node after the transition
fn advance_workflow(
    claim_alias: ClaimAlias,
    storage: &FileStorage,
    force_bypass: Option<&Option<String>>,
) -> Result<String> {
    // Load workflow context
    let mut context = load_workflow_context(storage)?;

    // Convert alias to claims
    let claims = claim_alias.to_claims(&context.workflow_state.current_node)?;

    // Evaluate transition
    let outcome = evaluate_transition(&context, &claims, storage, force_bypass)?;

    // Execute transition
    execute_transition(outcome, &mut context, storage)?;

    // Return the current node after transition
    Ok(context.workflow_state.current_node.clone())
}

pub fn next_prompt(
    claim_str: Option<&str>,
    force_bypass: Option<&Option<String>>,
    storage: &FileStorage,
) -> Result<String> {
    let claim_alias = match claim_str {
        Some(name) => ClaimAlias::Custom(name.to_string()),
        None => ClaimAlias::Next,
    };
    advance_workflow(claim_alias, storage, force_bypass)
}

pub fn done_prompt(
    claim_str: Option<&str>,
    force_bypass: Option<&Option<String>>,
    storage: &FileStorage,
) -> Result<String> {
    // Advance the workflow and get the resulting node
    let claim_alias = match claim_str {
        Some(name) => ClaimAlias::Custom(name.to_string()),
        None => ClaimAlias::Next,
    };
    let current_node = advance_workflow(claim_alias, storage, force_bypass)?;

    // Verify we're in the done phase
    if current_node != "done" {
        anyhow::bail!(
            "Expected workflow to reach 'done' phase, but it's at '{}' instead",
            current_node
        );
    }

    Ok(current_node)
}

pub fn reset_workflow(storage: &FileStorage) -> Result<()> {
    // Load current state to preserve session_metadata
    let state = storage.load()?;

    // Clear workflow fields but keep session_metadata and cumulative_totals
    let cleared_state = state.with_workflow(None);

    storage.save(&cleared_state)?;
    println!("{}", Theme::success("Workflow state cleared"));
    Ok(())
}

pub fn abort_workflow(storage: &FileStorage) -> Result<()> {
    // Load current state
    let state = storage.load()?;

    // Check if there's an active workflow
    let workflow_state = match state.workflow.as_ref() {
        Some(ws) => ws,
        None => {
            println!("No active workflow to abort.");
            return Ok(());
        }
    };

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

    let workflow_state = state
        .workflow
        .as_ref()
        .context("No workflow state found. Run 'hegel start <workflow>' first.")?;

    // Load workflow from YAML file based on mode
    let workflow_path = storage.workflow_path(&workflow_state.mode);
    let workflow = crate::engine::load_workflow(&workflow_path)
        .with_context(|| format!("Failed to load workflow: {}", workflow_state.mode))?;

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
    let prompt_text = node.prompt_text();

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

pub fn restart_workflow(storage: &FileStorage) -> Result<String> {
    advance_workflow(ClaimAlias::Restart, storage, None)
}

pub fn prev_prompt(storage: &FileStorage) -> Result<()> {
    use chrono::Utc;

    // Load current state
    let state = storage.load()?;

    let mut workflow_state = state
        .workflow
        .clone()
        .context("No workflow state found. Run 'hegel start <workflow>' first.")?;

    // Load workflow from YAML file based on mode
    let workflow_path = storage.workflow_path(&workflow_state.mode);
    let workflow = crate::engine::load_workflow(&workflow_path)
        .with_context(|| format!("Failed to load workflow: {}", workflow_state.mode))?;

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
    let updated_state = state.with_workflow(Some(workflow_state.clone()));
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
    let prompt_text = node.prompt_text();

    display_workflow_prompt(
        &to_node,
        &workflow_state.mode,
        prompt_text,
        workflow_state.is_handlebars,
        storage,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests;
