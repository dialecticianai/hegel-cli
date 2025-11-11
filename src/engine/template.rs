use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Validate guide name to prevent path traversal attacks
pub(crate) fn validate_guide_name(guide_name: &str) -> Result<()> {
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

        // Try filesystem first (allows user overrides), then fall back to embedded
        let guide_path = guides_dir.join(&guide_filename);
        let guide_content = if let Ok(content) = fs::read_to_string(&guide_path) {
            content
        } else if let Some(embedded) = crate::embedded::get_guide(&guide_filename) {
            embedded.to_string()
        } else {
            anyhow::bail!("Failed to load required guide: {}", guide_filename)
        };

        result = result.replace(&format!("{{{{{}}}}}", guide_name), &guide_content);
    }

    Ok(result)
}
