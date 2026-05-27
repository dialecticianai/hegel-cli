//! Trim synthetic cowboy rides to their last captured activity.
//!
//! Cowboy detection runs at the *next* workflow start, so a ride's raw window
//! ends whenever detection happened to run — inflating overnight gaps into
//! many idle hours. This pass re-ends each cowboy at its last bash/file/commit
//! timestamp (anchored start preserved), recomputing per-phase durations.

use anyhow::Result;

use crate::storage::archive::read_archives;
use crate::storage::{atomic_write_json, FileStorage};
use crate::theme::Theme;

/// Detect (and with `apply`, fix) over-long synthetic cowboy ride windows.
pub fn check_and_fix_cowboy_durations(
    storage: &FileStorage,
    apply: bool,
    json: bool,
) -> Result<()> {
    let state_dir = storage.state_dir();
    let mut archives = read_archives(state_dir)?;

    let mut trimmed_ids: Vec<String> = Vec::new();
    for archive in &mut archives {
        if archive.trim_cowboy_to_activity() {
            trimmed_ids.push(archive.workflow_id.clone());
        }
    }

    // Nothing to do — stay quiet (consistent with a clean doctor run).
    if trimmed_ids.is_empty() {
        return Ok(());
    }

    if json {
        let report = serde_json::json!({
            "cowboy_duration_trim": {
                "rides_trimmed": trimmed_ids.len(),
                "applied": apply,
                "archives": trimmed_ids,
            }
        });
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("{}", Theme::header("=== Cowboy Durations ==="));
        println!(
            "{} cowboy ride(s) end past their last activity",
            Theme::highlight(&trimmed_ids.len().to_string()),
        );
    }

    if apply {
        for archive in &archives {
            if trimmed_ids.contains(&archive.workflow_id) {
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
                Theme::success("✓ Trimmed cowboy rides to last activity")
            );
        }
    } else if !json {
        println!("  {}", Theme::secondary("Run with --apply to fix"));
    }

    Ok(())
}
