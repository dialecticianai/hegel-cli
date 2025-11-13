use anyhow::{bail, Result};
use chrono::Local;
use std::fs;
use std::path::PathBuf;

use crate::ddd::parse_ddd_structure;

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
    // Validate name format
    crate::ddd::validate_name_format(name)?;

    // Get today's date
    let today = Local::now().format("%Y%m%d").to_string();

    // Check if artifact with this name already exists today (any index or no index)
    let scan_result = parse_ddd_structure().unwrap_or_else(|_| crate::ddd::DddScanResult {
        artifacts: Vec::new(),
        issues: Vec::new(),
    });

    for artifact in &scan_result.artifacts {
        if let crate::ddd::DddArtifact::Feat(feat) = artifact {
            if feat.date == today && feat.name == name {
                bail!("Artifact already exists: {}", feat.dir_path().display());
            }
        }
    }

    // Check for existing artifacts today to determine index
    let index = determine_index(&today)?;

    // Build directory name
    let dir_name = if let Some(idx) = index {
        format!("{}-{}-{}", today, idx, name)
    } else {
        format!("{}-{}", today, name)
    };

    let dir_path = PathBuf::from(".ddd/feat").join(&dir_name);

    // Create directory
    fs::create_dir_all(&dir_path)?;

    // Output message
    println!("Created feature directory: {}", dir_path.display());
    println!("Write your planning documents to: {}", dir_path.display());

    Ok(())
}

/// Create a refactor artifact (output path only)
fn create_refactor(name: &str) -> Result<()> {
    // Validate name format
    crate::ddd::validate_name_format(name)?;

    // Get today's date
    let today = Local::now().format("%Y%m%d").to_string();

    // Build file name
    let file_name = format!("{}-{}.md", today, name);
    let file_path = PathBuf::from(".ddd/refactor").join(&file_name);

    // Check for duplicates
    if file_path.exists() {
        bail!("Artifact already exists: {}", file_path.display());
    }

    // Output path (don't create file)
    println!("Write your document to: {}", file_path.display());

    Ok(())
}

/// Create a report artifact (output path only)
fn create_report(name: &str) -> Result<()> {
    // Validate name format
    crate::ddd::validate_name_format(name)?;

    // Get today's date
    let today = Local::now().format("%Y%m%d").to_string();

    // Build file name
    let file_name = format!("{}-{}.md", today, name);
    let file_path = PathBuf::from(".ddd/report").join(&file_name);

    // Check for duplicates
    if file_path.exists() {
        bail!("Artifact already exists: {}", file_path.display());
    }

    // Output path (don't create file)
    println!("Write your document to: {}", file_path.display());

    Ok(())
}

/// Determine the index for a feat artifact on a given date
/// Returns None if this is the first artifact, Some(N) if there are existing artifacts
fn determine_index(date: &str) -> Result<Option<usize>> {
    // Scan existing artifacts
    let scan_result = parse_ddd_structure().unwrap_or_else(|_| crate::ddd::DddScanResult {
        artifacts: Vec::new(),
        issues: Vec::new(),
    });

    // Count feats with this date
    let mut same_day_count = 0;
    for artifact in &scan_result.artifacts {
        if let crate::ddd::DddArtifact::Feat(feat) = artifact {
            if feat.date == date {
                same_day_count += 1;
            }
        }
    }

    // If there are already artifacts today, assign next index
    if same_day_count > 0 {
        Ok(Some(same_day_count + 1))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn test_create_feat_basic() {
        let temp = TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        fs::create_dir_all(".ddd/feat").unwrap();

        let result = create_feat("my-feature");
        assert!(result.is_ok());

        let expected_path = PathBuf::from(".ddd/feat/20251113-my-feature");
        // Date will vary, so just check directory was created
        let entries: Vec<_> = fs::read_dir(".ddd/feat").unwrap().collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    #[serial]
    fn test_create_feat_with_auto_index() {
        let temp = TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
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
        env::set_current_dir(temp.path()).unwrap();
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
        env::set_current_dir(temp.path()).unwrap();
        fs::create_dir_all(".ddd/feat").unwrap();

        let result = create_feat("Invalid_Name");
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_create_refactor_outputs_path() {
        let temp = TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
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
        env::set_current_dir(temp.path()).unwrap();
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
        let result = determine_index("20991231");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    #[serial]
    fn test_determine_index_with_existing() {
        let temp = TempDir::new().unwrap();
        env::set_current_dir(temp.path()).unwrap();
        fs::create_dir_all(".ddd/feat").unwrap();

        // Create artifacts with specific date
        let date = "20251113";
        fs::create_dir_all(format!(".ddd/feat/{}-first", date)).unwrap();
        fs::create_dir_all(format!(".ddd/feat/{}-1-second", date)).unwrap();

        let result = determine_index(date);
        assert!(result.is_ok());
        // Should return index 3 (2 existing + 1)
        assert_eq!(result.unwrap(), Some(3));
    }
}
