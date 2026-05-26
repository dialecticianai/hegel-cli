//! `hegel md` — markdown file tree with DDD artifact awareness.
//!
//! - `scan`: filesystem walk + classification into a flat `MarkdownFile` list
//! - `tree`: build the directory tree model and attach DDD metadata
//! - `render`: terminal tree rendering
//! - `json`: machine-readable output

mod json;
mod render;
mod scan;
mod tree;

use anyhow::Result;

use scan::{scan_markdown_files, FileCategory, MarkdownFile};

/// Arguments for the markdown command
#[derive(Debug, Clone)]
pub struct MarkdownArgs {
    pub json: bool,
    pub no_ddd: bool,
    pub ddd: bool,
}

/// Execute the markdown tree command
pub fn run_markdown(args: MarkdownArgs) -> Result<()> {
    // Scan current directory for markdown files
    let files = scan_markdown_files()?;

    if files.is_empty() {
        println!("No markdown files found in current directory");
        return Ok(());
    }

    // Categorize files
    let (ddd_files, regular_files) = categorize_files(files, args.no_ddd, args.ddd);

    // Output based on mode
    if args.json {
        json::output_json(&ddd_files, &regular_files, args.no_ddd, args.ddd)?;
    } else {
        render::output_tree(&ddd_files, &regular_files, args.no_ddd, args.ddd)?;
    }

    Ok(())
}

/// Categorize files into DDD and regular
fn categorize_files(
    files: Vec<MarkdownFile>,
    no_ddd: bool,
    ddd: bool,
) -> (Vec<MarkdownFile>, Vec<MarkdownFile>) {
    if no_ddd {
        // Show only regular files
        let regular: Vec<_> = files
            .into_iter()
            .filter(|f| matches!(f.category, FileCategory::Regular))
            .collect();
        (vec![], regular)
    } else if ddd {
        // Show only DDD files
        let ddd_files: Vec<_> = files
            .into_iter()
            .filter(|f| matches!(f.category, FileCategory::Ddd { .. }))
            .collect();
        (ddd_files, vec![])
    } else {
        // Show both
        let mut ddd = Vec::new();
        let mut regular = Vec::new();

        for file in files {
            match file.category {
                FileCategory::Ddd { .. } => ddd.push(file),
                FileCategory::Regular => regular.push(file),
            }
        }

        (ddd, regular)
    }
}
