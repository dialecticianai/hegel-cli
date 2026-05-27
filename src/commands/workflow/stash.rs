//! Stash commands: save, list, restore, and drop workflow state.

use anyhow::{Context, Result};

use crate::engine::load_workflow;
use crate::storage::FileStorage;

use super::context::display_workflow_prompt;

/// Format an optional stash message for display: ` "msg"` when present, else empty.
fn fmt_stash_message(message: &Option<String>) -> String {
    match message {
        Some(m) => format!(r#" "{}""#, m),
        None => String::new(),
    }
}

pub fn stash_workflow(message: Option<String>, storage: &FileStorage) -> Result<()> {
    // Save stash via storage layer
    storage.save_stash(message.clone())?;

    // Display confirmation
    let stashes = storage.list_stashes()?;
    let stash = &stashes[0]; // Newest stash is always at index 0

    let msg_display = fmt_stash_message(&message);

    println!(
        "Saved working directory to stash@{{0}}: {}/{}{}",
        stash.workflow.mode, stash.workflow.current_node, msg_display
    );

    Ok(())
}

/// List all stashes
pub fn list_stashes(storage: &FileStorage) -> Result<()> {
    let stashes = storage.list_stashes()?;

    if stashes.is_empty() {
        println!("No stashes found");
        return Ok(());
    }

    for stash in stashes {
        // Format message
        let msg_display = fmt_stash_message(&stash.message);

        // Format relative time
        use chrono::{DateTime, Utc};
        let timestamp = DateTime::parse_from_rfc3339(&stash.timestamp)
            .with_context(|| format!("Invalid timestamp: {}", stash.timestamp))?;
        let now = Utc::now();
        let duration = now.signed_duration_since(timestamp);

        let time_ago = if duration.num_seconds() < 60 {
            "just now".to_string()
        } else if duration.num_minutes() < 60 {
            format!("{} minutes ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_days() < 7 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_weeks() < 4 {
            format!("{} weeks ago", duration.num_weeks())
        } else {
            format!("{} days ago", duration.num_days())
        };

        println!(
            "stash@{{{}}}: {}/{}{}  ({})",
            stash.index, stash.workflow.mode, stash.workflow.current_node, msg_display, time_ago
        );
    }

    Ok(())
}

/// Pop stash and restore workflow
pub fn pop_stash(index: Option<usize>, storage: &FileStorage) -> Result<()> {
    let index = index.unwrap_or(0);

    // Check for active workflow (ignore if at terminal node)
    let state = storage.load()?;
    if let Some(ref workflow_state) = state.workflow {
        if !crate::engine::is_terminal(&workflow_state.current_node) {
            anyhow::bail!(
                "Cannot restore stash: active workflow at '{}/{}'. Run 'hegel abort' or 'hegel stash' first.",
                workflow_state.mode,
                workflow_state.current_node
            );
        }
    }

    // Load stash
    let stash = storage.load_stash(index)?;

    // Restore state
    let restored = state.with_workflow(Some(stash.workflow.clone()));
    storage.save(&restored)?;

    // Delete stash
    storage.delete_stash(index)?;

    // Display restored workflow prompt
    let msg_display = fmt_stash_message(&stash.message);

    println!(
        "Restored stash@{{{}}}: {}/{}{}",
        index, stash.workflow.mode, stash.workflow.current_node, msg_display
    );
    println!();

    // Get workflow to display prompt
    let workflow_state = restored.workflow.context("No workflow in restored state")?;
    let workflow_path = storage.workflow_path(&workflow_state.mode);
    let workflow = load_workflow(&workflow_path)
        .with_context(|| format!("Failed to load workflow: {}", workflow_state.mode))?;

    let node = workflow
        .nodes
        .get(&stash.workflow.current_node)
        .with_context(|| format!("Node not found: {}", stash.workflow.current_node))?;

    let prompt_text = node.prompt_text();

    display_workflow_prompt(
        &stash.workflow.current_node,
        &stash.workflow.mode,
        prompt_text,
        stash.workflow.is_handlebars,
        storage,
    )?;

    Ok(())
}

/// Drop stash without restoring
pub fn drop_stash(index: Option<usize>, storage: &FileStorage) -> Result<()> {
    let index = index.unwrap_or(0);

    // Load stash to show what's being dropped
    let stash = storage.load_stash(index)?;

    let msg_display = fmt_stash_message(&stash.message);

    // Delete stash (skip confirmation in non-interactive mode)
    storage.delete_stash(index)?;

    println!(
        "Dropped stash@{{{}}}: {}/{}{}",
        index, stash.workflow.mode, stash.workflow.current_node, msg_display
    );

    Ok(())
}
