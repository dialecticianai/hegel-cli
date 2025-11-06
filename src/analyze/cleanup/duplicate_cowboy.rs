use anyhow::Result;
use std::path::Path;

use crate::storage::archive::WorkflowArchive;

use super::ArchiveCleanup;

/// Cleanup strategy: remove duplicate synthetic cowboy archives
///
/// Before the fix in commit 341957f, cowboy detection ran at workflow END
/// (in archive_and_cleanup), which caused the same inter-workflow activity
/// to be detected multiple times across sequential workflow completions.
///
/// This cleanup identifies duplicate synthetic cowboy archives with identical
/// workflow_id timestamps and removes all but the first occurrence.
///
/// # Detection Logic
///
/// A cowboy archive is considered duplicate if:
/// 1. It is synthetic (is_synthetic = true)
/// 2. It has mode = "cowboy"
/// 3. Another archive exists with the same workflow_id (timestamp)
///
/// # Repair Strategy
///
/// When duplicates are found, keep the FIRST occurrence and mark later
/// occurrences for removal. The first occurrence is most likely to have
/// complete metrics since it was created closest to the actual activity.
pub struct DuplicateCowboyCleanup;

impl DuplicateCowboyCleanup {
    pub fn new() -> Self {
        Self
    }
}

impl ArchiveCleanup for DuplicateCowboyCleanup {
    fn name(&self) -> &str {
        "duplicate cowboy removal"
    }

    fn needs_repair(&self, archive: &WorkflowArchive) -> bool {
        // Repair zero-duration synthetic cowboys (workflow_id == completed_at)
        archive.is_synthetic
            && archive.mode == "cowboy"
            && archive.workflow_id == archive.completed_at
    }

