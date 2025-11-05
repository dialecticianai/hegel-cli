use anyhow::Result;
use semver::{Version, VersionReq};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Agent runtime type
///
/// When executing agents (future implementation):
/// - Node.js/Python CLIs: Execute through user's shell (bash/zsh) to preserve
///   environment (nvm, volta, pyenv, etc.). This allows version managers to
///   automatically select the correct runtime version.
/// - Native binaries: Execute directly without shell wrapper.
///
/// The min_version field is informational only - version validation will be
/// handled by the runtime itself or version managers.
#[derive(Debug, Clone, PartialEq)]
pub enum AgentRuntime {
    /// Native binary (Rust, Go, etc.)
    Native,
    /// Node.js CLI (requires node in PATH)
    NodeJs { min_version: Option<&'static str> },
    /// Python CLI (requires python in PATH)
    Python { min_version: Option<&'static str> },
}

/// Runtime compatibility status
#[derive(Debug)]
pub enum RuntimeCompatibility {
    /// Runtime requirements satisfied with current PATH
    Compatible(String),
    /// Compatible version available via nvm
    NvmAvailable(PathBuf),
    /// Requirements not satisfied
    Incompatible(String),
    /// Unable to determine
    Unknown,
}

/// Expand tilde in path to home directory
pub fn expand_tilde(path: &str) -> Option<String> {
    if path.starts_with("~/") {
        if let Some(home) = std::env::var("HOME").ok() {
            return Some(path.replacen("~", &home, 1));
        }
    }
    Some(path.to_string())
}

/// Parse version string (e.g., "v18.20.8" or "18.20.8") into semver Version
pub fn parse_version(version: &str) -> Option<Version> {
    let version = version.trim().trim_start_matches('v');
    Version::parse(version).ok()
}

/// Get current node version by running `node --version`
fn get_current_node_version() -> Option<String> {
    let output = Command::new("node").arg("--version").output().ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Find compatible Node.js version in nvm directory
fn find_nvm_compatible_version(min_version: &str) -> Option<PathBuf> {
    let required = VersionReq::parse(&format!(">={}", min_version)).ok()?;

    // Check if nvm directory exists
    let nvm_dir = expand_tilde("~/.nvm/versions/node")?;
    let nvm_path = Path::new(&nvm_dir);

    if !nvm_path.exists() {
        return None;
    }

    // Read all installed versions
    let entries = std::fs::read_dir(nvm_path).ok()?;

    let mut compatible_versions = Vec::new();
    for entry in entries.flatten() {
        let version_name = entry.file_name().to_string_lossy().to_string();
        if let Some(actual) = parse_version(&version_name) {
            if required.matches(&actual) {
                compatible_versions.push((actual, entry.path()));
            }
        }
    }

    // Sort by version (descending) and return the highest compatible version
    compatible_versions.sort_by(|a, b| b.0.cmp(&a.0));

    compatible_versions
        .first()
        .map(|(_, path)| path.join("bin"))
}

/// Check if runtime requirements are satisfied
pub fn check_runtime_compatibility(runtime: &AgentRuntime) -> RuntimeCompatibility {
    match runtime {
        AgentRuntime::NodeJs { min_version } => {
            if let Some(min_ver) = min_version {
                let required = match VersionReq::parse(&format!(">={}", min_ver)) {
                    Ok(req) => req,
                    Err(_) => return RuntimeCompatibility::Unknown,
                };

                // Check current node version
                if let Some(current_str) = get_current_node_version() {
                    if let Some(current) = parse_version(&current_str) {
                        if required.matches(&current) {
                            return RuntimeCompatibility::Compatible(current_str);
                        }
                    }
                }

                // Current version too low or not found, check nvm
                if let Some(nvm_bin) = find_nvm_compatible_version(min_ver) {
                    return RuntimeCompatibility::NvmAvailable(nvm_bin);
                }

                // No compatible version found
                return RuntimeCompatibility::Incompatible(format!(
                    "Node.js >= {} required",
                    min_ver
                ));
            }

            // No min version specified, just check if node exists
            if get_current_node_version().is_some() {
                RuntimeCompatibility::Compatible("available".to_string())
            } else {
                RuntimeCompatibility::Incompatible("Node.js required".to_string())
            }
        }
        AgentRuntime::Python { min_version } => {
            // TODO: Similar logic for Python
            if min_version.is_some() {
                RuntimeCompatibility::Unknown
            } else {
                RuntimeCompatibility::Unknown
            }
        }
        AgentRuntime::Native => RuntimeCompatibility::Compatible("native".to_string()),
    }
}

/// Execute an agent with the given prompt and arguments
pub fn execute_agent(
    agent_name: &str,
    agent_path: &str,
    runtime: &AgentRuntime,
    cmd_args: &[String],
) -> Result<String> {
    // Create command with agent name
    let mut command = Command::new(agent_name);
    command.args(cmd_args);

    // If this is a Node.js agent with version requirements, check compatibility
    if let AgentRuntime::NodeJs { min_version } = runtime {
        if min_version.is_some() {
            match check_runtime_compatibility(runtime) {
                RuntimeCompatibility::NvmAvailable(nvm_bin) => {
                    // Prepend nvm bin directory to PATH
                    let current_path = std::env::var("PATH").unwrap_or_default();
                    let new_path = format!("{}:{}", nvm_bin.display(), current_path);
                    command.env("PATH", new_path);
                }
                RuntimeCompatibility::Incompatible(msg) => {
                    anyhow::bail!("Cannot execute {}: {}", agent_name, msg);
                }
                _ => {} // Compatible or Unknown - proceed normally
            }
        }
    }

    // Execute and capture output
    let output = command.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Agent '{}' failed with exit code {:?}\n{}",
            agent_name,
            output.status.code(),
            stderr
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert!(parse_version("18.20.8").is_some());
        assert!(parse_version("v18.20.8").is_some());
        assert!(parse_version("20.0.0").is_some());
        assert!(parse_version("v20.0.0").is_some());
        assert_eq!(
            parse_version("18.20.8").unwrap(),
            Version::parse("18.20.8").unwrap()
        );
        assert!(parse_version("invalid").is_none());
    }

    #[test]
    fn test_version_requirements() {
        let v18 = parse_version("18.20.8").unwrap();
        let v20 = parse_version("20.0.0").unwrap();
        let v22 = parse_version("22.20.0").unwrap();

        let req_18 = VersionReq::parse(">=18.0.0").unwrap();
        let req_20 = VersionReq::parse(">=20.0.0").unwrap();

        // v18 satisfies >=18.0.0 but not >=20.0.0
        assert!(req_18.matches(&v18));
        assert!(!req_20.matches(&v18));

        // v20 satisfies both
        assert!(req_18.matches(&v20));
        assert!(req_20.matches(&v20));

        // v22 satisfies both
        assert!(req_18.matches(&v22));
        assert!(req_20.matches(&v22));
    }

    #[test]
    fn test_runtime_compatibility_check() {
        // Test NodeJs with no min version
        let runtime = AgentRuntime::NodeJs { min_version: None };
        let compat = check_runtime_compatibility(&runtime);
        match compat {
            RuntimeCompatibility::Compatible(_) => {}
            RuntimeCompatibility::Incompatible(_) => {}
            _ => panic!("Expected Compatible or Incompatible"),
        }

        // Test Native runtime (always compatible)
        let runtime = AgentRuntime::Native;
        match check_runtime_compatibility(&runtime) {
            RuntimeCompatibility::Compatible(s) => assert_eq!(s, "native"),
            _ => panic!("Expected Native to be compatible"),
        }
    }
}
