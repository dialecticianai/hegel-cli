use anyhow::Result;
use std::path::Path;

use crate::metrics::git;
use crate::storage::archive::WorkflowArchive;

use super::ArchiveCleanup;

/// Cleanup strategy: backfill git commit metrics for archives
///
/// Archives created before git integration may be missing commit data.
/// This cleanup re-parses git history and attributes commits to phases.
pub struct GitBackfillCleanup;

impl ArchiveCleanup for GitBackfillCleanup {
    fn name(&self) -> &str {
        "git metrics backfill"
    }

    fn needs_repair(&self, archive: &WorkflowArchive) -> bool {
        // Check if all phases have no git commits
        archive.phases.iter().all(|p| p.git_commits.is_empty())
    }

    fn repair(
        &self,
        archive: &mut WorkflowArchive,
        state_dir: &Path,
        dry_run: bool,
    ) -> Result<bool> {
        // Check if repair is needed
        if !self.needs_repair(archive) {
            return Ok(false);
        }

        // Check if git repository exists
        if !git::has_git_repository(state_dir) {
            return Ok(false);
        }

        // If dry run, just report that repair is needed
        if dry_run {
            return Ok(true);
        }

        // Perform the actual repair
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

        Ok(true)
    }
}
