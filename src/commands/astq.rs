use anyhow::{Context, Result};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

/// Run ast-grep with the provided arguments
/// Wrapper optimized for LLM agents - provides clear feedback about results
pub fn run_astq(args: &[String]) -> Result<()> {
    // Build ast-grep from vendor if not already built
    let ast_grep_bin = build_ast_grep()?;

    // Execute ast-grep with all arguments, capturing output
    let mut child = Command::new(&ast_grep_bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to execute ast-grep")?;

    let stdout = child.stdout.take().context("Failed to capture stdout")?;
    let stderr = child.stderr.take().context("Failed to capture stderr")?;

    // Read and display stdout line by line
    let mut stdout_lines = Vec::new();
    for line in BufReader::new(stdout).lines() {
        let line = line?;
        println!("{}", line);
        stdout_lines.push(line);
    }

    // Read and display stderr
    let mut stderr_output = String::new();
    for line in BufReader::new(stderr).lines() {
        let line = line?;
        eprintln!("{}", line);
        stderr_output.push_str(&line);
        stderr_output.push('\n');
    }

    let status = child.wait().context("Failed to wait for ast-grep")?;

    // Provide helpful feedback for LLM agents
    if status.success() && stdout_lines.is_empty() && !stderr_output.contains("ERROR") {
        eprintln!("\n📭 No matches found for this pattern.");
        eprintln!("💡 Debugging tips:");
        eprintln!("   - Use --debug-query=ast to see how ast-grep parses your pattern");
        eprintln!(
            "   - Try simplifying the pattern (e.g., just 'green()' instead of '$X.green()')"
        );
        eprintln!("   - Check if the pattern needs to be a complete statement vs expression");
    }

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

/// Build ast-grep from vendor directory if needed
fn build_ast_grep() -> Result<std::path::PathBuf> {
    build_ast_grep_with_path("vendor/ast-grep")
}

/// Build ast-grep from a custom vendor path (exposed for testing)
fn build_ast_grep_with_path(vendor_path: &str) -> Result<std::path::PathBuf> {
    let vendor_path = std::path::Path::new(vendor_path);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_build_ast_grep_finds_existing_binary() {
        let temp_dir = TempDir::new().unwrap();
        let vendor_path = temp_dir.path().join("vendor-ast-grep");
        let target_dir = vendor_path.join("target/release");
        std::fs::create_dir_all(&target_dir).unwrap();

        let binary_path = target_dir.join("ast-grep");
        std::fs::write(&binary_path, "fake binary").unwrap();

        let result = build_ast_grep_with_path(vendor_path.to_str().unwrap()).unwrap();
        assert_eq!(result, binary_path);
    }

    #[test]
    fn test_build_ast_grep_fails_when_vendor_missing() {
        let result = build_ast_grep_with_path("/nonexistent/path");
        assert!(result.is_err());
    }
}
