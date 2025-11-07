//! Archive test helpers

use crate::metrics::git::GitCommit;
use crate::storage::archive::{
    PhaseArchive, TokenTotals, TransitionArchive, WorkflowArchive, WorkflowTotals,
};

/// Builder for creating test WorkflowArchive instances
///
/// Provides a fluent interface for creating archives with custom metrics.
/// Useful for testing cowboy detection, gap analysis, and archive-related functionality.
///
/// # Example
/// ```ignore
/// use crate::test_helpers::ArchiveBuilder;
///
/// let archive = ArchiveBuilder::new("2025-01-01T10:00:00Z", "2025-01-01T10:30:00Z")
///     .with_git_commits(vec![test_commit])
///     .build();
/// ```
pub struct ArchiveBuilder {
    workflow_id: String,
    completed_at: String,
    mode: String,
    is_synthetic: bool,
    git_commits: Vec<GitCommit>,
}

impl ArchiveBuilder {
    /// Create a new ArchiveBuilder with start and end times
    pub fn new(workflow_id: &str, completed_at: &str) -> Self {
        Self {
            workflow_id: workflow_id.to_string(),
            completed_at: completed_at.to_string(),
            mode: "discovery".to_string(),
            is_synthetic: false,
            git_commits: vec![],
        }
    }

    /// Set the workflow mode (default: "discovery")
    pub fn mode(mut self, mode: &str) -> Self {
        self.mode = mode.to_string();
        self
    }

    /// Mark as synthetic (default: false)
    pub fn synthetic(mut self, is_synthetic: bool) -> Self {
        self.is_synthetic = is_synthetic;
        self
    }

    /// Add git commits to the archive
    // TODO: Investigate if this method is still needed or can be removed
    #[allow(dead_code)]
    pub fn with_git_commits(mut self, commits: Vec<GitCommit>) -> Self {
        self.git_commits = commits;
        self
    }

    /// Build the WorkflowArchive
    pub fn build(self) -> WorkflowArchive {
        let git_count = self.git_commits.len();

        WorkflowArchive {
            workflow_id: self.workflow_id.clone(),
            mode: self.mode,
            completed_at: self.completed_at.clone(),
            session_id: None,
            is_synthetic: self.is_synthetic,
            phases: vec![PhaseArchive {
                phase_name: "spec".to_string(),
                start_time: self.workflow_id.clone(),
                end_time: Some(self.completed_at.clone()),
                duration_seconds: 900,
                tokens: TokenTotals::default(),
                bash_commands: vec![],
                file_modifications: vec![],
                git_commits: self.git_commits,
            }],
            transitions: vec![TransitionArchive {
                from_node: "START".to_string(),
                to_node: "spec".to_string(),
                timestamp: self.workflow_id,
            }],
            totals: WorkflowTotals {
                tokens: TokenTotals::default(),
                bash_commands: 0,
                file_modifications: 0,
                unique_files: 0,
                unique_commands: 0,
                git_commits: git_count,
            },
        }
    }
}

/// Create a simple test WorkflowArchive with specified start and end times
///
/// Convenience function for creating minimal archives without the builder.
/// For more complex archives with git commits, use `ArchiveBuilder`.
///
/// # Arguments
/// * `workflow_id` - Start timestamp (RFC3339 format)
/// * `completed_at` - End timestamp (RFC3339 format)
///
/// # Example
/// ```ignore
/// use crate::test_helpers::test_archive;
///
/// let archive = test_archive("2025-01-01T10:00:00Z", "2025-01-01T10:30:00Z");
/// write_archive(&archive, state_dir).unwrap();
/// ```
pub fn test_archive(workflow_id: &str, completed_at: &str) -> WorkflowArchive {
    ArchiveBuilder::new(workflow_id, completed_at).build()
}

/// Create a test GitCommit with specified timestamp
///
/// # Arguments
/// * `timestamp` - Commit timestamp (RFC3339 format)
///
/// # Example
/// ```ignore
/// let commit = test_git_commit("2025-01-01T10:15:00Z");
/// ```
pub fn test_git_commit(timestamp: &str) -> GitCommit {
    GitCommit {
        hash: "abc123def456".to_string(),
        author: "test@example.com".to_string(),
        timestamp: timestamp.to_string(),
        message: "test commit".to_string(),
        files_changed: 1,
        insertions: 10,
        deletions: 5,
    }
}
