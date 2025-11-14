use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Specification for an artifact file (e.g., SPEC.md, PLAN.md)
#[derive(Debug, Clone, PartialEq)]
pub struct ArtifactFileSpec {
    pub name: &'static str,
    pub required: bool,
}

/// Feature artifact with date, optional index, name, and file existence tracking
#[derive(Debug, Clone, PartialEq)]
pub struct FeatArtifact {
    pub date: String,
    pub index: Option<usize>,
    pub name: String,
    pub spec_exists: bool,
    pub plan_exists: bool,
}

impl FeatArtifact {
    /// File specifications for feat artifacts
    pub const FILES: &'static [ArtifactFileSpec] = &[
        ArtifactFileSpec {
            name: "SPEC.md",
            required: true,
        },
        ArtifactFileSpec {
            name: "PLAN.md",
            required: true,
        },
    ];

    /// Generate directory name with optional index
    pub fn dir_name(&self) -> String {
        if let Some(idx) = self.index {
            format!("{}-{}-{}", self.date, idx, self.name)
        } else {
            format!("{}-{}", self.date, self.name)
        }
    }

    /// Generate full directory path
    pub fn dir_path(&self) -> PathBuf {
        PathBuf::from(".ddd").join("feat").join(self.dir_name())
    }
}

/// Refactor artifact with date, optional index, and name
#[derive(Debug, Clone, PartialEq)]
pub struct RefactorArtifact {
    pub date: String,
    pub index: Option<usize>,
    pub name: String,
}

impl RefactorArtifact {
    /// Generate file name with .md extension
    pub fn file_name(&self) -> String {
        if let Some(idx) = self.index {
            format!("{}-{}-{}.md", self.date, idx, self.name)
        } else {
            format!("{}-{}.md", self.date, self.name)
        }
    }

    /// Generate full file path
    pub fn file_path(&self) -> PathBuf {
        PathBuf::from(".ddd")
            .join("refactor")
            .join(self.file_name())
    }
}

/// Report artifact with date, optional index, and name
#[derive(Debug, Clone, PartialEq)]
pub struct ReportArtifact {
    pub date: String,
    pub index: Option<usize>,
    pub name: String,
}

impl ReportArtifact {
    /// Generate file name with .md extension
    pub fn file_name(&self) -> String {
        if let Some(idx) = self.index {
            format!("{}-{}-{}.md", self.date, idx, self.name)
        } else {
            format!("{}-{}.md", self.date, self.name)
        }
    }

    /// Generate full file path
    pub fn file_path(&self) -> PathBuf {
        PathBuf::from(".ddd").join("report").join(self.file_name())
    }
}

/// DDD artifact enum wrapping all types
#[derive(Debug, Clone, PartialEq)]
pub enum DddArtifact {
    Feat(FeatArtifact),
    Refactor(RefactorArtifact),
    Report(ReportArtifact),
}

/// Type of validation issue
#[derive(Debug, Clone, PartialEq)]
pub enum IssueType {
    MissingDate,
    InvalidFormat,
    MissingIndex,
}

/// Validation issue for a malformed artifact
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub path: PathBuf,
    pub issue_type: IssueType,
    pub suggested_fix: String,
    /// For MissingIndex: the target filename with correct index
    pub target_name: Option<String>,
}

/// Result of scanning DDD artifacts
#[derive(Debug)]
pub struct DddScanResult {
    pub artifacts: Vec<DddArtifact>,
    pub issues: Vec<ValidationIssue>,
}

/// Parse name with optional index (format: YYYYMMDD[-N]-name)
/// Returns (date, index, name)
fn parse_name_with_index(parts: &[&str]) -> Result<(String, Option<usize>, String)> {
    // Must have at least date and name
    if parts.len() < 2 {
        return Err(anyhow!("Invalid name format: need at least date and name"));
    }

    // First part must be date (8 digits)
    let date = parts[0];
    validate_date_format(date)?;

    // Check if second part is an index
    if parts.len() >= 3 {
        if let Ok(index) = parts[1].parse::<usize>() {
            // Has index: date-index-name
            let name = parts[2..].join("-");
            validate_name_format(&name)?;
            return Ok((date.to_string(), Some(index), name));
        }
    }

    // No index: date-name
    let name = parts[1..].join("-");
    validate_name_format(&name)?;
    Ok((date.to_string(), None, name))
}

