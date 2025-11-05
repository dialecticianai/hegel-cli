mod sections;

use anyhow::Result;

use crate::metrics::parse_unified_metrics;
use crate::storage::FileStorage;
use crate::theme::Theme;
use sections::*;

pub fn analyze_metrics(
    storage: &FileStorage,
    export_dot: bool,
    fix_archives: bool,
    dry_run: bool,
    json: bool,
) -> Result<()> {
    // Handle archive repair if requested
    if fix_archives {
        return repair_archives(storage, dry_run || json, json);
    }

    // analyze command needs ALL metrics including archives
    let metrics = parse_unified_metrics(storage.state_dir(), true)?;

    // Export DOT format if requested
    if export_dot {
        render_workflow_graph_dot(&metrics)?;
        return Ok(());
    }

    // Otherwise, render full analysis
    println!("{}", Theme::header("=== Hegel Metrics Analysis ==="));
    println!();

    render_session(&metrics);
    render_tokens(&metrics);
    render_activity(&metrics);
    render_top_bash_commands(&metrics);
    render_command_output_summary(&metrics);
    render_top_file_modifications(&metrics);
    render_state_transitions(&metrics);
    render_phase_breakdown(&metrics.phase_metrics);
    render_workflow_graph(&metrics);

    Ok(())
}

/// Repair archives: backfill missing git metrics and rebuild cumulative totals
fn repair_archives(storage: &FileStorage, dry_run: bool, json: bool) -> Result<()> {
    use crate::metrics::git;
    use crate::storage::archive::{read_archives, write_archive};
    use serde::Serialize;

    let state_dir = storage.state_dir();

    // Read all existing archives
    let mut archives = read_archives(state_dir)?;

    if !json {
        println!(
            "{}",
            Theme::header("=== Archive Repair (Backfilling Missing Data) ===")
        );
        println!();

        if dry_run {
            println!(
                "{}",
                Theme::warning("DRY RUN MODE - No changes will be made")
            );
            println!();
        }

        if archives.is_empty() {
            println!("{}", Theme::warning("No archives found to repair"));
            return Ok(());
        }

        println!("Found {} archive(s) to check", archives.len());
        println!();
    }

    #[derive(Serialize)]
    struct ArchiveIssue {
        workflow_id: String,
        needs_git_metrics: bool,
        zero_token_phases: Vec<String>,
    }

    #[derive(Serialize)]
    struct RepairReport {
        total_archives: usize,
        archives_need_repair: usize,
        archives_missing_git: usize,
        synthetic_cowboy_created: usize,
        has_git_repository: bool,
        issues: Vec<ArchiveIssue>,
    }

    let mut repaired_count = 0;
    let mut needs_git_count = 0;
    let mut issues = Vec::new();

    // Check if we have a git repository
    let has_git = git::has_git_repository(state_dir);
    if !json && !has_git {
        println!(
            "{}",
            Theme::warning("No git repository found - skipping git backfill")
        );
        println!();
    }

    // Process each archive
    for archive in &mut archives {
        let mut needs_repair = false;
        let mut repairs = Vec::new();

        // Check if git commits are missing
        let missing_git = archive.phases.iter().all(|p| p.git_commits.is_empty());
        if missing_git && has_git {
            needs_repair = true;
            needs_git_count += 1;
            repairs.push("git metrics");
        }

        // Check for phases with zero token usage (warning only, no repair)
        let mut zero_token_phases = Vec::new();
        for phase in &archive.phases {
            let has_no_tokens = phase.tokens.input == 0
                && phase.tokens.output == 0
                && phase.tokens.cache_creation == 0
                && phase.tokens.cache_read == 0;
            if has_no_tokens {
                zero_token_phases.push(phase.phase_name.clone());
                if !json {
                    eprintln!(
                        "⚠️  Warning: Archive {} phase '{}' has no token usage recorded",
                        archive.workflow_id, phase.phase_name
                    );
                }
            }
        }

        // Collect issue data for JSON output
        if needs_repair || !zero_token_phases.is_empty() {
            issues.push(ArchiveIssue {
                workflow_id: archive.workflow_id.clone(),
                needs_git_metrics: missing_git && has_git,
                zero_token_phases,
            });
        }

        if needs_repair {
            if !json {
                println!(
                    "{} {}",
                    Theme::highlight(&archive.workflow_id),
                    Theme::secondary(&format!("(needs: {})", repairs.join(", ")))
                );
            }

            if !dry_run {
                // Backfill git metrics
                if missing_git && has_git {
                    backfill_git_metrics(archive, state_dir)?;
                }

                // Rewrite archive
                let archive_path = state_dir
                    .join("archive")
                    .join(format!("{}.json", archive.workflow_id));
                std::fs::remove_file(&archive_path)?;
                write_archive(archive, state_dir)?;

                if !json {
                    println!("  {}", Theme::success("✓ Repaired"));
                }
            } else if !json {
                println!("  {}", Theme::secondary("Would repair"));
            }

            repaired_count += 1;
        }
    }

    // Detect and create synthetic cowboy workflows for historical gaps
    let synthetic_count = detect_and_create_cowboy_archives(state_dir, &archives, dry_run, json)?;

    // Output results
    if json {
        let report = RepairReport {
            total_archives: archives.len(),
            archives_need_repair: repaired_count,
            archives_missing_git: needs_git_count,
            synthetic_cowboy_created: synthetic_count,
            has_git_repository: has_git,
            issues,
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!();
        println!(
            "{}",
            Theme::success(&format!(
                "Summary: {} archive(s) {} repair, {} cowboy workflow(s) {}",
                repaired_count,
                if dry_run { "need" } else { "repaired" },
                synthetic_count,
                if dry_run {
                    "would be created"
                } else {
                    "created"
                }
            ))
        );
        if needs_git_count > 0 {
            println!("  - {} missing git metrics", needs_git_count);
        }

        // Rebuild cumulative totals in state
        if !dry_run && repaired_count > 0 {
            println!();
            println!("{}", Theme::secondary("Rebuilding cumulative totals..."));
            rebuild_cumulative_totals(storage, &archives)?;
            println!("{}", Theme::success("✓ Cumulative totals updated"));
        }
    }

    Ok(())
}

/// Backfill git metrics for an archive by re-parsing git history
fn backfill_git_metrics(
    archive: &mut crate::storage::archive::WorkflowArchive,
    state_dir: &std::path::Path,
) -> Result<()> {
    use crate::metrics::git;

    let project_root = state_dir.parent().unwrap();

    // Use the archive's first transition timestamp as the since time
    let since_timestamp = archive
        .transitions
        .first()
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t.timestamp).ok())
        .map(|dt| dt.timestamp());

    // Parse git commits
    let git_commits = git::parse_git_commits(project_root, since_timestamp)?;

    // Convert phases to mutable PhaseMetrics for attribution
    let mut phase_metrics: Vec<crate::metrics::PhaseMetrics> = archive
        .phases
        .iter()
        .map(|p| crate::metrics::PhaseMetrics {
            phase_name: p.phase_name.clone(),
            start_time: p.start_time.clone(),
            end_time: p.end_time.clone(),
            duration_seconds: p.duration_seconds,
            token_metrics: Default::default(),
            bash_commands: vec![],
            file_modifications: vec![],
            git_commits: vec![],
            is_synthetic: archive.is_synthetic,
        })
        .collect();

    // Attribute commits to phases
    git::attribute_commits_to_phases(git_commits, &mut phase_metrics);

    // Update archive phases with git commits
    for (phase_archive, phase_metrics) in archive.phases.iter_mut().zip(phase_metrics.iter()) {
        phase_archive.git_commits = phase_metrics.git_commits.clone();
    }

    // Update totals
    archive.totals.git_commits = archive.phases.iter().map(|p| p.git_commits.len()).sum();

    Ok(())
}

