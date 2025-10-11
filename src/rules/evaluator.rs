use anyhow::Result;
use chrono::DateTime;
use regex::Regex;

use super::types::{RuleConfig, RuleEvaluationContext, RuleViolation};

/// Evaluate all rules and return the first violation (short-circuit)
pub fn evaluate_rules(
    rules: &[RuleConfig],
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    // Evaluate each rule in order, short-circuit on first violation
    for rule in rules {
        let violation = match rule {
            RuleConfig::RepeatedCommand { .. } => evaluate_repeated_command(rule, context)?,
            RuleConfig::RepeatedFileEdit { .. } => evaluate_repeated_file_edit(rule, context)?,
            RuleConfig::PhaseTimeout { .. } => evaluate_phase_timeout(rule, context)?,
            RuleConfig::TokenBudget { .. } => evaluate_token_budget(rule, context)?,
        };

        if violation.is_some() {
            return Ok(violation); // Short-circuit on first violation
        }
    }

    Ok(None) // No violations
}

/// Evaluate a repeated_file_edit rule
fn evaluate_repeated_file_edit(
    rule: &RuleConfig,
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    let (path_pattern, threshold, window) = match rule {
        RuleConfig::RepeatedFileEdit {
            path_pattern,
            threshold,
            window,
        } => (path_pattern, threshold, window),
        _ => return Ok(None),
    };

    // Calculate window bounds: [phase_start, phase_start + window]
    let phase_start = DateTime::parse_from_rfc3339(context.phase_start_time)?;
    let window_end = phase_start + chrono::Duration::seconds(*window as i64);

    // Compile regex if pattern provided
    let regex = if let Some(pat) = path_pattern {
        Some(Regex::new(pat)?)
    } else {
        None
    };

    // Filter file modifications by time window and pattern
    let matching_edits: Vec<_> = context
        .hook_metrics
        .file_modifications
        .iter()
        .filter(|file_mod| {
            // Filter by timestamp (within window: phase_start <= ts <= window_end)
            if let Some(ts) = &file_mod.timestamp {
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
                re.is_match(&file_mod.file_path)
            } else {
                true // No pattern = match all
            }
        })
        .collect();

    let count = matching_edits.len();

    if count >= *threshold {
        // Build recent events list (last 5)
        let recent_events: Vec<String> = matching_edits
            .iter()
            .rev()
            .take(5)
            .rev()
            .map(|file_mod| {
                format!(
                    "{}: {} ({})",
                    file_mod
                        .timestamp
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())[11..19]
                        .to_string(),
                    file_mod.file_path,
                    file_mod.tool
                )
            })
            .collect();

        Ok(Some(RuleViolation {
            rule_type: "Repeated File Edit".to_string(),
            diagnostic: format!("Files edited {} times in last {}s", count, window),
            suggestion: "You're thrashing the same files. Step back and write a failing test that captures the desired behavior, then implement the fix.".to_string(),
            recent_events,
        }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{BashCommand, FileModification, HookMetrics, PhaseMetrics, TokenMetrics};

    // ========== Unified Rule Evaluation Tests (Orchestration) ==========

    #[test]
    fn test_evaluate_rules_returns_first_violation() {
        // Setup: multiple rules, only first should trigger
        let commands = vec![
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
                stdout: None,
                stderr: None,
            };
            5
        ];

        let hook_metrics = Box::leak(Box::new(HookMetrics {
            total_events: 5,
            bash_commands: commands,
            file_modifications: vec![],
            session_start_time: None,
            session_end_time: None,
        }));

        let context = RuleEvaluationContext {
            current_phase: "code",
            phase_start_time: "2025-01-01T10:00:00Z",
            phase_metrics: None,
            hook_metrics,
        };

        let rules = vec![
            RuleConfig::RepeatedCommand {
                pattern: Some("cargo build".to_string()),
                threshold: 5,
                window: 120,
            },
            RuleConfig::TokenBudget { max_tokens: 1 }, // Would also trigger if checked
        ];

        let result = evaluate_rules(&rules, &context).unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().rule_type, "Repeated Command"); // First rule
    }

    #[test]
    fn test_evaluate_rules_no_violations() {
        let hook_metrics = Box::leak(Box::new(HookMetrics::default()));

        let context = RuleEvaluationContext {
            current_phase: "code",
            phase_start_time: "2025-01-01T10:00:00Z",
            phase_metrics: None,
            hook_metrics,
        };

        let rules = vec![
            RuleConfig::RepeatedCommand {
                pattern: Some("cargo build".to_string()),
                threshold: 5,
                window: 120,
            },
            RuleConfig::RepeatedFileEdit {
                path_pattern: Some(r"src/.*\.rs".to_string()),
                threshold: 8,
                window: 180,
            },
        ];

        let result = evaluate_rules(&rules, &context).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_rules_empty_rules() {
        let hook_metrics = Box::leak(Box::new(HookMetrics::default()));

        let context = RuleEvaluationContext {
            current_phase: "code",
            phase_start_time: "2025-01-01T10:00:00Z",
            phase_metrics: None,
            hook_metrics,
        };

        let result = evaluate_rules(&[], &context).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_rules_short_circuit() {
        // Second rule would trigger but first rule triggers first
        let commands = vec![
            BashCommand {
                command: "cargo build".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
                stdout: None,
                stderr: None,
            };
            5
        ];

        let edits = vec![
            FileModification {
                file_path: "src/main.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
            };
            10
        ];

        let hook_metrics = Box::leak(Box::new(HookMetrics {
            total_events: 15,
            bash_commands: commands,
            file_modifications: edits,
            session_start_time: None,
            session_end_time: None,
        }));

        let context = RuleEvaluationContext {
            current_phase: "code",
            phase_start_time: "2025-01-01T10:00:00Z",
            phase_metrics: None,
            hook_metrics,
        };

        let rules = vec![
            RuleConfig::RepeatedCommand {
                pattern: None,
                threshold: 5,
                window: 120,
            },
            RuleConfig::RepeatedFileEdit {
                path_pattern: None,
                threshold: 8,
                window: 180,
            },
        ];

        let result = evaluate_rules(&rules, &context).unwrap();

        assert!(result.is_some());
        // Should be first rule (RepeatedCommand) even though both would trigger
        assert_eq!(result.unwrap().rule_type, "Repeated Command");
    }

    #[test]
    fn test_evaluate_rules_mixed_types() {
        // Test with all four rule types
        let phase = Box::leak(Box::new(PhaseMetrics {
            phase_name: "code".to_string(),
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: Some("2025-01-01T10:15:00Z".to_string()), // 15 min = 900s
            duration_seconds: 900,
            token_metrics: TokenMetrics {
                total_input_tokens: 6000,
                total_output_tokens: 1000,
                total_cache_creation_tokens: 0,
                total_cache_read_tokens: 0,
                assistant_turns: 10,
            },
            bash_commands: vec![],
            file_modifications: vec![],
        }));

        let hook_metrics = Box::leak(Box::new(HookMetrics::default()));

        let context = RuleEvaluationContext {
            current_phase: "code",
            phase_start_time: &phase.start_time,
            phase_metrics: Some(phase),
            hook_metrics,
        };

        let rules = vec![
            RuleConfig::RepeatedCommand {
                pattern: None,
                threshold: 100, // Won't trigger
                window: 120,
            },
            RuleConfig::RepeatedFileEdit {
                path_pattern: None,
                threshold: 100, // Won't trigger
                window: 180,
            },
            RuleConfig::PhaseTimeout { max_duration: 600 }, // WILL trigger (900s > 600s)
            RuleConfig::TokenBudget { max_tokens: 10000 },  // Won't trigger
        ];

        let result = evaluate_rules(&rules, &context).unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().rule_type, "Phase Timeout");
    }

    #[test]
    fn test_evaluate_rules_all_pass() {
        // Realistic scenario where all rules are configured but none trigger
        let phase = Box::leak(Box::new(PhaseMetrics {
            phase_name: "code".to_string(),
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: Some("2025-01-01T10:05:00Z".to_string()), // 5 min = 300s
            duration_seconds: 300,
            token_metrics: TokenMetrics {
                total_input_tokens: 2000,
                total_output_tokens: 1500,
                total_cache_creation_tokens: 0,
                total_cache_read_tokens: 0,
                assistant_turns: 5,
            },
            bash_commands: vec![],
            file_modifications: vec![],
        }));

        let hook_metrics = Box::leak(Box::new(HookMetrics::default()));

        let context = RuleEvaluationContext {
            current_phase: "code",
            phase_start_time: &phase.start_time,
            phase_metrics: Some(phase),
            hook_metrics,
        };

        let rules = vec![
            RuleConfig::RepeatedCommand {
                pattern: Some("cargo build".to_string()),
                threshold: 10,
                window: 300,
            },
            RuleConfig::RepeatedFileEdit {
                path_pattern: Some(r"src/.*\.rs".to_string()),
                threshold: 15,
                window: 300,
            },
            RuleConfig::PhaseTimeout { max_duration: 600 },
            RuleConfig::TokenBudget { max_tokens: 5000 },
        ];

        let result = evaluate_rules(&rules, &context).unwrap();
        assert!(result.is_none()); // All rules pass
    }

    // ========== Individual Evaluator Tests ==========

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

    // ========== Repeated File Edit Evaluation Tests ==========

    fn test_context_with_file_edits(
        edits: Vec<crate::metrics::FileModification>,
    ) -> RuleEvaluationContext<'static> {
        let hook_metrics = Box::leak(Box::new(HookMetrics {
            total_events: edits.len(),
            bash_commands: vec![],
            file_modifications: edits,
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

    #[test]
    fn test_repeated_file_edit_triggers_at_threshold() {
        use crate::metrics::FileModification;

        let edits = vec![
            FileModification {
                file_path: "src/main.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
            },
            FileModification {
                file_path: "src/main.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:30Z".to_string()),
            },
            FileModification {
                file_path: "src/lib.rs".to_string(),
                tool: "Write".to_string(),
                timestamp: Some("2025-01-01T10:01:00Z".to_string()),
            },
            FileModification {
                file_path: "src/main.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:01:30Z".to_string()),
            },
            FileModification {
                file_path: "src/utils.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:02:00Z".to_string()),
            },
            FileModification {
                file_path: "src/main.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:02:30Z".to_string()),
            },
            FileModification {
                file_path: "src/lib.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:02:59Z".to_string()),
            },
            FileModification {
                file_path: "src/types.rs".to_string(),
                tool: "Write".to_string(),
                timestamp: Some("2025-01-01T10:03:00Z".to_string()),
            },
        ];

        let rule = RuleConfig::RepeatedFileEdit {
            path_pattern: Some(r"src/.*\.rs".to_string()),
            threshold: 8,
            window: 180,
        };

        let context = test_context_with_file_edits(edits);
        let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

        assert!(result.is_some());
        let violation = result.unwrap();
        assert_eq!(violation.rule_type, "Repeated File Edit");
        assert!(violation.diagnostic.contains("8"));
    }

    #[test]
    fn test_repeated_file_edit_below_threshold_no_trigger() {
        use crate::metrics::FileModification;

        let edits = vec![
            FileModification {
                file_path: "src/main.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
            },
            FileModification {
                file_path: "src/lib.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:30Z".to_string()),
            },
        ];

        let rule = RuleConfig::RepeatedFileEdit {
            path_pattern: Some(r"src/.*\.rs".to_string()),
            threshold: 5,
            window: 180,
        };

        let context = test_context_with_file_edits(edits);
        let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_repeated_file_edit_path_pattern_matches() {
        use crate::metrics::FileModification;

        let edits = vec![
            FileModification {
                file_path: "src/main.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
            },
            FileModification {
                file_path: "src/lib.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:10Z".to_string()),
            },
            FileModification {
                file_path: "README.md".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:20Z".to_string()),
            },
            FileModification {
                file_path: "src/utils.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:30Z".to_string()),
            },
            FileModification {
                file_path: "Cargo.toml".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:40Z".to_string()),
            },
        ];

        let rule = RuleConfig::RepeatedFileEdit {
            path_pattern: Some(r"src/.*\.rs".to_string()),
            threshold: 3,
            window: 180,
        };

        let context = test_context_with_file_edits(edits);
        let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

        assert!(result.is_some()); // 3 src/*.rs files match
    }

    #[test]
    fn test_repeated_file_edit_path_pattern_excludes() {
        use crate::metrics::FileModification;

        let edits = vec![
            FileModification {
                file_path: "README.md".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
            },
            FileModification {
                file_path: "Cargo.toml".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:10Z".to_string()),
            },
            FileModification {
                file_path: "src/main.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:20Z".to_string()),
            },
        ];

        let rule = RuleConfig::RepeatedFileEdit {
            path_pattern: Some(r"src/.*\.rs".to_string()),
            threshold: 3,
            window: 180,
        };

        let context = test_context_with_file_edits(edits);
        let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

        assert!(result.is_none()); // Only 1 src/*.rs file
    }

    #[test]
    fn test_repeated_file_edit_no_pattern_matches_all() {
        use crate::metrics::FileModification;

        let edits = vec![
            FileModification {
                file_path: "src/main.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
            },
            FileModification {
                file_path: "README.md".to_string(),
                tool: "Write".to_string(),
                timestamp: Some("2025-01-01T10:00:10Z".to_string()),
            },
            FileModification {
                file_path: "Cargo.toml".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:20Z".to_string()),
            },
        ];

        let rule = RuleConfig::RepeatedFileEdit {
            path_pattern: None, // Match ALL
            threshold: 3,
            window: 180,
        };

        let context = test_context_with_file_edits(edits);
        let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

        assert!(result.is_some()); // All 3 files match
    }

    #[test]
    fn test_repeated_file_edit_empty_list_no_trigger() {
        let rule = RuleConfig::RepeatedFileEdit {
            path_pattern: Some(r"src/.*\.rs".to_string()),
            threshold: 5,
            window: 180,
        };

        let context = test_context_with_file_edits(vec![]);
        let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_repeated_file_edit_outside_window_no_trigger() {
        use crate::metrics::FileModification;

        let edits = vec![
            // Outside window (before phase start)
            FileModification {
                file_path: "src/main.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T09:57:00Z".to_string()),
            },
            FileModification {
                file_path: "src/lib.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T09:58:00Z".to_string()),
            },
            // Inside window
            FileModification {
                file_path: "src/utils.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:00:00Z".to_string()),
            },
            FileModification {
                file_path: "src/types.rs".to_string(),
                tool: "Edit".to_string(),
                timestamp: Some("2025-01-01T10:01:00Z".to_string()),
            },
        ];

        let rule = RuleConfig::RepeatedFileEdit {
            path_pattern: Some(r"src/.*\.rs".to_string()),
            threshold: 5,
            window: 120,
        };

        let context = test_context_with_file_edits(edits);
        let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

        assert!(result.is_none()); // Only 2 in window, threshold is 5
    }

    // ========== Phase Timeout Evaluation Tests ==========

    fn test_context_with_phase(
        phase_metrics: crate::metrics::PhaseMetrics,
    ) -> RuleEvaluationContext<'static> {
        let phase_metrics_ref = Box::leak(Box::new(phase_metrics));
        let hook_metrics = Box::leak(Box::new(HookMetrics::default()));

        RuleEvaluationContext {
            current_phase: "code",
            phase_start_time: &phase_metrics_ref.start_time,
            phase_metrics: Some(phase_metrics_ref),
            hook_metrics,
        }
    }

    #[test]
    fn test_phase_timeout_triggers_when_exceeded() {
        use crate::metrics::{PhaseMetrics, TokenMetrics};

        let phase = PhaseMetrics {
            phase_name: "code".to_string(),
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: Some("2025-01-01T10:12:00Z".to_string()), // 12 minutes = 720s
            duration_seconds: 720,
            token_metrics: TokenMetrics::default(),
            bash_commands: vec![],
            file_modifications: vec![],
        };

        let rule = RuleConfig::PhaseTimeout { max_duration: 600 }; // 10 minutes

        let context = test_context_with_phase(phase);
        let result = evaluate_phase_timeout(&rule, &context).unwrap();

        assert!(result.is_some());
        let violation = result.unwrap();
        assert_eq!(violation.rule_type, "Phase Timeout");
        assert!(violation.diagnostic.contains("720"));
    }

    #[test]
    fn test_phase_timeout_no_trigger_below_limit() {
        use crate::metrics::{PhaseMetrics, TokenMetrics};

        let phase = PhaseMetrics {
            phase_name: "code".to_string(),
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: Some("2025-01-01T10:08:00Z".to_string()), // 8 minutes = 480s
            duration_seconds: 480,
            token_metrics: TokenMetrics::default(),
            bash_commands: vec![],
            file_modifications: vec![],
        };

        let rule = RuleConfig::PhaseTimeout { max_duration: 600 }; // 10 minutes

        let context = test_context_with_phase(phase);
        let result = evaluate_phase_timeout(&rule, &context).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_phase_timeout_active_phase_uses_current_time() {
        use crate::metrics::{PhaseMetrics, TokenMetrics};
        use chrono::Utc;

        // Active phase with no end_time
        let start = (Utc::now() - chrono::Duration::seconds(700)).to_rfc3339();
        let phase = PhaseMetrics {
            phase_name: "code".to_string(),
            start_time: start.clone(),
            end_time: None, // Active phase
            duration_seconds: 0,
            token_metrics: TokenMetrics::default(),
            bash_commands: vec![],
            file_modifications: vec![],
        };

        let rule = RuleConfig::PhaseTimeout { max_duration: 600 };

        let context = test_context_with_phase(phase);
        let result = evaluate_phase_timeout(&rule, &context).unwrap();

        // Should trigger since ~700s > 600s
        assert!(result.is_some());
    }

    #[test]
    fn test_phase_timeout_exact_limit_no_trigger() {
        use crate::metrics::{PhaseMetrics, TokenMetrics};

        let phase = PhaseMetrics {
            phase_name: "code".to_string(),
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: Some("2025-01-01T10:10:00Z".to_string()), // Exactly 600s
            duration_seconds: 600,
            token_metrics: TokenMetrics::default(),
            bash_commands: vec![],
            file_modifications: vec![],
        };

        let rule = RuleConfig::PhaseTimeout { max_duration: 600 };

        let context = test_context_with_phase(phase);
        let result = evaluate_phase_timeout(&rule, &context).unwrap();

        // Exactly at limit should NOT trigger (> not >=)
        assert!(result.is_none());
    }

    #[test]
    fn test_phase_timeout_zero_duration_no_trigger() {
        use crate::metrics::{PhaseMetrics, TokenMetrics};

        let phase = PhaseMetrics {
            phase_name: "code".to_string(),
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: Some("2025-01-01T10:00:00Z".to_string()), // Zero duration
            duration_seconds: 0,
            token_metrics: TokenMetrics::default(),
            bash_commands: vec![],
            file_modifications: vec![],
        };

        let rule = RuleConfig::PhaseTimeout { max_duration: 600 };

        let context = test_context_with_phase(phase);
        let result = evaluate_phase_timeout(&rule, &context).unwrap();

        assert!(result.is_none());
    }

    // ========== Token Budget Evaluation Tests ==========

    #[test]
    fn test_token_budget_triggers_when_exceeded() {
        use crate::metrics::{PhaseMetrics, TokenMetrics};

        let phase = PhaseMetrics {
            phase_name: "code".to_string(),
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: None,
            duration_seconds: 0,
            token_metrics: TokenMetrics {
                total_input_tokens: 4000,
                total_output_tokens: 2500,
                total_cache_creation_tokens: 0,
                total_cache_read_tokens: 0,
                assistant_turns: 10,
            },
            bash_commands: vec![],
            file_modifications: vec![],
        };

        let rule = RuleConfig::TokenBudget { max_tokens: 6000 };

        let context = test_context_with_phase(phase);
        let result = evaluate_token_budget(&rule, &context).unwrap();

        assert!(result.is_some());
        let violation = result.unwrap();
        assert_eq!(violation.rule_type, "Token Budget");
        assert!(violation.diagnostic.contains("6500")); // 4000 + 2500
        assert!(violation.diagnostic.contains("6000"));
    }

    #[test]
    fn test_token_budget_no_trigger_below_limit() {
        use crate::metrics::{PhaseMetrics, TokenMetrics};

        let phase = PhaseMetrics {
            phase_name: "code".to_string(),
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: None,
            duration_seconds: 0,
            token_metrics: TokenMetrics {
                total_input_tokens: 2000,
                total_output_tokens: 1500,
                total_cache_creation_tokens: 0,
                total_cache_read_tokens: 0,
                assistant_turns: 5,
            },
            bash_commands: vec![],
            file_modifications: vec![],
        };

        let rule = RuleConfig::TokenBudget { max_tokens: 5000 };

        let context = test_context_with_phase(phase);
        let result = evaluate_token_budget(&rule, &context).unwrap();

        assert!(result.is_none()); // 3500 < 5000
    }

    #[test]
    fn test_token_budget_exact_limit_no_trigger() {
        use crate::metrics::{PhaseMetrics, TokenMetrics};

        let phase = PhaseMetrics {
            phase_name: "code".to_string(),
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: None,
            duration_seconds: 0,
            token_metrics: TokenMetrics {
                total_input_tokens: 3000,
                total_output_tokens: 2000,
                total_cache_creation_tokens: 0,
                total_cache_read_tokens: 0,
                assistant_turns: 5,
            },
            bash_commands: vec![],
            file_modifications: vec![],
        };

        let rule = RuleConfig::TokenBudget { max_tokens: 5000 };

        let context = test_context_with_phase(phase);
        let result = evaluate_token_budget(&rule, &context).unwrap();

        assert!(result.is_none()); // Exactly 5000, should not trigger (> not >=)
    }

    #[test]
    fn test_token_budget_zero_tokens_no_trigger() {
        use crate::metrics::{PhaseMetrics, TokenMetrics};

        let phase = PhaseMetrics {
            phase_name: "code".to_string(),
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: None,
            duration_seconds: 0,
            token_metrics: TokenMetrics::default(),
            bash_commands: vec![],
            file_modifications: vec![],
        };

        let rule = RuleConfig::TokenBudget { max_tokens: 5000 };

        let context = test_context_with_phase(phase);
        let result = evaluate_token_budget(&rule, &context).unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_token_budget_includes_input_and_output() {
        use crate::metrics::{PhaseMetrics, TokenMetrics};

        let phase = PhaseMetrics {
            phase_name: "code".to_string(),
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: None,
            duration_seconds: 0,
            token_metrics: TokenMetrics {
                total_input_tokens: 3000,
                total_output_tokens: 3000,
                total_cache_creation_tokens: 500,
                total_cache_read_tokens: 1000,
                assistant_turns: 10,
            },
            bash_commands: vec![],
            file_modifications: vec![],
        };

        let rule = RuleConfig::TokenBudget { max_tokens: 5000 };

        let context = test_context_with_phase(phase);
        let result = evaluate_token_budget(&rule, &context).unwrap();

        // Total = 3000 + 3000 = 6000 (cache tokens not counted per SPEC)
        assert!(result.is_some());
    }

    #[test]
    fn test_token_budget_only_input_tokens() {
        use crate::metrics::{PhaseMetrics, TokenMetrics};

        let phase = PhaseMetrics {
            phase_name: "code".to_string(),
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: None,
            duration_seconds: 0,
            token_metrics: TokenMetrics {
                total_input_tokens: 6000,
                total_output_tokens: 0,
                total_cache_creation_tokens: 0,
                total_cache_read_tokens: 0,
                assistant_turns: 1,
            },
            bash_commands: vec![],
            file_modifications: vec![],
        };

        let rule = RuleConfig::TokenBudget { max_tokens: 5000 };

        let context = test_context_with_phase(phase);
        let result = evaluate_token_budget(&rule, &context).unwrap();

        assert!(result.is_some()); // 6000 > 5000
    }
}

/// Evaluate a token_budget rule
fn evaluate_token_budget(
    rule: &RuleConfig,
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    let max_tokens = match rule {
        RuleConfig::TokenBudget { max_tokens } => max_tokens,
        _ => return Ok(None),
    };

    let phase_metrics = match context.phase_metrics {
        Some(pm) => pm,
        None => return Ok(None), // No phase metrics available
    };

    // Calculate total tokens (input + output, per SPEC cache tokens excluded)
    let total_tokens = phase_metrics.token_metrics.total_input_tokens
        + phase_metrics.token_metrics.total_output_tokens;

    if total_tokens > *max_tokens {
        let recent_events = vec![
            format!(
                "Input tokens: {}",
                phase_metrics.token_metrics.total_input_tokens
            ),
            format!(
                "Output tokens: {}",
                phase_metrics.token_metrics.total_output_tokens
            ),
            format!("Total: {} (limit: {})", total_tokens, max_tokens),
            format!("Turns: {}", phase_metrics.token_metrics.assistant_turns),
        ];

        Ok(Some(RuleViolation {
            rule_type: "Token Budget".to_string(),
            diagnostic: format!(
                "{} phase used {} tokens (limit: {})",
                phase_metrics.phase_name, total_tokens, max_tokens
            ),
            suggestion: "You've exceeded the token budget for this phase. Consider simplifying scope, deferring non-critical work, or transitioning to document progress before resetting.".to_string(),
            recent_events,
        }))
    } else {
        Ok(None)
    }
}

/// Evaluate a phase_timeout rule
fn evaluate_phase_timeout(
    rule: &RuleConfig,
    context: &RuleEvaluationContext,
) -> Result<Option<RuleViolation>> {
    let max_duration = match rule {
        RuleConfig::PhaseTimeout { max_duration } => max_duration,
        _ => return Ok(None),
    };

    let phase_metrics = match context.phase_metrics {
        Some(pm) => pm,
        None => return Ok(None), // No phase metrics available
    };

    // Calculate duration
    let duration_secs = if let Some(ref end_time) = phase_metrics.end_time {
        // Completed phase - calculate from timestamps
        let start = DateTime::parse_from_rfc3339(&phase_metrics.start_time)?;
        let end = DateTime::parse_from_rfc3339(end_time)?;
        (end - start).num_seconds() as u64
    } else {
        // Active phase - calculate from current time
        let start = DateTime::parse_from_rfc3339(&phase_metrics.start_time)?;
        let now = chrono::Utc::now().with_timezone(start.offset());
        (now - start).num_seconds() as u64
    };

    if duration_secs > *max_duration {
        let minutes = duration_secs / 60;
        let seconds = duration_secs % 60;
        let limit_minutes = max_duration / 60;

        let recent_events = vec![
            format!("Phase start: {}", &phase_metrics.start_time[11..19]),
            format!("Duration: {}m {}s", minutes, seconds),
            format!("Limit: {}m", limit_minutes),
        ];

        Ok(Some(RuleViolation {
            rule_type: "Phase Timeout".to_string(),
            diagnostic: format!(
                "{} phase running for {}s (limit: {}s)",
                phase_metrics.phase_name, duration_secs, max_duration
            ),
            suggestion: "This phase is taking too long. Consider breaking the task into smaller steps, transitioning to LEARNINGS to document blockers, or resetting with simplified scope.".to_string(),
            recent_events,
        }))
    } else {
        Ok(None)
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
