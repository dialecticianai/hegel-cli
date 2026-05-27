//! Repair archived phase/transition metrics: drop records duplicated across
//! archives (the pre-Nov-3 re-archiving bug wrote nested cumulative snapshots),
//! compact each archive (intra-archive dedup + prune), then rebuild totals.

use std::collections::BTreeSet;

use anyhow::Result;

use crate::storage::archive::{dedup_across_archives, read_archives};
use crate::storage::{atomic_write_json, FileStorage};
use crate::theme::Theme;

/// Detect (and with `apply`, fix) phase-metric issues across all archives.
pub fn check_and_fix_phases(storage: &FileStorage, apply: bool, json: bool) -> Result<()> {
    let state_dir = storage.state_dir();
    let mut archives = read_archives(state_dir)?;

    // Track every archive whose content changed (cross-archive or intra-archive).
    let mut changed_ids: BTreeSet<String> = BTreeSet::new();

    // 1. Cross-archive dedup: strip phases/transitions duplicated across the
    //    nested cumulative snapshots, keeping one canonical copy of each.
    let cross = dedup_across_archives(&mut archives);
    changed_ids.extend(cross.affected.iter().cloned());

    // 2. Compact each archive in memory and tally what changed.
    let mut total_deduped = 0;
    let mut total_pruned = 0;
    for archive in &mut archives {
        let result = archive.compact_phases();
        if result.changed() {
            total_deduped += result.deduped;
            total_pruned += result.pruned;
            changed_ids.insert(archive.workflow_id.clone());
        }
    }

    // Nothing to do — stay quiet (consistent with a clean doctor run).
    if changed_ids.is_empty() {
        return Ok(());
    }

    if json {
        let report = serde_json::json!({
            "phase_compaction": {
                "archives_affected": changed_ids.len(),
                "cross_archive_phases_removed": cross.phases_removed,
                "cross_archive_transitions_removed": cross.transitions_removed,
                "duplicate_phases_removed": total_deduped,
                "empty_phases_pruned": total_pruned,
                "applied": apply,
                "archives": changed_ids,
            }
        });
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{}", Theme::header("=== Phase Metrics ==="));
        println!(
            "{} archive(s) need repair: {} cross-archive phase(s), {} cross-archive transition(s), {} duplicate phase(s), {} empty phase(s)",
            Theme::highlight(&changed_ids.len().to_string()),
            Theme::highlight(&cross.phases_removed.to_string()),
            Theme::highlight(&cross.transitions_removed.to_string()),
            Theme::highlight(&total_deduped.to_string()),
            Theme::highlight(&total_pruned.to_string()),
        );
    }

    if apply {
        // Overwrite the changed archives atomically, then rebuild cumulative totals.
        for archive in &archives {
            if changed_ids.contains(&archive.workflow_id) {
                let path = state_dir
                    .join("archive")
                    .join(format!("{}.json", archive.workflow_id));
                atomic_write_json(&path, archive)?;
            }
        }
        crate::analyze::totals::rebuild_cumulative_totals(storage, &archives)?;
        if !json {
            println!(
                "  {}",
                Theme::success("✓ Compacted phases and rebuilt cumulative totals")
            );
        }
    } else if !json {
        println!("  {}", Theme::secondary("Run with --apply to fix"));
    }

    Ok(())
}
