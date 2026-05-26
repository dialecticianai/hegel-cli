use std::path::PathBuf;

/// Specification for an artifact file (e.g., SPEC.md, PLAN.md)
#[derive(Debug, Clone, PartialEq)]
pub struct ArtifactFileSpec {
    pub name: &'static str,
    pub required: bool,
}

/// Build a `{date}[-{index}]-{name}{suffix}` artifact name.
fn dated_name(date: &str, index: Option<usize>, name: &str, suffix: &str) -> String {
    match index {
        Some(idx) => format!("{}-{}-{}{}", date, idx, name, suffix),
        None => format!("{}-{}{}", date, name, suffix),
    }
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
        dated_name(&self.date, self.index, &self.name, "")
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
        dated_name(&self.date, self.index, &self.name, ".md")
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
        dated_name(&self.date, self.index, &self.name, ".md")
    }

    /// Generate full file path
    pub fn file_path(&self) -> PathBuf {
        PathBuf::from(".ddd").join("report").join(self.file_name())
    }
}

/// Toy artifact with sequential number and name (format: toyN_name)
#[derive(Debug, Clone, PartialEq)]
pub struct ToyArtifact {
    pub number: usize,
    pub name: String,
    pub spec_exists: bool,
    pub plan_exists: bool,
    pub learnings_exists: bool,
    pub readme_exists: bool,
}

impl ToyArtifact {
    /// File specifications for toy artifacts
    pub const FILES: &'static [ArtifactFileSpec] = &[
        ArtifactFileSpec {
            name: "SPEC.md",
            required: true,
        },
        ArtifactFileSpec {
            name: "PLAN.md",
            required: true,
        },
        ArtifactFileSpec {
            name: "LEARNINGS.md",
            required: true,
        },
        ArtifactFileSpec {
            name: "README.md",
            required: false,
        },
    ];

    /// Generate directory name (format: toyN_name)
    pub fn dir_name(&self) -> String {
        format!("toy{}_{}", self.number, self.name)
    }

    /// Generate full directory path
    pub fn dir_path(&self) -> PathBuf {
        PathBuf::from(".ddd").join("toys").join(self.dir_name())
    }
}

/// DDD artifact enum wrapping all types
#[derive(Debug, Clone, PartialEq)]
pub enum DddArtifact {
    Feat(FeatArtifact),
    Refactor(RefactorArtifact),
    Report(ReportArtifact),
    Toy(ToyArtifact),
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
#[derive(Debug, Default)]
pub struct DddScanResult {
    pub artifacts: Vec<DddArtifact>,
    pub issues: Vec<ValidationIssue>,
}
