use anyhow::Result;

use crate::engine::is_terminal;
use crate::storage::FileStorage;
use crate::theme::Theme;

/// Get uncommitted changes summary (files modified, lines changed)
fn get_uncommitted_changes(storage: &FileStorage) -> Result<Option<String>> {
    let project_root = match storage.state_dir().parent() {
        Some(p) => p,
        None => return Ok(None),
    };

    let repo = match git2::Repository::open(project_root) {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    // Get diff between HEAD and working directory
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return Ok(None), // Detached HEAD or no commits
    };

    let head_tree = match head.peel_to_tree() {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };

    let mut opts = git2::DiffOptions::new();
    opts.include_untracked(true);

    let diff = match repo.diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut opts)) {
        Ok(d) => d,
        Err(_) => return Ok(None),
    };

    let stats = match diff.stats() {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };

    let files_changed = stats.files_changed();
    let insertions = stats.insertions();
    let deletions = stats.deletions();

    if files_changed == 0 {
        return Ok(None);
    }

    let total_lines = insertions + deletions;
    let files_str = if files_changed == 1 { "file" } else { "files" };
    let lines_str = if total_lines == 1 { "line" } else { "lines" };
    Ok(Some(format!(
        "uncommitted changes: {} {}, {} {}",
        files_changed, files_str, total_lines, lines_str
    )))
}

/// Show overall project status (meta-mode, workflow, etc.)
pub fn show_status(storage: &FileStorage) -> Result<()> {
    // Ensure git info is cached
    storage.ensure_git_info_cached()?;

    let state = storage.load()?;

    // Show git information
    if let Some(git_info) = &state.git_info {
        if git_info.has_repo {
            let branch = git_info.current_branch.as_deref().unwrap_or("(detached)");
            print!("{}: {}", Theme::label("Git"), Theme::highlight(branch));

            // Add uncommitted changes summary
            match get_uncommitted_changes(storage)? {
                Some(changes) => print!(" {}", Theme::secondary(&format!("({})", changes))),
                None => print!(" {}", Theme::secondary("(no changes)")),
            }
            println!();
        }
    }

    // Show workflow status if active
    if state.workflow.is_none() || state.workflow.is_none() {
        println!("{}", Theme::secondary("No active workflow"));
        println!(
            "Start a workflow with: {}",
            Theme::highlight("hegel start <workflow>")
        );
        return Ok(());
    }

    let workflow_state = state.workflow.as_ref().unwrap();

    // Check if we're in a terminal state (done or aborted)
    let in_terminal_state = is_terminal(&workflow_state.current_node);

    // Build single-line status: <mode> (<meta_mode>) node1->node2->[current]->...
    if in_terminal_state {
        println!("{}", Theme::secondary("No active workflow"));
        print!("{} ", Theme::label("Previous workflow:"));
    } else {
        print!("{}: ", Theme::label("Hegel"));
    }

    // Workflow mode (highlighted)
    print!("{}", Theme::highlight(&workflow_state.mode));

    // Meta-mode (if active, highlighted)
    if let Some(meta_mode) = &workflow_state.meta_mode {
        print!(" {}", Theme::highlight(&format!("({})", meta_mode.name)));
    }

    print!(" ");

    // Build node chain with colors
    for (i, node) in workflow_state.history.iter().enumerate() {
        if i > 0 {
            print!("{}", Theme::secondary("->"));
        }

        if node == &workflow_state.current_node {
            print!("[{}]", Theme::highlight(node));
        } else {
            print!("{}", Theme::secondary(node));
        }
    }

    // Add current node if not in history
    if !workflow_state
        .history
        .contains(&workflow_state.current_node)
    {
        print!("{}", Theme::secondary("->"));
        print!("[{}]", Theme::highlight(&workflow_state.current_node));
    }

    println!();

    Ok(())
}
