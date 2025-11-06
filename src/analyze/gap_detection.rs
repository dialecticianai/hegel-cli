//! Cowboy Gap Filler
//!
//! Ensures there is exactly one synthetic cowboy workflow filling each gap between non-synthetic workflows,
//! but ONLY for gaps that contain git activity.
//!
//! Algorithm:
//! 1. Find all non-synthetic workflows sorted by start time
//! 2. For each gap between consecutive non-synthetic workflows:
//!    a. Check if gap contains git commits
//!    b. If no activity: remove any cowboys in this gap
//!    c. If has activity:
//!       - Check if a cowboy already exists for this exact gap (matching start/end times)
//!       - If yes: keep it
//!       - If no: create one
//!       - Remove any other cowboys in this gap (duplicates or incorrect timestamps)

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::Path;

use crate::storage::archive::{write_archive, WorkflowArchive};

/// Ensures exactly one cowboy per gap between non-synthetic workflows that contains git activity
///
/// Only creates cowboys for gaps with git commits. Removes cowboys from gaps without activity.
/// Takes the current archives (which may include in-memory repairs not yet written to disk)
/// to correctly identify gaps even in dry-run mode
///
/// # Arguments
/// * `git_commits` - Optional git commits to use. If `None`, reads from filesystem (production).
///                   If `Some`, uses provided commits (testing).
pub fn ensure_cowboy_coverage(
    state_dir: &Path,
    archives: &[WorkflowArchive],
    git_commits: Option<&[crate::metrics::git::GitCommit]>,
    dry_run: bool,
) -> Result<(usize, usize)> {
    // Use provided archives (may have repairs applied)
    let mut archives = archives.to_vec();

    // Sort by workflow_id (start time)
    archives.sort_by(|a, b| a.workflow_id.cmp(&b.workflow_id));

    eprintln!(
        "DEBUG COWBOY_GAP_FILLER: Processing {} archives",
        archives.len()
    );

    // Find all non-synthetic workflows
    let real_workflows: Vec<_> = archives.iter().filter(|a| !a.is_synthetic).collect();

    eprintln!(
        "DEBUG COWBOY_GAP_FILLER: Found {} real workflows",
        real_workflows.len()
    );

    // Identify gaps between consecutive real workflows that have git activity
    // Either use provided commits (testing) or read from filesystem (production)
    let git_commits_vec;
    let empty_commits = vec![];
    let git_commits = if let Some(commits) = git_commits {
        commits
    } else if crate::metrics::git::has_git_repository(state_dir) {
        let project_root = state_dir.parent().unwrap();
        git_commits_vec =
            crate::metrics::git::parse_git_commits(project_root, None).unwrap_or_default();
        &git_commits_vec
    } else {
        &empty_commits
    };

    let mut gaps = Vec::new();
    for i in 0..real_workflows.len() {
        if i + 1 < real_workflows.len() {
            let current_end = parse_timestamp(&real_workflows[i].completed_at)?;
            let next_start = parse_timestamp(&real_workflows[i + 1].workflow_id)?;

            if next_start > current_end {
                // Check if there are any git commits in this gap
                let has_activity = git_commits.iter().any(|c| {
                    if let Ok(commit_time) = parse_timestamp(&c.timestamp) {
                        commit_time > current_end && commit_time < next_start
                    } else {
                        false
                    }
                });

                if has_activity {
                    gaps.push((current_end, next_start));
                    eprintln!(
                        "DEBUG COWBOY_GAP_FILLER: Gap {} to {} (has activity)",
                        current_end, next_start
                    );
                } else {
                    eprintln!(
                        "DEBUG COWBOY_GAP_FILLER: Skipping gap {} to {} (no activity)",
                        current_end, next_start
                    );
                }
            }
        }
    }

    eprintln!(
        "DEBUG COWBOY_GAP_FILLER: Found {} gaps with activity",
        gaps.len()
    );

    let mut cowboys_created = 0;
    let mut cowboys_removed = 0;

    // First, remove any cowboys in gaps WITHOUT activity
    for i in 0..real_workflows.len() {
        if i + 1 < real_workflows.len() {
            let current_end = parse_timestamp(&real_workflows[i].completed_at)?;
            let next_start = parse_timestamp(&real_workflows[i + 1].workflow_id)?;

            if next_start > current_end {
                // Check if this gap has activity
                let has_activity = git_commits.iter().any(|c| {
                    if let Ok(commit_time) = parse_timestamp(&c.timestamp) {
                        commit_time > current_end && commit_time < next_start
                    } else {
                        false
                    }
                });

                if !has_activity {
                    // Remove any cowboys in this gap
                    let cowboys_in_empty_gap: Vec<_> = archives
                        .iter()
                        .filter(|a| {
                            if !a.is_synthetic || a.mode != "cowboy" {
                                return false;
                            }

                            let cow_start = parse_timestamp(&a.workflow_id).ok();
                            let cow_end = parse_timestamp(&a.completed_at).ok();

                            if let (Some(cs), Some(ce)) = (cow_start, cow_end) {
                                cs >= current_end && ce <= next_start
                            } else {
                                false
                            }
                        })
                        .collect();

                    for cow in cowboys_in_empty_gap {
                        eprintln!(
                            "DEBUG COWBOY_GAP_FILLER: Removing cowboy {} from gap with no activity",
                            cow.workflow_id
                        );
                        if !dry_run {
                            let archive_path = state_dir
                                .join("archive")
                                .join(format!("{}.json", cow.workflow_id));
                            if archive_path.exists() {
                                std::fs::remove_file(&archive_path)?;
                            }
                        }
                        cowboys_removed += 1;
                    }
                }
            }
        }
    }

    // For each gap, ensure exactly one cowboy exists
    for (gap_start, gap_end) in gaps {
        // Find all cowboys that overlap with this gap
        let cowboys_in_gap: Vec<_> = archives
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                if !a.is_synthetic || a.mode != "cowboy" {
                    return false;
                }

                let cow_start = parse_timestamp(&a.workflow_id).ok();
                let cow_end = parse_timestamp(&a.completed_at).ok();

                if let (Some(cs), Some(ce)) = (cow_start, cow_end) {
                    // Cowboy overlaps if its range intersects the gap
                    cs < gap_end && ce > gap_start
                } else {
                    false
                }
            })
            .collect();

        eprintln!(
            "DEBUG COWBOY_GAP_FILLER: Gap {} to {} has {} cowboys",
            gap_start,
            gap_end,
            cowboys_in_gap.len()
        );

        // Check if we have a correctly-spanning cowboy
        let correct_cowboy = cowboys_in_gap.iter().find(|(_, cow)| {
            let cow_start = parse_timestamp(&cow.workflow_id).ok();
            let cow_end = parse_timestamp(&cow.completed_at).ok();

            cow_start == Some(gap_start) && cow_end == Some(gap_end)
        });

        if correct_cowboy.is_some() {
            eprintln!("DEBUG COWBOY_GAP_FILLER: Correct cowboy exists for gap");

            // Remove all OTHER cowboys in this gap
            for (_idx, cow) in &cowboys_in_gap {
                let cow_start = parse_timestamp(&cow.workflow_id).ok();
                let cow_end = parse_timestamp(&cow.completed_at).ok();

                // Remove if not the correct one
                if cow_start != Some(gap_start) || cow_end != Some(gap_end) {
                    eprintln!(
                        "DEBUG COWBOY_GAP_FILLER: Removing duplicate/incorrect cowboy {}",
                        cow.workflow_id
                    );
                    if !dry_run {
                        let archive_path = state_dir
                            .join("archive")
                            .join(format!("{}.json", cow.workflow_id));
                        if archive_path.exists() {
                            std::fs::remove_file(&archive_path)?;
                        }
                    }
                    cowboys_removed += 1;
                }
            }
        } else {
            // No correct cowboy exists
            eprintln!("DEBUG COWBOY_GAP_FILLER: No correct cowboy, need to create one");

            // Remove all incorrect cowboys
            for (_idx, cow) in &cowboys_in_gap {
                eprintln!(
                    "DEBUG COWBOY_GAP_FILLER: Removing incorrect cowboy {}",
                    cow.workflow_id
                );
                if !dry_run {
                    let archive_path = state_dir
                        .join("archive")
                        .join(format!("{}.json", cow.workflow_id));
                    if archive_path.exists() {
                        std::fs::remove_file(&archive_path)?;
                    }
                }
                cowboys_removed += 1;
            }

            // Create new cowboy for this gap
            eprintln!(
                "DEBUG COWBOY_GAP_FILLER: Creating cowboy for gap {} to {}",
                gap_start, gap_end
            );
            if !dry_run {
                create_cowboy_for_gap(gap_start, gap_end, state_dir)?;
            }
            cowboys_created += 1;
        }
    }

    Ok((cowboys_created, cowboys_removed))
}

