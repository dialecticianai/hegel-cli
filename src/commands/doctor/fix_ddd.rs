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
            // Determine if this is a feat directory or refactor/report file
            let is_feat_dir = issue.path.to_string_lossy().contains("/.ddd/feat/");
            let is_refactor = issue.path.to_string_lossy().contains("/.ddd/refactor/");
            let is_report = issue.path.to_string_lossy().contains("/.ddd/report/");

            if is_feat_dir {
                // Feat directory missing date prefix
                // Example: "markdown-review" -> "20251110-markdown-review"
                let name = file_name.replace('_', "-");

                // Check if there are other feats on this date to determine if we need an index
                use crate::ddd::parse_ddd_structure;
                let scan_result = parse_ddd_structure()?;

                let mut same_date_count = 0;
                for artifact in &scan_result.artifacts {
                    if let crate::ddd::DddArtifact::Feat(feat) = artifact {
                        if feat.date == git_date {
                            same_date_count += 1;
                        }
                    }
                }

                // If there are other feats on this date, add index
                if same_date_count > 0 {
                    format!("{}-{}-{}", git_date, same_date_count + 1, name)
                } else {
                    format!("{}-{}", git_date, name)
                }
            } else if is_refactor || is_report {
                // File with underscores or other formatting issues
                // Just replace underscores with hyphens
                // Example: "20251107-e2e-performance-LEARNINGS.md" (has uppercase, already has date)
                file_name.replace('_', "-").to_string()
            } else {
                // Unknown path type
                return Ok(None);
            }
        }
        IssueType::MissingIndex => {
            // Determine the index for this artifact
            // Parse the file name to get date and name
            use crate::ddd::parse_single_file_name;
            let (date, _index, name) = parse_single_file_name(file_name)?;

            // Scan existing artifacts to determine the correct index
            use crate::ddd::parse_ddd_structure;
            let scan_result = parse_ddd_structure()?;

            // Count artifacts of the same type on this date that come before this one alphabetically
            let artifact_type = if issue.path.to_string_lossy().contains("/refactor/") {
                "refactor"
            } else if issue.path.to_string_lossy().contains("/report/") {
                "report"
            } else {
                return Ok(None);
            };

            let mut same_date_artifacts: Vec<String> = Vec::new();
            for artifact in &scan_result.artifacts {
                match (artifact_type, artifact) {
                    ("refactor", crate::ddd::DddArtifact::Refactor(r)) if r.date == date => {
                        same_date_artifacts.push(r.name.clone());
                    }
                    ("report", crate::ddd::DddArtifact::Report(r)) if r.date == date => {
                        same_date_artifacts.push(r.name.clone());
                    }
                    _ => {}
                }
            }

            // Sort alphabetically and find position
            same_date_artifacts.sort();
            let position = same_date_artifacts
                .iter()
                .position(|n| n == &name)
                .unwrap_or(0);

            // Index is position + 1
            let index = position + 1;
            format!("{}-{}-{}.md", date, index, name)
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
