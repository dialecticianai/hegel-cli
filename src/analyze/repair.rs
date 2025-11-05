use anyhow::Result;
use serde::Serialize;

use crate::metrics::git;
use crate::storage::archive::{read_archives, write_archive};
use crate::storage::FileStorage;
use crate::theme::Theme;

use super::backfill::backfill_git_metrics;
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
        needs_git_metrics: bool,
        zero_token_phases: Vec<String>,
    }

    #[derive(Serialize)]
    struct RepairReport {
        total_archives: usize,
        archives_need_repair: usize,
        archives_missing_git: usize,
        synthetic_cowboy_created: usize,
        has_git_repository: bool,
        issues: Vec<ArchiveIssue>,
    }

    let mut repaired_count = 0;
    let mut needs_git_count = 0;
    let mut issues = Vec::new();

    // Check if we have a git repository
    let has_git = git::has_git_repository(state_dir);
    if !json && !has_git {
        println!(
            "{}",
            Theme::warning("No git repository found - skipping git backfill")
        );
        println!();
    }

    // Process each archive
    for archive in &mut archives {
        let mut needs_repair = false;
        let mut repairs = Vec::new();

        // Check if git commits are missing
        let missing_git = archive.phases.iter().all(|p| p.git_commits.is_empty());
        if missing_git && has_git {
            needs_repair = true;
            needs_git_count += 1;
            repairs.push("git metrics");
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
        if needs_repair || !zero_token_phases.is_empty() {
            issues.push(ArchiveIssue {
                workflow_id: archive.workflow_id.clone(),
                needs_git_metrics: missing_git && has_git,
                zero_token_phases,
            });
        }

        if needs_repair {
            if !json {
                println!(
                    "{} {}",
                    Theme::highlight(&archive.workflow_id),
                    Theme::secondary(&format!("(needs: {})", repairs.join(", ")))
                );
            }

            if !dry_run {
                // Backfill git metrics
                if missing_git && has_git {
                    backfill_git_metrics(archive, state_dir)?;
                }

                // Rewrite archive
                let archive_path = state_dir
                    .join("archive")
                    .join(format!("{}.json", archive.workflow_id));
                std::fs::remove_file(&archive_path)?;
                write_archive(archive, state_dir)?;

                if !json {
                    println!("  {}", Theme::success("✓ Repaired"));
                }
            } else if !json {
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
            archives_missing_git: needs_git_count,
            synthetic_cowboy_created: synthetic_count,
            has_git_repository: has_git,
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
        if needs_git_count > 0 {
            println!("  - {} missing git metrics", needs_git_count);
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