/// Parse feat directory name (format: YYYYMMDD[-N]-name)
pub fn parse_feat_name(name: &str) -> Result<(String, Option<usize>, String)> {
    let parts: Vec<&str> = name.split('-').collect();
    parse_name_with_index(&parts)
}

/// Parse single file name (format: YYYYMMDD[-N]-name.md)
pub fn parse_single_file_name(name: &str) -> Result<(String, Option<usize>, String)> {
    // Remove .md extension
    let name_without_ext = name
        .strip_suffix(".md")
        .ok_or_else(|| anyhow!("File must have .md extension"))?;

    let parts: Vec<&str> = name_without_ext.split('-').collect();
    parse_name_with_index(&parts)
}

/// Validate date format (YYYYMMDD)
pub fn validate_date_format(date: &str) -> Result<()> {
    if date.len() != 8 {
        return Err(anyhow!("Date must be 8 digits (YYYYMMDD), got: {}", date));
    }

    if !date.chars().all(|c| c.is_ascii_digit()) {
        return Err(anyhow!("Date must contain only digits, got: {}", date));
    }

    Ok(())
}

/// Validate name format (lowercase with hyphens)
pub fn validate_name_format(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Name cannot be empty"));
    }

    if name.starts_with('-') || name.ends_with('-') {
        return Err(anyhow!("Name cannot start or end with hyphen: {}", name));
    }

    for ch in name.chars() {
        if !ch.is_lowercase() && !ch.is_ascii_digit() && ch != '-' {
            return Err(anyhow!(
                "Name must be lowercase with hyphens, got: {}",
                name
            ));
        }
    }

    Ok(())
}

