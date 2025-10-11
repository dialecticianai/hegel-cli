use anyhow::Result;
use chrono::DateTime;
use regex::Regex;

use super::types::{RuleConfig, RuleEvaluationContext, RuleViolation};

/// Evaluate all rules and return the first violation (short-circuit)
pub fn evaluate_rules(
    _rules: &[RuleConfig],
    _context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    // TODO: Implement in Step 7 (orchestration)
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{BashCommand, HookMetrics, PhaseMetrics, TokenMetrics};

    fn test_context_with_commands(commands: Vec<BashCommand>) -> RuleEvaluationContext<'static> {
        let hook_metrics = Box::leak(Box::new(HookMetrics {
            total_events: commands.len(),
            bash_commands: commands,
            file_modifications: vec![],
            session_start_time: None,
            session_end_time: None,
        }));

        RuleEvaluationContext {
            current_phase: "code",
            phase_start_time: "2025-01-01T10:00:00Z",
            phase_metrics: None,
            hook_metrics,
        }
    }

    // ========== Repeated Command Evaluation Tests ==========

    #[test]
    fn test_repeated_command_triggers_at_threshold() {
        // 5 commands in 120s window with threshold=5 should trigger
        let commands = vec![
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:00:30Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:01:00Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:01:30Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:01:59Z".to_string()),
                stdout: None,
                stderr: None,
            },
        ];

        let rule = RuleConfig::RepeatedCommand {
            pattern: Some("cargo build".to_string()),
            threshold: 5,
            window: 120,
        };

        let context = test_context_with_commands(commands);
        let result = evaluate_repeated_command(&rule, &context).unwrap();

        assert!(result.is_some());
        let violation = result.unwrap();
        assert_eq!(violation.rule_type, "Repeated Command");
        assert!(violation.diagnostic.contains("5"));
    }

    #[test]
    fn test_repeated_command_below_threshold_no_trigger() {
        // 4 commands with threshold=5 should NOT trigger
        let commands = vec![
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:00:30Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:01:00Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:01:30Z".to_string()),
                stdout: None,
                stderr: None,
            },
        ];

        let rule = RuleConfig::RepeatedCommand {
            pattern: Some("cargo build".to_string()),
            threshold: 5,
            window: 120,
        };

        let context = test_context_with_commands(commands);
        let result = evaluate_repeated_command(&rule, &context).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_repeated_command_outside_window_no_trigger() {
        // 6 commands but 3 outside the window (>120s old) should NOT trigger (threshold=5)
        let commands = vec![
            // Outside window (older than 120s from phase start)
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T09:57:00Z".to_string()), // 3 min before start
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T09:58:00Z".to_string()), // 2 min before start
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T09:59:00Z".to_string()), // 1 min before start
                stdout: None,
                stderr: None,
            },
            // Inside window (within 120s from phase start)
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:01:00Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:01:59Z".to_string()),
                stdout: None,
                stderr: None,
            },
        ];

        let rule = RuleConfig::RepeatedCommand {
            pattern: Some("cargo build".to_string()),
            threshold: 5,
            window: 120,
        };

        let context = test_context_with_commands(commands);
        let result = evaluate_repeated_command(&rule, &context).unwrap();

        // Only 3 commands in window, threshold is 5
        assert!(result.is_none());
    }

    #[test]
    fn test_repeated_command_regex_matches_multiple() {
        // Pattern "cargo (build|test)" should match both commands
        let commands = vec![
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo test".to_string(),
                timestamp: Some("2025-01-01T10:00:30Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:01:00Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo test".to_string(),
                timestamp: Some("2025-01-01T10:01:30Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:01:59Z".to_string()),
                stdout: None,
                stderr: None,
            },
        ];

        let rule = RuleConfig::RepeatedCommand {
            pattern: Some("cargo (build|test)".to_string()),
            threshold: 5,
            window: 120,
        };

        let context = test_context_with_commands(commands);
        let result = evaluate_repeated_command(&rule, &context).unwrap();

        assert!(result.is_some()); // 5 matches total
    }

    #[test]
    fn test_repeated_command_regex_excludes_non_matching() {
        // Pattern "cargo (build|test)" should NOT match "cargo fmt" or "git status"
        let commands = vec![
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "git status".to_string(),
                timestamp: Some("2025-01-01T10:00:10Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo fmt".to_string(),
                timestamp: Some("2025-01-01T10:00:20Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo test".to_string(),
                timestamp: Some("2025-01-01T10:00:30Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "git status".to_string(),
                timestamp: Some("2025-01-01T10:00:40Z".to_string()),
                stdout: None,
                stderr: None,
            },
        ];

        let rule = RuleConfig::RepeatedCommand {
            pattern: Some("cargo (build|test)".to_string()),
            threshold: 5,
            window: 120,
        };

        let context = test_context_with_commands(commands);
        let result = evaluate_repeated_command(&rule, &context).unwrap();

        // Only 2 matches (cargo build, cargo test), threshold is 5
        assert!(result.is_none());
    }

    #[test]
    fn test_repeated_command_no_pattern_matches_all() {
        // No pattern (None) should match ALL commands
        let commands = vec![
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "git status".to_string(),
                timestamp: Some("2025-01-01T10:00:10Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "ls -la".to_string(),
                timestamp: Some("2025-01-01T10:00:20Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "pwd".to_string(),
                timestamp: Some("2025-01-01T10:00:30Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "echo hello".to_string(),
                timestamp: Some("2025-01-01T10:00:40Z".to_string()),
                stdout: None,
                stderr: None,
            },
        ];

        let rule = RuleConfig::RepeatedCommand {
            pattern: None, // Match ALL
            threshold: 5,
            window: 120,
        };

        let context = test_context_with_commands(commands);
        let result = evaluate_repeated_command(&rule, &context).unwrap();

        assert!(result.is_some()); // All 5 commands match
    }

    #[test]
    fn test_repeated_command_empty_list_no_trigger() {
        let rule = RuleConfig::RepeatedCommand {
            pattern: Some("cargo build".to_string()),
            threshold: 5,
            window: 120,
        };

        let context = test_context_with_commands(vec![]);
        let result = evaluate_repeated_command(&rule, &context).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_repeated_command_exact_window_boundary() {
        // Commands at exactly 120s boundary should be included
        let commands = vec![
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()), // Exactly at phase start
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:01:00Z".to_string()),
                stdout: None,
                stderr: None,
            },
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:02:00Z".to_string()), // Exactly 120s after start
                stdout: None,
                stderr: None,
            },
        ];

        let rule = RuleConfig::RepeatedCommand {
            pattern: Some("cargo build".to_string()),
            threshold: 3,
            window: 120,
        };

        let context = test_context_with_commands(commands);
        let result = evaluate_repeated_command(&rule, &context).unwrap();

        assert!(result.is_some()); // All 3 should be included
    }
}

/// Evaluate a repeated_command rule
fn evaluate_repeated_command(
    rule: &RuleConfig,
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    let (pattern, threshold, window) = match rule {
        RuleConfig::RepeatedCommand {
            pattern,
            threshold,
            window,
        } => (pattern, threshold, window),
        _ => return Ok(None),
    };

    // Calculate window bounds: [phase_start, phase_start + window]
    let phase_start = DateTime::parse_from_rfc3339(context.phase_start_time)?;
    let window_end = phase_start + chrono::Duration::seconds(*window as i64);

    // Compile regex if pattern provided
    let regex = if let Some(pat) = pattern {
        Some(Regex::new(pat)?)
    } else {
        None
    };

    // Filter commands by time window and pattern
    let matching_commands: Vec<_> = context
        .hook_metrics
        .bash_commands
        .iter()
        .filter(|cmd| {
            // Filter by timestamp (within window: phase_start <= ts <= window_end)
            if let Some(ts) = &cmd.timestamp {
                if let Ok(timestamp) = DateTime::parse_from_rfc3339(ts) {
                    if timestamp < phase_start || timestamp > window_end {
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }

            // Filter by pattern (if provided)
            if let Some(ref re) = regex {
                re.is_match(&cmd.command)
            } else {
                true // No pattern = match all
            }
        })
        .collect();

    let count = matching_commands.len();

    if count >= *threshold {
        // Build recent events list (last 5)
        let recent_events: Vec<String> = matching_commands
            .iter()
            .rev()
            .take(5)
            .rev()
            .map(|cmd| {
                format!(
                    "{}: {}",
                    cmd.timestamp.as_ref().unwrap_or(&"unknown".to_string())[11..19].to_string(),
                    cmd.command
                )
            })
            .collect();

        Ok(Some(RuleViolation {
            rule_type: "Repeated Command".to_string(),
            diagnostic: format!("Command executed {} times in last {}s", count, window),
            suggestion: "You're stuck in a build loop. Review the error message carefully. Consider using TDD: write a failing test first, then fix the specific issue.".to_string(),
            recent_events,
        }))
    } else {
        Ok(None)
    }
}
