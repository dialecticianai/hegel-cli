use crate::config::HegelConfig;
use crate::metrics::{BashCommand, FileModification, HookMetrics, PhaseMetrics, TokenMetrics};
use crate::rules::evaluator::*;
use crate::rules::types::{RuleConfig, RuleEvaluationContext};

// ========== Test Helpers ==========

/// Base timestamp for all tests (2025-01-01 10:00:00 UTC)
const BASE_TIME: &str = "2025-01-01T10:00:00Z";

/// Create timestamp offset from BASE_TIME by seconds
fn time(offset_secs: i64) -> String {
    let base = chrono::DateTime::parse_from_rfc3339(BASE_TIME).unwrap();
    (base + chrono::Duration::seconds(offset_secs))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Create BashCommand with minimal boilerplate
fn cmd(command: &str, time_offset: i64) -> BashCommand {
    BashCommand {
        command: command.to_string(),
        timestamp: Some(time(time_offset)),
        stdout: None,
        stderr: None,
    }
}

/// Create FileModification with minimal boilerplate
fn edit(path: &str, time_offset: i64) -> FileModification {
    FileModification {
        file_path: path.to_string(),
        tool: "Edit".to_string(),
        timestamp: Some(time(time_offset)),
    }
}

/// Create PhaseMetrics with sensible defaults
///
/// Creates a completed phase with known duration for testing.
/// Uses BASE_TIME as start, and BASE_TIME + duration_secs as end.
fn phase(duration_secs: u64, input_tokens: u64, output_tokens: u64) -> PhaseMetrics {
    use crate::test_helpers::test_phase_metrics_with;

    PhaseMetrics {
        phase_name: "code".to_string(),
        start_time: BASE_TIME.to_string(),
        end_time: Some(time(duration_secs as i64)), // Completed phase
        duration_seconds: duration_secs,
        token_metrics: TokenMetrics {
            total_input_tokens: input_tokens,
            total_output_tokens: output_tokens,
            total_cache_creation_tokens: 0,
            total_cache_read_tokens: 0,
            assistant_turns: 10,
        },
        ..test_phase_metrics_with(true)
    }
}

/// Create RuleEvaluationContext with commands
fn ctx_with_cmds(commands: Vec<BashCommand>) -> RuleEvaluationContext<'static> {
    let hook_metrics = Box::leak(Box::new(HookMetrics {
        total_events: commands.len(),
        bash_commands: commands,
        file_modifications: vec![],
        session_start_time: None,
        session_end_time: None,
    }));

    let config = Box::leak(Box::new(HegelConfig::default()));
    let all_phase_metrics = Box::leak(Box::new([]));

    RuleEvaluationContext {
        current_phase: "code",
        phase_start_time: Some(Box::leak(Box::new(BASE_TIME.to_string()))),
        all_phase_metrics,
        hook_metrics,
        config,
        git_info: None,
    }
}

/// Create RuleEvaluationContext with file edits
fn ctx_with_edits(edits: Vec<FileModification>) -> RuleEvaluationContext<'static> {
    let hook_metrics = Box::leak(Box::new(HookMetrics {
        total_events: edits.len(),
        bash_commands: vec![],
        file_modifications: edits,
        session_start_time: None,
        session_end_time: None,
    }));

    let config = Box::leak(Box::new(HegelConfig::default()));
    let all_phase_metrics = Box::leak(Box::new([]));

    RuleEvaluationContext {
        current_phase: "code",
        phase_start_time: Some(Box::leak(Box::new(BASE_TIME.to_string()))),
        all_phase_metrics,
        hook_metrics,
        config,
        git_info: None,
    }
}

/// Create RuleEvaluationContext with phase metrics
fn ctx_with_phase(phase: PhaseMetrics) -> RuleEvaluationContext<'static> {
    let phase_vec = Box::leak(Box::new(vec![phase]));
    let hook_metrics = Box::leak(Box::new(HookMetrics::default()));
    let config = Box::leak(Box::new(HegelConfig::default()));

    RuleEvaluationContext {
        current_phase: "code",
        phase_start_time: Some(&phase_vec[0].start_time),
        all_phase_metrics: phase_vec.as_slice(),
        hook_metrics,
        config,
        git_info: None,
    }
}

// ========== Unified Rule Evaluation Tests (Orchestration) ==========

#[test]
fn test_evaluate_rules_returns_first_violation() {
    let context = ctx_with_cmds(vec![cmd("cargo build", 0); 5]);

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
    assert_eq!(result.unwrap().rule_type, "Repeated Command");
}

