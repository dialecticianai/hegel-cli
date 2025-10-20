use serde::{Deserialize, Serialize};

/// Top-level guardrails configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardRailsConfig {
    #[serde(default)]
    pub git: Option<CommandGuardrails>,
    #[serde(default)]
    pub docker: Option<CommandGuardrails>,
}

/// Guardrails for a specific command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandGuardrails {
    /// List of blocked patterns with reasons
    #[serde(default)]
    pub blocked: Vec<BlockedRule>,
    /// List of allowed patterns (if specified, acts as allowlist)
    #[serde(default)]
    pub allowed: Vec<String>,
}

/// A blocked command pattern with reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedRule {
    pub pattern: String,
    pub reason: String,
}

/// Result of matching a command against guardrails
#[derive(Debug, Clone, PartialEq)]
pub enum RuleMatch {
    /// Command is explicitly allowed
    Allowed,
    /// Command is blocked with reason
    Blocked(String),
    /// No explicit allow/block rules matched (default allow)
    NoMatch,
}

impl GuardRailsConfig {
    /// Create empty config (all commands allowed)
    pub fn empty() -> Self {
        Self {
            git: None,
            docker: None,
        }
    }

    /// Get guardrails for a specific command
    pub fn get_command_guardrails(&self, command: &str) -> Option<&CommandGuardrails> {
        match command {
            "git" => self.git.as_ref(),
            "docker" => self.docker.as_ref(),
            _ => None,
        }
    }
}

impl CommandGuardrails {
    /// Check if a command invocation should be blocked
    pub fn evaluate(&self, args: &[String]) -> RuleMatch {
        let args_str = args.join(" ");

        // Check blocked patterns first
        for rule in &self.blocked {
            if let Ok(re) = regex::Regex::new(&rule.pattern) {
                if re.is_match(&args_str) {
                    return RuleMatch::Blocked(rule.reason.clone());
                }
            }
        }

        // If allowlist exists, check if command matches any allowed pattern
        if !self.allowed.is_empty() {
            for pattern in &self.allowed {
                if let Ok(re) = regex::Regex::new(pattern) {
                    if re.is_match(&args_str) {
                        return RuleMatch::Allowed;
                    }
                }
                // Also support literal prefix matching
                if args_str.starts_with(pattern) {
                    return RuleMatch::Allowed;
                }
            }
            // If allowlist exists but nothing matched, block
            return RuleMatch::Blocked(
                "Command not in allowlist. See .hegel/guardrails.yaml".to_string(),
            );
        }

        // No rules matched (default allow)
        RuleMatch::NoMatch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocked_rule_matches() {
        let guardrails = CommandGuardrails {
            blocked: vec![BlockedRule {
                pattern: "clean -fd".to_string(),
                reason: "Destructive operation".to_string(),
            }],
            allowed: vec![],
        };

        let result = guardrails.evaluate(&["clean".to_string(), "-fd".to_string()]);
        assert!(matches!(result, RuleMatch::Blocked(_)));
    }

    #[test]
    fn test_blocked_rule_regex() {
        let guardrails = CommandGuardrails {
            blocked: vec![BlockedRule {
                pattern: r"commit.*--no-verify".to_string(),
                reason: "Bypasses hooks".to_string(),
            }],
            allowed: vec![],
        };

        let result = guardrails.evaluate(&[
            "commit".to_string(),
            "-m".to_string(),
            "test".to_string(),
            "--no-verify".to_string(),
        ]);
        assert!(matches!(result, RuleMatch::Blocked(_)));
    }

    #[test]
    fn test_allowlist_matches() {
        let guardrails = CommandGuardrails {
            blocked: vec![],
            allowed: vec!["status".to_string(), "log".to_string()],
        };

        let result = guardrails.evaluate(&["status".to_string()]);
        assert_eq!(result, RuleMatch::Allowed);

        let result = guardrails.evaluate(&["log".to_string(), "--oneline".to_string()]);
        assert_eq!(result, RuleMatch::Allowed);
    }

    #[test]
    fn test_allowlist_blocks_non_matching() {
        let guardrails = CommandGuardrails {
            blocked: vec![],
            allowed: vec!["status".to_string()],
        };

        let result = guardrails.evaluate(&["push".to_string()]);
        assert!(matches!(result, RuleMatch::Blocked(_)));
    }

    #[test]
    fn test_no_rules_allows_everything() {
        let guardrails = CommandGuardrails {
            blocked: vec![],
            allowed: vec![],
        };

        let result = guardrails.evaluate(&["anything".to_string()]);
        assert_eq!(result, RuleMatch::NoMatch);
    }
}
