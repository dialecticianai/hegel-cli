use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Known agent CLIs to detect with common installation locations
const KNOWN_AGENTS: &[(&str, &str, &[&str])] = &[
    (
        "claude",
        "Claude Code CLI (Anthropic)",
        &["~/.claude/local/claude", "~/.claude/claude"],
    ),
    ("aider", "AI pair programming (aider.chat)", &[]),
    ("copilot", "GitHub Copilot CLI", &[]),
    ("codex", "OpenAI Codex CLI", &[]),
    ("gemini", "Google Gemini CLI", &[]),
    ("cody", "Sourcegraph Cody CLI", &[]),
];

/// Agent detection result
#[derive(Debug)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub path: String,
    pub available: bool,
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

    for (name, description, fallback_paths) in KNOWN_AGENTS {
        // First try `which` command
        let output = Command::new("which").arg(name).output();

        let (available, path) = match output {
            Ok(result) if result.status.success() => {
                let path = String::from_utf8_lossy(&result.stdout).trim().to_string();
                (true, path)
            }
            _ => {
                // If `which` fails, check fallback locations
                let mut found = false;
                let mut found_path = String::new();

                for fallback in *fallback_paths {
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
            name: name.to_string(),
            description: description.to_string(),
            path,
            available,
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
            },
            Agent {
                name: "aider".to_string(),
                description: "AI pair programming".to_string(),
                path: String::new(),
                available: false,
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
