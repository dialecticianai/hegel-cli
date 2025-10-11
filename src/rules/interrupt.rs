use super::types::RuleViolation;

/// Generate interrupt prompt from rule violation
pub fn generate_interrupt_prompt(violation: &RuleViolation) -> String {
    let mut prompt = String::new();

    // Header
    prompt.push_str(&format!("⚠️  **{}**\n\n", violation.rule_type));

    // Diagnostic
    prompt.push_str(&format!("{}\n\n", violation.diagnostic));

    // Recent events (if any)
    if !violation.recent_events.is_empty() {
        prompt.push_str("**Recent Activity:**\n");
        for event in &violation.recent_events {
            prompt.push_str(&format!("- {}\n", event));
        }
        prompt.push_str("\n");
    }

    // Suggestion
    prompt.push_str(&format!("**Suggestion:** {}\n\n", violation.suggestion));

    // Decision prompt
    prompt.push_str("**What next?**\n");
    prompt.push_str("- Fix the issue and continue: `hegel continue`\n");
    prompt.push_str("- Escalate to human for review\n");

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_interrupt_repeated_command() {
        let violation = RuleViolation {
            rule_type: "Repeated Command".to_string(),
            diagnostic: "Command executed 5 times in last 120s".to_string(),
            suggestion: "You're stuck in a build loop. Review the error carefully.".to_string(),
            recent_events: vec![
                "10:00:00: cargo build".to_string(),
                "10:00:30: cargo build".to_string(),
                "10:01:00: cargo build".to_string(),
            ],
        };

        let prompt = generate_interrupt_prompt(&violation);

        assert!(prompt.contains("⚠️  **Repeated Command**"));
        assert!(prompt.contains("Command executed 5 times in last 120s"));
        assert!(prompt.contains("You're stuck in a build loop"));
        assert!(prompt.contains("**Recent Activity:**"));
        assert!(prompt.contains("10:00:00: cargo build"));
        assert!(prompt.contains("`hegel continue`"));
    }

    #[test]
    fn test_generate_interrupt_repeated_file_edit() {
        let violation = RuleViolation {
            rule_type: "Repeated File Edit".to_string(),
            diagnostic: "Files edited 8 times in last 180s".to_string(),
            suggestion: "Step back and write a failing test first.".to_string(),
            recent_events: vec![
                "10:00:00: src/main.rs (Edit)".to_string(),
                "10:00:30: src/lib.rs (Edit)".to_string(),
            ],
        };

        let prompt = generate_interrupt_prompt(&violation);

        assert!(prompt.contains("⚠️  **Repeated File Edit**"));
        assert!(prompt.contains("Files edited 8 times"));
        assert!(prompt.contains("write a failing test"));
        assert!(prompt.contains("src/main.rs (Edit)"));
    }

    #[test]
    fn test_generate_interrupt_phase_timeout() {
        let violation = RuleViolation {
            rule_type: "Phase Timeout".to_string(),
            diagnostic: "code phase running for 720s (limit: 600s)".to_string(),
            suggestion: "Consider breaking into smaller steps.".to_string(),
            recent_events: vec![
                "Phase start: 10:00:00".to_string(),
                "Duration: 12m 0s".to_string(),
                "Limit: 10m".to_string(),
            ],
        };

        let prompt = generate_interrupt_prompt(&violation);

        assert!(prompt.contains("⚠️  **Phase Timeout**"));
        assert!(prompt.contains("720s (limit: 600s)"));
        assert!(prompt.contains("breaking into smaller steps"));
        assert!(prompt.contains("Duration: 12m 0s"));
    }

    #[test]
    fn test_generate_interrupt_token_budget() {
        let violation = RuleViolation {
            rule_type: "Token Budget".to_string(),
            diagnostic: "code phase used 6500 tokens (limit: 6000)".to_string(),
            suggestion: "Consider simplifying scope.".to_string(),
            recent_events: vec![
                "Input tokens: 4000".to_string(),
                "Output tokens: 2500".to_string(),
                "Total: 6500 (limit: 6000)".to_string(),
                "Turns: 10".to_string(),
            ],
        };

        let prompt = generate_interrupt_prompt(&violation);

        assert!(prompt.contains("⚠️  **Token Budget**"));
        assert!(prompt.contains("6500 tokens (limit: 6000)"));
        assert!(prompt.contains("simplifying scope"));
        assert!(prompt.contains("Input tokens: 4000"));
        assert!(prompt.contains("Output tokens: 2500"));
    }

    #[test]
    fn test_generate_interrupt_empty_recent_events() {
        let violation = RuleViolation {
            rule_type: "Test Rule".to_string(),
            diagnostic: "Test diagnostic".to_string(),
            suggestion: "Test suggestion".to_string(),
            recent_events: vec![],
        };

        let prompt = generate_interrupt_prompt(&violation);

        assert!(prompt.contains("⚠️  **Test Rule**"));
        assert!(prompt.contains("Test diagnostic"));
        assert!(prompt.contains("Test suggestion"));
        assert!(!prompt.contains("**Recent Activity:**"));
    }

    #[test]
    fn test_generate_interrupt_has_decision_prompt() {
        let violation = RuleViolation {
            rule_type: "Test".to_string(),
            diagnostic: "Test".to_string(),
            suggestion: "Test".to_string(),
            recent_events: vec![],
        };

        let prompt = generate_interrupt_prompt(&violation);

        assert!(prompt.contains("**What next?**"));
        assert!(prompt.contains("`hegel continue`"));
        assert!(prompt.contains("Escalate to human"));
    }

    #[test]
    fn test_generate_interrupt_formatting() {
        let violation = RuleViolation {
            rule_type: "Test".to_string(),
            diagnostic: "Line 1\nLine 2".to_string(),
            suggestion: "Do this".to_string(),
            recent_events: vec!["Event 1".to_string()],
        };

        let prompt = generate_interrupt_prompt(&violation);

        // Should preserve newlines in diagnostic
        assert!(prompt.contains("Line 1\nLine 2"));
        // Should have proper section headers
        assert!(prompt.contains("**Recent Activity:**"));
        assert!(prompt.contains("**Suggestion:**"));
        assert!(prompt.contains("**What next?**"));
    }
}
