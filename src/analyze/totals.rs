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
        cumulative += &archive.totals;
    }

    state.cumulative_totals = Some(cumulative);
    storage.save(&state)?;

    Ok(())
}
