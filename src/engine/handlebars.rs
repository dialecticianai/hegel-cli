use anyhow::{anyhow, Result};
use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
    RenderErrorReason, ScopedJson,
};
use serde::{Deserialize, Serialize};
use serde_json::value::Value as Json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Context structure for Handlebars templates
/// Wraps user context for extensibility (future: config, metadata, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlebarsContext {
    pub context: HashMap<String, String>,
    // Future fields: config, metadata, etc.
}

/// Validate partial name to prevent path traversal attacks
fn validate_partial_name(name: &str) -> Result<()> {
    // Reject empty names
    if name.is_empty() {
        return Err(anyhow!("Partial name cannot be empty"));
    }

    // Reject path traversal attempts
    if name.contains("..") {
        return Err(anyhow!("Path traversal not allowed in partial names"));
    }

    // Reject absolute paths
    if name.starts_with('/') || name.starts_with('\\') {
        return Err(anyhow!("Absolute paths not allowed in partial names"));
    }

    // Allow partials/ subdirectory only
    if name.contains('/') || name.contains('\\') {
        if !name.starts_with("partials/") {
            return Err(anyhow!(
                "Subdirectories not allowed in partial names (except partials/)"
            ));
        }
        // Validate that partials/ is followed by a simple name (no further slashes)
        let after_prefix = name.strip_prefix("partials/").unwrap();
        if after_prefix.contains('/') || after_prefix.contains('\\') {
            return Err(anyhow!("Nested subdirectories not allowed in partials/"));
        }
    }

    Ok(())
}

/// Register partials from guides directories
/// Precedence: guides/partials/*.hbs > guides/*.hbs
fn register_partials(registry: &mut Handlebars, guides_dir: &Path) -> Result<()> {
    use std::collections::HashSet;

    let mut registered: HashSet<String> = HashSet::new();

    // First, register from guides/partials/*.hbs (higher precedence)
    let partials_dir = guides_dir.join("partials");
    if partials_dir.exists() && partials_dir.is_dir() {
        for entry in fs::read_dir(&partials_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process .hbs files
            if path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Validate the partial name
                    validate_partial_name(stem)?;

                    // Read and register the partial
                    let content = fs::read_to_string(&path)?;
                    registry.register_template_string(stem, content)?;
                    registered.insert(stem.to_string());
                }
            }
        }
    }

    // Then, register from guides/*.hbs (fallback, lower precedence)
    if guides_dir.exists() && guides_dir.is_dir() {
        for entry in fs::read_dir(guides_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process .hbs files
            if path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Skip if already registered from partials/
                    if registered.contains(stem) {
                        continue;
                    }

                    // Validate the partial name
                    validate_partial_name(stem)?;

                    // Read and register the partial
                    let content = fs::read_to_string(&path)?;
                    registry.register_template_string(stem, content)?;
                }
            }
        }
    }

    Ok(())
}

/// Custom eq helper for string equality comparison
/// Used in conditionals like {{#if (eq x "value")}}
#[derive(Clone, Copy)]
struct EqHelper;

impl HelperDef for EqHelper {
    fn call_inner<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        _: &'reg Handlebars<'reg>,
        _: &'rc Context,
        _: &mut RenderContext<'reg, 'rc>,
    ) -> Result<ScopedJson<'rc>, RenderError> {
        // Get two parameters to compare
        let param0 = h
            .param(0)
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("eq", 0))?;
        let param1 = h
            .param(1)
            .ok_or(RenderErrorReason::ParamNotFoundForIndex("eq", 1))?;

        // Compare values as JSON
        let equal = param0.value() == param1.value();

        Ok(ScopedJson::Derived(Json::Bool(equal)))
    }
}

