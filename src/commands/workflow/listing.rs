//! Listing commands: available workflows and guides.

use anyhow::Result;
use colored::Colorize;

use crate::engine::load_workflow;
use crate::storage::FileStorage;

/// List all available workflows
pub fn list_workflows(storage: &FileStorage) -> Result<()> {
    use std::collections::HashSet;
    use std::fs;

    // Get embedded workflows
    let embedded: HashSet<String> = crate::embedded::list_workflows()
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Get filesystem workflows
    let workflows_dir = storage.workflows_dir();
    let mut filesystem = HashSet::new();

    if let Ok(entries) = fs::read_dir(&workflows_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    filesystem.insert(name.to_string());
                }
            }
        }
    }

    // Separate into local-only and embedded (or both)
    let mut local_only: Vec<String> = filesystem.difference(&embedded).cloned().collect();
    local_only.sort();

    let mut embedded_workflows: Vec<String> = embedded.iter().cloned().collect();
    embedded_workflows.sort();

    // Display local-only workflows first
    if !local_only.is_empty() {
        println!("Local-only:\n");
        for workflow in &local_only {
            let workflow_path = storage.workflow_path(workflow);
            let flow = match load_workflow(&workflow_path) {
                Ok(wf) => extract_node_flow(&wf),
                Err(_) => "".to_string(),
            };

            if flow.is_empty() {
                println!("  {}", workflow);
            } else {
                println!("  {}\n    {}", workflow, flow.dimmed());
            }
        }
        println!();
    }

    // Display embedded workflows
    if !embedded_workflows.is_empty() {
        println!("Embedded:\n");
        for workflow in &embedded_workflows {
            // Load workflow to extract node flow
            let workflow_path = storage.workflow_path(workflow);
            let flow = match load_workflow(&workflow_path) {
                Ok(wf) => extract_node_flow(&wf),
                Err(_) => "".to_string(),
            };

            if flow.is_empty() {
                println!("  {}", workflow);
            } else {
                println!("  {}\n    {}", workflow, flow.dimmed());
            }
        }
    }

    Ok(())
}

/// Extract a simple node flow visualization from a workflow
pub(crate) fn extract_node_flow(workflow: &crate::engine::Workflow) -> String {
    use std::collections::{HashMap, HashSet};

    let start = &workflow.start_node;
    let mut visited = HashSet::new();
    let mut flow = Vec::new();
    let mut current = start.clone();

    // Build a map of node -> next node (taking first transition)
    let mut next_map: HashMap<String, Option<String>> = HashMap::new();
    for (node_name, node) in &workflow.nodes {
        let next = node
            .transitions
            .first()
            .map(|t| t.to.clone())
            .filter(|to| to != node_name); // Skip self-loops
        next_map.insert(node_name.clone(), next);
    }

    // Follow the chain from start node
    flow.push(current.clone());
    visited.insert(current.clone());

    while let Some(Some(next)) = next_map.get(&current) {
        if visited.contains(next) {
            break; // Prevent infinite loops
        }
        flow.push(next.clone());
        visited.insert(next.clone());
        current = next.clone();
    }

    flow.join(" → ")
}

/// List all available guides or show embedded guide content
pub fn list_guides(show_embedded: Option<&str>, storage: &FileStorage) -> Result<()> {
    // If --show-embedded flag is provided, display that guide's content
    if let Some(guide_name) = show_embedded {
        return show_embedded_guide(guide_name);
    }

    use std::collections::HashSet;
    use std::fs;

    // Get embedded guides
    let embedded: HashSet<String> = crate::embedded::list_guides()
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Get filesystem guides
    let guides_dir = storage.guides_dir();
    let mut filesystem = HashSet::new();

    if let Ok(entries) = fs::read_dir(&guides_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    filesystem.insert(name.to_string());
                }
            }
        }
    }

    // Combine and sort
    let mut all_guides: Vec<String> = embedded.union(&filesystem).cloned().collect();
    all_guides.sort();

    println!("Available guides:");
    for guide in &all_guides {
        let mut markers = Vec::new();
        if embedded.contains(guide) {
            markers.push("embedded");
        }
        if filesystem.contains(guide) {
            markers.push("local");
        }

        if markers.is_empty() {
            println!("  {}", guide);
        } else {
            println!("  {} ({})", guide, markers.join(", "));
        }
    }

    Ok(())
}

/// Display the content of an embedded guide
fn show_embedded_guide(guide_name: &str) -> Result<()> {
    // Auto-add .md extension if missing
    let normalized_name = if guide_name.ends_with(".md") {
        guide_name.to_string()
    } else {
        format!("{}.md", guide_name)
    };

    // Try to get the embedded guide
    match crate::embedded::get_guide(&normalized_name) {
        Some(content) => {
            println!("{}", content);
            Ok(())
        }
        None => {
            anyhow::bail!(
                "Embedded guide '{}' not found. Run 'hegel guides' to see available guides.",
                normalized_name
            );
        }
    }
}
