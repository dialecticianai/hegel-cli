use anyhow::Result;

use crate::metrics::git;

/// Backfill git metrics for an archive by re-parsing git history
pub fn backfill_git_metrics(
    archive: &mut crate::storage::archive::WorkflowArchive,
    state_dir: &std::path::Path,
) -> Result<()> {
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
