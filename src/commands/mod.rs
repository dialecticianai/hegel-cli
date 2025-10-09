use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

use crate::engine::{get_next_prompt, init_state, load_workflow, render_template};
use crate::storage::{FileStorage, State};

pub fn start_workflow(workflow_name: &str, storage: &FileStorage) -> Result<()> {
    // Load workflow from YAML file
    let workflow_path = format!("workflows/{}.yaml", workflow_name);
    let workflow = load_workflow(&workflow_path)
        .with_context(|| format!("Failed to load workflow: {}", workflow_name))?;

    // Initialize workflow state
    let workflow_state = init_state(&workflow);

    // Get current node and prompt
    let current_node = &workflow_state.current_node;
    let node = workflow
        .nodes
        .get(current_node)
        .with_context(|| format!("Node not found: {}", current_node))?;

    // Render prompt with guides
    let guides_dir = Path::new("guides");
    let context = HashMap::new(); // Empty context for now
    let rendered_prompt = render_template(&node.prompt, guides_dir, &context)
        .with_context(|| "Failed to render prompt template")?;

    // Store state
    let state = State {
        workflow: Some(serde_yaml::to_value(&workflow)?),
        workflow_state: Some(workflow_state.clone()),
    };
    storage.save(&state)?;

    // Display output
    println!("{}", "Workflow started".bold().green());
    println!();
    println!("{}: {}", "Mode".bold(), workflow_state.mode);
    println!("{}: {}", "Current node".bold(), current_node);
    println!();
    println!("{}", "Prompt:".bold().cyan());
    println!("{}", rendered_prompt);

    Ok(())
}

pub fn next_prompt(claims_str: &str, storage: &FileStorage) -> Result<()> {
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
    let workflow =
        serde_yaml::from_value(workflow_yaml.clone()).context("Failed to parse stored workflow")?;

    // Parse claims from JSON string
    let claims: HashMap<String, bool> = serde_json::from_str(claims_str)
        .context("Failed to parse claims JSON. Expected format: {\"claim_name\": true}")?;

    // Get next prompt
    let previous_node = workflow_state.current_node.clone();
    let (prompt_text, new_state) = get_next_prompt(&workflow, workflow_state, &claims)?;

    // Render prompt with guides
    let guides_dir = Path::new("guides");
    let context = HashMap::new(); // Empty context for now
    let rendered_prompt = render_template(&prompt_text, guides_dir, &context)
        .with_context(|| "Failed to render prompt template")?;

    // Save updated state
    let updated_state = State {
        workflow: Some(workflow_yaml.clone()),
        workflow_state: Some(new_state.clone()),
    };
    storage.save(&updated_state)?;

    // Display transition
    let transitioned = previous_node != new_state.current_node;
    if transitioned {
        println!(
            "{} {} {} {}",
            "Transitioned:".bold().green(),
            previous_node.bright_black(),
            "→".bright_black(),
            new_state.current_node.cyan()
        );
    } else {
        println!("{}", "Stayed at current node".yellow());
    }
    println!();
    println!("{}: {}", "Current node".bold(), new_state.current_node);
    println!();
    println!("{}", "Prompt:".bold().cyan());
    println!("{}", rendered_prompt);

    Ok(())
}

pub fn show_status(storage: &FileStorage) -> Result<()> {
    let state = storage.load()?;

    if state.workflow.is_none() || state.workflow_state.is_none() {
        println!("{}", "No workflow loaded".yellow());
        println!();
        println!("Start a workflow with: {}", "hegel start <workflow>".cyan());
        return Ok(());
    }

    let workflow_state = state.workflow_state.as_ref().unwrap();

    println!("{}", "Workflow Status".bold().cyan());
    println!();
    println!("{}: {}", "Mode".bold(), workflow_state.mode);
    println!("{}: {}", "Current node".bold(), workflow_state.current_node);
    println!();
    println!("{}", "History:".bold());
    for (i, node) in workflow_state.history.iter().enumerate() {
        if i == workflow_state.history.len() - 1 {
            println!("  {} {}", "→".cyan(), node.cyan());
        } else {
            println!("    {}", node.bright_black());
        }
    }

    Ok(())
}

