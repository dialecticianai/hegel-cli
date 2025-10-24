use anyhow::Result;
use semver::{Version, VersionReq};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Agent metadata: (name, description, fallback_paths, runtime)
struct AgentMetadata {
    name: &'static str,
    description: &'static str,
    fallback_paths: &'static [&'static str],
    runtime: AgentRuntime,
}

/// Known agent CLIs to detect with common installation locations and runtime info
const KNOWN_AGENTS: &[AgentMetadata] = &[
    AgentMetadata {
        name: "claude",
        description: "Claude Code CLI (Anthropic)",
        fallback_paths: &["~/.claude/local/claude", "~/.claude/claude"],
        runtime: AgentRuntime::NodeJs {
            min_version: Some("18.0.0"),
        },
    },
    AgentMetadata {
        name: "aider",
        description: "AI pair programming (aider.chat)",
        fallback_paths: &[],
        runtime: AgentRuntime::Python {
            min_version: Some("3.8.0"),
        },
    },
    AgentMetadata {
        name: "copilot",
        description: "GitHub Copilot CLI",
        fallback_paths: &[],
        runtime: AgentRuntime::NodeJs { min_version: None },
    },
    AgentMetadata {
        name: "codex",
        description: "OpenAI Codex CLI",
        fallback_paths: &[],
        runtime: AgentRuntime::NodeJs { min_version: None },
    },
    AgentMetadata {
        name: "gemini",
        description: "Google Gemini CLI",
        fallback_paths: &[],
        runtime: AgentRuntime::NodeJs {
            min_version: Some("20.0.0"),
        },
    },
    AgentMetadata {
        name: "cody",
        description: "Sourcegraph Cody CLI",
        fallback_paths: &[],
        runtime: AgentRuntime::NodeJs { min_version: None },
    },
];

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

/// Agent detection result
#[derive(Debug)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub path: String,
    pub available: bool,
    pub runtime: AgentRuntime,
}

/// Expand tilde in path to home directory
fn expand_tilde(path: &str) -> Option<String> {
    if path.starts_with("~/") {
        if let Some(home) = std::env::var("HOME").ok() {
            return Some(path.replacen("~", &home, 1));
        }
    }
    Some(path.to_string())
}

