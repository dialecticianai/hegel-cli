use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashMap;

use crate::engine::{get_next_prompt, init_state, load_workflow};
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
    println!("{}", node.prompt);

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
    let workflow = serde_yaml::from_value(workflow_yaml.clone())
        .context("Failed to parse stored workflow")?;

    // Parse claims from JSON string
    let claims: HashMap<String, bool> = serde_json::from_str(claims_str)
        .context("Failed to parse claims JSON. Expected format: {\"claim_name\": true}")?;

    // Get next prompt
    let previous_node = workflow_state.current_node.clone();
    let (prompt_text, new_state) = get_next_prompt(&workflow, workflow_state, &claims)?;

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
    println!("{}", prompt_text);

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
