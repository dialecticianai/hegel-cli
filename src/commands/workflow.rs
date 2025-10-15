use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;

use crate::engine::{get_next_prompt, init_state, load_workflow, render_template};
use crate::storage::{FileStorage, State};

/// Claim aliases for ergonomic workflow transitions
#[derive(Debug, Clone, PartialEq)]
pub enum ClaimAlias {
    /// Happy-path: {"{current_node}_complete": true}
    Next,
    /// Repeat current phase: {"{current_node}_complete": false}
    Repeat,
    /// Restart workflow cycle: {"restart_cycle": true}
    Restart,
    /// Custom claim JSON
    Custom(String),
}

impl ClaimAlias {
    /// Convert claim alias to HashMap for engine consumption
    pub fn to_claims(&self, current_node: &str) -> Result<HashMap<String, bool>> {
        match self {
            Self::Next => Ok(HashMap::from([(
                format!("{}_complete", current_node),
                true,
            )])),
            Self::Repeat => Ok(HashMap::from([(
                format!("{}_complete", current_node),
                false,
            )])),
            Self::Restart => Ok(HashMap::from([("restart_cycle".to_string(), true)])),
            Self::Custom(json) => serde_json::from_str(json)
                .context("Failed to parse claims JSON. Expected format: {\"claim_name\": true}"),
        }
    }
}

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
        session_metadata: None,
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

