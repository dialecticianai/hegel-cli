use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Configuration for finding and executing external binaries
pub struct ExternalBinary {
    /// Name of the binary (e.g., "mirror", "hegel-pm")
    pub name: &'static str,
    /// Relative path from hegel-cli to the adjacent repo (for development)
    pub adjacent_repo_path: &'static str,
    /// User-friendly build instructions
    pub build_instructions: &'static str,
}

/// Configuration for finding and executing external npm packages
pub struct ExternalNpmPackage {
    /// Name of the npm package (e.g., "hegel-ide")
    pub name: &'static str,
    /// Relative path from hegel-cli to the adjacent repo (for development)
    pub adjacent_repo_path: &'static str,
    /// User-friendly build instructions
    pub build_instructions: &'static str,
}

impl ExternalBinary {
    /// Find the binary in known locations
    pub fn find(&self) -> Result<PathBuf> {
        // Check common locations
        let candidates = vec![
            // Adjacent repo (development)
            format!("{}/target/release/{}", self.adjacent_repo_path, self.name),
            // System PATH
            self.name.to_string(),
        ];

        for candidate in &candidates {
            let path = std::path::Path::new(candidate);
            if path.exists() {
                return Ok(path.to_path_buf());
            }
        }

        // Try to find in PATH
        if let Ok(output) = Command::new("which").arg(self.name).output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                let path = path_str.trim();
                if !path.is_empty() {
                    return Ok(PathBuf::from(path));
                }
            }
        }

        anyhow::bail!(
            "{} binary not found. {}",
            self.name,
            self.build_instructions
        )
    }

    /// Execute the binary with the given arguments
    pub fn execute(&self, args: &[String]) -> Result<()> {
        let bin_path = self.find()?;

        let mut cmd = Command::new(&bin_path);
        cmd.args(args);

        // Pass through HEGEL_SESSION_ID if present
        if let Ok(session_id) = std::env::var("HEGEL_SESSION_ID") {
            cmd.env("HEGEL_SESSION_ID", session_id);
        }

        let status = cmd
            .status()
            .with_context(|| format!("Failed to execute {}", self.name))?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }

        Ok(())
    }
}

impl ExternalNpmPackage {
    /// Execute the npm package with the given arguments
    pub fn execute(&self, args: &[String]) -> Result<()> {
        // Try adjacent repo first (development mode)
        let adjacent_package_json = format!("{}/package.json", self.adjacent_repo_path);
        if std::path::Path::new(&adjacent_package_json).exists() {
            let mut cmd = Command::new("npm");
            cmd.arg("start")
                .arg("--prefix")
                .arg(self.adjacent_repo_path);

            if !args.is_empty() {
                cmd.arg("--").args(args);
            }

            // Pass through HEGEL_SESSION_ID if present
            if let Ok(session_id) = std::env::var("HEGEL_SESSION_ID") {
                cmd.env("HEGEL_SESSION_ID", session_id);
            }

            // Pass through current working directory
            if let Ok(cwd) = std::env::current_dir() {
                cmd.env("HEGEL_IDE_CWD", cwd);
            }

            let status = cmd
                .status()
                .with_context(|| format!("Failed to execute npm start for {}", self.name))?;

            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }

            return Ok(());
        }

        // Try npx (published package)
        let mut cmd = Command::new("npx");
        cmd.arg(self.name).args(args);

        // Pass through HEGEL_SESSION_ID if present
        if let Ok(session_id) = std::env::var("HEGEL_SESSION_ID") {
            cmd.env("HEGEL_SESSION_ID", session_id);
        }

        // Pass through current working directory
        if let Ok(cwd) = std::env::current_dir() {
            cmd.env("HEGEL_IDE_CWD", cwd);
        }

        let status = cmd
            .status()
            .with_context(|| format!("Failed to execute npx {}", self.name))?;

        if !status.success() {
            if status.code() == Some(127) || status.code().is_none() {
                // Command not found
                anyhow::bail!(
                    "{} package not found. {}",
                    self.name,
                    self.build_instructions
                );
            }
            std::process::exit(status.code().unwrap_or(1));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_binary_find_checks_adjacent_repo() {
        let bin = ExternalBinary {
            name: "nonexistent-test-binary",
            adjacent_repo_path: "../nonexistent-repo",
            build_instructions: "Build instructions here",
        };

        let result = bin.find();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_external_npm_package_configuration() {
        // Test documents the expected structure without executing commands
        let pkg = ExternalNpmPackage {
            name: "test-package",
            adjacent_repo_path: "../test-repo",
            build_instructions: "Test build instructions",
        };

        assert_eq!(pkg.name, "test-package");
        assert_eq!(pkg.adjacent_repo_path, "../test-repo");
        assert_eq!(pkg.build_instructions, "Test build instructions");
    }
}
