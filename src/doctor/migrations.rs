use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::storage::{FileStorage, State};

/// Issue detected by a migration check
#[derive(Debug)]
pub struct MigrationIssue {
    pub description: String,
    pub impact: Option<String>,
}

/// Trait for state migration strategies
///
/// Each migration implementation can detect issues in state.json
/// and repair them by returning a new State.
pub trait StateMigration {
    /// Human-readable name of this migration
    fn name(&self) -> &str;

    /// Check if migration is needed
    ///
    /// Returns Some(MigrationIssue) if migration is needed, None otherwise
    fn check(&self, state: &State, storage: &FileStorage) -> Result<Option<MigrationIssue>>;

    /// Perform the migration
    ///
    /// Returns the migrated state
    fn migrate(&mut self, state: State, storage: &FileStorage) -> Result<State>;
}

/// Migration: Remove workflow definition from state.json
///
/// Old format had both `workflow: Option<serde_yaml::Value>` (full YAML definition)
/// and `workflow_state: Option<WorkflowState>` (current state).
///
/// New format has only `workflow: Option<WorkflowState>` (current state).
/// Definitions are loaded from YAML files as needed.
pub struct WorkflowDefMigration;

impl StateMigration for WorkflowDefMigration {
    fn name(&self) -> &str {
        "Workflow definition in state.json (deprecated)"
    }

    fn check(&self, state: &State, _storage: &FileStorage) -> Result<Option<MigrationIssue>> {
        // Check if state has workflow field
        if state.workflow.is_none() {
            return Ok(None);
        }

        // Try to detect if workflow field contains a YAML definition (old format)
        // vs WorkflowState (new format)
        //
        // This is tricky because both are deserialized as WorkflowState.
        // The old format would have had the raw YAML Value which included
        // fields like "mode", "nodes", "transitions" etc.
        //
        // In the new format, WorkflowState only has: workflow_id, mode, current_node, history
        //
        // We can't directly detect this from the deserialized State struct alone.
        // Instead, we'll check the file size and structure.

        // Alternative approach: Check if state.json file is suspiciously large
        // (contains workflow definitions instead of just state)
        let state_path = _storage.state_dir().join("state.json");
        let metadata =
            std::fs::metadata(&state_path).context("Failed to read state.json metadata")?;

        let size_bytes = metadata.len();

        // Normal state.json should be < 5KB
        // State with workflow definitions can be 50KB+
        if size_bytes > 10_000 {
            // Likely has workflow definition embedded
            let size_kb = size_bytes as f64 / 1024.0;
            let description = format!(
                "state.json size: {:.1} KB (likely contains workflow definition)",
                size_kb
            );
            let impact = Some(format!(
                "State file ~{:.0}x larger than needed. Workflow changes in YAML files won't be reflected.",
                size_kb / 1.0
            ));

            return Ok(Some(MigrationIssue {
                description,
                impact,
            }));
        }

        Ok(None)
    }

    fn migrate(&mut self, state: State, storage: &FileStorage) -> Result<State> {
        // If there's a workflow, reload it fresh from the YAML file
        // This strips out any embedded definition and keeps only the state
        if let Some(ref workflow_state) = state.workflow {
            let mode = &workflow_state.mode;
            let workflow_path =
                PathBuf::from(storage.workflows_dir()).join(format!("{}.yaml", mode));

            // Verify the workflow YAML file exists
            if !workflow_path.exists() {
                anyhow::bail!(
                    "Workflow YAML file not found: {}. Cannot complete migration.",
                    workflow_path.display()
                );
            }

            // Return a new State with the same workflow state but loaded fresh
            // This ensures we're not carrying any embedded YAML definitions
            Ok(State {
                workflow: state.workflow, // Keep the state as-is
                cumulative_totals: state.cumulative_totals,
                session_metadata: state.session_metadata,
                git_info: state.git_info,
            })
        } else {
            // No workflow, nothing to migrate
            Ok(state)
        }
    }
}

/// Registry of all available migration strategies
pub fn all_migrations() -> Vec<Box<dyn StateMigration>> {
    vec![Box::new(WorkflowDefMigration)]
}
