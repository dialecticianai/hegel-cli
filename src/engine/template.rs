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

    // Allow templates/ subdirectory (for template includes)
    // Reject all other subdirectories
    if guide_name.contains('/') || guide_name.contains('\\') {
        if !guide_name.starts_with("templates/") {
            return Err(anyhow!(
                "Subdirectories not allowed in guide names (except templates/)"
            ));
        }
        // Validate that templates/ is followed by a simple name (no further slashes)
        let after_prefix = guide_name.strip_prefix("templates/").unwrap();
        if after_prefix.contains('/') || after_prefix.contains('\\') {
            return Err(anyhow!("Nested subdirectories not allowed in templates/"));
        }
    }

    Ok(())
}

/// Render a template string by replacing placeholders with guide content and context variables
///
/// Placeholder types:
/// - {{UPPERCASE}} - Load guide from guides/UPPERCASE.md (required, error if missing)
/// - {{templates/name}} - Load template from guides/templates/name.md (supports nesting)
/// - {{?lowercase}} - Replace with context value if present, empty string otherwise (optional)
/// - {{lowercase}} - Replace with context value (required, error if missing)
///
/// Processing order:
/// 1. Expand all context variables ({{lowercase}}, {{?lowercase}}) - single pass
/// 2. Expand all guide placeholders ({{UPPERCASE}}, {{templates/name}}) - recursive up to 10 levels
///
/// This two-phase approach allows:
/// - Guide names to contain variables: {{templates/code_map_{{style}}}} → {{templates/code_map_hierarchical}}
/// - Loaded guides to contain variables that get expanded in phase 1
pub fn render_template(
    template: &str,
    guides_dir: &Path,
    context: &HashMap<String, String>,
) -> Result<String> {
    let mut result = template.to_string();

    // Interleave context variable expansion and guide loading
    // This allows: {{GUIDE}} loads "{{templates/file_{{var}}}}" → expand {{var}} → load {{templates/file_X}}
    const MAX_ITERATIONS: usize = 10;
    for _iteration in 0..MAX_ITERATIONS {
        let before = result.clone();

        // Expand context variables
        result = expand_context_variables(&result, context)?;

        // Expand guides
        result = expand_guides(&result, guides_dir)?;

        // If nothing changed, we're done
        if result == before {
            break;
        }
    }

    Ok(result)
}

/// Expand context variables in a string (one pass)
fn expand_context_variables(text: &str, context: &HashMap<String, String>) -> Result<String> {
    let mut result = text.to_string();

    // Handle optional context variables ({{?lowercase}})
    let optional_re = Regex::new(r"\{\{\?([a-z_]+)\}\}").unwrap();
    for cap in optional_re.captures_iter(text) {
        let var_name = &cap[1];
        let replacement = context.get(var_name).map(|s| s.as_str()).unwrap_or("");
        result = result.replace(&format!("{{{{?{}}}}}", var_name), replacement);
    }

    // Handle required context variables ({{lowercase}})
    let required_re = Regex::new(r"\{\{([a-z_]+)\}\}").unwrap();
    for cap in required_re.captures_iter(text) {
        let var_name = &cap[1];
        let value = context
            .get(var_name)
            .with_context(|| format!("Required context variable missing: {}", var_name))?;
        result = result.replace(&format!("{{{{{}}}}}", var_name), value);
    }

    Ok(result)
}