/// Parse version string (e.g., "v18.20.8" or "18.20.8") into semver Version
fn parse_version(version: &str) -> Option<Version> {
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
fn check_runtime_compatibility(runtime: &AgentRuntime) -> RuntimeCompatibility {
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

/// Runtime compatibility status
#[derive(Debug)]
enum RuntimeCompatibility {
    /// Runtime requirements satisfied with current PATH
    Compatible(String),
    /// Compatible version available via nvm
    NvmAvailable(PathBuf),
    /// Requirements not satisfied
    Incompatible(String),
    /// Unable to determine
    Unknown,
}

/// Detect installed agent CLIs using `which` command and fallback locations
pub fn detect_agents() -> Result<Vec<Agent>> {
    let mut agents = Vec::new();

    for meta in KNOWN_AGENTS {
        // First try `which` command
        let output = Command::new("which").arg(meta.name).output();

        let (available, path) = match output {
            Ok(result) if result.status.success() => {
                let path = String::from_utf8_lossy(&result.stdout).trim().to_string();
                (true, path)
            }
            _ => {
                // If `which` fails, check fallback locations
                let mut found = false;
                let mut found_path = String::new();

                for fallback in meta.fallback_paths {
                    if let Some(expanded) = expand_tilde(fallback) {
                        if Path::new(&expanded).exists() {
                            found = true;
                            found_path = expanded;
                            break;
                        }
                    }
                }

                (found, found_path)
            }
        };

        agents.push(Agent {
            name: meta.name.to_string(),
            description: meta.description.to_string(),
            path,
            available,
            runtime: meta.runtime.clone(),
        });
    }

    Ok(agents)
}

/// Display detected agents
pub fn display_agents(agents: &[Agent]) {
    println!("Detected Agent CLIs:\n");

    let available: Vec<_> = agents.iter().filter(|a| a.available).collect();
    let unavailable: Vec<_> = agents.iter().filter(|a| !a.available).collect();

    if !available.is_empty() {
        println!("Available:");
        for agent in &available {
            println!("  ✓ {} - {}", agent.name, agent.description);
            println!("    Path: {}", agent.path);

            // Show runtime requirements and compatibility
            match &agent.runtime {
                AgentRuntime::NodeJs { min_version } => {
                    if let Some(version) = min_version {
                        println!("    Runtime: Node.js >= {}", version);

                        // Check compatibility
                        match check_runtime_compatibility(&agent.runtime) {
                            RuntimeCompatibility::Compatible(current) => {
                                println!("    Status: ✓ Compatible (using {})", current);
                            }
                            RuntimeCompatibility::NvmAvailable(nvm_path) => {
                                println!(
                                    "    Status: ⚠ Current node version too low. Compatible version found at:"
                                );
                                println!("            {}", nvm_path.display());
                                println!(
                                    "            Run: export PATH={}:$PATH",
                                    nvm_path.display()
                                );
                            }
                            RuntimeCompatibility::Incompatible(msg) => {
                                println!("    Status: ✗ {}", msg);
                                println!("            Install Node.js {} or higher", version);
                            }
                            RuntimeCompatibility::Unknown => {
                                println!("    Status: ? Unable to determine compatibility");
                            }
                        }
                    } else {
                        println!("    Runtime: Node.js");
                    }
                }
                AgentRuntime::Python { min_version } => {
                    if let Some(version) = min_version {
                        println!("    Runtime: Python >= {}", version);
                    } else {
                        println!("    Runtime: Python");
                    }
                }
                AgentRuntime::Native => {
                    println!("    Runtime: Native binary");
                }
            }
        }
        println!();
    }

    if !unavailable.is_empty() {
        println!("Not installed:");
        for agent in unavailable {
            println!("  ✗ {} - {}", agent.name, agent.description);
        }
    }

    if available.is_empty() {
        println!("No agent CLIs detected. Install an agent CLI to use `hegel fork`.");
    }
}

/// Execute an agent with the given prompt and arguments
fn execute_agent(agent: &Agent, prompt: Option<&str>, args: &[String]) -> Result<String> {
    // Determine command and arguments based on agent
    let (cmd, cmd_args) = match agent.name.as_str() {
        "codex" => {
            // codex uses `codex exec "prompt"` for non-interactive mode
            let mut exec_args = vec!["exec".to_string()];

            // Add passthrough args first (before prompt)
            exec_args.extend_from_slice(args);

            // Add prompt if provided
            if let Some(p) = prompt {
                exec_args.push(p.to_string());
            }

            ("codex", exec_args)
        }
        "gemini" => {
            // gemini uses positional argument for prompt
            let mut gemini_args = Vec::new();

            // Add passthrough args first
            gemini_args.extend_from_slice(args);

            // Add prompt if provided
            if let Some(p) = prompt {
                gemini_args.push(p.to_string());
            }

            ("gemini", gemini_args)
        }
        _ => {
            // Default: just pass prompt and args
            let mut default_args = Vec::new();
            default_args.extend_from_slice(args);
            if let Some(p) = prompt {
                default_args.push(p.to_string());
            }
            (agent.name.as_str(), default_args)
        }
    };

    // Check if we need to modify PATH for Node.js version compatibility
    let mut command = Command::new(cmd);
    command.args(&cmd_args);

    // If this is a Node.js agent with version requirements, check compatibility
    if let AgentRuntime::NodeJs { min_version } = &agent.runtime {
        if min_version.is_some() {
            match check_runtime_compatibility(&agent.runtime) {
                RuntimeCompatibility::NvmAvailable(nvm_bin) => {
                    // Prepend nvm bin directory to PATH
                    let current_path = std::env::var("PATH").unwrap_or_default();
                    let new_path = format!("{}:{}", nvm_bin.display(), current_path);
                    command.env("PATH", new_path);
                }
                RuntimeCompatibility::Incompatible(msg) => {
                    anyhow::bail!("Cannot execute {}: {}", agent.name, msg);
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
            agent.name,
            output.status.code(),
            stderr
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Handle `hegel fork` command
pub fn handle_fork(agent_name: Option<&str>, prompt: Option<&str>, args: &[String]) -> Result<()> {
    let agents = detect_agents()?;

    // If no agent specified, just list available agents
    if agent_name.is_none() {
        display_agents(&agents);
        return Ok(());
    }

    let agent_name = agent_name.unwrap();

    // Find the requested agent
    let agent = agents
        .iter()
        .find(|a| a.name == agent_name && a.available)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Agent '{}' not found or not available. Run `hegel fork` to see available agents.",
                agent_name
            )
        })?;

    // Execute the agent
    let output = execute_agent(agent, prompt, args)?;

    // Print output to stdout for the calling agent to consume
    print!("{}", output);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_agents() {
        // Just verify it doesn't crash
        let result = detect_agents();
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_agents() {
        let agents = vec![
            Agent {
                name: "claude".to_string(),
                description: "Claude Code CLI".to_string(),
                path: "/usr/local/bin/claude".to_string(),
                available: true,
                runtime: AgentRuntime::NodeJs {
                    min_version: Some("18.0.0"),
                },
            },
            Agent {
                name: "aider".to_string(),
                description: "AI pair programming".to_string(),
                path: String::new(),
                available: false,
                runtime: AgentRuntime::Python {
                    min_version: Some("3.8.0"),
                },
            },
        ];

        // Just verify display doesn't crash
        display_agents(&agents);
    }

    #[test]
    fn test_handle_fork_list_agents() {
        // Verify handle_fork lists agents when no agent specified
        let result = handle_fork(None, None, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_fork_agent_not_found() {
        // Verify error when agent doesn't exist
        let result = handle_fork(Some("nonexistent"), Some("test prompt"), &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not found or not available"));
    }

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
