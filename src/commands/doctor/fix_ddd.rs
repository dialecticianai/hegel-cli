use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

use crate::ddd::{parse_ddd_structure, IssueType, ValidationIssue};
use crate::theme::Theme;

/// Get git creation date for a file/directory
fn lookup_git_date(path: &PathBuf) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(&[
            "log",
            "--follow",
            "--format=%ad",
            "--date=short",
            &path.display().to_string(),
        ])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // Get last line (earliest commit)
    if let Some(date_str) = lines.last() {
        // Convert YYYY-MM-DD to YYYYMMDD
        let date = date_str.replace('-', "");
        Ok(Some(date))
    } else {
        Ok(None)
    }
}

/// Check if a file is tracked by git
fn is_tracked(path: &PathBuf) -> Result<bool> {
    let output = Command::new("git")
        .args(&["ls-files", &path.display().to_string()])
        .output()?;

    Ok(output.status.success() && !output.stdout.is_empty())
}

/// Suggest a fix for a validation issue
fn suggest_fix(issue: &ValidationIssue) -> Result<Option<(PathBuf, String)>> {
    // Get git date for the artifact
    let git_date = match lookup_git_date(&issue.path)? {
        Some(date) => date,
        None => {
            // Can't fix without a date
            return Ok(None);
        }
    };

    // Extract the name from the path
    let file_name = issue
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;

    // Build new name based on issue type
    let new_name = match issue.issue_type {
        IssueType::InvalidFormat | IssueType::MissingDate => {
            // The file already has underscores instead of hyphens in the name
            // Just replace underscores with hyphens to match the spec
            // For example: "20251104-1-non_phase_commits" -> "20251104-1-non-phase-commits"
            file_name.replace('_', "-").to_string()
        }
        IssueType::MissingIndex => {
            // For MissingIndex, this would need more complex logic
            // to determine the correct index - skip for now
            return Ok(None);
        }
    };

    // Build new path
    let new_path = issue.path.parent().unwrap().join(&new_name);

    Ok(Some((new_path, new_name)))
}

/// Apply fix by renaming artifact
fn apply_fix(issue: &ValidationIssue, new_path: &PathBuf) -> Result<()> {
    // Check if tracked by git
    if is_tracked(&issue.path)? {
        // Use git mv
        let output = Command::new("git")
            .args(&[
                "mv",
                &issue.path.display().to_string(),
                &new_path.display().to_string(),
            ])
            .output()?;

        if !output.status.success() {
            anyhow::bail!("git mv failed: {}", String::from_utf8_lossy(&output.stderr));
        }
    } else {
        // Use std::fs::rename for untracked files
        std::fs::rename(&issue.path, new_path)?;
    }

    Ok(())
}

/// Check and fix DDD artifacts
pub fn check_and_fix_ddd(apply: bool, json: bool) -> Result<()> {
    // Check for DDD artifact issues
    let ddd_issues = if PathBuf::from(".ddd").exists() {
        match parse_ddd_structure() {
            Ok(scan_result) => scan_result.issues,
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    // Display DDD issues if found
    if !ddd_issues.is_empty() {
        if !json {
            println!("{}", Theme::header("=== DDD Artifact Issues ==="));
            println!();
            println!(
                "{}  Found {} malformed artifact(s):",
                Theme::warning("⚠"),
                ddd_issues.len()
            );
            println!();

            if apply {
                // Apply mode: fix issues
                let mut fixed_count = 0;
                for issue in &ddd_issues {
                    match suggest_fix(issue) {
                        Ok(Some((new_path, _))) => match apply_fix(issue, &new_path) {
                            Ok(()) => {
                                println!(
                                    "{}  {} → {}",
                                    Theme::success("✓"),
                                    issue.path.display(),
                                    new_path.display()
                                );
                                fixed_count += 1;
                            }
                            Err(e) => {
                                println!("{}  {} ({})", Theme::error("✗"), issue.path.display(), e);
                            }
                        },
                        Ok(None) => {
                            println!(
                                "{}  {} (cannot auto-fix)",
                                Theme::warning("⚠"),
                                issue.path.display()
                            );
                        }
                        Err(e) => {
                            println!("{}  {} ({})", Theme::error("✗"), issue.path.display(), e);
                        }
                    }
                }
                println!();
                println!(
                    "{}",
                    Theme::success(&format!("Fixed {} artifact(s)", fixed_count))
                );
            } else {
                // Dry-run mode: show planned fixes
                for (i, issue) in ddd_issues.iter().enumerate() {
                    println!(
                        "  {}. {} ({:?})",
                        i + 1,
                        issue.path.display(),
                        issue.issue_type
                    );

                    // Try to suggest a fix
                    match suggest_fix(issue) {
                        Ok(Some((new_path, _))) => {
                            println!("     → Rename to: {}", new_path.display());
                            if let Ok(Some(date)) = lookup_git_date(&issue.path) {
                                println!("        (added on {})", date);
                            }
                        }
                        Ok(None) => {
                            println!("     → {}", issue.suggested_fix);
                        }
                        Err(_) => {
                            println!("     → {}", issue.suggested_fix);
                        }
                    }
                    println!();
                }

                println!("Run 'hegel doctor --apply' to fix these issues");
            }
            println!();
        }
    }

    Ok(())
}
