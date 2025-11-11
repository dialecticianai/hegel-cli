// Git commit tracking and attribution for workflow phases

use std::path::Path;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Git commit metadata with diff statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitCommit {
    /// Commit SHA (abbreviated, 7 chars)
    pub hash: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Commit message (first line only)
    pub message: String,
    /// Author name
    pub author: String,
    /// Number of files changed
    pub files_changed: usize,
    /// Lines inserted
    pub insertions: usize,
    /// Lines deleted
    pub deletions: usize,
}

/// Check if a git repository exists at the project root
///
/// Returns `true` if a git repository can be opened at the parent directory
/// of the provided state directory (assumed to be `.hegel`).
///
/// Never panics or propagates errors - returns `false` for any failure case.
pub fn has_git_repository(state_dir: &Path) -> bool {
    let project_root = match state_dir.parent() {
        Some(p) => p,
        None => return false,
    };

    git2::Repository::open(project_root).is_ok()
}

/// Parse git commits from repository with optional timestamp filter
///
/// Returns commits from HEAD with metadata and diff statistics.
/// Filters by `since` Unix timestamp if provided.
pub fn parse_git_commits(project_root: &Path, since: Option<i64>) -> Result<Vec<GitCommit>> {
    let repo = git2::Repository::open(project_root)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut commits = Vec::new();

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        // Filter by timestamp
        let commit_time = commit.time().seconds();
        if let Some(since_time) = since {
            if commit_time < since_time {
                continue;
            }
        }

        // Get diff stats
        let stats = get_commit_stats(&repo, &commit)?;

        // Convert to GitCommit
        let git_commit = GitCommit {
            hash: format!("{:.7}", oid),
            timestamp: timestamp_to_iso8601(commit_time),
            message: commit
                .message()
                .unwrap_or("")
                .lines()
                .next()
                .unwrap_or("")
                .to_string(),
            author: commit.author().name().unwrap_or("").to_string(),
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        };

        commits.push(git_commit);
    }

    Ok(commits)
}

/// Get diff statistics for a commit
fn get_commit_stats(repo: &git2::Repository, commit: &git2::Commit) -> Result<git2::DiffStats> {
    let old_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    let diff = repo.diff_tree_to_tree(old_tree.as_ref(), Some(&commit.tree()?), None)?;

    Ok(diff.stats()?)
}

/// Convert Unix timestamp to ISO 8601 string
pub(crate) fn timestamp_to_iso8601(timestamp: i64) -> String {
    Utc.timestamp_opt(timestamp, 0).unwrap().to_rfc3339()
}

/// Attribute commits to workflow phases by timestamp
///
/// For each commit, finds the phase whose time range contains the commit timestamp
/// and adds the commit to that phase's git_commits vector.
///
/// Commits outside all phase ranges are ignored.
pub fn attribute_commits_to_phases(
    commits: Vec<GitCommit>,
    phases: &mut [crate::metrics::PhaseMetrics],
) {
    use chrono::DateTime;

    for commit in commits {
        let commit_time = match DateTime::parse_from_rfc3339(&commit.timestamp) {
            Ok(dt) => dt,
            Err(_) => continue, // Skip commits with invalid timestamps
        };

        // Find matching phase (iterate in reverse to match newest phases first)
        for phase in phases.iter_mut().rev() {
            let phase_start = match DateTime::parse_from_rfc3339(&phase.start_time) {
                Ok(dt) => dt,
                Err(_) => continue,
            };

            // Check if commit is after phase start
            if commit_time < phase_start {
                continue;
            }

            // Check if commit is before phase end (if phase has ended)
            let in_range = if let Some(end_time) = &phase.end_time {
                match DateTime::parse_from_rfc3339(end_time) {
                    Ok(phase_end) => commit_time <= phase_end,
                    Err(_) => false,
                }
            } else {
                // Active phase (no end time) - all commits after start are included
                true
            };

            if in_range {
                phase.git_commits.push(commit.clone());
                break; // Commit attributed to first matching phase
            }
        }
    }
}
