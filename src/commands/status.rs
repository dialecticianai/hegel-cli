use anyhow::Result;

use crate::storage::FileStorage;
use crate::theme::Theme;

/// Show overall project status (meta-mode, workflow, etc.)
pub fn show_status(storage: &FileStorage) -> Result<()> {
    let state = storage.load()?;

    println!("{}", Theme::header("Project Status"));
    println!();

    // Show meta-mode
    if let Some(workflow_state) = &state.workflow_state {
        if let Some(meta_mode) = &workflow_state.meta_mode {
            println!("{}: {}", Theme::label("Meta-mode"), meta_mode.name);
        } else {
            println!(
                "{}: {}",
                Theme::label("Meta-mode"),
                Theme::secondary("none")
            );
        }
    } else {
        println!(
            "{}: {}",
            Theme::label("Meta-mode"),
            Theme::secondary("none")
        );
    }

    println!();

    // Show workflow status if active
    if state.workflow.is_none() || state.workflow_state.is_none() {
        println!("{}", Theme::secondary("No active workflow"));
        println!();
        println!(
            "Start a workflow with: {}",
            Theme::highlight("hegel start <workflow>")
        );
        return Ok(());
    }

    let workflow_state = state.workflow_state.as_ref().unwrap();

    println!("{}", Theme::header("Workflow Status"));
    println!();
    println!("{}: {}", Theme::label("Mode"), workflow_state.mode);
    println!(
        "{}: {}",
        Theme::label("Current node"),
        workflow_state.current_node
    );
    println!();
    println!("{}", Theme::label("History:"));
    for (i, node) in workflow_state.history.iter().enumerate() {
        if i == workflow_state.history.len() - 1 {
            println!("  {} {}", Theme::highlight("→"), Theme::highlight(node));
        } else {
            println!("    {}", Theme::secondary(node));
        }
    }

    Ok(())
}