/// Render Handlebars template with guide/partial loading
pub fn render_template_hbs(
    template: &str,
    guides_dir: &Path,
    hbs_context: &HandlebarsContext,
) -> Result<String> {
    // Initialize Handlebars registry
    let mut hbs = Handlebars::new();

    // Register partials from guides directories
    register_partials(&mut hbs, guides_dir)?;

    // Register custom eq helper
    hbs.register_helper("eq", Box::new(EqHelper));

    // Render the template
    // HandlebarsContext serializes to {context: {...}, future: config, etc.}
    let result = hbs.render_template(template, hbs_context)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    // Helper to create HandlebarsContext from HashMap
    fn hbs_ctx(context: &HashMap<String, String>) -> HandlebarsContext {
        HandlebarsContext {
            context: context.clone(),
        }
    }

    // ========== Step 2: Partial Loading Tests ==========

    #[test]
    fn test_register_partials_from_partials_dir() {
        let temp_dir = TempDir::new().unwrap();
        let guides_dir = temp_dir.path();

        // Create guides/partials directory with a test partial
        let partials_dir = guides_dir.join("partials");
        fs::create_dir_all(&partials_dir).unwrap();
        fs::write(partials_dir.join("test.hbs"), "partial content").unwrap();

        let mut registry = Handlebars::new();
        register_partials(&mut registry, guides_dir).unwrap();

        // Verify partial was registered
        assert!(registry.has_template("test"));
    }

    #[test]
    fn test_register_partials_from_guides_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let guides_dir = temp_dir.path();

        // Create guides/*.hbs file (fallback location)
        fs::write(guides_dir.join("fallback.hbs"), "fallback content").unwrap();

        let mut registry = Handlebars::new();
        register_partials(&mut registry, guides_dir).unwrap();

        // Verify partial was registered from fallback
        assert!(registry.has_template("fallback"));
    }

    #[test]
    fn test_register_partials_precedence() {
        let temp_dir = TempDir::new().unwrap();
        let guides_dir = temp_dir.path();

        // Create same-named file in both locations
        let partials_dir = guides_dir.join("partials");
        fs::create_dir_all(&partials_dir).unwrap();
        fs::write(partials_dir.join("priority.hbs"), "from partials/").unwrap();
        fs::write(guides_dir.join("priority.hbs"), "from guides/").unwrap();

        let mut registry = Handlebars::new();
        register_partials(&mut registry, guides_dir).unwrap();

        // Verify partials/ version takes precedence
        let result = registry
            .render("priority", &HashMap::<String, String>::new())
            .unwrap();
        assert_eq!(result, "from partials/");
    }

    #[test]
    fn test_validate_partial_name_valid() {
        assert!(validate_partial_name("simple").is_ok());
        assert!(validate_partial_name("with_underscore").is_ok());
        assert!(validate_partial_name("partials/nested").is_ok());
    }

    #[test]
    fn test_validate_partial_name_rejects_path_traversal() {
        let result = validate_partial_name("../etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Path traversal"));
    }

    #[test]
    fn test_validate_partial_name_rejects_absolute_path() {
        let result = validate_partial_name("/etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Absolute paths"));
    }

    #[test]
    fn test_validate_partial_name_rejects_other_subdirs() {
        let result = validate_partial_name("other/file");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Subdirectories not allowed"));
    }

    #[test]
    fn test_validate_partial_name_rejects_nested_subdirs() {
        let result = validate_partial_name("partials/nested/file");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Nested subdirectories"));
    }

    // ========== Step 3: Core Rendering Tests ==========

    #[test]
    fn test_render_simple_variable_substitution() {
        let temp_dir = TempDir::new().unwrap();
        let guides_dir = temp_dir.path();

        let mut context = HashMap::new();
        context.insert("name".to_string(), "World".to_string());

        let result =
            render_template_hbs("Hello {{context.name}}!", guides_dir, &hbs_ctx(&context)).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_render_with_partial_inclusion() {
        let temp_dir = TempDir::new().unwrap();
        let guides_dir = temp_dir.path();

        // Create a partial
        let partials_dir = guides_dir.join("partials");
        fs::create_dir_all(&partials_dir).unwrap();
        fs::write(partials_dir.join("greeting.hbs"), "Hello {{context.name}}!").unwrap();

        let mut context = HashMap::new();
        context.insert("name".to_string(), "World".to_string());

        let result = render_template_hbs("{{> greeting}}", guides_dir, &hbs_ctx(&context)).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_render_with_conditional() {
        let temp_dir = TempDir::new().unwrap();
        let guides_dir = temp_dir.path();

        let template = "{{#if context.show}}Visible{{else}}Hidden{{/if}}";

        // Test true case
        let mut context = HashMap::new();
        context.insert("show".to_string(), "true".to_string());
        let result = render_template_hbs(template, guides_dir, &hbs_ctx(&context)).unwrap();
        assert_eq!(result, "Visible");

        // Test false case (empty string is falsy)
        context.insert("show".to_string(), "".to_string());
        let result = render_template_hbs(template, guides_dir, &hbs_ctx(&context)).unwrap();
        assert_eq!(result, "Hidden");
    }

    #[test]
    fn test_render_with_eq_helper() {
        let temp_dir = TempDir::new().unwrap();
        let guides_dir = temp_dir.path();

        let template =
            "{{#if (eq context.style \"hierarchical\")}}Hierarchical{{else}}Monolithic{{/if}}";

        // Test eq returns true
        let mut context = HashMap::new();
        context.insert("style".to_string(), "hierarchical".to_string());
        let result = render_template_hbs(template, guides_dir, &hbs_ctx(&context)).unwrap();
        assert_eq!(result, "Hierarchical");

        // Test eq returns false
        context.insert("style".to_string(), "monolithic".to_string());
        let result = render_template_hbs(template, guides_dir, &hbs_ctx(&context)).unwrap();
        assert_eq!(result, "Monolithic");
    }

    #[test]
    fn test_render_error_missing_partial() {
        let temp_dir = TempDir::new().unwrap();
        let guides_dir = temp_dir.path();

        let result =
            render_template_hbs("{{> nonexistent}}", guides_dir, &hbs_ctx(&HashMap::new()));
        assert!(result.is_err());
    }

    #[test]
    fn test_render_error_invalid_syntax() {
        let temp_dir = TempDir::new().unwrap();
        let guides_dir = temp_dir.path();

        let result = render_template_hbs("{{#if unclosed", guides_dir, &hbs_ctx(&HashMap::new()));
        assert!(result.is_err());
    }

    // ========== Step 4: Dual-Engine Routing Tests ==========

    #[test]
    fn test_routing_to_handlebars_engine() {
        let temp_dir = TempDir::new().unwrap();
        let guides_dir = temp_dir.path();

        // Create a partial for Handlebars
        let partials_dir = guides_dir.join("partials");
        fs::create_dir_all(&partials_dir).unwrap();
        fs::write(partials_dir.join("test.hbs"), "HBS: {{context.value}}").unwrap();

        let mut context = HashMap::new();
        context.insert("value".to_string(), "works".to_string());

        // Route to Handlebars engine
        let result = crate::engine::render_prompt(
            "{{> test}}",
            true, // is_handlebars = true
            guides_dir,
            &context,
        )
        .unwrap();

        assert_eq!(result, "HBS: works");
    }

    #[test]
    fn test_routing_to_markdown_engine() {
        use crate::test_helpers::test_guides;

        let (_temp_dir, guides_dir) = test_guides();
        let context = HashMap::new();

        // Route to Markdown engine (existing template system)
        let result = crate::engine::render_prompt(
            "{{SPEC_WRITING}}",
            false, // is_handlebars = false
            &guides_dir,
            &context,
        )
        .unwrap();

        // Verify it's using the old engine (loads guides/SPEC_WRITING.md)
        assert!(result.contains("# SPEC Writing Guide"));
    }

    #[test]
    fn test_both_engines_coexist() {
        use crate::test_helpers::test_guides;

        let (_temp_dir, guides_dir) = test_guides();

        // Old engine works
        let result1 =
            crate::engine::render_prompt("{{SPEC_WRITING}}", false, &guides_dir, &HashMap::new())
                .unwrap();
        assert!(result1.contains("# SPEC Writing Guide"));

        // New engine works in same test run
        let temp_dir2 = TempDir::new().unwrap();
        let guides_dir2 = temp_dir2.path();
        let partials_dir = guides_dir2.join("partials");
        fs::create_dir_all(&partials_dir).unwrap();
        fs::write(partials_dir.join("test.hbs"), "New engine").unwrap();

        let result2 =
            crate::engine::render_prompt("{{> test}}", true, guides_dir2, &HashMap::new()).unwrap();
        assert_eq!(result2, "New engine");
    }

    // ========== Step 6: Code Map Proof-of-Concept Tests ==========

    #[test]
    fn test_code_map_hierarchical_mode() {
        // Use actual guides directory
        let guides_dir = std::path::Path::new("guides");

        let mut context = HashMap::new();
        context.insert("code_map_style".to_string(), "hierarchical".to_string());

        let result = render_template_hbs("{{> code_map}}", guides_dir, &hbs_ctx(&context)).unwrap();

        // Verify hierarchical content is present
        assert!(result.contains("Larger Projects"));
        assert!(result.contains("Hierarchical Principles"));
        assert!(result.contains("Non-recursive"));

        // Verify monolithic content is NOT present
        assert!(!result.contains("Small Projects"));
        assert!(!result.contains("When to Switch to Hierarchical"));
    }

    #[test]
    fn test_code_map_monolithic_mode() {
        // Use actual guides directory
        let guides_dir = std::path::Path::new("guides");

        let mut context = HashMap::new();
        context.insert("code_map_style".to_string(), "monolithic".to_string());

        let result = render_template_hbs("{{> code_map}}", guides_dir, &hbs_ctx(&context)).unwrap();

        // Verify monolithic content is present
        assert!(result.contains("Small Projects"));
        assert!(result.contains("When to Use Monolithic Mode"));
        assert!(result.contains("When to Switch to Hierarchical"));

        // Verify hierarchical content is NOT present
        assert!(!result.contains("Larger Projects"));
        assert!(!result.contains("Hierarchical Principles"));
    }

    #[test]
    fn test_code_map_partial_precedence() {
        // Test that guides/partials/code_map.hbs takes precedence over
        // guides/templates/code_map_*.md files

        let guides_dir = std::path::Path::new("guides");
        let mut context = HashMap::new();
        context.insert("code_map_style".to_string(), "hierarchical".to_string());

        let result = render_template_hbs("{{> code_map}}", guides_dir, &hbs_ctx(&context)).unwrap();

        // The .hbs file uses conditional logic (if/else), while the .md files are separate
        // If we get content that varies based on the context variable, we're using .hbs
        assert!(result.contains("Larger Projects"));

        // Change context and verify different output (proves conditional logic works)
        context.insert("code_map_style".to_string(), "monolithic".to_string());
        let result2 =
            render_template_hbs("{{> code_map}}", guides_dir, &hbs_ctx(&context)).unwrap();

        assert!(result2.contains("Small Projects"));
        assert!(result != result2); // Different content for different context
    }

    // ========== Step 1: Original Placeholder Test ==========

    #[test]
    fn test_basic_rendering_placeholder() {
        // Placeholder test for basic Handlebars rendering
        let temp_dir = TempDir::new().unwrap();
        let context = HashMap::new();
        let result = render_template_hbs(
            "Hello {{context.name}}",
            temp_dir.path(),
            &hbs_ctx(&context),
        );
        assert!(result.is_ok());
    }
}
