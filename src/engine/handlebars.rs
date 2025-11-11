use anyhow::{anyhow, Result};
use handlebars::{
    Context, Handlebars, Helper, HelperDef, RenderContext, RenderError, RenderErrorReason,
    ScopedJson,
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
pub(crate) fn validate_partial_name(name: &str) -> Result<()> {
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
pub(crate) fn register_partials(registry: &mut Handlebars, guides_dir: &Path) -> Result<()> {
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
