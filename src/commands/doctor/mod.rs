#[cfg(test)]
mod tests;

use anyhow::Result;
use clap::Args;
use serde::Serialize;
use std::collections::HashMap;

use crate::doctor::{all_migrations, rescue_state_file};
use crate::storage::FileStorage;
use crate::theme::Theme;

#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// Perform a dry-run without modifying files
    #[arg(long)]
    pub dry_run: bool,

    /// Show verbose output
    #[arg(long, short)]
    pub verbose: bool,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn doctor_command(args: DoctorArgs, storage: &FileStorage) -> Result<()> {
    let state_path = storage.state_dir().join("state.json");

    // Check if state.json exists
    if !state_path.exists() {
        if args.json {
            let report = DoctorReport {
                state_file_exists: false,
                issues_found: 0,
                migrations_needed: HashMap::new(),
                migrations_applied: 0,
                dry_run: args.dry_run,
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!(
                "{}",
                Theme::success("No state file found - nothing to check")
            );
        }
        return Ok(());
    }

    if !args.json {
        println!(
            "{}",
            Theme::header("=== Hegel Doctor: State Health Check ===")
        );
        println!();

        if args.dry_run {
            println!(
                "{}",
                Theme::warning("DRY RUN MODE - No changes will be made")
            );
            println!();
        }
    }

    // Try to rescue corrupted state files first (unless in dry-run mode)
    let _rescued = if !args.dry_run {
        match rescue_state_file(storage) {
            Ok(true) => {
                if !args.json {
                    println!("{}", Theme::success("✓ Rescued corrupted state.json"));
                    println!();
                }
                true
            }
            Ok(false) => false,
            Err(e) => {
                // Rescue failed, but continue - maybe load() will work
                if args.verbose && !args.json {
                    eprintln!("Rescue attempt failed: {}", e);
                }
                false
            }
        }
    } else {
        false
    };

    // Load current state
    let state = match storage.load() {
        Ok(s) => s,
        Err(e) => {
            // If load fails and we're NOT in dry-run mode, we have a real problem
            if args.json {
                let report = DoctorReport {
                    state_file_exists: true,
                    issues_found: 1,
                    migrations_needed: {
                        let mut m = HashMap::new();
                        m.insert("rescue".to_string(), vec![e.to_string()]);
                        m
                    },
                    migrations_applied: 0,
                    dry_run: args.dry_run,
                };
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("{} Failed to load state.json", Theme::error("✗"));
                println!();
                println!("Error: {}", e);
                println!();
                if args.dry_run {
                    println!("Run without --dry-run to attempt automatic repair.");
                } else {
                    println!("State file may be corrupted beyond automatic repair.");
                    println!("Consider manual intervention or restoring from backup.");
                }
            }
            return Err(e);
        }
    };

    // Get all migration strategies
    let mut migrations = all_migrations();

    // Check which migrations are needed
    let mut issues = Vec::new();
    let mut migrations_by_type: HashMap<String, Vec<String>> = HashMap::new();

    for migration in &migrations {
        if let Some(issue) = migration.check(&state, storage)? {
            if args.verbose && !args.json {
                println!(
                    "{} {}",
                    Theme::warning("⚠"),
                    Theme::highlight(migration.name())
                );
                println!("  {}", Theme::secondary(&issue.description));
                if let Some(ref impact) = issue.impact {
                    println!("  Impact: {}", impact);
                }
                println!();
            }

            migrations_by_type
                .entry(migration.name().to_string())
                .or_insert_with(Vec::new)
                .push(issue.description.clone());

            issues.push(issue);
        }
    }

    // Report findings
    if issues.is_empty() {
        if args.json {
            let report = DoctorReport {
                state_file_exists: true,
                issues_found: 0,
                migrations_needed: HashMap::new(),
                migrations_applied: 0,
                dry_run: args.dry_run,
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!("{}", Theme::success("✓ State file is healthy"));
            println!();
            println!("Run 'hegel status' to verify everything works.");
        }
        return Ok(());
    }

    // Display issues summary
    if !args.json {
        println!("{} Found {} issue(s):", Theme::warning("⚠"), issues.len());
        println!();

        for (i, issue) in issues.iter().enumerate() {
            println!("  {}. {}", i + 1, Theme::highlight(&migrations[i].name()));
            println!("     {}", issue.description);
            if let Some(ref impact) = issue.impact {
                println!("     • {}", impact);
            }
        }
        println!();
    }

    // Apply migrations
    let mut applied_count = 0;
    if !args.dry_run {
        if !args.json {
            println!("{}", Theme::secondary("Migrating state.json..."));
        }

        let mut current_state = state;

        for migration in &mut migrations {
            if migration.check(&current_state, storage)?.is_some() {
                current_state = migration.migrate(current_state, storage)?;
                applied_count += 1;

                if args.verbose && !args.json {
                    println!("  {} {}", Theme::success("✓"), migration.name());
                }
            }
        }

        // Save the migrated state
        storage.save(&current_state)?;

        if !args.json {
            println!();
            println!(
                "{}",
                Theme::success(&format!(
                    "✓ Migration complete! Applied {} migration(s)",
                    applied_count
                ))
            );
            println!();
            println!("Run 'hegel status' to verify everything works.");
        }
    } else {
        if !args.json {
            println!(
                "{}",
                Theme::warning(&format!(
                    "Dry-run mode: {} migration(s) would be applied",
                    issues.len()
                ))
            );
            println!("Run without --dry-run to perform migration.");
        }
    }

    // JSON output
    if args.json {
        let report = DoctorReport {
            state_file_exists: true,
            issues_found: issues.len(),
            migrations_needed: migrations_by_type,
            migrations_applied: applied_count,
            dry_run: args.dry_run,
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
    }

    Ok(())
}

#[derive(Serialize)]
struct DoctorReport {
    state_file_exists: bool,
    issues_found: usize,
    migrations_needed: HashMap<String, Vec<String>>,
    migrations_applied: usize,
    dry_run: bool,
}
