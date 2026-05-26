use super::parse::{parse_feat_name, parse_single_file_name, parse_toy_name, validate_date_format};
use super::*;
use std::path::PathBuf;

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
