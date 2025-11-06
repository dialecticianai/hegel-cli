use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

use crate::storage::archive::{read_archives, write_archive};
use crate::storage::FileStorage;
use crate::theme::Theme;

use super::cleanup::all_cleanups;
use super::totals::rebuild_cumulative_totals;

/// Repair archives: backfill missing git metrics and rebuild cumulative totals
pub fn repair_archives(storage: &FileStorage, dry_run: bool, json: bool) -> Result<()> {
    let state_dir = storage.state_dir();

    // Read all existing archives
    let mut archives = read_archives(state_dir)?;

    if !json {
        println!(
            "{}",
            Theme::header("=== Archive Repair (Backfilling Missing Data) ===")
        );
        println!();

        if dry_run {
            println!(
                "{}",
                Theme::warning("DRY RUN MODE - No changes will be made")
            );
            println!();
        }

        if archives.is_empty() {
            println!("{}", Theme::warning("No archives found to repair"));
            return Ok(());
        }

        println!("Found {} archive(s) to check", archives.len());
        println!();
    }

    #[derive(Serialize)]
    struct ArchiveIssue {
        workflow_id: String,
        repairs_needed: Vec<String>,
        zero_token_phases: Vec<String>,
    }

    #[derive(Serialize)]
    struct RepairReport {
        total_archives: usize,
        archives_need_repair: usize,
        repairs_by_type: HashMap<String, usize>,
        synthetic_cowboy_created: usize,
        issues: Vec<ArchiveIssue>,
    }

    let mut repaired_count = 0;
    let mut repairs_by_type: HashMap<String, usize> = HashMap::new();
    let mut issues = Vec::new();

    // Get all cleanup strategies
    let mut cleanups = all_cleanups();

    // Process each archive
    for archive in &mut archives {
        let mut repairs_needed = Vec::new();

        // Check which cleanups are needed for this archive
        for cleanup in &cleanups {
            if cleanup.needs_repair(archive) {
                repairs_needed.push(cleanup.name().to_string());
                *repairs_by_type
                    .entry(cleanup.name().to_string())
                    .or_insert(0) += 1;
            }
        }

        // Check for phases with zero token usage (warning only, no repair)
        let mut zero_token_phases = Vec::new();
        for phase in &archive.phases {
            let has_no_tokens = phase.tokens.input == 0
                && phase.tokens.output == 0
                && phase.tokens.cache_creation == 0
                && phase.tokens.cache_read == 0;
            if has_no_tokens {
                zero_token_phases.push(phase.phase_name.clone());
                if !json {
                    eprintln!(
                        "⚠️  Warning: Archive {} phase '{}' has no token usage recorded",
                        archive.workflow_id, phase.phase_name
                    );
                }
            }
        }

        // Collect issue data for JSON output
        if !repairs_needed.is_empty() || !zero_token_phases.is_empty() {
            issues.push(ArchiveIssue {
                workflow_id: archive.workflow_id.clone(),
                repairs_needed: repairs_needed.clone(),
                zero_token_phases,
            });
        }

        if !repairs_needed.is_empty() {
            if !json {
                println!(
                    "{} {}",
                    Theme::highlight(&archive.workflow_id),
                    Theme::secondary(&format!("(needs: {})", repairs_needed.join(", ")))
                );
            }

            // Apply all repairs
            let mut archive_modified = false;
            for cleanup in &cleanups {
                let repaired = cleanup.repair(archive, state_dir, dry_run)?;
                if repaired {
                    archive_modified = true;
                }
            }

            if archive_modified && !dry_run {
                // Rewrite archive
                let archive_path = state_dir
                    .join("archive")
                    .join(format!("{}.json", archive.workflow_id));
                std::fs::remove_file(&archive_path)?;
                write_archive(archive, state_dir)?;

                if !json {
                    println!("  {}", Theme::success("✓ Repaired"));
                }
            } else if !json && dry_run {
                println!("  {}", Theme::secondary("Would repair"));
            }

            repaired_count += 1;
        }
    }

    // Reload archives after repair phase to get updated data
    // (repair phase may have modified archive timestamps)
    if repaired_count > 0 && !dry_run {
        archives = read_archives(state_dir)?;

        // Re-sort archives by workflow_id (timestamp) since repairs may have changed end times
        // This ensures consecutive duplicate detection works correctly
        archives.sort_by(|a, b| a.workflow_id.cmp(&b.workflow_id));
    }

    // Run post-processing hooks (for batch operations like duplicate removal)
    let mut total_removed = 0;
    for cleanup in &mut cleanups {
        let to_remove = cleanup.post_process(&mut archives, state_dir, dry_run)?;

        if !to_remove.is_empty() {
            let count = to_remove.len();
            total_removed += count;
            *repairs_by_type
                .entry(cleanup.name().to_string())
                .or_insert(0) += count;

            if !json {
                for &index in &to_remove {
                    if dry_run {
                        println!(
                            "{} {}",
                            Theme::highlight(&archives[index].workflow_id),
                            Theme::secondary(&format!("({} - would be removed)", cleanup.name()))
                        );
                    } else {
                        println!(
                            "{} {}",
                            Theme::highlight(&archives[index].workflow_id),
                            Theme::secondary(&format!("({} - removing)", cleanup.name()))
                        );
                    }
                }
            }

            // DON'T remove from archives vec yet or delete files - gap detection needs to see them
        }
    }

    repaired_count += total_removed;

    // Write debug log to file
    use std::io::Write;
    let mut debug_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/hegel_repair_debug.log")?;

    writeln!(
        debug_file,
        "\n=== REPAIR DEBUG SESSION {} ===",
        chrono::Utc::now()
    )?;
    writeln!(debug_file, "DEBUG REPAIR: About to run gap detection")?;
    writeln!(
        debug_file,
        "DEBUG REPAIR: archives.len() = {}",
        archives.len()
    )?;
    writeln!(
        debug_file,
        "DEBUG REPAIR: total_removed = {}",
        total_removed
    )?;
    writeln!(debug_file, "DEBUG REPAIR: dry_run = {}", dry_run)?;

    // Print archive timeline
    writeln!(debug_file, "DEBUG REPAIR: Archive timeline:")?;
    for (i, archive) in archives.iter().enumerate() {
        writeln!(
            debug_file,
            "  [{}] {} - mode={} synthetic={}",
            i, archive.workflow_id, archive.mode, archive.is_synthetic
        )?;
    }

    eprintln!("DEBUG REPAIR: About to run gap detection");
    eprintln!("DEBUG REPAIR: archives.len() = {}", archives.len());
    eprintln!("DEBUG REPAIR: total_removed = {}", total_removed);
    eprintln!("DEBUG REPAIR: dry_run = {}", dry_run);

    // Print archive timeline
    eprintln!("DEBUG REPAIR: Archive timeline:");
    for (i, archive) in archives.iter().enumerate() {
        eprintln!(
            "  [{}] {} - mode={} synthetic={}",
            i, archive.workflow_id, archive.mode, archive.is_synthetic
        );
    }

    // Ensure exactly one cowboy per gap between non-synthetic workflows
    // Pass the in-memory archives which include repairs (even in dry-run mode)
    let (cowboys_created, _cowboys_removed) =
        super::gap_detection::ensure_cowboy_coverage(state_dir, &archives, dry_run)?;
    let synthetic_count = cowboys_created;

    writeln!(
        debug_file,
        "DEBUG REPAIR: Gap detection returned synthetic_count = {}",
        synthetic_count
    )?;
    eprintln!(
        "DEBUG REPAIR: Gap detection returned synthetic_count = {}",
        synthetic_count
    );

    // NOW delete duplicate cowboy files and remove from archives vec (after gap detection)
    if !dry_run && total_removed > 0 {
        writeln!(debug_file, "DEBUG REPAIR: Entering deletion phase")?;
        eprintln!("DEBUG REPAIR: Entering deletion phase");
        for cleanup in &mut cleanups {
            writeln!(
                debug_file,
                "DEBUG REPAIR: Calling post_process AGAIN for cleanup: {}",
                cleanup.name()
            )?;
            eprintln!(
                "DEBUG REPAIR: Calling post_process AGAIN for cleanup: {}",
                cleanup.name()
            );
            let to_remove = cleanup.post_process(&mut archives, state_dir, true)?; // Get indices again
            writeln!(
                debug_file,
                "DEBUG REPAIR: Second post_process returned {} indices",
                to_remove.len()
            )?;
            eprintln!(
                "DEBUG REPAIR: Second post_process returned {} indices",
                to_remove.len()
            );

            // Delete files
            for &index in &to_remove {
                let archive = &archives[index];
                writeln!(
                    debug_file,
                    "DEBUG REPAIR: Deleting file for archive[{}]: {}",
                    index, archive.workflow_id
                )?;
                eprintln!(
                    "DEBUG REPAIR: Deleting file for archive[{}]: {}",
                    index, archive.workflow_id
                );
                let archive_path = state_dir
                    .join("archive")
                    .join(format!("{}.json", archive.workflow_id));
                if archive_path.exists() {
                    std::fs::remove_file(&archive_path)?;
                }
            }

            // Remove from archives vec (in reverse to maintain indices)
            for &index in to_remove.iter().rev() {
                writeln!(
                    debug_file,
                    "DEBUG REPAIR: Removing archive[{}] from vec",
                    index
                )?;
                eprintln!("DEBUG REPAIR: Removing archive[{}] from vec", index);
                archives.remove(index);
            }
        }
        writeln!(
            debug_file,
            "DEBUG REPAIR: After deletion, archives.len() = {}",
            archives.len()
        )?;
        eprintln!(
            "DEBUG REPAIR: After deletion, archives.len() = {}",
            archives.len()
        );
    }

    // Output results
    if json {
        let report = RepairReport {
            total_archives: archives.len(),
            archives_need_repair: repaired_count,
            repairs_by_type,
            synthetic_cowboy_created: synthetic_count,
            issues,
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!();
        println!(
            "{}",
            Theme::success(&format!(
                "Summary: {} archive(s) {} repair, {} cowboy workflow(s) {}",
                repaired_count,
                if dry_run { "need" } else { "repaired" },
                synthetic_count,
                if dry_run {
                    "would be created"
                } else {
                    "created"
                }
            ))
        );
        for (repair_type, count) in &repairs_by_type {
            println!("  - {}: {} archive(s)", repair_type, count);
        }

        // Rebuild cumulative totals in state
        if !dry_run && repaired_count > 0 {
            println!();
            println!("{}", Theme::secondary("Rebuilding cumulative totals..."));
            rebuild_cumulative_totals(storage, &archives)?;
            println!("{}", Theme::success("✓ Cumulative totals updated"));
        }
    }

    Ok(())
}
