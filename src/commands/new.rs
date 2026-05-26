use anyhow::{bail, Result};
use chrono::Local;
use std::fs;

use crate::ddd::{
    parse_ddd_structure, DddArtifact, FeatArtifact, RefactorArtifact, ReportArtifact,
};

/// Arguments for the new command
#[derive(Debug, Clone)]
pub struct NewArgs {
    pub artifact_type: ArtifactType,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum ArtifactType {
    Feat,
    Refactor,
    Report,
}

/// Execute the new artifact command
pub fn run_new(args: NewArgs) -> Result<()> {
    match args.artifact_type {
        ArtifactType::Feat => create_feat(&args.name),
        ArtifactType::Refactor => create_refactor(&args.name),
        ArtifactType::Report => create_report(&args.name),
    }
}

/// Create a feat directory
fn create_feat(name: &str) -> Result<()> {
    crate::ddd::validate_name_format(name)?;
    let today = Local::now().format("%Y%m%d").to_string();

    // Reject a same-name artifact already created today (any index)
    let scan_result = parse_ddd_structure().unwrap_or_default();
    for artifact in &scan_result.artifacts {
        if let DddArtifact::Feat(feat) = artifact {
            if feat.date == today && feat.name == name {
                bail!("Artifact already exists: {}", feat.dir_path().display());
            }
        }
    }

    let index = determine_index_for_artifact(&today, "feat")?;
    let dir_path = FeatArtifact {
        date: today,
        index,
        name: name.to_string(),
        spec_exists: false,
        plan_exists: false,
    }
    .dir_path();

    fs::create_dir_all(&dir_path)?;
    println!("Created feature directory: {}", dir_path.display());
    println!("Write your planning documents to: {}", dir_path.display());

    Ok(())
}

/// Create a refactor artifact (output path only)
fn create_refactor(name: &str) -> Result<()> {
    crate::ddd::validate_name_format(name)?;
    let today = Local::now().format("%Y%m%d").to_string();

    let scan_result = parse_ddd_structure().unwrap_or_default();
    for artifact in &scan_result.artifacts {
        if let DddArtifact::Refactor(refactor) = artifact {
            if refactor.date == today && refactor.name == name {
                bail!(
                    "Artifact already exists: {}",
                    refactor.file_path().display()
                );
            }
        }
    }

    let index = determine_index_for_artifact(&today, "refactor")?;
    let file_path = RefactorArtifact {
        date: today,
        index,
        name: name.to_string(),
    }
    .file_path();

    // Output path (don't create file)
    println!("Write your document to: {}", file_path.display());

    Ok(())
}

/// Create a report artifact (output path only)
fn create_report(name: &str) -> Result<()> {
    crate::ddd::validate_name_format(name)?;
    let today = Local::now().format("%Y%m%d").to_string();

    let scan_result = parse_ddd_structure().unwrap_or_default();
    for artifact in &scan_result.artifacts {
        if let DddArtifact::Report(report) = artifact {
            if report.date == today && report.name == name {
                bail!("Artifact already exists: {}", report.file_path().display());
            }
        }
    }

    let index = determine_index_for_artifact(&today, "report")?;
    let file_path = ReportArtifact {
        date: today,
        index,
        name: name.to_string(),
    }
    .file_path();

    // Output path (don't create file)
    println!("Write your document to: {}", file_path.display());

    Ok(())
}

/// Determine the next index for an artifact of `artifact_type` on `date`.
/// Returns None if this is the first artifact, Some(N) if there are existing ones.
fn determine_index_for_artifact(date: &str, artifact_type: &str) -> Result<Option<usize>> {
    let scan_result = parse_ddd_structure().unwrap_or_default();

    let same_day_count = scan_result
        .artifacts
        .iter()
        .filter(|artifact| match (artifact_type, artifact) {
            ("feat", DddArtifact::Feat(a)) => a.date == date,
            ("refactor", DddArtifact::Refactor(a)) => a.date == date,
            ("report", DddArtifact::Report(a)) => a.date == date,
            _ => false,
        })
        .count();

    if same_day_count > 0 {
        Ok(Some(same_day_count + 1))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::DirGuard;
    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn test_create_feat_basic() {
        let temp = TempDir::new().unwrap();
        let _guard = DirGuard::new(temp.path()).unwrap();
        fs::create_dir_all(".ddd/feat").unwrap();

        let result = create_feat("my-feature");
        assert!(result.is_ok());

        // Date will vary, so just check the directory was created
        let entries: Vec<_> = fs::read_dir(".ddd/feat").unwrap().collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    #[serial]
    fn test_create_feat_with_auto_index() {
        let temp = TempDir::new().unwrap();
        let _guard = DirGuard::new(temp.path()).unwrap();
        fs::create_dir_all(".ddd/feat").unwrap();

        // Create first feat
        create_feat("first-feature").unwrap();

        // Create second feat (should get index)
        create_feat("second-feature").unwrap();

        let entries: Vec<_> = fs::read_dir(".ddd/feat").unwrap().collect();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    #[serial]
    fn test_create_feat_duplicate_error() {
        let temp = TempDir::new().unwrap();
        let _guard = DirGuard::new(temp.path()).unwrap();
        fs::create_dir_all(".ddd/feat").unwrap();

        create_feat("my-feature").unwrap();
        let result = create_feat("my-feature");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    #[serial]
    fn test_create_feat_invalid_name() {
        let temp = TempDir::new().unwrap();
        let _guard = DirGuard::new(temp.path()).unwrap();
        fs::create_dir_all(".ddd/feat").unwrap();

        let result = create_feat("Invalid_Name");
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_create_refactor_outputs_path() {
        let temp = TempDir::new().unwrap();
        let _guard = DirGuard::new(temp.path()).unwrap();
        fs::create_dir_all(".ddd/refactor").unwrap();

        let result = create_refactor("my-refactor");
        assert!(result.is_ok());

        // Should NOT create the file
        let entries: Vec<_> = fs::read_dir(".ddd/refactor").unwrap().collect();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    #[serial]
    fn test_create_report_outputs_path() {
        let temp = TempDir::new().unwrap();
        let _guard = DirGuard::new(temp.path()).unwrap();
        fs::create_dir_all(".ddd/report").unwrap();

        let result = create_report("my-report");
        assert!(result.is_ok());

        // Should NOT create the file
        let entries: Vec<_> = fs::read_dir(".ddd/report").unwrap().collect();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    #[serial]
    fn test_determine_index_no_existing() {
        let result = determine_index_for_artifact("20991231", "feat");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    #[serial]
    fn test_determine_index_with_existing() {
        let temp = TempDir::new().unwrap();
        let _guard = DirGuard::new(temp.path()).unwrap();
        fs::create_dir_all(".ddd/feat").unwrap();

        // Create artifacts with specific date
        let date = "20251113";
        fs::create_dir_all(format!(".ddd/feat/{}-first", date)).unwrap();
        fs::create_dir_all(format!(".ddd/feat/{}-1-second", date)).unwrap();

        let result = determine_index_for_artifact(date, "feat");
        assert!(result.is_ok());
        // Should return index 3 (2 existing + 1)
        assert_eq!(result.unwrap(), Some(3));
    }
}
