use anyhow::{Context, Result};
use std::process::Command;

/// Run ast-grep with the provided arguments
pub fn run_astq(args: &[String]) -> Result<()> {
    // Build ast-grep from vendor if not already built
    let ast_grep_bin = build_ast_grep()?;

    // Execute ast-grep with all arguments passed through
    let status = Command::new(&ast_grep_bin)
        .args(args)
        .status()
        .context("Failed to execute ast-grep")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Build ast-grep from vendor directory if needed
fn build_ast_grep() -> Result<std::path::PathBuf> {
    let vendor_path = std::path::Path::new("vendor/ast-grep");
    let target_bin = vendor_path.join("target/release/ast-grep");

    // Check if binary exists
    if target_bin.exists() {
        return Ok(target_bin);
    }

    // Build if not exists
    eprintln!("Building ast-grep from vendor (first run only)...");
    let status = Command::new("cargo")
        .args(&["build", "--release", "--package", "ast-grep"])
        .current_dir(vendor_path)
        .status()
        .context("Failed to build ast-grep")?;

    if !status.success() {
        anyhow::bail!("Failed to build ast-grep");
    }

    Ok(target_bin)
}
