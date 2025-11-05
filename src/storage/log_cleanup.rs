//! Log cleanup utilities for managing hooks.jsonl and states.jsonl
//!
//! Deletes log files after archiving workflows.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Delete hooks.jsonl and states.jsonl log files
pub fn cleanup_logs<P: AsRef<Path>>(state_dir: P) -> Result<()> {
    let state_dir = state_dir.as_ref();
    let hooks_path = state_dir.join("hooks.jsonl");
    let states_path = state_dir.join("states.jsonl");

    if hooks_path.exists() {
        fs::remove_file(&hooks_path)
            .with_context(|| format!("Failed to delete hooks.jsonl: {:?}", hooks_path))?;
    }

    if states_path.exists() {
        fs::remove_file(&states_path)
            .with_context(|| format!("Failed to delete states.jsonl: {:?}", states_path))?;
    }

    Ok(())
}