/// Detect and create synthetic cowboy archives for historical gaps
fn detect_and_create_cowboy_archives(
    state_dir: &std::path::Path,
    archives: &[crate::storage::archive::WorkflowArchive],
    dry_run: bool,
    json: bool,
) -> Result<usize> {
    use crate::metrics::cowboy::{build_synthetic_cowboy_archive, identify_cowboy_workflows};
    use crate::metrics::{git, parse_hooks_file};
    use crate::storage::archive::write_archive;

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

    // Identify cowboy workflows from gaps
    let cowboy_groups = identify_cowboy_workflows(
        &bash_commands,
        &file_modifications,
        &git_commits,
        &[], // transcript events - skip for now
        archives,
    )?;

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

/// Rebuild cumulative totals in state from all archives
fn rebuild_cumulative_totals(
    storage: &FileStorage,
    archives: &[crate::storage::archive::WorkflowArchive],
) -> Result<()> {
    use crate::storage::archive::WorkflowTotals;

    let mut state = storage.load()?;
    let mut cumulative = WorkflowTotals::default();

    // Sum up all archive totals
    for archive in archives {
        cumulative.tokens.input += archive.totals.tokens.input;
        cumulative.tokens.output += archive.totals.tokens.output;
        cumulative.tokens.cache_creation += archive.totals.tokens.cache_creation;
        cumulative.tokens.cache_read += archive.totals.tokens.cache_read;
        cumulative.tokens.assistant_turns += archive.totals.tokens.assistant_turns;
        cumulative.bash_commands += archive.totals.bash_commands;
        cumulative.file_modifications += archive.totals.file_modifications;
        cumulative.unique_files += archive.totals.unique_files;
        cumulative.unique_commands += archive.totals.unique_commands;
        cumulative.git_commits += archive.totals.git_commits;
    }

    state.cumulative_totals = Some(cumulative);
    storage.save(&state)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn test_analyze_empty_state() {
        // Empty state directory - should not error
        let (_temp_dir, storage) = test_storage_with_files(None, None);

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_session_data() {
        // State with session ID and token metrics
        let hooks = vec![
            r#"{"session_id":"test-session-123","hook_event_name":"SessionStart","timestamp":"2025-01-01T10:00:00Z"}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(Some(&hooks), None);

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_tokens() {
        // State with token metrics
        let transcript_events = vec![
            r#"{"type":"assistant","timestamp":"2025-01-01T10:05:00Z","message":{"usage":{"input_tokens":100,"output_tokens":50,"cache_creation_input_tokens":200,"cache_read_input_tokens":300}}}"#,
        ];
        let (_transcript_temp, transcript_path) = create_transcript_file(&transcript_events);
        let hook = hook_with_transcript(&transcript_path, "test", "2025-01-01T10:00:00Z");
        let (_temp_dir, storage) = test_storage_with_files(Some(&[&hook]), None);

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_bash_commands() {
        // State with bash commands
        let hooks = vec![
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:05:00Z","tool_input":{"command":"cargo build"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:06:00Z","tool_input":{"command":"cargo test"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:07:00Z","tool_input":{"command":"cargo build"}}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(Some(&hooks), None);

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_file_modifications() {
        // State with file modifications
        let hooks = vec![
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Edit","timestamp":"2025-01-01T10:05:00Z","tool_input":{"file_path":"src/main.rs"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Write","timestamp":"2025-01-01T10:06:00Z","tool_input":{"file_path":"README.md"}}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(Some(&hooks), None);

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_state_transitions() {
        // State with workflow transitions
        let states = vec![
            r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"START","to_node":"spec","phase":"spec","mode":"discovery"}"#,
            r#"{"timestamp":"2025-01-01T10:15:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"spec","to_node":"plan","phase":"plan","mode":"discovery"}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(None, Some(&states));

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_phase_metrics() {
        // Full workflow with phase metrics
        let states = vec![
            r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"START","to_node":"spec","phase":"spec","mode":"discovery"}"#,
            r#"{"timestamp":"2025-01-01T10:15:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"spec","to_node":"plan","phase":"plan","mode":"discovery"}"#,
        ];
        let hooks = vec![
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:05:00Z","tool_input":{"command":"cargo build"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Edit","timestamp":"2025-01-01T10:10:00Z","tool_input":{"file_path":"spec.md"}}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(Some(&hooks), Some(&states));

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_active_phase() {
        // Active phase (no end_time) should display correctly
        let states = vec![
            r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"START","to_node":"spec","phase":"spec","mode":"discovery"}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(None, Some(&states));

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_with_long_command() {
        // Very long bash command should be truncated
        let long_command = "a".repeat(100);
        let hook_str = format!(
            r#"{{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:05:00Z","tool_input":{{"command":"{}"}}}}"#,
            long_command
        );
        let (_temp_dir, storage) = test_storage_with_files(Some(&[&hook_str]), None);

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_comprehensive() {
        // Test all sections together
        let transcript_events = vec![
            r#"{"type":"assistant","timestamp":"2025-01-01T10:05:00Z","message":{"usage":{"input_tokens":100,"output_tokens":50}}}"#,
            r#"{"type":"assistant","timestamp":"2025-01-01T10:20:00Z","message":{"usage":{"input_tokens":150,"output_tokens":75}}}"#,
        ];
        let (_transcript_temp, transcript_path) = create_transcript_file(&transcript_events);

        let states = vec![
            r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"START","to_node":"spec","phase":"spec","mode":"discovery"}"#,
            r#"{"timestamp":"2025-01-01T10:15:00Z","workflow_id":"2025-01-01T10:00:00Z","from_node":"spec","to_node":"plan","phase":"plan","mode":"discovery"}"#,
        ];

        let hook = hook_with_transcript(
            &transcript_path,
            "test-comprehensive",
            "2025-01-01T10:00:00Z",
        );
        let hooks = vec![
            hook.as_str(),
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:05:00Z","tool_input":{"command":"cargo build"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Edit","timestamp":"2025-01-01T10:10:00Z","tool_input":{"file_path":"spec.md"}}"#,
            r#"{"session_id":"test","hook_event_name":"PostToolUse","tool_name":"Bash","timestamp":"2025-01-01T10:20:00Z","tool_input":{"command":"cargo test"}}"#,
        ];
        let (_temp_dir, storage) = test_storage_with_files(Some(&hooks), Some(&states));

        let result = analyze_metrics(&storage, false, false, false, false);
        assert!(result.is_ok());
    }
}
