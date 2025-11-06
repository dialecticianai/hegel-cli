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

    // Use default needs_repair() which returns false
    // Duplicate detection happens entirely in post_process()

    fn repair(
        &self,
        _archive: &mut WorkflowArchive,
        _state_dir: &Path,
        _dry_run: bool,
    ) -> Result<bool> {
        // This cleanup uses post_process() instead of repair()
        // Individual archives can't be repaired in isolation
        Ok(false)
    }

    fn post_process(
        &mut self,
        archives: &mut [WorkflowArchive],
        _state_dir: &Path,
        _dry_run: bool,
    ) -> Result<Vec<usize>> {
        // Remove consecutive synthetic cowboys: A->B->C->D->E where B,C,D are cowboys becomes A->B->E
        // Keep the first cowboy in any consecutive sequence, remove the rest
        let mut to_remove = Vec::new();
        let mut prev_was_cowboy = false;

        eprintln!(
            "DEBUG: Checking {} archives for consecutive cowboys",
            archives.len()
        );
        for (index, archive) in archives.iter().enumerate() {
            let is_cowboy = archive.is_synthetic && archive.mode == "cowboy";

            if is_cowboy {
                eprintln!(
                    "DEBUG: [{}] {} - cowboy (prev_was_cowboy={})",
                    index, archive.workflow_id, prev_was_cowboy
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
        WorkflowArchive {
            workflow_id: workflow_id.to_string(),
            mode: "cowboy".to_string(),
            completed_at: workflow_id.to_string(),
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