/// Core workflow advancement logic used by next, repeat, and restart
fn advance_workflow(claim_alias: ClaimAlias, storage: &FileStorage) -> Result<()> {
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

    // Convert alias to claims
    let claims = claim_alias.to_claims(&workflow_state.current_node)?;

    // Get next prompt (pass state_dir for rule evaluation)
    let previous_node = workflow_state.current_node.clone();
    let (prompt_text, new_state) =
        get_next_prompt(&workflow, workflow_state, &claims, storage.state_dir())?;

    // Render prompt with guides
    let guides_dir = Path::new("guides");
    let context = HashMap::new(); // Empty context for now
    let rendered_prompt = render_template(&prompt_text, guides_dir, &context)
        .with_context(|| "Failed to render prompt template")?;

    // Save updated state (preserve session_metadata from loaded state)
    let updated_state = State {
        workflow: Some(workflow_yaml.clone()),
        workflow_state: Some(new_state.clone()),
        session_metadata: state.session_metadata.clone(),
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
    // Load current state to preserve session_metadata
    let state = storage.load()?;

    // Clear workflow fields but keep session_metadata
    let cleared_state = State {
        workflow: None,
        workflow_state: None,
        session_metadata: state.session_metadata,
    };

    storage.save(&cleared_state)?;
    println!("{}", "Workflow state cleared".green());
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
    let guides_dir = Path::new("guides");
    let context = HashMap::new(); // Empty context for now
    let rendered_prompt = render_template(&node.prompt, guides_dir, &context)
        .with_context(|| "Failed to render prompt template")?;

    // Display output
    println!("{}", "Re-displaying current prompt".yellow());
    println!("{}: {}", "Current node".bold(), current_node);
    println!();
    println!("{}", "Prompt:".bold().cyan());
    println!("{}", rendered_prompt);

    Ok(())
}

pub fn restart_workflow(storage: &FileStorage) -> Result<()> {
    advance_workflow(ClaimAlias::Restart, storage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    // ========== start_workflow Tests ==========

    #[test]
    fn test_start_workflow_success() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        let state = storage.load().unwrap();
        assert!(state.workflow.is_some());
        assert_state_eq(&state, "spec", "discovery", &["spec"]);
        // Verify workflow_id is set and parseable
        let wf_id = state.workflow_state.unwrap().workflow_id.unwrap();
        use chrono::DateTime;
        assert!(DateTime::parse_from_rfc3339(&wf_id).is_ok());
    }

    #[test]
    fn test_start_workflow_missing_file() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        let result = start_workflow("nonexistent", &storage);
        assert!(
            result.is_err()
                && result
                    .unwrap_err()
                    .to_string()
                    .contains("Failed to load workflow")
        );
    }

    // ========== next_prompt Tests ==========

    #[test]
    fn test_next_prompt_successful_transition() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        next_prompt(Some(r#"{"spec_complete": true}"#), &storage).unwrap();
        assert_state_eq(
            &storage.load().unwrap(),
            "plan",
            "discovery",
            &["spec", "plan"],
        );
    }

    #[test]
    fn test_next_prompt_no_matching_transition() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        next_prompt(Some(r#"{"wrong_claim": true}"#), &storage).unwrap();
        assert_state_eq(&storage.load().unwrap(), "spec", "discovery", &["spec"]);
    }

    #[test]
    fn test_next_prompt_no_workflow_loaded() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        let result = next_prompt(Some(r#"{"spec_complete": true}"#), &storage);
        assert!(
            result.is_err()
                && result
                    .unwrap_err()
                    .to_string()
                    .contains("No workflow loaded")
        );
    }

    #[test]
    fn test_next_prompt_invalid_json() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        let result = next_prompt(Some("not valid json"), &storage);
        assert!(
            result.is_err()
                && result
                    .unwrap_err()
                    .to_string()
                    .contains("Failed to parse claims JSON")
        );
    }

    #[test]
    fn test_next_prompt_multiple_transitions() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        next_prompt(Some(r#"{"spec_complete": true}"#), &storage).unwrap();
        assert_state_eq(
            &storage.load().unwrap(),
            "plan",
            "discovery",
            &["spec", "plan"],
        );
        next_prompt(Some(r#"{"plan_complete": true}"#), &storage).unwrap();
        assert_state_eq(
            &storage.load().unwrap(),
            "done",
            "discovery",
            &["spec", "plan", "done"],
        );
    }

    // ========== show_status Tests ==========

    #[test]
    fn test_show_status_with_workflow() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        assert!(show_status(&storage).is_ok());
    }

    #[test]
    fn test_show_status_no_workflow() {
        let (_temp_dir, storage) = test_storage();
        assert!(show_status(&storage).is_ok());
    }

    #[test]
    fn test_show_status_after_transitions() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        next_prompt(Some(r#"{"spec_complete": true}"#), &storage).unwrap();
        assert!(show_status(&storage).is_ok());
        assert_state_eq(
            &storage.load().unwrap(),
            "plan",
            "discovery",
            &["spec", "plan"],
        );
    }

    // ========== reset_workflow Tests ==========

    #[test]
    fn test_reset_workflow_clears_state() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        assert!(storage.load().unwrap().workflow.is_some());
        reset_workflow(&storage).unwrap();
        let state = storage.load().unwrap();
        assert!(state.workflow.is_none() && state.workflow_state.is_none());
    }

    #[test]
    fn test_reset_workflow_when_no_state() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        assert!(reset_workflow(&storage).is_ok());
    }

    #[test]
    fn test_reset_workflow_preserves_session_metadata() {
        use crate::storage::SessionMetadata;

        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();

        // Add session metadata manually
        let mut state = storage.load().unwrap();
        state.session_metadata = Some(SessionMetadata {
            session_id: "test-session".to_string(),
            transcript_path: "/tmp/transcript.jsonl".to_string(),
            started_at: "2025-01-01T10:00:00Z".to_string(),
        });
        storage.save(&state).unwrap();

        // Reset workflow
        reset_workflow(&storage).unwrap();

        // Verify workflow cleared but session preserved
        let state = storage.load().unwrap();
        assert!(state.workflow.is_none());
        assert!(state.workflow_state.is_none());
        assert!(state.session_metadata.is_some());
        assert_eq!(state.session_metadata.unwrap().session_id, "test-session");
    }

    #[test]
    #[ignore] // FLAKY: Fails intermittently due to working directory race condition when run with other tests
    fn test_reset_then_start_new_workflow() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        next_prompt(Some(r#"{"spec_complete": true}"#), &storage).unwrap();
        reset_workflow(&storage).unwrap();
        start_workflow("discovery", &storage).unwrap();
        assert_state_eq(&storage.load().unwrap(), "spec", "discovery", &["spec"]);
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_full_workflow_cycle() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        assert_state_eq(&storage.load().unwrap(), "spec", "discovery", &["spec"]);
        next_prompt(Some(r#"{"spec_complete": true}"#), &storage).unwrap();
        assert_state_eq(
            &storage.load().unwrap(),
            "plan",
            "discovery",
            &["spec", "plan"],
        );
        next_prompt(Some(r#"{"plan_complete": true}"#), &storage).unwrap();
        assert_state_eq(
            &storage.load().unwrap(),
            "done",
            "discovery",
            &["spec", "plan", "done"],
        );
    }

    // ========== State Transition Logging Tests ==========

    #[test]
    fn test_next_prompt_logs_state_transition() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        next_prompt(Some(r#"{"spec_complete": true}"#), &storage).unwrap();
        let event = read_jsonl_line(&storage.state_dir().join("states.jsonl"), 0);
        assert_eq!(event["from_node"], "spec");
        assert_eq!(event["to_node"], "plan");
        assert_eq!(event["phase"], "plan");
        assert_eq!(event["mode"], "discovery");
    }

    #[test]
    fn test_next_prompt_logs_multiple_transitions() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        next_prompt(Some(r#"{"spec_complete": true}"#), &storage).unwrap();
        next_prompt(Some(r#"{"plan_complete": true}"#), &storage).unwrap();
        let events = read_jsonl_all(&storage.state_dir().join("states.jsonl"));
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["from_node"], "spec");
        assert_eq!(events[0]["to_node"], "plan");
        assert_eq!(events[1]["from_node"], "plan");
        assert_eq!(events[1]["to_node"], "done");
    }

    #[test]
    fn test_next_prompt_no_log_when_no_transition() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        next_prompt(Some(r#"{"wrong_claim": true}"#), &storage).unwrap();
        assert_eq!(
            count_jsonl_lines(&storage.state_dir().join("states.jsonl")),
            0
        );
    }

    #[test]
    fn test_state_transition_includes_workflow_id() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        let workflow_id = storage
            .load()
            .unwrap()
            .workflow_state
            .as_ref()
            .unwrap()
            .workflow_id
            .as_ref()
            .unwrap()
            .clone();
        next_prompt(Some(r#"{"spec_complete": true}"#), &storage).unwrap();
        let event = read_jsonl_line(&storage.state_dir().join("states.jsonl"), 0);
        assert_eq!(event["workflow_id"], workflow_id.as_str());
    }

    // ========== repeat_prompt Tests ==========

    #[test]
    fn test_continue_with_active_workflow_returns_current_node_prompt() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        assert!(repeat_prompt(&storage).is_ok());
    }

    #[test]
    fn test_continue_with_no_workflow_loaded_returns_error() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        let result = repeat_prompt(&storage);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No workflow loaded"));
    }

    #[test]
    fn test_continue_renders_template_with_guides() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        // If template rendering fails, repeat_prompt will error
        assert!(repeat_prompt(&storage).is_ok());
    }

    #[test]
    fn test_continue_does_not_change_workflow_state() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        let state_before = storage.load().unwrap();
        repeat_prompt(&storage).unwrap();
        let state_after = storage.load().unwrap();

        // State should be identical
        assert_state_eq(&state_before, "spec", "discovery", &["spec"]);
        assert_state_eq(&state_after, "spec", "discovery", &["spec"]);
    }

    #[test]
    fn test_continue_does_not_log_state_transition() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        repeat_prompt(&storage).unwrap();

        // Should be no state transitions logged
        assert_eq!(
            count_jsonl_lines(&storage.state_dir().join("states.jsonl")),
            0
        );
    }

    // ========== Implicit Next Tests ==========

    #[test]
    fn test_next_prompt_implicit_happy_path() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        // Omit claims - should infer {"spec_complete": true}
        next_prompt(None, &storage).unwrap();
        assert_state_eq(
            &storage.load().unwrap(),
            "plan",
            "discovery",
            &["spec", "plan"],
        );
    }

    #[test]
    fn test_next_prompt_implicit_multiple_transitions() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        next_prompt(None, &storage).unwrap(); // spec -> plan
        next_prompt(None, &storage).unwrap(); // plan -> done
        assert_state_eq(
            &storage.load().unwrap(),
            "done",
            "discovery",
            &["spec", "plan", "done"],
        );
    }

    // ========== Restart Workflow Tests ==========

    #[test]
    fn test_restart_workflow_returns_to_spec() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        start_workflow("discovery", &storage).unwrap();
        next_prompt(None, &storage).unwrap(); // spec -> plan
        assert_state_eq(
            &storage.load().unwrap(),
            "plan",
            "discovery",
            &["spec", "plan"],
        );

        restart_workflow(&storage).unwrap(); // plan -> spec
        assert_state_eq(
            &storage.load().unwrap(),
            "spec",
            "discovery",
            &["spec", "plan", "spec"],
        );
    }

    #[test]
    fn test_restart_workflow_no_workflow_loaded() {
        let (_temp_dir, storage, _guard) = setup_workflow_env();
        let result = restart_workflow(&storage);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No workflow loaded"));
    }

    // ========== Production Workflow Tests ==========

    #[test]
    #[cfg_attr(coverage, ignore)] // Skip during coverage runs (different working directory)
    fn test_discovery_workflow_loads_with_rules() {
        use crate::engine::load_workflow;

        let (_temp_dir, _guard) = setup_production_workflows();
        let workflow = load_workflow("workflows/discovery.yaml").unwrap();
        assert_eq!(workflow.mode, "discovery");

        // Verify code node has rules
        let code_node = &workflow.nodes["code"];
        assert_eq!(code_node.rules.len(), 3);
    }

    #[test]
    #[cfg_attr(coverage, ignore)] // Skip during coverage runs (different working directory)
    fn test_execution_workflow_loads_with_rules() {
        use crate::engine::load_workflow;

        let (_temp_dir, _guard) = setup_production_workflows();
        let workflow = load_workflow("workflows/execution.yaml").unwrap();
        assert_eq!(workflow.mode, "execution");

        // Verify code node has rules (4 rules including phase_timeout)
        let code_node = &workflow.nodes["code"];
        assert_eq!(code_node.rules.len(), 4);
    }

    #[test]
    #[cfg_attr(coverage, ignore)] // Skip during coverage runs (different working directory)
    fn test_discovery_workflow_rules_are_valid() {
        use crate::engine::load_workflow;

        let (_temp_dir, _guard) = setup_production_workflows();
        let workflow = load_workflow("workflows/discovery.yaml").unwrap();
        let code_node = &workflow.nodes["code"];

        // Validate all rules (errors would panic)
        for rule in &code_node.rules {
            rule.validate().unwrap();
        }
    }

    #[test]
    #[cfg_attr(coverage, ignore)] // Skip during coverage runs (different working directory)
    fn test_execution_workflow_rules_are_valid() {
        use crate::engine::load_workflow;

        let (_temp_dir, _guard) = setup_production_workflows();
        let workflow = load_workflow("workflows/execution.yaml").unwrap();
        let code_node = &workflow.nodes["code"];

        // Validate all rules (errors would panic)
        for rule in &code_node.rules {
            rule.validate().unwrap();
        }
    }
}