fn parse_timestamp(ts: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(ts)?.with_timezone(&Utc))
}

fn create_cowboy_for_gap(start: DateTime<Utc>, end: DateTime<Utc>, state_dir: &Path) -> Result<()> {
    use crate::metrics::{
        build_phase_metrics, parse_hooks_file, HookMetrics, StateTransitionEvent, UnifiedMetrics,
    };
    use crate::storage::FileStorage;

    let workflow_id = start.to_rfc3339();

    // Create synthetic state transitions for this gap
    let state_transitions = vec![
        StateTransitionEvent {
            timestamp: start.to_rfc3339(),
            workflow_id: Some(workflow_id.clone()),
            from_node: "START".to_string(),
            to_node: "ride".to_string(),
            phase: "ride".to_string(),
            mode: "cowboy".to_string(),
        },
        StateTransitionEvent {
            timestamp: end.to_rfc3339(),
            workflow_id: Some(workflow_id.clone()),
            from_node: "ride".to_string(),
            to_node: "done".to_string(),
            phase: "ride".to_string(),
            mode: "cowboy".to_string(),
        },
    ];

    // Parse hooks if available
    let hooks_path = state_dir.join("hooks.jsonl");
    let hook_metrics = if hooks_path.exists() {
        parse_hooks_file(&hooks_path)?
    } else {
        HookMetrics::default()
    };

    // Discover all transcript files for this project
    let project_root = state_dir.parent().unwrap();
    let transcript_files = crate::adapters::list_transcript_files(project_root).unwrap_or_default();

    // Build phase metrics using shared aggregation logic (filters by time range)
    let mut phase_metrics =
        build_phase_metrics(&state_transitions, &hook_metrics, &transcript_files, None)?;

    // Also load current state for session_id tracking
    let storage = FileStorage::new(state_dir)?;
    let state = storage.load()?;

    // Parse git commits and attribute to phases
    if crate::metrics::git::has_git_repository(state_dir) {
        let project_root = state_dir.parent().unwrap();
        let git_commits = crate::metrics::git::parse_git_commits(project_root, None)
            .unwrap_or_default()
            .into_iter()
            .filter(|c| {
                if let Ok(commit_time) = parse_timestamp(&c.timestamp) {
                    commit_time >= start && commit_time < end
                } else {
                    false
                }
            })
            .collect::<Vec<_>>();

        crate::metrics::git::attribute_commits_to_phases(git_commits, &mut phase_metrics);
    }

    // Mark phases as synthetic
    for phase in &mut phase_metrics {
        phase.is_synthetic = true;
        phase.workflow_id = Some(workflow_id.clone());
    }

    // Build UnifiedMetrics
    let token_metrics = phase_metrics
        .first()
        .map(|p| p.token_metrics.clone())
        .unwrap_or_default();

    let metrics = UnifiedMetrics {
        hook_metrics,
        token_metrics,
        state_transitions,
        session_id: state.session_metadata.map(|s| s.session_id),
        phase_metrics,
        git_commits: vec![], // Git commits already attributed to phases
    };

    // Create archive (synthetic cowboy)
    let mut archive = WorkflowArchive::from_metrics(&metrics, &workflow_id, true)?;
    archive.completed_at = end.to_rfc3339();

    write_archive(&archive, state_dir)?;

    Ok(())
}
