use anyhow::{Context, Result};
use std::fs;

use crate::storage::FileStorage;

/// Attempt to rescue a corrupted state.json file
///
/// This handles the case where state.json is in an old format that can't be
/// deserialized by the current State struct. Common issues:
/// - Old `workflow_state` field instead of `workflow`
/// - Missing required fields in WorkflowState
///
/// Returns Ok(true) if rescue was successful, Ok(false) if no rescue was needed
pub fn rescue_state_file(storage: &FileStorage) -> Result<bool> {
    let state_path = storage.state_dir().join("state.json");

    if !state_path.exists() {
        return Ok(false);
    }

    // Try to load as raw JSON
    let content = fs::read_to_string(&state_path)
        .with_context(|| format!("Failed to read state file: {:?}", state_path))?;

    let mut json: serde_json::Value =
        serde_json::from_str(&content).context("State file is not valid JSON")?;

    let mut needs_rescue = false;

    // Check if state has old `workflow_state` field instead of `workflow`
    if let Some(obj) = json.as_object_mut() {
        if obj.contains_key("workflow_state") && !obj.contains_key("workflow") {
            // Migrate: workflow_state → workflow
            if let Some(workflow_state) = obj.remove("workflow_state") {
                obj.insert("workflow".to_string(), workflow_state);
                needs_rescue = true;
            }
        }

        // Remove old `workflow` field if it's a YAML definition (object with "nodes", "transitions", etc)
        // vs WorkflowState (object with "current_node", "mode", "history")
        if let Some(workflow) = obj.get("workflow") {
            if let Some(workflow_obj) = workflow.as_object() {
                // If workflow has "nodes" or "transitions", it's a definition (BAD)
                // If it has "current_node", it's a WorkflowState (GOOD)
                if workflow_obj.contains_key("nodes") || workflow_obj.contains_key("transitions") {
                    // This is a workflow definition, not state - remove it
                    obj.remove("workflow");
                    needs_rescue = true;
                }
            }
        }
    }

    if needs_rescue {
        // Write the rescued state back
        let rescued_content =
            serde_json::to_string_pretty(&json).context("Failed to serialize rescued state")?;

        fs::write(&state_path, rescued_content)
            .with_context(|| format!("Failed to write rescued state file: {:?}", state_path))?;

        Ok(true)
    } else {
        Ok(false)
    }
}
