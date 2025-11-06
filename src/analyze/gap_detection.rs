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
pub fn ensure_cowboy_coverage(
    state_dir: &Path,
    archives: &[WorkflowArchive],
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
    // Read git commits to check if gaps have activity
    let git_commits = if crate::metrics::git::has_git_repository(state_dir) {
        let project_root = state_dir.parent().unwrap();
        crate::metrics::git::parse_git_commits(project_root, None).unwrap_or_default()
    } else {
        vec![]
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
    // Read git commits in this time range
    let git_commits = if crate::metrics::git::has_git_repository(state_dir) {
        let project_root = state_dir.parent().unwrap();
        crate::metrics::git::parse_git_commits(project_root, None)
            .unwrap_or_default()
            .into_iter()
            .filter(|c| {
                if let Ok(commit_time) = parse_timestamp(&c.timestamp) {
                    commit_time >= start && commit_time < end
                } else {
                    false
                }
            })
            .collect()
    } else {
        vec![]
    };

    // Build cowboy group
    let group = crate::metrics::cowboy::CowboyActivityGroup {
        start_time: start,
        end_time: end,
        bash_commands: vec![],
        file_modifications: vec![],
        git_commits,
        transcript_events: vec![],
    };

    let archive = crate::metrics::cowboy::build_synthetic_cowboy_archive(&group)?;
    write_archive(&archive, state_dir)?;

    Ok(())
}
