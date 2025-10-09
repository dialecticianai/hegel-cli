use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Validate guide name to prevent path traversal attacks
fn validate_guide_name(guide_name: &str) -> Result<()> {
    // Reject empty names
    if guide_name.is_empty() {
        return Err(anyhow!("Guide name cannot be empty"));
    }

    // Reject path traversal attempts
    if guide_name.contains("..") {
        return Err(anyhow!("Path traversal not allowed in guide names"));
    }

    // Reject absolute paths
    if guide_name.starts_with('/') || guide_name.starts_with('\\') {
        return Err(anyhow!("Absolute paths not allowed in guide names"));
    }

    // Reject subdirectories
    if guide_name.contains('/') || guide_name.contains('\\') {
        return Err(anyhow!("Subdirectories not allowed in guide names"));
    }

    Ok(())
}

/// Render a template string by replacing placeholders with guide content and context variables
///
/// Placeholder types:
/// - {{UPPERCASE}} - Load guide from guides/UPPERCASE.md (required, error if missing)
/// - {{?lowercase}} - Replace with context value if present, empty string otherwise (optional)
/// - {{lowercase}} - Replace with context value (required, error if missing)
pub fn render_template(
    template: &str,
    guides_dir: &Path,
    context: &HashMap<String, String>,
) -> Result<String> {
    let mut result = template.to_string();

    // First, handle guide placeholders ({{UPPERCASE}})
    // Use permissive regex and validate afterwards for security
    let guide_re = Regex::new(r"\{\{([^{}?]+)\}\}").unwrap();
    for cap in guide_re.captures_iter(template) {
        let guide_name = &cap[1];

        // Skip if it's lowercase (context variable, handled later)
        if guide_name.chars().all(|c| c.is_lowercase() || c == '_') {
            continue;
        }

        // Validate guide name for security
        validate_guide_name(guide_name)?;

        let guide_path = guides_dir.join(format!("{}.md", guide_name));

        let guide_content = fs::read_to_string(&guide_path)
            .with_context(|| format!("Failed to load required guide: {}.md", guide_name))?;

        result = result.replace(&format!("{{{{{}}}}}", guide_name), &guide_content);
    }

    // Handle optional context variables ({{?lowercase}})
    let optional_re = Regex::new(r"\{\{\?([a-z_]+)\}\}").unwrap();
    for cap in optional_re.captures_iter(&result.clone()) {
        let var_name = &cap[1];
        let replacement = context.get(var_name).map(|s| s.as_str()).unwrap_or("");
        result = result.replace(&format!("{{{{?{}}}}}", var_name), replacement);
    }

    // Handle required context variables ({{lowercase}})
    let required_re = Regex::new(r"\{\{([a-z_]+)\}\}").unwrap();
    for cap in required_re.captures_iter(&result.clone()) {
        let var_name = &cap[1];
        let value = context
            .get(var_name)
            .with_context(|| format!("Required context variable missing: {}", var_name))?;
        result = result.replace(&format!("{{{{{}}}}}", var_name), value);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Helper to create test guides directory
    fn create_test_guides_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let guides_path = temp_dir.path().join("guides");
        std::fs::create_dir(&guides_path).unwrap();

        // Create test guide files
        std::fs::write(
            guides_path.join("SPEC_WRITING.md"),
            "# SPEC Writing Guide\n\nWrite a specification document.",
        )
        .unwrap();

        std::fs::write(
            guides_path.join("PLAN_WRITING.md"),
            "# PLAN Writing Guide\n\nWrite an implementation plan.",
        )
        .unwrap();

        temp_dir
    }

    // ========== Optional Placeholder Tests ==========

    #[test]
    fn test_optional_placeholder_with_value() {
        let template = "Hello {{?name}}!";
        let mut context = HashMap::new();
        context.insert("name".to_string(), "World".to_string());

        let result = render_template(template, &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_optional_placeholder_without_value() {
        let template = "Hello {{?name}}!";
        let context = HashMap::new();

        let result = render_template(template, &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "Hello !");
    }

    #[test]
    fn test_optional_placeholder_with_underscores() {
        let template = "Goal: {{?user_goals}}";
        let mut context = HashMap::new();
        context.insert("user_goals".to_string(), "Focus on performance".to_string());

        let result = render_template(template, &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "Goal: Focus on performance");
    }

    #[test]
    fn test_multiple_optional_placeholders_mixed() {
        let template = "Task: {{task}}\n\n{{?context_a}}\n\n{{?context_b}}\n\nBegin.";
        let mut context = HashMap::new();
        context.insert("task".to_string(), "Write tests".to_string());
        context.insert("context_b".to_string(), "Use Rust".to_string());

        let result = render_template(template, &PathBuf::from("guides"), &context).unwrap();
        assert!(result.contains("Write tests"));
        assert!(result.contains("Use Rust"));
        assert!(!result.contains("context_a"));
    }

    // ========== Required Placeholder Tests ==========

    #[test]
    fn test_required_placeholder_with_value() {
        let template = "Write a {{doc_type}}.";
        let mut context = HashMap::new();
        context.insert("doc_type".to_string(), "SPEC.md".to_string());

        let result = render_template(template, &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "Write a SPEC.md.");
    }

    #[test]
    fn test_required_placeholder_missing() {
        let template = "Hello {{name}}!";
        let context = HashMap::new();

        let result = render_template(template, &PathBuf::from("guides"), &context);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Required context variable missing"));
        assert!(err_msg.contains("name"));
    }

    #[test]
    fn test_multiple_required_placeholders() {
        let template = "Write a {{doc_type}} for {{system}}.";
        let mut context = HashMap::new();
        context.insert("doc_type".to_string(), "SPEC.md".to_string());
        context.insert("system".to_string(), "workflow engine".to_string());

        let result = render_template(template, &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "Write a SPEC.md for workflow engine.");
    }

    #[test]
    fn test_same_placeholder_appears_multiple_times() {
        let template = "{{name}} wrote a {{doc_type}}. The {{doc_type}} was good.";
        let mut context = HashMap::new();
        context.insert("name".to_string(), "Alice".to_string());
        context.insert("doc_type".to_string(), "SPEC".to_string());

        let result = render_template(template, &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "Alice wrote a SPEC. The SPEC was good.");
        assert_eq!(result.matches("SPEC").count(), 2);
    }

    #[test]
    fn test_empty_string_value() {
        let template = "Header: {{header}}\n\nBody";
        let mut context = HashMap::new();
        context.insert("header".to_string(), "".to_string());

        let result = render_template(template, &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "Header: \n\nBody");
    }

    #[test]
    fn test_extra_context_keys_ignored() {
        let template = "Write a {{doc_type}}.";
        let mut context = HashMap::new();
        context.insert("doc_type".to_string(), "SPEC.md".to_string());
        context.insert("extra_key".to_string(), "ignored".to_string());
        context.insert("another_key".to_string(), "also ignored".to_string());

        let result = render_template(template, &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "Write a SPEC.md.");
        assert!(!result.contains("extra_key"));
        assert!(!result.contains("ignored"));
    }

    // ========== Guide Loading Tests ==========

    #[test]
    fn test_guide_placeholder_loads_file() {
        let temp_dir = create_test_guides_dir();
        let guides_path = temp_dir.path().join("guides");

        let template = "Follow this guide:\n\n{{SPEC_WRITING}}";
        let context = HashMap::new();

        let result = render_template(template, &guides_path, &context).unwrap();
        assert!(result.contains("SPEC Writing Guide"));
        assert!(result.contains("specification document"));
    }

    #[test]
    fn test_guide_placeholder_missing_file() {
        let temp_dir = create_test_guides_dir();
        let guides_path = temp_dir.path().join("guides");

        let template = "{{NONEXISTENT}}";
        let context = HashMap::new();

        let result = render_template(template, &guides_path, &context);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to load required guide"));
        assert!(err_msg.contains("NONEXISTENT.md"));
    }

    #[test]
    fn test_multiple_guide_placeholders() {
        let temp_dir = create_test_guides_dir();
        let guides_path = temp_dir.path().join("guides");

        let template = "Phase 1:\n{{SPEC_WRITING}}\n\n---\n\nPhase 2:\n{{PLAN_WRITING}}";
        let context = HashMap::new();

        let result = render_template(template, &guides_path, &context).unwrap();
        assert!(result.contains("SPEC Writing Guide"));
        assert!(result.contains("PLAN Writing Guide"));
    }

    // ========== Mixed Placeholder Tests ==========

    #[test]
    fn test_mixed_required_and_optional_and_guides() {
        let temp_dir = create_test_guides_dir();
        let guides_path = temp_dir.path().join("guides");

        let template = "Write a {{doc_type}} for {{system}}.\n\n{{SPEC_WRITING}}\n\n{{?user_context}}\n\nFollow TDD.";
        let mut context = HashMap::new();
        context.insert("doc_type".to_string(), "SPEC.md".to_string());
        context.insert("system".to_string(), "template engine".to_string());

        let result = render_template(template, &guides_path, &context).unwrap();
        assert!(result.contains("Write a SPEC.md for template engine."));
        assert!(result.contains("SPEC Writing Guide"));
        assert!(result.contains("Follow TDD."));
        assert!(!result.contains("user_context"));
    }

    // ========== Empty Template Tests ==========

    #[test]
    fn test_empty_template() {
        let template = "This has no placeholders.";
        let context = HashMap::new();

        let result = render_template(template, &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "This has no placeholders.");
    }

    #[test]
    fn test_template_with_only_text() {
        let template = "Just plain text, no variables at all.";
        let context = HashMap::new();

        let result = render_template(template, &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "Just plain text, no variables at all.");
    }

    // ========== Security Tests ==========

    #[test]
    fn test_path_traversal_rejected() {
        let temp_dir = create_test_guides_dir();
        let guides_path = temp_dir.path().join("guides");

        // Try to escape guides directory with ..
        let template = "{{../../../etc/passwd}}";
        let context = HashMap::new();

        let result = render_template(template, &guides_path, &context);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Path traversal not allowed") || err_msg.contains("Failed to load")
        );
    }

    #[test]
    fn test_path_traversal_with_dots_rejected() {
        let temp_dir = create_test_guides_dir();
        let guides_path = temp_dir.path().join("guides");

        let template = "{{..SPEC_WRITING}}";
        let context = HashMap::new();

        let result = render_template(template, &guides_path, &context);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Path traversal not allowed") || err_msg.contains("Failed to load")
        );
    }

    #[test]
    fn test_absolute_path_unix_rejected() {
        let temp_dir = create_test_guides_dir();
        let guides_path = temp_dir.path().join("guides");

        let template = "{{/etc/passwd}}";
        let context = HashMap::new();

        let result = render_template(template, &guides_path, &context);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Absolute paths not allowed") || err_msg.contains("Failed to load")
        );
    }

    #[test]
    fn test_subdirectory_rejected() {
        let temp_dir = create_test_guides_dir();
        let guides_path = temp_dir.path().join("guides");

        let template = "{{subdir/GUIDE}}";
        let context = HashMap::new();

        let result = render_template(template, &guides_path, &context);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Subdirectories not allowed") || err_msg.contains("Failed to load")
        );
    }

    #[test]
    fn test_validate_guide_name_empty() {
        let result = validate_guide_name("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_guide_name_path_traversal() {
        let result = validate_guide_name("../etc/passwd");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Path traversal not allowed"));
    }

    #[test]
    fn test_validate_guide_name_absolute_path() {
        let result = validate_guide_name("/etc/passwd");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Absolute paths not allowed"));
    }

    #[test]
    fn test_validate_guide_name_subdirectory() {
        let result = validate_guide_name("subdir/guide");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Subdirectories not allowed"));
    }

    #[test]
    fn test_validate_guide_name_valid() {
        let result = validate_guide_name("SPEC_WRITING");
        assert!(result.is_ok());

        let result2 = validate_guide_name("PLAN_WRITING");
        assert!(result2.is_ok());
    }
}
