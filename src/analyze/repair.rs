use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

use crate::storage::archive::{read_archives, write_archive};
use crate::storage::FileStorage;
use crate::theme::Theme;

use super::cleanup::all_cleanups;
use super::gap_detection::detect_and_create_cowboy_archives;
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
    let cleanups = all_cleanups();

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

    // Detect and create synthetic cowboy workflows for historical gaps
    let synthetic_count = detect_and_create_cowboy_archives(state_dir, &archives, dry_run, json)?;

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
