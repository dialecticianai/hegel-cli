use anyhow::{anyhow, Result};
use std::path::PathBuf;

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

/// Refactor artifact with date and name
#[derive(Debug, Clone, PartialEq)]
pub struct RefactorArtifact {
    pub date: String,
    pub name: String,
}

impl RefactorArtifact {
    /// Generate file name with .md extension
    pub fn file_name(&self) -> String {
        format!("{}-{}.md", self.date, self.name)
    }

    /// Generate full file path
    pub fn file_path(&self) -> PathBuf {
        PathBuf::from(".ddd")
            .join("refactor")
            .join(self.file_name())
    }
}

/// Report artifact with date and name
#[derive(Debug, Clone, PartialEq)]
pub struct ReportArtifact {
    pub date: String,
    pub name: String,
}

impl ReportArtifact {
    /// Generate file name with .md extension
    pub fn file_name(&self) -> String {
        format!("{}-{}.md", self.date, self.name)
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
}

/// Result of scanning DDD artifacts
#[derive(Debug)]
pub struct DddScanResult {
    pub artifacts: Vec<DddArtifact>,
    pub issues: Vec<ValidationIssue>,
}

/// Parse feat directory name (format: YYYYMMDD[-N]-name)
pub fn parse_feat_name(name: &str) -> Result<(String, Option<usize>, String)> {
    let parts: Vec<&str> = name.split('-').collect();

    // Must have at least date and name
    if parts.len() < 2 {
        return Err(anyhow!("Invalid feat name format: {}", name));
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

/// Parse single file name (format: YYYYMMDD-name.md)
pub fn parse_single_file_name(name: &str) -> Result<(String, String)> {
    // Remove .md extension
    let name = name
        .strip_suffix(".md")
        .ok_or_else(|| anyhow!("File must have .md extension"))?;

    let parts: Vec<&str> = name.split('-').collect();

    if parts.len() < 2 {
        return Err(anyhow!("Invalid file name format: {}", name));
    }

    let date = parts[0];
    validate_date_format(date)?;

    let name = parts[1..].join("-");
    validate_name_format(&name)?;

    Ok((date.to_string(), name))
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
    fn test_refactor_file_name() {
        let refactor = RefactorArtifact {
            date: "20251104".to_string(),
            name: "large-files".to_string(),
        };

        assert_eq!(refactor.file_name(), "20251104-large-files.md");
    }

    #[test]
    fn test_refactor_file_path() {
        let refactor = RefactorArtifact {
            date: "20251104".to_string(),
            name: "large-files".to_string(),
        };

        assert_eq!(
            refactor.file_path(),
            PathBuf::from(".ddd/refactor/20251104-large-files.md")
        );
    }

    #[test]
    fn test_report_file_name() {
        let report = ReportArtifact {
            date: "20251010".to_string(),
            name: "tui-dep-review".to_string(),
        };

        assert_eq!(report.file_name(), "20251010-tui-dep-review.md");
    }

    #[test]
    fn test_report_file_path() {
        let report = ReportArtifact {
            date: "20251010".to_string(),
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
    fn test_parse_single_file_valid() {
        let (date, name) = parse_single_file_name("20251104-large-files.md").unwrap();
        assert_eq!(date, "20251104");
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
}