/// Expand guide placeholders in a string (one pass)
fn expand_guides(text: &str, guides_dir: &Path) -> Result<String> {
    let mut result = text.to_string();

    // First, check for any invalid guide patterns and error explicitly
    // This catches security issues like {{/etc/passwd}} or {{../foo}}
    let all_placeholders_re = Regex::new(r"\{\{([^{}?]+)\}\}").unwrap();
    for cap in all_placeholders_re.captures_iter(text) {
        let name = &cap[1];

        // Skip if it's a lowercase variable (context vars are handled separately)
        if name.chars().all(|c| c.is_lowercase() || c == '_') {
            continue;
        }

        // Skip if it matches valid guide pattern (will be processed below)
        if name.chars().next().map_or(false, |c| c.is_uppercase()) || name.starts_with("templates/")
        {
            continue;
        }

        // If we got here, it's an invalid pattern - validate to get proper error
        validate_guide_name(name)?;
    }

    // Now expand valid guide patterns
    // Pattern: starts with uppercase OR "templates/"
    let guide_re = Regex::new(r"\{\{([A-Z_][A-Z0-9_]*|templates/[a-z_][a-z0-9_]*)\}\}").unwrap();

    for cap in guide_re.captures_iter(text) {
        let guide_name = &cap[1];

        // Validate guide name for security (should pass since we pre-validated)
        validate_guide_name(guide_name)?;

        let guide_filename = format!("{}.md", guide_name);

        // Try embedded guide first, then fall back to filesystem
        let guide_content = if let Some(embedded) = crate::embedded::get_guide(&guide_filename) {
            embedded.to_string()
        } else {
            // Fall back to guides_dir (for user overrides or local development)
            let guide_path = guides_dir.join(&guide_filename);
            fs::read_to_string(&guide_path)
                .with_context(|| format!("Failed to load required guide: {}", guide_filename))?
        };

        result = result.replace(&format!("{{{{{}}}}}", guide_name), &guide_content);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use std::path::PathBuf;

    // ========== Optional Placeholder Tests ==========

    #[test]
    fn test_optional_placeholder_with_value() {
        let context = ctx().add("name", "World").build();
        let result =
            render_template("Hello {{?name}}!", &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_optional_placeholder_without_value() {
        let result = render_template(
            "Hello {{?name}}!",
            &PathBuf::from("guides"),
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(result, "Hello !");
    }

    #[test]
    fn test_optional_placeholder_with_underscores() {
        let context = ctx().add("user_goals", "Focus on performance").build();
        let result =
            render_template("Goal: {{?user_goals}}", &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "Goal: Focus on performance");
    }

    #[test]
    fn test_multiple_optional_placeholders_mixed() {
        let context = ctx()
            .add("task", "Write tests")
            .add("context_b", "Use Rust")
            .build();
        let result = render_template(
            "Task: {{task}}\n\n{{?context_a}}\n\n{{?context_b}}\n\nBegin.",
            &PathBuf::from("guides"),
            &context,
        )
        .unwrap();
        assert!(result.contains("Write tests"));
        assert!(result.contains("Use Rust"));
        assert!(!result.contains("context_a"));
    }

    // ========== Required Placeholder Tests ==========

    #[test]
    fn test_required_placeholder_with_value() {
        let context = ctx().add("doc_type", "SPEC.md").build();
        let result =
            render_template("Write a {{doc_type}}.", &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "Write a SPEC.md.");
    }

    #[test]
    fn test_required_placeholder_missing() {
        let result = render_template("Hello {{name}}!", &PathBuf::from("guides"), &HashMap::new());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Required context variable missing") && err_msg.contains("name"));
    }

    #[test]
    fn test_multiple_required_placeholders() {
        let context = ctx()
            .add("doc_type", "SPEC.md")
            .add("system", "workflow engine")
            .build();
        let result = render_template(
            "Write a {{doc_type}} for {{system}}.",
            &PathBuf::from("guides"),
            &context,
        )
        .unwrap();
        assert_eq!(result, "Write a SPEC.md for workflow engine.");
    }

    #[test]
    fn test_same_placeholder_appears_multiple_times() {
        let context = ctx().add("name", "Alice").add("doc_type", "SPEC").build();
        let result = render_template(
            "{{name}} wrote a {{doc_type}}. The {{doc_type}} was good.",
            &PathBuf::from("guides"),
            &context,
        )
        .unwrap();
        assert_eq!(result, "Alice wrote a SPEC. The SPEC was good.");
        assert_eq!(result.matches("SPEC").count(), 2);
    }

    #[test]
    fn test_empty_string_value() {
        let context = ctx().add("header", "").build();
        let result = render_template(
            "Header: {{header}}\n\nBody",
            &PathBuf::from("guides"),
            &context,
        )
        .unwrap();
        assert_eq!(result, "Header: \n\nBody");
    }

    #[test]
    fn test_extra_context_keys_ignored() {
        let context = ctx()
            .add("doc_type", "SPEC.md")
            .add("extra_key", "ignored")
            .add("another_key", "also ignored")
            .build();
        let result =
            render_template("Write a {{doc_type}}.", &PathBuf::from("guides"), &context).unwrap();
        assert_eq!(result, "Write a SPEC.md.");
        assert!(!result.contains("extra_key") && !result.contains("ignored"));
    }

    // ========== Guide Loading Tests ==========

    #[test]
    fn test_guide_placeholder_loads_file() {
        let (_temp_dir, guides_path) = test_guides();
        let result = render_template(
            "Follow this guide:\n\n{{SPEC_WRITING}}",
            &guides_path,
            &HashMap::new(),
        )
        .unwrap();
        // Check mechanism: guide loaded and placeholder replaced (content can change)
        assert!(result.starts_with("Follow this guide:\n\n"));
        assert!(result.len() > 100); // Guide content was loaded (non-trivial length)
        assert!(!result.contains("{{SPEC_WRITING}}")); // Placeholder was replaced
    }

    #[test]
    fn test_guide_placeholder_missing_file() {
        let (_temp_dir, guides_path) = test_guides();
        let result = render_template("{{NONEXISTENT}}", &guides_path, &HashMap::new());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to load required guide") && err_msg.contains("NONEXISTENT.md")
        );
    }

    #[test]
    fn test_multiple_guide_placeholders() {
        let (_temp_dir, guides_path) = test_guides();
        let result = render_template(
            "Phase 1:\n{{SPEC_WRITING}}\n\n---\n\nPhase 2:\n{{PLAN_WRITING}}",
            &guides_path,
            &HashMap::new(),
        )
        .unwrap();
        // Check mechanism: both guides loaded, placeholders replaced
        assert!(result.starts_with("Phase 1:\n"));
        assert!(result.contains("\n---\n"));
        assert!(result.len() > 200); // Both guides loaded
        assert!(!result.contains("{{SPEC_WRITING}}"));
        assert!(!result.contains("{{PLAN_WRITING}}"));
    }

    // ========== Mixed Placeholder Tests ==========

    #[test]
    fn test_mixed_required_and_optional_and_guides() {
        let (_temp_dir, guides_path) = test_guides();
        let context = ctx()
            .add("doc_type", "SPEC.md")
            .add("system", "template engine")
            .build();
        let template = "Write a {{doc_type}} for {{system}}.\n\n{{SPEC_WRITING}}\n\n{{?user_context}}\n\nFollow TDD.";
        let result = render_template(template, &guides_path, &context).unwrap();
        // Check all mechanisms work together
        assert!(result.contains("Write a SPEC.md for template engine.")); // Required vars
        assert!(result.contains("Follow TDD.")); // Static text preserved
        assert!(result.len() > 100); // Guide loaded
        assert!(!result.contains("{{SPEC_WRITING}}")); // Guide placeholder replaced
        assert!(!result.contains("user_context")); // Optional var removed (not provided)
    }

    // ========== Empty Template Tests ==========

    #[test]
    fn test_empty_template() {
        let result = render_template(
            "This has no placeholders.",
            &PathBuf::from("guides"),
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(result, "This has no placeholders.");
    }

    #[test]
    fn test_template_with_only_text() {
        let result = render_template(
            "Just plain text, no variables at all.",
            &PathBuf::from("guides"),
            &HashMap::new(),
        )
        .unwrap();
        assert_eq!(result, "Just plain text, no variables at all.");
    }

    // ========== Security Tests ==========

    #[test]
    fn test_path_traversal_rejected() {
        let (_temp_dir, guides_path) = test_guides();
        for template in ["{{../../../etc/passwd}}", "{{..SPEC_WRITING}}"] {
            let result = render_template(template, &guides_path, &HashMap::new());
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Path traversal not allowed"));
        }
    }

    #[test]
    fn test_absolute_path_unix_rejected() {
        let (_temp_dir, guides_path) = test_guides();
        let result = render_template("{{/etc/passwd}}", &guides_path, &HashMap::new());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Absolute paths not allowed"));
    }

    #[test]
    fn test_subdirectory_rejected() {
        let (_temp_dir, guides_path) = test_guides();
        let result = render_template("{{subdir/GUIDE}}", &guides_path, &HashMap::new());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Subdirectories not allowed"));
    }

    #[test]
    fn test_validate_guide_name_errors() {
        assert!(validate_guide_name("")
            .unwrap_err()
            .to_string()
            .contains("cannot be empty"));
        assert!(validate_guide_name("../etc/passwd")
            .unwrap_err()
            .to_string()
            .contains("Path traversal not allowed"));
        assert!(validate_guide_name("/etc/passwd")
            .unwrap_err()
            .to_string()
            .contains("Absolute paths not allowed"));
        assert!(validate_guide_name("subdir/guide")
            .unwrap_err()
            .to_string()
            .contains("Subdirectories not allowed"));
    }

    #[test]
    fn test_validate_guide_name_valid() {
        assert!(validate_guide_name("SPEC_WRITING").is_ok());
        assert!(validate_guide_name("PLAN_WRITING").is_ok());
        assert!(validate_guide_name("templates/mirror_workflow").is_ok());
    }

    // ========== Template Include Tests ==========

    #[test]
    fn test_template_include_basic() {
        let (_temp_dir, guides_path) = test_guides();

        // Create a template file
        let templates_dir = guides_path.join("templates");
        std::fs::create_dir_all(&templates_dir).unwrap();
        std::fs::write(templates_dir.join("footer.md"), "This is a common footer.").unwrap();

        // Create a guide that uses the template
        std::fs::write(
            guides_path.join("TEST_GUIDE.md"),
            "# Test Guide\n\n{{templates/footer}}",
        )
        .unwrap();

        let result = render_template("{{TEST_GUIDE}}", &guides_path, &HashMap::new()).unwrap();

        assert!(result.contains("# Test Guide"));
        assert!(result.contains("This is a common footer."));
        assert!(!result.contains("{{templates/footer}}"));
    }

    #[test]
    fn test_template_include_with_context_variables() {
        let (_temp_dir, guides_path) = test_guides();

        // Create a template that uses context variables
        let templates_dir = guides_path.join("templates");
        std::fs::create_dir_all(&templates_dir).unwrap();
        std::fs::write(
            templates_dir.join("doc_header.md"),
            "# {{doc_type}} Document\n\nFile: {{doc_filename}}",
        )
        .unwrap();

        let context = ctx()
            .add("doc_type", "SPEC")
            .add("doc_filename", "SPEC.md")
            .build();

        let result = render_template(
            "{{templates/doc_header}}\n\nContent here.",
            &guides_path,
            &context,
        )
        .unwrap();

        assert!(result.contains("# SPEC Document"));
        assert!(result.contains("File: SPEC.md"));
        assert!(result.contains("Content here."));
    }

    #[test]
    fn test_template_include_nested() {
        let (_temp_dir, guides_path) = test_guides();

        // Create nested templates
        let templates_dir = guides_path.join("templates");
        std::fs::create_dir_all(&templates_dir).unwrap();

        std::fs::write(templates_dir.join("inner.md"), "Inner template content").unwrap();

        std::fs::write(
            templates_dir.join("outer.md"),
            "Outer starts\n{{templates/inner}}\nOuter ends",
        )
        .unwrap();

        let result = render_template("{{templates/outer}}", &guides_path, &HashMap::new()).unwrap();

        assert!(result.contains("Outer starts"));
        assert!(result.contains("Inner template content"));
        assert!(result.contains("Outer ends"));
        assert!(!result.contains("{{templates/"));
    }

    #[test]
    fn test_template_include_missing_file() {
        let (_temp_dir, guides_path) = test_guides();

        let templates_dir = guides_path.join("templates");
        std::fs::create_dir_all(&templates_dir).unwrap();

        let result = render_template("{{templates/nonexistent}}", &guides_path, &HashMap::new());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to load required guide"));
        assert!(err_msg.contains("templates/nonexistent.md"));
    }

    #[test]
    fn test_validate_templates_subdirectory_allowed() {
        assert!(validate_guide_name("templates/footer").is_ok());
        assert!(validate_guide_name("templates/mirror_workflow").is_ok());
    }

    #[test]
    fn test_validate_nested_templates_rejected() {
        let result = validate_guide_name("templates/nested/file");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Nested subdirectories not allowed"));
    }

    #[test]
    fn test_validate_other_subdirectories_still_rejected() {
        let result = validate_guide_name("other/guide");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Subdirectories not allowed in guide names (except templates/)"));
    }

    // ========== Nested Variable Substitution Tests ==========

    #[test]
    fn test_nested_variable_in_template_path() {
        let (_temp_dir, guides_path) = test_guides();

        // Create variant template files
        let templates_dir = guides_path.join("templates");
        std::fs::create_dir_all(&templates_dir).unwrap();
        std::fs::write(
            templates_dir.join("variant_a.md"),
            "This is variant A content",
        )
        .unwrap();
        std::fs::write(
            templates_dir.join("variant_b.md"),
            "This is variant B content",
        )
        .unwrap();

        // Test with variant_a
        let context = ctx().add("style", "variant_a").build();
        let result =
            render_template("Content: {{templates/{{style}}}}", &guides_path, &context).unwrap();
        assert!(result.contains("This is variant A content"));
        assert!(!result.contains("variant B"));

        // Test with variant_b
        let context = ctx().add("style", "variant_b").build();
        let result =
            render_template("Content: {{templates/{{style}}}}", &guides_path, &context).unwrap();
        assert!(result.contains("This is variant B content"));
        assert!(!result.contains("variant A"));
    }

    #[test]
    fn test_code_map_style_variant_integration() {
        let (_temp_dir, guides_path) = test_guides();

        // Create template variants (templates/ works because validation allows it)
        let templates_dir = guides_path.join("templates");
        std::fs::create_dir_all(&templates_dir).unwrap();
        std::fs::write(
            templates_dir.join("style_monolithic.md"),
            "## Monolithic Structure\nSingle file approach.",
        )
        .unwrap();
        std::fs::write(
            templates_dir.join("style_hierarchical.md"),
            "## Hierarchical Structure\nOne file per directory.",
        )
        .unwrap();

        // Create a guide with nested variable substitution
        std::fs::write(
            guides_path.join("CUSTOM_GUIDE.md"),
            "# Custom Guide\n\n{{templates/style_{{code_map_style}}}}\n\nEnd of guide.",
        )
        .unwrap();

        // Test hierarchical style
        let context = ctx().add("code_map_style", "hierarchical").build();
        let result = render_template("{{CUSTOM_GUIDE}}", &guides_path, &context).unwrap();
        assert!(result.contains("Hierarchical Structure"));
        assert!(result.contains("One file per directory"));
        assert!(!result.contains("Monolithic"));

        // Test monolithic style
        let context = ctx().add("code_map_style", "monolithic").build();
        let result = render_template("{{CUSTOM_GUIDE}}", &guides_path, &context).unwrap();
        assert!(result.contains("Monolithic Structure"));
        assert!(result.contains("Single file approach"));
        assert!(!result.contains("Hierarchical"));
    }

    #[test]
    fn test_multiple_nested_variables() {
        let (_temp_dir, guides_path) = test_guides();

        let templates_dir = guides_path.join("templates");
        std::fs::create_dir_all(&templates_dir).unwrap();
        std::fs::write(templates_dir.join("header_en.md"), "English Header").unwrap();
        std::fs::write(templates_dir.join("footer_compact.md"), "Compact Footer").unwrap();

        let context = ctx().add("lang", "en").add("layout", "compact").build();

        let result = render_template(
            "{{templates/header_{{lang}}}}\n\n{{templates/footer_{{layout}}}}",
            &guides_path,
            &context,
        )
        .unwrap();

        assert!(result.contains("English Header"));
        assert!(result.contains("Compact Footer"));
    }

    #[test]
    fn test_nested_variable_missing_context() {
        let (_temp_dir, guides_path) = test_guides();

        let templates_dir = guides_path.join("templates");
        std::fs::create_dir_all(&templates_dir).unwrap();

        // Missing 'style' in context should error
        let result = render_template(
            "{{templates/variant_{{style}}}}",
            &guides_path,
            &HashMap::new(),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Required context variable missing"));
    }
}
