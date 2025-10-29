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

/// Get path to ast-grep binary (built at compile time or from system)
fn build_ast_grep() -> Result<std::path::PathBuf> {
    hegel_ast_grep::get_ast_grep_path()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_ast_grep_returns_valid_path() {
        let result = build_ast_grep();

        // In dev/test builds (no bundle-ast-grep feature), this will:
        // - Succeed if system ast-grep is installed
        // - Fail with helpful message if not installed
        // We don't assert success because CI/dev environments may vary

        if let Ok(path) = result {
            // If it succeeds, path should be absolute
            assert!(path.is_absolute());
        }
        // If it fails, that's also valid (no system ast-grep)
    }
}
