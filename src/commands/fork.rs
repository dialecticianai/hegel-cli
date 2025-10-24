use anyhow::Result;
use std::path::Path;
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

            // Show runtime requirements
            match &agent.runtime {
                AgentRuntime::NodeJs { min_version } => {
                    if let Some(version) = min_version {
                        println!("    Runtime: Node.js >= {}", version);
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

/// Handle `hegel fork` command (no args - detection only)
pub fn handle_fork() -> Result<()> {
    let agents = detect_agents()?;
    display_agents(&agents);
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
    fn test_handle_fork() {
        // Verify handle_fork doesn't crash
        let result = handle_fork();
        assert!(result.is_ok());
    }
}
