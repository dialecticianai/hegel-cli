mod aborted;
mod duplicate_cowboy;
mod git;

use anyhow::Result;
use std::path::Path;

pub use aborted::AbortedNodeCleanup;
pub use duplicate_cowboy::DuplicateCowboyCleanup;
pub use git::GitBackfillCleanup;

/// Trait for archive cleanup strategies
///
/// Each cleanup implementation can detect issues in archived workflows
/// and repair them by mutating the archive in place.
pub trait ArchiveCleanup {
    /// Human-readable name of this cleanup (e.g., "git metrics backfill")
    fn name(&self) -> &str;

    /// Check if this archive needs repair by this cleanup
    ///
    /// Default implementation returns false. Override for per-archive cleanups.
    /// Batch cleanups (using post_process) can leave this as default.
    fn needs_repair(&self, _archive: &crate::storage::archive::WorkflowArchive) -> bool {
        false
    }

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

    /// Post-process archives after all individual repairs are complete
    ///
    /// This hook is called after all archives have been processed through
    /// the repair() method. It allows cleanup strategies to perform batch
    /// operations that require visibility across all archives.
    ///
    /// # Arguments
    /// * `archives` - All archives (mutable for repairs)
    /// * `state_dir` - Path to .hegel directory
    /// * `dry_run` - If true, detect issues but don't mutate
    ///
    /// # Returns
    /// Indices of archives to remove from the collection
    fn post_process(
        &mut self,
        _archives: &mut [crate::storage::archive::WorkflowArchive],
        _state_dir: &Path,
        _dry_run: bool,
    ) -> Result<Vec<usize>> {
        // Default: no post-processing needed
        Ok(Vec::new())
    }
}

/// Registry of all available cleanup strategies
pub fn all_cleanups() -> Vec<Box<dyn ArchiveCleanup>> {
    vec![
        Box::new(GitBackfillCleanup),
        Box::new(AbortedNodeCleanup),
        Box::new(DuplicateCowboyCleanup::new()),
    ]
}
