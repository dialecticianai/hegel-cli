//! DDD artifact management: parsing, validation, and scanning of `.ddd/`.
//!
//! - `types`: artifact structs/enums + path/name helpers
//! - `parse`: name parsing + format validation
//! - `scan`: directory scanning into a `DddScanResult`
//! - `index`: same-day index disambiguation (git-timestamp ordering)

mod index;
mod parse;
mod scan;
mod types;

pub use parse::validate_name_format;
pub use scan::parse_ddd_structure_in;
pub use types::{
    DddArtifact, DddScanResult, FeatArtifact, IssueType, RefactorArtifact, ReportArtifact,
    ToyArtifact, ValidationIssue,
};

use anyhow::Result;

/// Parse DDD artifacts from `.ddd/` in the current directory.
pub fn parse_ddd_structure() -> Result<DddScanResult> {
    parse_ddd_structure_in(None)
}

#[cfg(test)]
mod tests;
