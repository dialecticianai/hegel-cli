use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::engine::{get_next_prompt, init_state, load_workflow, render_template};
use crate::storage::{FileStorage, State};

pub fn start_workflow(workflow_name: &str, storage: &FileStorage) -> Result<()> {
    use chrono::Utc;

    // Load workflow from YAML file
    let workflow_path = format!("workflows/{}.yaml", workflow_name);
    let workflow = load_workflow(&workflow_path)
        .with_context(|| format!("Failed to load workflow: {}", workflow_name))?;

    // Initialize workflow state with workflow_id (ISO timestamp)
    let mut workflow_state = init_state(&workflow);
    workflow_state.workflow_id = Some(Utc::now().to_rfc3339());

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

    // Log state transition if a transition occurred
    let transitioned = previous_node != new_state.current_node;
    if transitioned {
        storage.log_state_transition(
            &previous_node,
            &new_state.current_node,
            &new_state.mode,
            new_state.workflow_id.as_deref(),
        )?;
    }

    // Display transition
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

/// Process a hook event JSON string and write to hooks.jsonl with timestamp
fn process_hook_event(hook_json: &str, storage: &FileStorage) -> Result<()> {
    use chrono::Utc;

    // Parse and validate JSON
    let mut hook_value: serde_json::Value =
        serde_json::from_str(hook_json).context("Invalid JSON received")?;

    // Inject timestamp if not present
    if let serde_json::Value::Object(ref mut map) = hook_value {
        if !map.contains_key("timestamp") {
            map.insert(
                "timestamp".to_string(),
                serde_json::Value::String(Utc::now().to_rfc3339()),
            );
        }
    }

    // Serialize back to JSON
    let enriched_json =
        serde_json::to_string(&hook_value).context("Failed to serialize enriched hook event")?;

    // Get hooks.jsonl path using the storage's state dir
    let state_dir = storage.state_dir();
    let hooks_file = state_dir.join("hooks.jsonl");

    // Ensure directory exists (should already exist from storage init, but be safe)
    std::fs::create_dir_all(&state_dir)
        .with_context(|| format!("Failed to create state directory: {:?}", state_dir))?;

    // Append hook JSON to hooks.jsonl
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&hooks_file)
        .with_context(|| format!("Failed to open hooks file: {:?}", hooks_file))?;

    writeln!(file, "{}", enriched_json)
        .with_context(|| format!("Failed to write to hooks file: {:?}", hooks_file))?;

    Ok(())
}