#[test]
fn test_evaluate_rules_no_violations() {
    let context = ctx_with_cmds(vec![]);

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
    let context = ctx_with_cmds(vec![]);
    let result = evaluate_rules(&[], &context).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_evaluate_rules_short_circuit() {
    let hook_metrics = Box::leak(Box::new(HookMetrics {
        total_events: 15,
        bash_commands: vec![cmd("cargo build", 0); 5],
        file_modifications: vec![edit("src/main.rs", 0); 10],
        session_start_time: None,
        session_end_time: None,
    }));

    let config = Box::leak(Box::new(HegelConfig::default()));
    let all_phase_metrics = Box::leak(Box::new([]));

    let context = RuleEvaluationContext {
        current_phase: "code",
        phase_start_time: Some(Box::leak(Box::new(BASE_TIME.to_string()))),
        all_phase_metrics,
        hook_metrics,
        config,
        git_info: None,
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
    assert_eq!(result.unwrap().rule_type, "Repeated Command");
}

#[test]
fn test_evaluate_rules_mixed_types() {
    let context = ctx_with_phase(phase(900, 6000, 1000));

    let rules = vec![
        RuleConfig::RepeatedCommand {
            pattern: None,
            threshold: 100,
            window: 120,
        },
        RuleConfig::RepeatedFileEdit {
            path_pattern: None,
            threshold: 100,
            window: 180,
        },
        RuleConfig::PhaseTimeout { max_duration: 600 }, // Triggers (900s > 600s)
        RuleConfig::TokenBudget { max_tokens: 10000 },
    ];

    let result = evaluate_rules(&rules, &context).unwrap();

    assert!(result.is_some());
    assert_eq!(result.unwrap().rule_type, "Phase Timeout");
}

#[test]
fn test_evaluate_rules_all_pass() {
    let context = ctx_with_phase(phase(300, 2000, 1500));

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
    assert!(result.is_none());
}

// ==========Repeated Command Evaluation Tests ==========

#[test]
fn test_repeated_command_triggers_at_threshold() {
    let commands = vec![
        cmd("cargo build", 0),
        cmd("cargo build", 30),
        cmd("cargo build", 60),
        cmd("cargo build", 90),
        cmd("cargo build", 119),
    ];

    let rule = RuleConfig::RepeatedCommand {
        pattern: Some("cargo build".to_string()),
        threshold: 5,
        window: 120,
    };

    let context = ctx_with_cmds(commands);
    let result = evaluate_repeated_command(&rule, &context).unwrap();

    assert!(result.is_some());
    let violation = result.unwrap();
    assert_eq!(violation.rule_type, "Repeated Command");
    assert!(violation.diagnostic.contains("5"));
}

#[test]
fn test_repeated_command_below_threshold_no_trigger() {
    let commands = vec![
        cmd("cargo build", 0),
        cmd("cargo build", 30),
        cmd("cargo build", 60),
        cmd("cargo build", 90),
    ];

    let rule = RuleConfig::RepeatedCommand {
        pattern: Some("cargo build".to_string()),
        threshold: 5,
        window: 120,
    };

    let context = ctx_with_cmds(commands);
    let result = evaluate_repeated_command(&rule, &context).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_repeated_command_outside_window_no_trigger() {
    let commands = vec![
        cmd("cargo build", -180), // 3 min before
        cmd("cargo build", -120), // 2 min before
        cmd("cargo build", -60),  // 1 min before
        cmd("cargo build", 0),    // Inside window
        cmd("cargo build", 60),
        cmd("cargo build", 119),
    ];

    let rule = RuleConfig::RepeatedCommand {
        pattern: Some("cargo build".to_string()),
        threshold: 5,
        window: 120,
    };

    let context = ctx_with_cmds(commands);
    let result = evaluate_repeated_command(&rule, &context).unwrap();

    assert!(result.is_none()); // Only 3 in window
}

#[test]
fn test_repeated_command_regex_matches_multiple() {
    let commands = vec![
        cmd("cargo build", 0),
        cmd("cargo test", 30),
        cmd("cargo build", 60),
        cmd("cargo test", 90),
        cmd("cargo build", 119),
    ];

    let rule = RuleConfig::RepeatedCommand {
        pattern: Some("cargo (build|test)".to_string()),
        threshold: 5,
        window: 120,
    };

    let context = ctx_with_cmds(commands);
    let result = evaluate_repeated_command(&rule, &context).unwrap();

    assert!(result.is_some());
}

#[test]
fn test_repeated_command_regex_excludes_non_matching() {
    let commands = vec![
        cmd("cargo build", 0),
        cmd("git status", 10),
        cmd("cargo fmt", 20),
        cmd("cargo test", 30),
        cmd("git status", 40),
    ];

    let rule = RuleConfig::RepeatedCommand {
        pattern: Some("cargo (build|test)".to_string()),
        threshold: 5,
        window: 120,
    };

    let context = ctx_with_cmds(commands);
    let result = evaluate_repeated_command(&rule, &context).unwrap();

    assert!(result.is_none()); // Only 2 matches
}

#[test]
fn test_repeated_command_no_pattern_matches_all() {
    let commands = vec![
        cmd("cargo build", 0),
        cmd("git status", 10),
        cmd("ls -la", 20),
        cmd("pwd", 30),
        cmd("echo hello", 40),
    ];

    let rule = RuleConfig::RepeatedCommand {
        pattern: None,
        threshold: 5,
        window: 120,
    };

    let context = ctx_with_cmds(commands);
    let result = evaluate_repeated_command(&rule, &context).unwrap();

    assert!(result.is_some());
}

#[test]
fn test_repeated_command_empty_list_no_trigger() {
    let rule = RuleConfig::RepeatedCommand {
        pattern: Some("cargo build".to_string()),
        threshold: 5,
        window: 120,
    };

    let context = ctx_with_cmds(vec![]);
    let result = evaluate_repeated_command(&rule, &context).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_repeated_command_exact_window_boundary() {
    let commands = vec![
        cmd("cargo build", 0),
        cmd("cargo build", 60),
        cmd("cargo build", 120),
    ];

    let rule = RuleConfig::RepeatedCommand {
        pattern: Some("cargo build".to_string()),
        threshold: 3,
        window: 120,
    };

    let context = ctx_with_cmds(commands);
    let result = evaluate_repeated_command(&rule, &context).unwrap();

    assert!(result.is_some());
}

// ========== Repeated File Edit Evaluation Tests ==========

#[test]
fn test_repeated_file_edit_triggers_at_threshold() {
    let edits = vec![
        edit("src/main.rs", 0),
        edit("src/main.rs", 30),
        edit("src/lib.rs", 60),
        edit("src/main.rs", 90),
        edit("src/utils.rs", 120),
        edit("src/main.rs", 150),
        edit("src/lib.rs", 179),
        edit("src/types.rs", 180),
    ];

    let rule = RuleConfig::RepeatedFileEdit {
        path_pattern: Some(r"src/.*\.rs".to_string()),
        threshold: 8,
        window: 180,
    };

    let context = ctx_with_edits(edits);
    let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

    assert!(result.is_some());
    let violation = result.unwrap();
    assert_eq!(violation.rule_type, "Repeated File Edit");
    assert!(violation.diagnostic.contains("8"));
}

#[test]
fn test_repeated_file_edit_below_threshold_no_trigger() {
    let edits = vec![edit("src/main.rs", 0), edit("src/lib.rs", 30)];

    let rule = RuleConfig::RepeatedFileEdit {
        path_pattern: Some(r"src/.*\.rs".to_string()),
        threshold: 5,
        window: 180,
    };

    let context = ctx_with_edits(edits);
    let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_repeated_file_edit_path_pattern_matches() {
    let edits = vec![
        edit("src/main.rs", 0),
        edit("src/lib.rs", 10),
        edit("README.md", 20),
        edit("src/utils.rs", 30),
        edit("Cargo.toml", 40),
    ];

    let rule = RuleConfig::RepeatedFileEdit {
        path_pattern: Some(r"src/.*\.rs".to_string()),
        threshold: 3,
        window: 180,
    };

    let context = ctx_with_edits(edits);
    let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

    assert!(result.is_some());
}

#[test]
fn test_repeated_file_edit_path_pattern_excludes() {
    let edits = vec![
        edit("README.md", 0),
        edit("Cargo.toml", 10),
        edit("src/main.rs", 20),
    ];

    let rule = RuleConfig::RepeatedFileEdit {
        path_pattern: Some(r"src/.*\.rs".to_string()),
        threshold: 3,
        window: 180,
    };

    let context = ctx_with_edits(edits);
    let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_repeated_file_edit_no_pattern_matches_all() {
    let edits = vec![
        edit("src/main.rs", 0),
        edit("README.md", 10),
        edit("Cargo.toml", 20),
    ];

    let rule = RuleConfig::RepeatedFileEdit {
        path_pattern: None,
        threshold: 3,
        window: 180,
    };

    let context = ctx_with_edits(edits);
    let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

    assert!(result.is_some());
}

#[test]
fn test_repeated_file_edit_empty_list_no_trigger() {
    let rule = RuleConfig::RepeatedFileEdit {
        path_pattern: Some(r"src/.*\.rs".to_string()),
        threshold: 5,
        window: 180,
    };

    let context = ctx_with_edits(vec![]);
    let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_repeated_file_edit_outside_window_no_trigger() {
    let edits = vec![
        edit("src/main.rs", -180),
        edit("src/lib.rs", -120),
        edit("src/utils.rs", 0),
        edit("src/types.rs", 60),
    ];

    let rule = RuleConfig::RepeatedFileEdit {
        path_pattern: Some(r"src/.*\.rs".to_string()),
        threshold: 5,
        window: 120,
    };

    let context = ctx_with_edits(edits);
    let result = evaluate_repeated_file_edit(&rule, &context).unwrap();

    assert!(result.is_none());
}

// ========== Phase Timeout Evaluation Tests ==========

#[test]
fn test_phase_timeout_triggers_when_exceeded() {
    let rule = RuleConfig::PhaseTimeout { max_duration: 600 };
    let context = ctx_with_phase(phase(720, 0, 0));
    let result = evaluate_phase_timeout(&rule, &context).unwrap();

    assert!(result.is_some());
    let violation = result.unwrap();
    assert_eq!(violation.rule_type, "Phase Timeout");
    assert!(violation.diagnostic.contains("720"));
}

#[test]
fn test_phase_timeout_no_trigger_below_limit() {
    let rule = RuleConfig::PhaseTimeout { max_duration: 600 };
    let context = ctx_with_phase(phase(480, 0, 0));
    let result = evaluate_phase_timeout(&rule, &context).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_phase_timeout_active_phase_uses_current_time() {
    use chrono::Utc;

    use crate::test_helpers::test_phase_metrics_with;

    let start = (Utc::now() - chrono::Duration::seconds(700)).to_rfc3339();
    let phase_vec = Box::leak(Box::new(vec![PhaseMetrics {
        phase_name: "code".to_string(),
        start_time: start.clone(),
        end_time: None,
        duration_seconds: 0,
        token_metrics: TokenMetrics::default(),
        ..test_phase_metrics_with(true)
    }]));

    let config = Box::leak(Box::new(HegelConfig::default()));

    let context = RuleEvaluationContext {
        current_phase: "code",
        phase_start_time: Some(&phase_vec[0].start_time),
        all_phase_metrics: phase_vec.as_slice(),
        hook_metrics: Box::leak(Box::new(HookMetrics::default())),
        config,
        git_info: None,
    };

    let rule = RuleConfig::PhaseTimeout { max_duration: 600 };
    let result = evaluate_phase_timeout(&rule, &context).unwrap();

    assert!(result.is_some());
}

#[test]
fn test_phase_timeout_exact_limit_no_trigger() {
    let rule = RuleConfig::PhaseTimeout { max_duration: 600 };
    let context = ctx_with_phase(phase(600, 0, 0));
    let result = evaluate_phase_timeout(&rule, &context).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_phase_timeout_zero_duration_no_trigger() {
    let rule = RuleConfig::PhaseTimeout { max_duration: 600 };
    let context = ctx_with_phase(phase(0, 0, 0));
    let result = evaluate_phase_timeout(&rule, &context).unwrap();

    assert!(result.is_none());
}

// ========== Token Budget Evaluation Tests ==========

#[test]
fn test_token_budget_triggers_when_exceeded() {
    let rule = RuleConfig::TokenBudget { max_tokens: 6000 };
    let context = ctx_with_phase(phase(0, 4000, 2500));
    let result = evaluate_token_budget(&rule, &context).unwrap();

    assert!(result.is_some());
    let violation = result.unwrap();
    assert_eq!(violation.rule_type, "Token Budget");
    assert!(violation.diagnostic.contains("6500"));
    assert!(violation.diagnostic.contains("6000"));
}

#[test]
fn test_token_budget_no_trigger_below_limit() {
    let rule = RuleConfig::TokenBudget { max_tokens: 5000 };
    let context = ctx_with_phase(phase(0, 2000, 1500));
    let result = evaluate_token_budget(&rule, &context).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_token_budget_exact_limit_no_trigger() {
    let rule = RuleConfig::TokenBudget { max_tokens: 5000 };
    let context = ctx_with_phase(phase(0, 3000, 2000));
    let result = evaluate_token_budget(&rule, &context).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_token_budget_zero_tokens_no_trigger() {
    let rule = RuleConfig::TokenBudget { max_tokens: 5000 };
    let context = ctx_with_phase(phase(0, 0, 0));
    let result = evaluate_token_budget(&rule, &context).unwrap();

    assert!(result.is_none());
}

#[test]
fn test_token_budget_includes_input_and_output() {
    let rule = RuleConfig::TokenBudget { max_tokens: 5000 };
    let context = ctx_with_phase(phase(0, 3000, 3000));
    let result = evaluate_token_budget(&rule, &context).unwrap();

    assert!(result.is_some()); // 6000 > 5000
}

#[test]
fn test_token_budget_only_input_tokens() {
    let rule = RuleConfig::TokenBudget { max_tokens: 5000 };
    let context = ctx_with_phase(phase(0, 6000, 0));
    let result = evaluate_token_budget(&rule, &context).unwrap();

    assert!(result.is_some());
}