/// Parse DDD artifacts from .ddd/ directory structure
/// This doesn't re-scan - it parses existing directory/file names
///
/// # Arguments
/// * `root_dir` - Optional root directory (defaults to current directory)
pub fn parse_ddd_structure_in(root_dir: Option<&std::path::Path>) -> Result<DddScanResult> {
    let mut artifacts = Vec::new();
    let mut issues = Vec::new();

    let root = root_dir.unwrap_or_else(|| std::path::Path::new("."));

    // Parse feat/ directories
    let feat_dir = root.join(".ddd/feat");
    if feat_dir.exists() {
        for entry in std::fs::read_dir(&feat_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let dir_name = entry.file_name().to_string_lossy().to_string();

            match parse_feat_name(&dir_name) {
                Ok((date, index, name)) => {
                    let dir_path = entry.path();
                    let spec_exists = dir_path.join("SPEC.md").exists();
                    let plan_exists = dir_path.join("PLAN.md").exists();

                    artifacts.push(DddArtifact::Feat(FeatArtifact {
                        date,
                        index,
                        name,
                        spec_exists,
                        plan_exists,
                    }));
                }
                Err(_) => {
                    issues.push(ValidationIssue {
                        path: entry.path(),
                        issue_type: IssueType::InvalidFormat,
                        suggested_fix: format!("Rename {} to YYYYMMDD[-N]-name format", dir_name),
                        target_name: None,
                    });
                }
            }
        }
    }

    // Parse refactor/ files
    let refactor_dir = root.join(".ddd/refactor");
    if refactor_dir.exists() {
        for entry in std::fs::read_dir(&refactor_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy().to_string();

            match parse_single_file_name(&file_name) {
                Ok((date, index, name)) => {
                    artifacts.push(DddArtifact::Refactor(RefactorArtifact {
                        date,
                        index,
                        name,
                    }));
                }
                Err(_) => {
                    issues.push(ValidationIssue {
                        path: entry.path(),
                        issue_type: IssueType::InvalidFormat,
                        suggested_fix: format!(
                            "Rename {} to YYYYMMDD[-N]-name.md format",
                            file_name
                        ),
                        target_name: None,
                    });
                }
            }
        }
    }

    // Parse report/ files
    let report_dir = root.join(".ddd/report");
    if report_dir.exists() {
        for entry in std::fs::read_dir(&report_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy().to_string();

            match parse_single_file_name(&file_name) {
                Ok((date, index, name)) => {
                    artifacts.push(DddArtifact::Report(ReportArtifact { date, index, name }));
                }
                Err(_) => {
                    issues.push(ValidationIssue {
                        path: entry.path(),
                        issue_type: IssueType::InvalidFormat,
                        suggested_fix: format!(
                            "Rename {} to YYYYMMDD[-N]-name.md format",
                            file_name
                        ),
                        target_name: None,
                    });
                }
            }
        }
    }

    // Detect missing indexes (multiple artifacts on same date without indexes)
    detect_missing_indexes(&artifacts, &mut issues);

    Ok(DddScanResult { artifacts, issues })
}

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
fn detect_missing_indexes(artifacts: &[DddArtifact], issues: &mut Vec<ValidationIssue>) {
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

/// Parse DDD artifacts from .ddd/ in current directory
pub fn parse_ddd_structure() -> Result<DddScanResult> {
    parse_ddd_structure_in(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feat_dir_name_without_index() {
        let feat = FeatArtifact {
            date: "20251104".to_string(),
            index: None,
            name: "my-feature".to_string(),
            spec_exists: false,
            plan_exists: false,
        };

        assert_eq!(feat.dir_name(), "20251104-my-feature");
    }

    #[test]
    fn test_feat_dir_name_with_index() {
        let feat = FeatArtifact {
            date: "20251104".to_string(),
            index: Some(1),
            name: "my-feature".to_string(),
            spec_exists: false,
            plan_exists: false,
        };

        assert_eq!(feat.dir_name(), "20251104-1-my-feature");
    }

    #[test]
    fn test_feat_dir_path() {
        let feat = FeatArtifact {
            date: "20251104".to_string(),
            index: Some(2),
            name: "another-feature".to_string(),
            spec_exists: true,
            plan_exists: true,
        };

        assert_eq!(
            feat.dir_path(),
            PathBuf::from(".ddd/feat/20251104-2-another-feature")
        );
    }

    #[test]
    fn test_refactor_file_name_without_index() {
        let refactor = RefactorArtifact {
            date: "20251104".to_string(),
            index: None,
            name: "large-files".to_string(),
        };

        assert_eq!(refactor.file_name(), "20251104-large-files.md");
    }

    #[test]
    fn test_refactor_file_name_with_index() {
        let refactor = RefactorArtifact {
            date: "20251104".to_string(),
            index: Some(2),
            name: "large-files".to_string(),
        };

        assert_eq!(refactor.file_name(), "20251104-2-large-files.md");
    }

    #[test]
    fn test_refactor_file_path() {
        let refactor = RefactorArtifact {
            date: "20251104".to_string(),
            index: None,
            name: "large-files".to_string(),
        };

        assert_eq!(
            refactor.file_path(),
            PathBuf::from(".ddd/refactor/20251104-large-files.md")
        );
    }

    #[test]
    fn test_report_file_name_without_index() {
        let report = ReportArtifact {
            date: "20251010".to_string(),
            index: None,
            name: "tui-dep-review".to_string(),
        };

        assert_eq!(report.file_name(), "20251010-tui-dep-review.md");
    }

    #[test]
    fn test_report_file_name_with_index() {
        let report = ReportArtifact {
            date: "20251010".to_string(),
            index: Some(1),
            name: "tui-dep-review".to_string(),
        };

        assert_eq!(report.file_name(), "20251010-1-tui-dep-review.md");
    }

    #[test]
    fn test_report_file_path() {
        let report = ReportArtifact {
            date: "20251010".to_string(),
            index: None,
            name: "tui-dep-review".to_string(),
        };

        assert_eq!(
            report.file_path(),
            PathBuf::from(".ddd/report/20251010-tui-dep-review.md")
        );
    }

    #[test]
    fn test_parse_feat_with_index() {
        let (date, index, name) = parse_feat_name("20251104-1-non-phase-commits").unwrap();
        assert_eq!(date, "20251104");
        assert_eq!(index, Some(1));
        assert_eq!(name, "non-phase-commits");
    }

    #[test]
    fn test_parse_feat_without_index() {
        let (date, index, name) = parse_feat_name("20251104-my-feature").unwrap();
        assert_eq!(date, "20251104");
        assert_eq!(index, None);
        assert_eq!(name, "my-feature");
    }

    #[test]
    fn test_parse_feat_multi_hyphen_name() {
        let (date, index, name) = parse_feat_name("20251104-multi-word-feature-name").unwrap();
        assert_eq!(date, "20251104");
        assert_eq!(index, None);
        assert_eq!(name, "multi-word-feature-name");
    }

    #[test]
    fn test_parse_feat_invalid_missing_date() {
        let result = parse_feat_name("my-feature");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_feat_invalid_date_format() {
        let result = parse_feat_name("2025-11-04-feature");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_single_file_without_index() {
        let (date, index, name) = parse_single_file_name("20251104-large-files.md").unwrap();
        assert_eq!(date, "20251104");
        assert_eq!(index, None);
        assert_eq!(name, "large-files");
    }

    #[test]
    fn test_parse_single_file_with_index() {
        let (date, index, name) = parse_single_file_name("20251104-2-large-files.md").unwrap();
        assert_eq!(date, "20251104");
        assert_eq!(index, Some(2));
        assert_eq!(name, "large-files");
    }

    #[test]
    fn test_parse_single_file_missing_extension() {
        let result = parse_single_file_name("20251104-large-files");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_single_file_invalid_date() {
        let result = parse_single_file_name("bad-date-name.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_date_format_valid() {
        assert!(validate_date_format("20251104").is_ok());
    }

    #[test]
    fn test_validate_date_format_too_short() {
        assert!(validate_date_format("2025110").is_err());
    }

    #[test]
    fn test_validate_date_format_too_long() {
        assert!(validate_date_format("202511040").is_err());
    }

    #[test]
    fn test_validate_date_format_non_digits() {
        assert!(validate_date_format("2025-1-4").is_err());
    }

    #[test]
    fn test_validate_name_format_valid() {
        assert!(validate_name_format("my-feature").is_ok());
        assert!(validate_name_format("feature-2").is_ok());
    }

    #[test]
    fn test_validate_name_format_empty() {
        assert!(validate_name_format("").is_err());
    }

    #[test]
    fn test_validate_name_format_trailing_hyphen() {
        assert!(validate_name_format("my-feature-").is_err());
    }

    #[test]
    fn test_validate_name_format_uppercase() {
        assert!(validate_name_format("My-Feature").is_err());
    }

    #[test]
    fn test_parse_empty_ddd() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();

        // Create .ddd directory structure (empty)
        std::fs::create_dir_all(temp_dir.path().join(".ddd/feat")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join(".ddd/refactor")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join(".ddd/report")).unwrap();

        let result = parse_ddd_structure_in(Some(temp_dir.path())).unwrap();
        assert_eq!(result.artifacts.len(), 0);
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_parse_valid_feat() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();

        // Create feat directory with SPEC.md
        std::fs::create_dir_all(temp_dir.path().join(".ddd/feat/20251104-my-feature")).unwrap();
        std::fs::write(
            temp_dir
                .path()
                .join(".ddd/feat/20251104-my-feature/SPEC.md"),
            "test",
        )
        .unwrap();

        let result = parse_ddd_structure_in(Some(temp_dir.path())).unwrap();
        assert_eq!(result.artifacts.len(), 1);

        if let DddArtifact::Feat(feat) = &result.artifacts[0] {
            assert_eq!(feat.date, "20251104");
            assert_eq!(feat.name, "my-feature");
            assert!(feat.spec_exists);
            assert!(!feat.plan_exists);
        } else {
            panic!("Expected Feat artifact");
        }
    }

    #[test]
    fn test_parse_invalid_feat() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();

        // Create feat directory with invalid name (missing date)
        std::fs::create_dir_all(temp_dir.path().join(".ddd/feat/my-feature")).unwrap();

        let result = parse_ddd_structure_in(Some(temp_dir.path())).unwrap();
        assert_eq!(result.artifacts.len(), 0);
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].issue_type, IssueType::InvalidFormat);
    }
}