pub fn handle_hook(_event_name: &str, storage: &FileStorage) -> Result<()> {
    // Read JSON from stdin
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut hook_json = String::new();
    stdin_lock
        .read_line(&mut hook_json)
        .context("Failed to read hook JSON from stdin")?;

    // Trim whitespace and process
    process_hook_event(hook_json.trim(), storage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
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
        let guard = WorkingDirGuard::new();
        let temp_dir = TempDir::new().unwrap();

        // Create workflows directory
        let workflows_dir = temp_dir.path().join("workflows");
        fs::create_dir(&workflows_dir).unwrap();

        // Create guides directory
        let guides_dir = temp_dir.path().join("guides");
        fs::create_dir(&guides_dir).unwrap();

        // Create test workflow using shared constant
        fs::write(workflows_dir.join("discovery.yaml"), TEST_WORKFLOW_YAML).unwrap();

        // Create storage
        let storage_dir = temp_dir.path().join("state");
        let storage = FileStorage::new(&storage_dir).unwrap();

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

        // Verify workflow_id is set to ISO timestamp
        assert!(workflow_state.workflow_id.is_some());
        let workflow_id = workflow_state.workflow_id.unwrap();
        // Should be parseable as ISO timestamp
        use chrono::DateTime;
        assert!(DateTime::parse_from_rfc3339(&workflow_id).is_ok());
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

    // ========== handle_hook Tests ==========

    #[test]
    fn test_hook_injects_timestamp() {
        let (temp_dir, storage) = test_storage();

        let hook_json =
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Read"}"#;

        // Process the hook event
        process_hook_event(hook_json, &storage).unwrap();

        // Read and parse the hooks.jsonl file
        let hooks_file = temp_dir.path().join("hooks.jsonl");
        let parsed = read_jsonl_line(&hooks_file, 0);
        assert!(parsed.get("timestamp").is_some());
        assert_eq!(parsed["session_id"], "test");
        assert_eq!(parsed["hook_event_name"], "PostToolUse");
        assert_eq!(parsed["tool_name"], "Read");
    }

    #[test]
    fn test_hook_preserves_existing_timestamp() {
        let (temp_dir, storage) = test_storage();

        let hook_json =
            r#"{"session_id":"test","timestamp":"2025-01-01T00:00:00Z","tool_name":"Edit"}"#;

        process_hook_event(hook_json, &storage).unwrap();

        let hooks_file = temp_dir.path().join("hooks.jsonl");
        let parsed = read_jsonl_line(&hooks_file, 0);
        assert_eq!(parsed["timestamp"], "2025-01-01T00:00:00Z"); // Original timestamp preserved
    }

    #[test]
    fn test_hook_appends_multiple_events() {
        let (temp_dir, storage) = test_storage();

        process_hook_event(r#"{"event":"first"}"#, &storage).unwrap();
        process_hook_event(r#"{"event":"second"}"#, &storage).unwrap();

        let hooks_file = temp_dir.path().join("hooks.jsonl");
        let events = read_jsonl_all(&hooks_file);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["event"], "first");
        assert_eq!(events[1]["event"], "second");
        assert!(events[0].get("timestamp").is_some());
        assert!(events[1].get("timestamp").is_some());
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_full_workflow_cycle() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Complete workflow: start -> spec -> plan -> done (using test workflow)
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

    // ========== State Transition Logging Tests ==========

    #[test]
    fn test_next_prompt_logs_state_transition() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Start workflow
        start_workflow("discovery", &storage).unwrap();

        // Transition from spec to plan
        next_prompt(r#"{"spec_complete": true}"#, &storage).unwrap();

        // Verify transition was logged to states.jsonl
        let states_file = storage.state_dir().join("states.jsonl");
        assert!(states_file.exists());

        let event = read_jsonl_line(&states_file, 0);
        assert_eq!(event["from_node"], "spec");
        assert_eq!(event["to_node"], "plan");
        assert_eq!(event["phase"], "plan");
        assert_eq!(event["mode"], "discovery");
    }

    #[test]
    fn test_next_prompt_logs_multiple_transitions() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Start workflow
        start_workflow("discovery", &storage).unwrap();

        // Make multiple transitions
        next_prompt(r#"{"spec_complete": true}"#, &storage).unwrap();
        next_prompt(r#"{"plan_complete": true}"#, &storage).unwrap();

        // Verify all transitions were logged
        let states_file = storage.state_dir().join("states.jsonl");
        let events = read_jsonl_all(&states_file);
        assert_eq!(events.len(), 2);

        assert_eq!(events[0]["from_node"], "spec");
        assert_eq!(events[0]["to_node"], "plan");

        assert_eq!(events[1]["from_node"], "plan");
        assert_eq!(events[1]["to_node"], "done");
    }

    #[test]
    fn test_next_prompt_no_log_when_no_transition() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Start workflow
        start_workflow("discovery", &storage).unwrap();

        // Try to transition with wrong claim (should stay at spec)
        next_prompt(r#"{"wrong_claim": true}"#, &storage).unwrap();

        // Verify no transition was logged
        let states_file = storage.state_dir().join("states.jsonl");
        assert_eq!(count_jsonl_lines(&states_file), 0);
    }

    #[test]
    fn test_state_transition_includes_workflow_id() {
        let (temp_dir, storage, _workflows, _guides, _guard) = setup_test_env();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Start workflow (which generates workflow_id)
        start_workflow("discovery", &storage).unwrap();

        let state = storage.load().unwrap();
        let workflow_id = state
            .workflow_state
            .as_ref()
            .unwrap()
            .workflow_id
            .as_ref()
            .unwrap();

        // Transition
        next_prompt(r#"{"spec_complete": true}"#, &storage).unwrap();

        // Verify workflow_id is in logged event
        let states_file = storage.state_dir().join("states.jsonl");
        let event = read_jsonl_line(&states_file, 0);
        assert_eq!(event["workflow_id"], workflow_id.as_str());
    }
}
