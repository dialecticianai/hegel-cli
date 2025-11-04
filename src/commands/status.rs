use anyhow::Result;

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
    Ok(Some(format!("{}f, {}l", files_changed, total_lines)))
}

/// Show overall project status (meta-mode, workflow, etc.)
pub fn show_status(storage: &FileStorage) -> Result<()> {
    // Ensure git info is cached
    storage.ensure_git_info_cached()?;

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

    // Show git information
    if let Some(git_info) = &state.git_info {
        if git_info.has_repo {
            let branch = git_info.current_branch.as_deref().unwrap_or("(detached)");
            print!("{}: {}", Theme::label("Git"), branch);

            // Add uncommitted changes summary
            if let Some(changes) = get_uncommitted_changes(storage)? {
                print!(" {}", Theme::secondary(&format!("({})", changes)));
            }
            println!();
        } else {
            println!(
                "{}: {}",
                Theme::label("Git"),
                Theme::secondary("not in git repo")
            );
        }
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
