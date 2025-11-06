use anyhow::Result;

use crate::metrics::cowboy::{build_synthetic_cowboy_archive, identify_cowboy_workflows};
use crate::metrics::{git, parse_hooks_file};
use crate::storage::archive::write_archive;
use crate::theme::Theme;

/// Detect and create synthetic cowboy archives for historical gaps
pub fn detect_and_create_cowboy_archives(
    state_dir: &std::path::Path,
    archives: &[crate::storage::archive::WorkflowArchive],
    dry_run: bool,
    json: bool,
) -> Result<usize> {
    if !json {
        println!();
        println!("{}", Theme::label("Detecting cowboy workflows..."));
    }

    // Parse all historical data
    let hooks_path = state_dir.join("hooks.jsonl");
    let (bash_commands, file_modifications) = if hooks_path.exists() {
        let hook_metrics = parse_hooks_file(&hooks_path)?;
        (hook_metrics.bash_commands, hook_metrics.file_modifications)
    } else {
        (vec![], vec![])
    };

    let git_commits = if git::has_git_repository(state_dir) {
        let project_root = state_dir.parent().unwrap();
        git::parse_git_commits(project_root, None).unwrap_or_default()
    } else {
        vec![]
    };

    use std::io::Write;
    let mut debug_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/hegel_repair_debug.log")?;

    writeln!(
        debug_file,
        "DEBUG GAP: About to call identify_cowboy_workflows"
    )?;
    writeln!(debug_file, "DEBUG GAP: archives.len() = {}", archives.len())?;
    writeln!(
        debug_file,
        "DEBUG GAP: bash_commands.len() = {}",
        bash_commands.len()
    )?;
    writeln!(
        debug_file,
        "DEBUG GAP: file_modifications.len() = {}",
        file_modifications.len()
    )?;
    writeln!(
        debug_file,
        "DEBUG GAP: git_commits.len() = {}",
        git_commits.len()
    )?;

    eprintln!("DEBUG GAP: About to call identify_cowboy_workflows");
    eprintln!("DEBUG GAP: archives.len() = {}", archives.len());
    eprintln!("DEBUG GAP: bash_commands.len() = {}", bash_commands.len());
    eprintln!(
        "DEBUG GAP: file_modifications.len() = {}",
        file_modifications.len()
    );
    eprintln!("DEBUG GAP: git_commits.len() = {}", git_commits.len());

    // Identify cowboy workflows from gaps
    let cowboy_groups = identify_cowboy_workflows(
        &bash_commands,
        &file_modifications,
        &git_commits,
        &[], // transcript events - skip for now
        archives,
    )?;

    writeln!(
        debug_file,
        "DEBUG GAP: identify_cowboy_workflows returned {} groups",
        cowboy_groups.len()
    )?;
    for (i, group) in cowboy_groups.iter().enumerate() {
        writeln!(
            debug_file,
            "DEBUG GAP: Group[{}]: {} to {} ({} bash, {} files, {} commits)",
            i,
            group.start_time,
            group.end_time,
            group.bash_commands.len(),
            group.file_modifications.len(),
            group.git_commits.len()
        )?;
    }

    eprintln!(
        "DEBUG GAP: identify_cowboy_workflows returned {} groups",
        cowboy_groups.len()
    );
    for (i, group) in cowboy_groups.iter().enumerate() {
        eprintln!(
            "DEBUG GAP: Group[{}]: {} to {} ({} bash, {} files, {} commits)",
            i,
            group.start_time,
            group.end_time,
            group.bash_commands.len(),
            group.file_modifications.len(),
            group.git_commits.len()
        );
    }

    if cowboy_groups.is_empty() {
        if !json {
            println!("  {}", Theme::success("No gaps found"));
        }
        return Ok(0);
    }

    if !json {
        println!("  Found {} cowboy workflow gap(s)", cowboy_groups.len());
    }

    let mut created_count = 0;
    for group in cowboy_groups {
        let synthetic_archive = build_synthetic_cowboy_archive(&group)?;
        let archive_path = state_dir
            .join("archive")
            .join(format!("{}.json", synthetic_archive.workflow_id));

        // Skip if already exists (idempotent)
        if archive_path.exists() {
            if !json {
                println!(
                    "  {} (already exists)",
                    Theme::secondary(&synthetic_archive.workflow_id)
                );
            }
            continue;
        }

        if dry_run {
            if !json {
                println!(
                    "  {} (would create)",
                    Theme::highlight(&synthetic_archive.workflow_id)
                );
            }
        } else {
            write_archive(&synthetic_archive, state_dir)?;
            if !json {
                println!(
                    "  {} {}",
                    Theme::success("✓"),
                    Theme::highlight(&synthetic_archive.workflow_id)
                );
            }
            created_count += 1;
        }
    }

    Ok(created_count)
}
