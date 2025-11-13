use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

use crate::doctor::{all_migrations, rescue_state_file};
use crate::storage::FileStorage;
use crate::theme::Theme;

#[derive(Serialize)]
pub struct StateReport {
    pub state_file_exists: bool,
    pub issues_found: usize,
    pub migrations_needed: HashMap<String, Vec<String>>,
    pub migrations_applied: usize,
    pub dry_run: bool,
}

/// Check and fix state file issues
pub fn check_and_fix_state(
    storage: &FileStorage,
    apply: bool,
    verbose: bool,
    json: bool,
) -> Result<Option<StateReport>> {
    let state_path = storage.state_dir().join("state.json");

    // Check if state.json exists
    if !state_path.exists() {
        if json {
            let report = StateReport {
                state_file_exists: false,
                issues_found: 0,
                migrations_needed: HashMap::new(),
                migrations_applied: 0,
                dry_run: !apply,
            };
            return Ok(Some(report));
        } else {
            println!(
                "{}",
                Theme::success("No state file found - nothing to check")
            );
        }
        return Ok(None);
    }

    if !json {
        println!(
            "{}",
            Theme::header("=== Hegel Doctor: State Health Check ===")
        );
        println!();

        if !apply {
            println!(
                "{}",
                Theme::warning("Detection mode - use --apply to fix issues")
            );
            println!();
        }
    }

    // Try to rescue corrupted state files first (unless in detection mode)
    let _rescued = if apply {
        match rescue_state_file(storage) {
            Ok(true) => {
                if !json {
                    println!("{}", Theme::success("✓ Rescued corrupted state.json"));
                    println!();
                }
                true
            }
            Ok(false) => false,
            Err(e) => {
                // Rescue failed, but continue - maybe load() will work
                if verbose && !json {
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
            if json {
                let report = StateReport {
                    state_file_exists: true,
                    issues_found: 1,
                    migrations_needed: {
                        let mut m = HashMap::new();
                        m.insert("rescue".to_string(), vec![e.to_string()]);
                        m
                    },
                    migrations_applied: 0,
                    dry_run: !apply,
                };
                return Ok(Some(report));
            } else {
                println!("{} Failed to load state.json", Theme::error("✗"));
                println!();
                println!("Error: {}", e);
                println!();
                if !apply {
                    println!("Run with --apply to attempt automatic repair.");
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
            if verbose && !json {
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
        if json {
            let report = StateReport {
                state_file_exists: true,
                issues_found: 0,
                migrations_needed: HashMap::new(),
                migrations_applied: 0,
                dry_run: !apply,
            };
            return Ok(Some(report));
        } else {
            println!("{}", Theme::success("✓ State file is healthy"));
            println!();
            println!("Run 'hegel status' to verify everything works.");
        }
        return Ok(None);
    }

    // Display issues summary
    if !json {
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
    if apply {
        if !json {
            println!("{}", Theme::secondary("Migrating state.json..."));
        }

        let mut current_state = state;

        for migration in &mut migrations {
            if migration.check(&current_state, storage)?.is_some() {
                current_state = migration.migrate(current_state, storage)?;
                applied_count += 1;

                if verbose && !json {
                    println!("  {} {}", Theme::success("✓"), migration.name());
                }
            }
        }

        // Save the migrated state
        storage.save(&current_state)?;

        if !json {
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
        if !json {
            println!(
                "{}",
                Theme::warning(&format!(
                    "Detection mode: {} migration(s) needed",
                    issues.len()
                ))
            );
            println!("Run with --apply to perform migration.");
        }
    }

    // Return report
    if json {
        let report = StateReport {
            state_file_exists: true,
            issues_found: issues.len(),
            migrations_needed: migrations_by_type,
            migrations_applied: applied_count,
            dry_run: !apply,
        };
        Ok(Some(report))
    } else {
        Ok(None)
    }
}
