mod aborted;
mod git;

use anyhow::Result;
use std::path::Path;

pub use aborted::AbortedNodeCleanup;
pub use git::GitBackfillCleanup;

/// Trait for archive cleanup strategies
///
/// Each cleanup implementation can detect issues in archived workflows
/// and repair them by mutating the archive in place.
pub trait ArchiveCleanup {
    /// Human-readable name of this cleanup (e.g., "git metrics backfill")
    fn name(&self) -> &str;

    /// Check if this archive needs repair by this cleanup
    fn needs_repair(&self, archive: &crate::storage::archive::WorkflowArchive) -> bool;

    /// Perform the repair on the archive
    ///
    /// # Arguments
    /// * `archive` - The archive to repair (only mutated if dry_run=false)
    /// * `state_dir` - Path to .hegel directory (for context like git repo access)
    /// * `dry_run` - If true, detect issues but don't mutate the archive
    ///
    /// # Returns
    /// True if repair was needed (and performed if dry_run=false), false otherwise
    fn repair(
        &self,
        archive: &mut crate::storage::archive::WorkflowArchive,
        state_dir: &Path,
        dry_run: bool,
    ) -> Result<bool>;
}

/// Registry of all available cleanup strategies
pub fn all_cleanups() -> Vec<Box<dyn ArchiveCleanup>> {
    vec![Box::new(GitBackfillCleanup), Box::new(AbortedNodeCleanup)]
}
