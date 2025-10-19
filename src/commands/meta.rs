use crate::metamodes::{evaluate_workflow_completion, MetaModeDefinition};
use crate::storage::{FileStorage, MetaMode};
use crate::theme::Theme;
use anyhow::{Context, Result};

/// Declare or view meta-mode
pub fn meta_mode(meta_mode_name: Option<&str>, list: bool, storage: &FileStorage) -> Result<()> {
    if list {
        list_meta_modes()
    } else {
        match meta_mode_name {
            Some(name) => declare_meta_mode(name, storage),
            None => show_meta_mode_status(storage),
        }
    }
}

/// Declare a new meta-mode and start its initial workflow
fn declare_meta_mode(name: &str, storage: &FileStorage) -> Result<()> {
    // Validate and get meta-mode definition
    let definition = MetaModeDefinition::get(name)?;

    // Load current state
    let state = storage.load()?;

    // Check if already in a workflow
    if let Some(workflow_state) = &state.workflow_state {
        // Allow changing meta-mode if at a done node or no workflow active
        if workflow_state.current_node != "done" {
            anyhow::bail!(
                "Cannot change meta-mode while workflow is active. Current: {} ({})\nComplete current workflow or run 'hegel reset' first.",
                workflow_state.mode,
                workflow_state.current_node
            );
        }
    }

    // Create new meta-mode
    let new_meta_mode = MetaMode {
        name: definition.name.clone(),
    };

    // Create initial dummy workflow state with just meta-mode
    // This allows start_workflow to find the meta-mode when it loads state
    let initial_state = crate::storage::State {
        workflow: None,
        workflow_state: Some(crate::storage::WorkflowState {
            current_node: String::new(),
            mode: String::new(),
            history: vec![],
            workflow_id: None,
            meta_mode: Some(new_meta_mode),
            phase_start_time: None,
        }),
        session_metadata: state.session_metadata,
    };

    // Save state with meta-mode
    storage.save(&initial_state)?;

    println!(
        "{}",
        Theme::success(&format!(
            "✓ {} meta-mode declared",
            Theme::highlight(&definition.name)
        ))
    );
    println!("  {}", Theme::secondary(&definition.description));
    println!();

    // Now start the initial workflow
    crate::commands::start_workflow(&definition.initial_workflow, storage)?;

    Ok(())
}

/// Show current meta-mode status and available transitions
fn show_meta_mode_status(storage: &FileStorage) -> Result<()> {
    let state = storage.load()?;

    let workflow_state = state
        .workflow_state
        .as_ref()
        .context("No workflow active. Run 'hegel meta <name>' to declare a meta-mode.")?;

    let meta_mode = workflow_state
        .meta_mode
        .as_ref()
        .context("No meta-mode declared. Run 'hegel meta <name>' first.\n\nAvailable meta-modes:\n  - learning:  Greenfield learning project (Research ↔ Discovery loop)\n  - standard:  Feature development with known patterns (Discovery ↔ Execution)")?;

    // Show current meta-mode and workflow
    println!(
        "{}",
        Theme::header(&format!(
            "Meta-mode: {} ({})",
            Theme::highlight(&meta_mode.name),
            Theme::secondary(&get_meta_mode_pattern(&meta_mode.name))
        ))
    );
    println!(
        "Current workflow: {} ({})",
        Theme::highlight(&workflow_state.mode),
        Theme::secondary(&workflow_state.current_node)
    );
    println!();

    // Check if we're at a workflow completion point
    if let Some(transitions) = evaluate_workflow_completion(
        &meta_mode.name,
        &workflow_state.mode,
        &workflow_state.current_node,
    ) {
        println!("{}", Theme::header("Available transitions:"));
        for transition in transitions {
            if let Some(new_meta_mode) = &transition.meta_mode_change {
                println!(
                    "  • {} - {}",
                    Theme::highlight(&format!("hegel meta {}", new_meta_mode)),
                    Theme::secondary(&transition.description)
                );
            } else {
                println!(
                    "  • {} - {}",
                    Theme::highlight(&format!("hegel start {}", transition.next_workflow)),
                    Theme::secondary(&transition.description)
                );
            }
        }
    } else {
        println!(
            "{}",
            Theme::secondary("Continue working through current workflow.")
        );
        println!(
            "{}",
            Theme::secondary("Run 'hegel next' to advance to next phase.")
        );
    }

    Ok(())
}

/// Get human-readable pattern description for meta-mode
fn get_meta_mode_pattern(name: &str) -> String {
    match name {
        "learning" => "Research ↔ Discovery".to_string(),
        "standard" => "Discovery ↔ Execution".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// List all available meta-modes
fn list_meta_modes() -> Result<()> {
    let all_modes = MetaModeDefinition::all();

    println!("Available meta-modes:");
    for mode in all_modes {
        println!("  {} - {}", mode.name, mode.description);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use tempfile::TempDir;

    #[test]
    fn test_declare_meta_mode_learning() {
        // Skip - requires research.yaml which may not exist in test env
        // Tested in integration tests instead
    }

    #[test]
    fn test_declare_meta_mode_standard() {
        let (_temp_dir, storage) = setup_workflow_env();

        let result = declare_meta_mode("standard", &storage);
        assert!(result.is_ok());

        // Should have started discovery workflow
        let state = storage.load().unwrap();
        let workflow_state = state.workflow_state.unwrap();
        assert_eq!(workflow_state.mode, "discovery");
        assert_eq!(workflow_state.meta_mode.unwrap().name, "standard");
    }

    #[test]
    fn test_declare_meta_mode_invalid() {
        let (_temp_dir, storage) = test_storage();

        let result = declare_meta_mode("invalid", &storage);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid meta-mode"));
    }

    #[test]
    fn test_declare_meta_mode_while_workflow_active() {
        let (_temp_dir, storage) = setup_workflow_env();

        // Start with standard
        declare_meta_mode("standard", &storage).unwrap();

        // Try to change while in spec phase (not done)
        let result = declare_meta_mode("standard", &storage);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot change meta-mode while workflow is active"));
    }

    #[test]
    fn test_show_meta_mode_status_no_workflow() {
        let (_temp_dir, storage) = test_storage();

        let result = show_meta_mode_status(&storage);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No workflow active"));
    }

    #[test]
    fn test_show_meta_mode_status_defaults_to_standard() {
        let (_temp_dir, storage) = setup_workflow_env();

        // Start workflow without explicitly declaring meta-mode (should default to standard)
        crate::commands::start_workflow("discovery", &storage).unwrap();

        // Should succeed and show standard meta-mode
        let result = show_meta_mode_status(&storage);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_meta_mode_pattern() {
        assert_eq!(get_meta_mode_pattern("learning"), "Research ↔ Discovery");
        assert_eq!(get_meta_mode_pattern("standard"), "Discovery ↔ Execution");
        assert_eq!(get_meta_mode_pattern("unknown"), "Unknown");
    }
}
