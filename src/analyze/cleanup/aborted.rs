use anyhow::Result;
use std::path::Path;

use crate::engine::is_terminal;
use crate::storage::archive::{TransitionArchive, WorkflowArchive};

use super::ArchiveCleanup;

/// Cleanup strategy: add synthetic ABORTED terminal nodes for incomplete workflows
///
/// Workflows that were aborted before the ABORTED node feature may be missing
/// terminal transitions. This cleanup detects incomplete workflows and adds
/// a synthetic ABORTED transition.
pub struct AbortedNodeCleanup;

impl ArchiveCleanup for AbortedNodeCleanup {
    fn name(&self) -> &str {
        "aborted node backfill"
    }

    fn needs_repair(&self, archive: &WorkflowArchive) -> bool {
        // An archive needs repair if it has no terminal transition
        let has_terminal = archive.transitions.iter().any(|t| is_terminal(&t.to_node));

        !has_terminal
    }

    fn repair(
        &self,
        archive: &mut WorkflowArchive,
        _state_dir: &Path,
        dry_run: bool,
    ) -> Result<bool> {
        // Check if repair is needed
        if !self.needs_repair(archive) {
            return Ok(false);
        }

        // If dry run, just report that repair is needed
        if dry_run {
            return Ok(true);
        }

        // Get the last transition to determine from_node
        let last_transition = archive.transitions.last();

        if let Some(last_trans) = last_transition {
            // Create synthetic aborted transition
            let aborted_transition = TransitionArchive {
                from_node: last_trans.to_node.clone(),
                to_node: "aborted".to_string(),
                timestamp: archive.completed_at.clone(), // Use archive completion time
            };

            archive.transitions.push(aborted_transition);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::archive::{PhaseArchive, TokenTotals, WorkflowTotals};

    fn test_archive_incomplete() -> WorkflowArchive {
        WorkflowArchive {
            workflow_id: "2025-01-01T10:00:00Z".to_string(),
            mode: "discovery".to_string(),
            completed_at: "2025-01-01T11:00:00Z".to_string(),
            session_id: None,
            is_synthetic: false,
            phases: vec![PhaseArchive {
                phase_name: "spec".to_string(),
                start_time: "2025-01-01T10:00:00Z".to_string(),
                end_time: Some("2025-01-01T10:30:00Z".to_string()),
                duration_seconds: 1800,
                tokens: TokenTotals::default(),
                bash_commands: vec![],
                file_modifications: vec![],
                git_commits: vec![],
            }],
            transitions: vec![TransitionArchive {
                from_node: "START".to_string(),
                to_node: "spec".to_string(),
                timestamp: "2025-01-01T10:00:00Z".to_string(),
            }],
            totals: WorkflowTotals::default(),
        }
    }

    fn test_archive_complete() -> WorkflowArchive {
        let mut archive = test_archive_incomplete();
        archive.transitions.push(TransitionArchive {
            from_node: "spec".to_string(),
            to_node: "done".to_string(),
            timestamp: "2025-01-01T11:00:00Z".to_string(),
        });
        archive
    }

    #[test]
    fn test_needs_repair_incomplete_workflow() {
        let cleanup = AbortedNodeCleanup;
        let archive = test_archive_incomplete();
        assert!(cleanup.needs_repair(&archive));
    }

    #[test]
    fn test_needs_repair_complete_workflow() {
        let cleanup = AbortedNodeCleanup;
        let archive = test_archive_complete();
        assert!(!cleanup.needs_repair(&archive));
    }

    #[test]
    fn test_repair_adds_aborted_transition() {
        let cleanup = AbortedNodeCleanup;
        let mut archive = test_archive_incomplete();
        let temp_dir = tempfile::TempDir::new().unwrap();

        let repaired = cleanup
            .repair(&mut archive, temp_dir.path(), false)
            .unwrap();

        assert!(repaired);
        assert_eq!(archive.transitions.len(), 2);
        assert_eq!(archive.transitions[1].from_node, "spec");
        assert_eq!(archive.transitions[1].to_node, "aborted");
        assert_eq!(archive.transitions[1].timestamp, "2025-01-01T11:00:00Z");
    }

    #[test]
    fn test_repair_dry_run_does_not_mutate() {
        let cleanup = AbortedNodeCleanup;
        let mut archive = test_archive_incomplete();
        let temp_dir = tempfile::TempDir::new().unwrap();

        let repaired = cleanup.repair(&mut archive, temp_dir.path(), true).unwrap();

        assert!(repaired); // Reports repair is needed
        assert_eq!(archive.transitions.len(), 1); // But doesn't mutate
    }

    #[test]
    fn test_repair_complete_archive_no_op() {
        let cleanup = AbortedNodeCleanup;
        let mut archive = test_archive_complete();
        let temp_dir = tempfile::TempDir::new().unwrap();

        let repaired = cleanup
            .repair(&mut archive, temp_dir.path(), false)
            .unwrap();

        assert!(!repaired);
        assert_eq!(archive.transitions.len(), 2); // Unchanged
    }
}
