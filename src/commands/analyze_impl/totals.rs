use anyhow::Result;

use crate::storage::archive::WorkflowTotals;
use crate::storage::FileStorage;

/// Rebuild cumulative totals in state from all archives
pub fn rebuild_cumulative_totals(
    storage: &FileStorage,
    archives: &[crate::storage::archive::WorkflowArchive],
) -> Result<()> {
    let mut state = storage.load()?;
    let mut cumulative = WorkflowTotals::default();

    // Sum up all archive totals
    for archive in archives {
        cumulative.tokens.input += archive.totals.tokens.input;
        cumulative.tokens.output += archive.totals.tokens.output;
        cumulative.tokens.cache_creation += archive.totals.tokens.cache_creation;
        cumulative.tokens.cache_read += archive.totals.tokens.cache_read;
        cumulative.tokens.assistant_turns += archive.totals.tokens.assistant_turns;
        cumulative.bash_commands += archive.totals.bash_commands;
        cumulative.file_modifications += archive.totals.file_modifications;
        cumulative.unique_files += archive.totals.unique_files;
        cumulative.unique_commands += archive.totals.unique_commands;
        cumulative.git_commits += archive.totals.git_commits;
    }

    state.cumulative_totals = Some(cumulative);
    storage.save(&state)?;

    Ok(())
}
