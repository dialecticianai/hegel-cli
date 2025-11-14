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

/// Artifact type for path classification
enum ArtifactKind {
    Feat,
    Refactor,
    Report,
}

/// Determine artifact kind from path
fn classify_artifact(path: &PathBuf) -> Option<ArtifactKind> {
    let path_str = path.to_string_lossy();

    // Check for feat directory (multiple possible path formats)
    if path_str.contains("/.ddd/feat/")
        || path_str.contains("\\.ddd\\feat\\")
        || path_str.contains(".ddd/feat/")
        || path_str.contains(".ddd\\feat\\")
    {
        return Some(ArtifactKind::Feat);
    }

    // Check for refactor/report files
    if path_str.contains("/refactor/") || path_str.contains("\\refactor\\") {
        return Some(ArtifactKind::Refactor);
    }
    if path_str.contains("/report/") || path_str.contains("\\report\\") {
        return Some(ArtifactKind::Report);
    }

    None
}

/// Fix InvalidFormat for feat directory (add date prefix and optional index)
fn fix_feat_dir(file_name: &str, git_date: &str) -> Result<String> {
    use crate::ddd::{parse_ddd_structure, DddArtifact};

    let name = file_name.replace('_', "-");
    let scan_result = parse_ddd_structure()?;

    // Count existing feats on this date
    let same_date_count = scan_result
        .artifacts
        .iter()
        .filter_map(|a| match a {
            DddArtifact::Feat(f) if f.date == git_date => Some(f),
            _ => None,
        })
        .count();

    // Add index if there are other feats on this date
    Ok(if same_date_count > 0 {
        format!("{}-{}-{}", git_date, same_date_count + 1, name)
    } else {
        format!("{}-{}", git_date, name)
    })
}

/// Fix InvalidFormat for refactor/report file (normalize formatting)
fn fix_file_artifact(file_name: &str) -> String {
    file_name.replace('_', "-").to_lowercase()
}

/// Suggest a fix for a validation issue
fn suggest_fix(issue: &ValidationIssue) -> Result<Option<(PathBuf, String)>> {
    // If target_name is pre-computed (e.g., MissingIndex), use it
    if let Some(target_name) = &issue.target_name {
        let new_path = issue.path.parent().unwrap().join(target_name);
        return Ok(Some((new_path, target_name.clone())));
    }

    // Only InvalidFormat/MissingDate need dynamic fixes
    if !matches!(
        issue.issue_type,
        IssueType::InvalidFormat | IssueType::MissingDate
    ) {
        return Ok(None);
    }

    // Get file name
    let file_name = issue
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;

    // Determine artifact kind
    let kind = classify_artifact(&issue.path).ok_or_else(|| {
        anyhow::anyhow!("Unknown artifact type for path: {}", issue.path.display())
    })?;

    // Generate new name based on artifact kind
    let new_name = match kind {
        ArtifactKind::Feat => {
            let git_date = lookup_git_date(&issue.path)?
                .ok_or_else(|| anyhow::anyhow!("Cannot determine git date for feat directory"))?;
            fix_feat_dir(file_name, &git_date)?
        }
        ArtifactKind::Refactor | ArtifactKind::Report => fix_file_artifact(file_name),
    };

    // Filter out no-op renames
    if file_name == new_name {
        return Ok(None);
    }

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
