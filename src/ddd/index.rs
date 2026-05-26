use std::path::PathBuf;

use super::types::{DddArtifact, IssueType, RefactorArtifact, ReportArtifact, ValidationIssue};

/// Get git creation timestamp (seconds since epoch) for a file
fn get_git_timestamp(path: &std::path::Path) -> Option<i64> {
    use std::process::Command;

    let output = Command::new("git")
        .args(&[
            "log",
            "--follow",
            "--format=%at",
            "--diff-filter=A",
            &path.display().to_string(),
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().last()?.parse().ok()
}

/// Trait for artifacts that can have optional indexes
trait IndexableArtifact {
    fn date(&self) -> &str;
    fn name(&self) -> &str;
    fn index(&self) -> Option<usize>;
    fn file_path(&self) -> PathBuf;
}

impl IndexableArtifact for RefactorArtifact {
    fn date(&self) -> &str {
        &self.date
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn index(&self) -> Option<usize> {
        self.index
    }
    fn file_path(&self) -> PathBuf {
        RefactorArtifact::file_path(self)
    }
}

impl IndexableArtifact for ReportArtifact {
    fn date(&self) -> &str {
        &self.date
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn index(&self) -> Option<usize> {
        self.index
    }
    fn file_path(&self) -> PathBuf {
        ReportArtifact::file_path(self)
    }
}

/// Detect and assign indexes for artifacts on the same date (generic helper)
fn detect_missing_indexes_for_type<T: IndexableArtifact>(
    artifacts: &[&T],
    issues: &mut Vec<ValidationIssue>,
) {
    use std::collections::HashMap;

    // Group by date
    let mut by_date: HashMap<&str, Vec<&T>> = HashMap::new();
    for artifact in artifacts {
        by_date.entry(artifact.date()).or_default().push(*artifact);
    }

    // Process each date that has multiple artifacts
    for (date, mut group) in by_date {
        if group.len() <= 1 {
            continue;
        }

        // Sort by git timestamp (chronological order)
        group.sort_by_key(|a| get_git_timestamp(&a.file_path()).unwrap_or(i64::MAX));

        // Assign indexes chronologically to those missing them
        for (idx, artifact) in group.iter().enumerate() {
            if artifact.index().is_none() {
                let target_index = idx + 1;
                let target_name = format!("{}-{}-{}.md", date, target_index, artifact.name());

                issues.push(ValidationIssue {
                    path: artifact.file_path(),
                    issue_type: IssueType::MissingIndex,
                    suggested_fix: format!(
                        "Add index to disambiguate from other {} artifacts",
                        date
                    ),
                    target_name: Some(target_name),
                });
            }
        }
    }
}

/// Detect artifacts that need indexes (multiple artifacts on same date without indexes)
pub(crate) fn detect_missing_indexes(artifacts: &[DddArtifact], issues: &mut Vec<ValidationIssue>) {
    // Collect refactor artifacts
    let refactors: Vec<&RefactorArtifact> = artifacts
        .iter()
        .filter_map(|a| match a {
            DddArtifact::Refactor(r) => Some(r),
            _ => None,
        })
        .collect();
    detect_missing_indexes_for_type(&refactors, issues);

    // Collect report artifacts
    let reports: Vec<&ReportArtifact> = artifacts
        .iter()
        .filter_map(|a| match a {
            DddArtifact::Report(r) => Some(r),
            _ => None,
        })
        .collect();
    detect_missing_indexes_for_type(&reports, issues);
}
