use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Rule configuration enum with all four rule types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleConfig {
    RepeatedCommand {
        #[serde(skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        threshold: usize,
        window: u64,
    },
    RepeatedFileEdit {
        #[serde(skip_serializing_if = "Option::is_none")]
        path_pattern: Option<String>,
        threshold: usize,
        window: u64,
    },
    PhaseTimeout {
        max_duration: u64,
    },
    TokenBudget {
        max_tokens: u64,
    },
}

/// Rule violation with diagnostic information
#[derive(Debug, Clone)]
pub struct RuleViolation {
    pub rule_type: String,
    pub diagnostic: String,
    pub suggestion: String,
    pub recent_events: Vec<String>,
}

/// Rule evaluation context for assessing violations
#[derive(Debug, Clone)]
pub struct RuleEvaluationContext<'a> {
    pub current_phase: &'a str,
    pub phase_start_time: &'a str,
    pub phase_metrics: Option<&'a crate::metrics::PhaseMetrics>,
    pub hook_metrics: &'a crate::metrics::HookMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== RuleConfig Deserialization Tests ==========

    #[test]
    fn test_deserialize_repeated_command_with_all_fields() {
        let yaml = r#"
type: repeated_command
pattern: "cargo (build|test)"
threshold: 5
window: 120
"#;
        let rule: RuleConfig = serde_yaml::from_str(yaml).unwrap();

        match rule {
            RuleConfig::RepeatedCommand {
                pattern,
                threshold,
                window,
            } => {
                assert_eq!(pattern, Some("cargo (build|test)".to_string()));
                assert_eq!(threshold, 5);
                assert_eq!(window, 120);
            }
            _ => panic!("Expected RepeatedCommand variant"),
        }
    }

    #[test]
    fn test_deserialize_repeated_command_with_no_pattern() {
        let yaml = r#"
type: repeated_command
threshold: 3
window: 60
"#;
        let rule: RuleConfig = serde_yaml::from_str(yaml).unwrap();

        match rule {
            RuleConfig::RepeatedCommand {
                pattern,
                threshold,
                window,
            } => {
                assert_eq!(pattern, None);
                assert_eq!(threshold, 3);
                assert_eq!(window, 60);
            }
            _ => panic!("Expected RepeatedCommand variant"),
        }
    }

    #[test]
    fn test_deserialize_repeated_file_edit_with_all_fields() {
        let yaml = r#"
type: repeated_file_edit
path_pattern: "src/.*\\.rs"
threshold: 8
window: 180
"#;
        let rule: RuleConfig = serde_yaml::from_str(yaml).unwrap();

        match rule {
            RuleConfig::RepeatedFileEdit {
                path_pattern,
                threshold,
                window,
            } => {
                assert_eq!(path_pattern, Some("src/.*\\.rs".to_string()));
                assert_eq!(threshold, 8);
                assert_eq!(window, 180);
            }
            _ => panic!("Expected RepeatedFileEdit variant"),
        }
    }

    #[test]
    fn test_deserialize_repeated_file_edit_with_no_pattern() {
        let yaml = r#"
type: repeated_file_edit
threshold: 6
window: 120
"#;
        let rule: RuleConfig = serde_yaml::from_str(yaml).unwrap();

        match rule {
            RuleConfig::RepeatedFileEdit {
                path_pattern,
                threshold,
                window,
            } => {
                assert_eq!(path_pattern, None);
                assert_eq!(threshold, 6);
                assert_eq!(window, 120);
            }
            _ => panic!("Expected RepeatedFileEdit variant"),
        }
    }

    #[test]
    fn test_deserialize_phase_timeout() {
        let yaml = r#"
type: phase_timeout
max_duration: 600
"#;
        let rule: RuleConfig = serde_yaml::from_str(yaml).unwrap();

        match rule {
            RuleConfig::PhaseTimeout { max_duration } => {
                assert_eq!(max_duration, 600);
            }
            _ => panic!("Expected PhaseTimeout variant"),
        }
    }

    #[test]
    fn test_deserialize_token_budget() {
        let yaml = r#"
type: token_budget
max_tokens: 5000
"#;
        let rule: RuleConfig = serde_yaml::from_str(yaml).unwrap();

        match rule {
            RuleConfig::TokenBudget { max_tokens } => {
                assert_eq!(max_tokens, 5000);
            }
            _ => panic!("Expected TokenBudget variant"),
        }
    }

    #[test]
    fn test_deserialize_unknown_type_returns_error() {
        let yaml = r#"
type: nonexistent_rule
threshold: 5
"#;
        let result: Result<RuleConfig, serde_yaml::Error> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown variant") || err.contains("nonexistent_rule"));
    }

    #[test]
    fn test_deserialize_missing_required_field_returns_error() {
        // Missing threshold for repeated_command
        let yaml = r#"
type: repeated_command
window: 60
"#;
        let result: Result<RuleConfig, serde_yaml::Error> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("missing field") || err.contains("threshold"));
    }
}