    fn repair(
        &self,
        archive: &mut WorkflowArchive,
        state_dir: &Path,
        _dry_run: bool,
    ) -> Result<bool> {
        // Fix zero-duration cowboys by updating completed_at
        // This happens when old cowboy archives were created with the same start/end timestamp

        if !self.needs_repair(archive) {
            return Ok(false);
        }

        // NOTE: Unlike other repairs, this ALWAYS mutates the in-memory archive, even in dry-run.
        // This is necessary so gap_detection.rs sees corrected timestamps when analyzing gaps.
        // The dry_run flag only affects whether we persist changes to disk (handled by caller).

        // Read all archives to find the next workflow after this cowboy
        let all_archives = crate::storage::archive::read_archives(state_dir)?;
        let current_start =
            chrono::DateTime::parse_from_rfc3339(&archive.workflow_id)?.with_timezone(&chrono::Utc);

        // Find next workflow (any workflow that starts after this one)
        let next_workflow_start = all_archives
            .iter()
            .filter_map(|a| chrono::DateTime::parse_from_rfc3339(&a.workflow_id).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .filter(|dt| *dt > current_start)
            .min();

        let new_completed_at = if let Some(next_start) = next_workflow_start {
            next_start.to_rfc3339()
        } else {
            // No next workflow, use current time
            chrono::Utc::now().to_rfc3339()
        };

        // Update the in-memory archive
        archive.completed_at = new_completed_at.clone();

        // Update the done transition timestamp if it exists
        if let Some(done_transition) = archive.transitions.iter_mut().find(|t| t.to_node == "done")
        {
            done_transition.timestamp = new_completed_at.clone();
        }

        // Update phase end_time if it exists
        if let Some(phase) = archive.phases.first_mut() {
            phase.end_time = Some(new_completed_at.clone());

            // Recalculate duration
            if let (Ok(start), Ok(end)) = (
                chrono::DateTime::parse_from_rfc3339(&phase.start_time),
                chrono::DateTime::parse_from_rfc3339(&new_completed_at),
            ) {
                phase.duration_seconds = (end - start).num_seconds() as u64;
            }
        }

        Ok(true)
    }

    fn post_process(
        &mut self,
        archives: &mut [WorkflowArchive],
        _state_dir: &Path,
        _dry_run: bool,
    ) -> Result<Vec<usize>> {
        // Remove consecutive synthetic cowboys: A->B->C->D->E where B,C,D are cowboys becomes A->B->E
        // Keep the first cowboy in any consecutive sequence, remove the rest
        // Skip zero-duration cowboys (they need to be fixed first in the repair phase)
        let mut to_remove = Vec::new();
        let mut prev_was_cowboy = false;

        eprintln!(
            "DEBUG: Checking {} archives for consecutive cowboys",
            archives.len()
        );
        for (index, archive) in archives.iter().enumerate() {
            // Skip zero-duration cowboys - they haven't been repaired yet
            let is_zero_duration = archive.workflow_id == archive.completed_at;
            let is_cowboy = archive.is_synthetic && archive.mode == "cowboy" && !is_zero_duration;

            if archive.is_synthetic && archive.mode == "cowboy" {
                eprintln!(
                    "DEBUG: [{}] {} - cowboy (prev_was_cowboy={}, zero_duration={})",
                    index, archive.workflow_id, prev_was_cowboy, is_zero_duration
                );
            }

            // If this is a cowboy AND previous was a cowboy, remove this one
            if is_cowboy && prev_was_cowboy {
                eprintln!(
                    "DEBUG: [{}] MARKING FOR REMOVAL: {}",
                    index, archive.workflow_id
                );
                to_remove.push(index);
            } else {
                prev_was_cowboy = is_cowboy;
            }
        }

        eprintln!("DEBUG: Total marked for removal: {}", to_remove.len());

        // Return indices to remove - caller will delete files AFTER gap detection
        // (so gap detection can see existing cowboys and not recreate them)
        Ok(to_remove)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::archive::{PhaseArchive, TokenTotals, TransitionArchive, WorkflowTotals};

    fn test_cowboy_archive(workflow_id: &str, is_synthetic: bool) -> WorkflowArchive {
        // Create non-zero-duration cowboy by adding 30 minutes to completed_at
        use chrono::{DateTime, Duration, Utc};
        let start = DateTime::parse_from_rfc3339(workflow_id)
            .unwrap()
            .with_timezone(&Utc);
        let end = start + Duration::minutes(30);
        let completed_at = end.to_rfc3339();

        WorkflowArchive {
            workflow_id: workflow_id.to_string(),
            mode: "cowboy".to_string(),
            completed_at,
            session_id: None,
            is_synthetic,
            phases: vec![PhaseArchive {
                phase_name: "ride".to_string(),
                start_time: workflow_id.to_string(),
                end_time: Some(workflow_id.to_string()),
                duration_seconds: 1800,
                tokens: TokenTotals::default(),
                bash_commands: vec![],
                file_modifications: vec![],
                git_commits: vec![],
            }],
            transitions: vec![TransitionArchive {
                from_node: "START".to_string(),
                to_node: "ride".to_string(),
                timestamp: workflow_id.to_string(),
            }],
            totals: WorkflowTotals::default(),
        }
    }

    fn test_discovery_archive(workflow_id: &str) -> WorkflowArchive {
        WorkflowArchive {
            workflow_id: workflow_id.to_string(),
            mode: "discovery".to_string(),
            completed_at: workflow_id.to_string(),
            session_id: None,
            is_synthetic: false,
            phases: vec![PhaseArchive {
                phase_name: "spec".to_string(),
                start_time: workflow_id.to_string(),
                end_time: Some(workflow_id.to_string()),
                duration_seconds: 1800,
                tokens: TokenTotals::default(),
                bash_commands: vec![],
                file_modifications: vec![],
                git_commits: vec![],
            }],
            transitions: vec![TransitionArchive {
                from_node: "START".to_string(),
                to_node: "spec".to_string(),
                timestamp: workflow_id.to_string(),
            }],
            totals: WorkflowTotals::default(),
        }
    }

    #[test]
    fn test_consecutive_cowboys_detected() {
        let mut cleanup = DuplicateCowboyCleanup::new();

        //  Simulate archive list: A->B->C->D->E where B,C,D are consecutive cowboys
        let mut archives = vec![
            test_discovery_archive("2025-01-01T09:00:00Z"), // A: regular workflow
            test_cowboy_archive("2025-01-01T10:00:00Z", true), // B: cowboy (keep)
            test_cowboy_archive("2025-01-01T11:00:00Z", true), // C: cowboy (remove - consecutive)
            test_cowboy_archive("2025-01-01T12:00:00Z", true), // D: cowboy (remove - consecutive)
            test_discovery_archive("2025-01-01T13:00:00Z"), // E: regular workflow
        ];

        let to_remove = cleanup
            .post_process(&mut archives, std::path::Path::new("."), true)
            .unwrap();

        // Should mark indices 2 and 3 for removal (C and D)
        assert_eq!(to_remove, vec![2, 3]);
    }

    #[test]
    fn test_non_consecutive_cowboys_not_removed() {
        let mut cleanup = DuplicateCowboyCleanup::new();

        // Cowboys separated by regular workflows
        let mut archives = vec![
            test_cowboy_archive("2025-01-01T10:00:00Z", true), // Cowboy (keep)
            test_discovery_archive("2025-01-01T11:00:00Z"),    // Regular workflow
            test_cowboy_archive("2025-01-01T12:00:00Z", true), // Cowboy (keep - not consecutive)
        ];

        let to_remove = cleanup
            .post_process(&mut archives, std::path::Path::new("."), true)
            .unwrap();

        // No duplicates - cowboys are separated
        assert_eq!(to_remove, Vec::<usize>::new());
    }
}
