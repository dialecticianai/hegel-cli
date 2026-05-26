use anyhow::Result;

use super::index::detect_missing_indexes;
use super::parse::{parse_feat_name, parse_single_file_name, parse_toy_name};
use super::types::{
    DddArtifact, DddScanResult, FeatArtifact, IssueType, RefactorArtifact, ReportArtifact,
    ToyArtifact, ValidationIssue,
};

/// Whether a scanned `.ddd/<subdir>` holds artifact directories or files.
enum EntryKind {
    Dir,
    File,
}

/// Scan one `.ddd/<subdir>`, parsing each matching entry into an artifact.
///
/// `parse` returns `Some(artifact)` for a valid entry name (it receives the
/// entry's path for any file-existence checks) or `None` for an unparseable
/// name, which is recorded as an `InvalidFormat` issue suggesting `valid_fmt`.
fn scan_artifact_dir(
    root: &std::path::Path,
    subdir: &str,
    kind: EntryKind,
    valid_fmt: &str,
    artifacts: &mut Vec<DddArtifact>,
    issues: &mut Vec<ValidationIssue>,
    mut parse: impl FnMut(&str, &std::path::Path) -> Option<DddArtifact>,
) -> Result<()> {
    let dir = root.join(".ddd").join(subdir);
    if !dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let matches = match kind {
            EntryKind::Dir => entry.file_type()?.is_dir(),
            EntryKind::File => entry.file_type()?.is_file(),
        };
        if !matches {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        match parse(&name, &entry.path()) {
            Some(artifact) => artifacts.push(artifact),
            None => issues.push(ValidationIssue {
                path: entry.path(),
                issue_type: IssueType::InvalidFormat,
                suggested_fix: format!("Rename {} to {}", name, valid_fmt),
                target_name: None,
            }),
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

    scan_artifact_dir(
        root,
        "feat",
        EntryKind::Dir,
        "YYYYMMDD[-N]-name format",
        &mut artifacts,
        &mut issues,
        |name, path| {
            let (date, index, name) = parse_feat_name(name).ok()?;
            Some(DddArtifact::Feat(FeatArtifact {
                date,
                index,
                name,
                spec_exists: path.join("SPEC.md").exists(),
                plan_exists: path.join("PLAN.md").exists(),
            }))
        },
    )?;

    scan_artifact_dir(
        root,
        "refactor",
        EntryKind::File,
        "YYYYMMDD[-N]-name.md format",
        &mut artifacts,
        &mut issues,
        |name, _| {
            let (date, index, name) = parse_single_file_name(name).ok()?;
            Some(DddArtifact::Refactor(RefactorArtifact {
                date,
                index,
                name,
            }))
        },
    )?;

    scan_artifact_dir(
        root,
        "report",
        EntryKind::File,
        "YYYYMMDD[-N]-name.md format",
        &mut artifacts,
        &mut issues,
        |name, _| {
            let (date, index, name) = parse_single_file_name(name).ok()?;
            Some(DddArtifact::Report(ReportArtifact { date, index, name }))
        },
    )?;

    scan_artifact_dir(
        root,
        "toys",
        EntryKind::Dir,
        "toyN_name format",
        &mut artifacts,
        &mut issues,
        |name, path| {
            let (number, name) = parse_toy_name(name).ok()?;
            Some(DddArtifact::Toy(ToyArtifact {
                number,
                name,
                spec_exists: path.join("SPEC.md").exists(),
                plan_exists: path.join("PLAN.md").exists(),
                learnings_exists: path.join("LEARNINGS.md").exists(),
                readme_exists: path.join("README.md").exists(),
            }))
        },
    )?;

    // Detect missing indexes (multiple artifacts on same date without indexes)
    detect_missing_indexes(&artifacts, &mut issues);

    Ok(DddScanResult { artifacts, issues })
}