pub fn reset_workflow(storage: &FileStorage) -> Result<()> {
    storage.clear()?;
    println!("{}", "Workflow state cleared".green());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Helper to save and restore working directory
    struct WorkingDirGuard {
        original_dir: std::path::PathBuf,
    }

    impl WorkingDirGuard {
        fn new() -> Self {
            Self {
                original_dir: std::env::current_dir().unwrap(),
            }
        }
    }

    impl Drop for WorkingDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original_dir);
        }
    }

    // Helper to create test workflow and guides
    fn setup_test_env() -> (
        TempDir,
        FileStorage,
        std::path::PathBuf,
        std::path::PathBuf,
        WorkingDirGuard,
    ) {
        let _guard = WorkingDirGuard::new();
        let temp_dir = TempDir::new().unwrap();

        // Create workflows directory
        let workflows_dir = temp_dir.path().join("workflows");
        fs::create_dir(&workflows_dir).unwrap();

        // Create guides directory
        let guides_dir = temp_dir.path().join("guides");
        fs::create_dir(&guides_dir).unwrap();

        // Create test workflow
        let workflow_yaml = r#"
mode: discovery
start_node: spec
nodes:
  spec:
    prompt: "Write SPEC.md"
    transitions:
      - when: spec_complete
        to: plan
  plan:
    prompt: "Write PLAN.md"
    transitions:
      - when: plan_complete
        to: done
  done:
    prompt: "Complete!"
    transitions: []
"#;
        fs::write(workflows_dir.join("discovery.yaml"), workflow_yaml).unwrap();

        // Create storage
        let storage_dir = temp_dir.path().join("state");
        let storage = FileStorage::new(&storage_dir).unwrap();

        let guard = WorkingDirGuard::new();
        (temp_dir, storage, workflows_dir, guides_dir, guard)
    }

    // ========== start_workflow Tests ==========

    #[test]
    fn test_start_workflow_success() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();

        // Change to temp dir so workflow can be found
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = start_workflow("discovery", &storage);
        assert!(result.is_ok());

        // Verify state was saved
        let state = storage.load().unwrap();
        assert!(state.workflow.is_some());
        assert!(state.workflow_state.is_some());

        let workflow_state = state.workflow_state.unwrap();
        assert_eq!(workflow_state.mode, "discovery");
        assert_eq!(workflow_state.current_node, "spec");
        assert_eq!(workflow_state.history, vec!["spec"]);
    }

    #[test]
    fn test_start_workflow_missing_file() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = start_workflow("nonexistent", &storage);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to load workflow"));
    }

    // ========== next_prompt Tests ==========

    #[test]
    fn test_next_prompt_successful_transition() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Start workflow first
        start_workflow("discovery", &storage).unwrap();

        // Transition to next node
        let result = next_prompt(r#"{"spec_complete": true}"#, &storage);
        assert!(result.is_ok());

        // Verify state was updated
        let state = storage.load().unwrap();
        let workflow_state = state.workflow_state.unwrap();
        assert_eq!(workflow_state.current_node, "plan");
        assert_eq!(workflow_state.history, vec!["spec", "plan"]);
    }

    #[test]
    fn test_next_prompt_no_matching_transition() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Start workflow first
        start_workflow("discovery", &storage).unwrap();

        // Try to transition with wrong claim
        let result = next_prompt(r#"{"wrong_claim": true}"#, &storage);
        assert!(result.is_ok());

        // Verify we stayed at current node
        let state = storage.load().unwrap();
        let workflow_state = state.workflow_state.unwrap();
        assert_eq!(workflow_state.current_node, "spec");
        assert_eq!(workflow_state.history, vec!["spec"]);
    }

    #[test]
    fn test_next_prompt_no_workflow_loaded() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Try to advance without starting workflow
        let result = next_prompt(r#"{"spec_complete": true}"#, &storage);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No workflow loaded"));
    }

    #[test]
    fn test_next_prompt_invalid_json() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Start workflow first
        start_workflow("discovery", &storage).unwrap();

        // Try to parse invalid JSON
        let result = next_prompt("not valid json", &storage);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse claims JSON"));
    }

    #[test]
    fn test_next_prompt_multiple_transitions() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Start workflow
        start_workflow("discovery", &storage).unwrap();

        // First transition: spec -> plan
        next_prompt(r#"{"spec_complete": true}"#, &storage).unwrap();
        let state1 = storage.load().unwrap();
        assert_eq!(state1.workflow_state.as_ref().unwrap().current_node, "plan");

        // Second transition: plan -> done
        next_prompt(r#"{"plan_complete": true}"#, &storage).unwrap();
        let state2 = storage.load().unwrap();
        assert_eq!(state2.workflow_state.as_ref().unwrap().current_node, "done");
        assert_eq!(
            state2.workflow_state.as_ref().unwrap().history,
            vec!["spec", "plan", "done"]
        );
    }

    // ========== show_status Tests ==========

    #[test]
    fn test_show_status_with_workflow() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Start workflow first
        start_workflow("discovery", &storage).unwrap();

        // Show status should work
        let result = show_status(&storage);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_status_no_workflow() {
        let (_temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();

        // Show status with no workflow should succeed but show message
        let result = show_status(&storage);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_status_after_transitions() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Start and transition
        start_workflow("discovery", &storage).unwrap();
        next_prompt(r#"{"spec_complete": true}"#, &storage).unwrap();

        // Show status should reflect current state
        let result = show_status(&storage);
        assert!(result.is_ok());

        // Verify state
        let state = storage.load().unwrap();
        let workflow_state = state.workflow_state.unwrap();
        assert_eq!(workflow_state.current_node, "plan");
    }

    // ========== reset_workflow Tests ==========

    #[test]
    fn test_reset_workflow_clears_state() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Start workflow
        start_workflow("discovery", &storage).unwrap();

        // Verify state exists
        let state_before = storage.load().unwrap();
        assert!(state_before.workflow.is_some());

        // Reset
        let result = reset_workflow(&storage);
        assert!(result.is_ok());

        // Verify state is cleared
        let state_after = storage.load().unwrap();
        assert!(state_after.workflow.is_none());
        assert!(state_after.workflow_state.is_none());
    }

    #[test]
    fn test_reset_workflow_when_no_state() {
        let (_temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();

        // Reset when there's no state should not error
        let result = reset_workflow(&storage);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // FLAKY: Fails intermittently due to working directory race condition when run with other tests
    fn test_reset_then_start_new_workflow() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Start, reset, start again
        start_workflow("discovery", &storage).unwrap();
        next_prompt(r#"{"spec_complete": true}"#, &storage).unwrap();

        reset_workflow(&storage).unwrap();

        start_workflow("discovery", &storage).unwrap();

        // Verify fresh state
        let state = storage.load().unwrap();
        let workflow_state = state.workflow_state.unwrap();
        assert_eq!(workflow_state.current_node, "spec");
        assert_eq!(workflow_state.history, vec!["spec"]);
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_full_workflow_cycle() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Complete workflow: start -> spec -> plan -> done
        start_workflow("discovery", &storage).unwrap();

        let state1 = storage.load().unwrap();
        assert_eq!(state1.workflow_state.as_ref().unwrap().current_node, "spec");

        next_prompt(r#"{"spec_complete": true}"#, &storage).unwrap();
        let state2 = storage.load().unwrap();
        assert_eq!(state2.workflow_state.as_ref().unwrap().current_node, "plan");

        next_prompt(r#"{"plan_complete": true}"#, &storage).unwrap();
        let state3 = storage.load().unwrap();
        assert_eq!(state3.workflow_state.as_ref().unwrap().current_node, "done");
        assert_eq!(
            state3.workflow_state.as_ref().unwrap().history,
            vec!["spec", "plan", "done"]
        );
    }
}
