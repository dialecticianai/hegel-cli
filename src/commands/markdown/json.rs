//! JSON output for the markdown tree.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::ddd::parse_ddd_structure;

use super::scan::{FileCategory, MarkdownFile};

/// JSON output schema
#[derive(Debug, Serialize, Deserialize)]
struct MarkdownTree {
    #[serde(skip_serializing_if = "Option::is_none")]
    ddd_documents: Option<Vec<FileEntry>>,
    other_markdown: Vec<FileEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    validation_issues: Option<Vec<ValidationIssueEntry>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileEntry {
    path: String,
    lines: usize,
    size_bytes: u64,
    last_modified: String,
    ephemeral: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidationIssueEntry {
    path: String,
    issue_type: String,
    suggested_fix: String,
}

/// Output as JSON
pub(crate) fn output_json(
    ddd_files: &[MarkdownFile],
    regular_files: &[MarkdownFile],
    no_ddd: bool,
    ddd: bool,
) -> Result<()> {
    let ddd_documents = if no_ddd || ddd_files.is_empty() {
        None
    } else {
        Some(
            ddd_files
                .iter()
                .map(|f| FileEntry {
                    path: f.path.display().to_string(),
                    lines: f.lines.unwrap_or(0),
                    size_bytes: f.size_bytes,
                    last_modified: f.last_modified.to_rfc3339(),
                    ephemeral: matches!(f.category, FileCategory::Ddd { ephemeral: true }),
                })
                .collect(),
        )
    };

    let other_markdown = if ddd {
        vec![]
    } else {
        regular_files
            .iter()
            .map(|f| FileEntry {
                path: f.path.display().to_string(),
                lines: f.lines.unwrap_or(0),
                size_bytes: f.size_bytes,
                last_modified: f.last_modified.to_rfc3339(),
                ephemeral: false,
            })
            .collect()
    };

    // Get validation issues from DDD scan
    let validation_issues = if !no_ddd {
        parse_ddd_structure().ok().and_then(|scan_result| {
            if scan_result.issues.is_empty() {
                None
            } else {
                Some(
                    scan_result
                        .issues
                        .iter()
                        .map(|issue| ValidationIssueEntry {
                            path: issue.path.display().to_string(),
                            issue_type: format!("{:?}", issue.issue_type),
                            suggested_fix: issue.suggested_fix.clone(),
                        })
                        .collect(),
                )
            }
        })
    } else {
        None
    };

    let tree = MarkdownTree {
        ddd_documents,
        other_markdown,
        validation_issues,
    };

    let json = serde_json::to_string_pretty(&tree)?;
    println!("{}", json);

    Ok(())
}
