use anyhow::Result;
use std::process::Command;

/// Get path to ast-grep binary (built at compile time or from system)
pub fn get_ast_grep_path() -> Result<std::path::PathBuf> {
    // The binary path is set by build.rs at compile time
    let bin_path = env!("AST_GREP_BIN_PATH");

    // If bundled (feature enabled), use the compiled binary
    if !bin_path.is_empty() {
        let path = std::path::PathBuf::from(bin_path);
        if !path.exists() {
            anyhow::bail!(
                "Bundled ast-grep binary not found at {}. This should have been built during compilation.",
                bin_path
            );
        }
        return Ok(path);
    }

    // Otherwise, fall back to system ast-grep (dev/test builds)
    // Check if ast-grep is installed on PATH
    if let Ok(output) = Command::new("which").arg("ast-grep").output() {
        if output.status.success() {
            let system_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !system_path.is_empty() {
                return Ok(std::path::PathBuf::from(system_path));
            }
        }
    }

    anyhow::bail!(
        "ast-grep not found. Either:\n\
         1. Install ast-grep: cargo install ast-grep-cli\n\
         2. Or build with bundled binary: cargo build --features bundle-ast-grep"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ast_grep_path_returns_valid_path() {
        let result = get_ast_grep_path();

        // In dev/test builds (no bundle feature), this will:
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
