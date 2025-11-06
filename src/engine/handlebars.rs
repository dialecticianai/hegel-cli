use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

/// Render Handlebars template with guide/partial loading
pub fn render_template_hbs(
    template: &str,
    guides_dir: &Path,
    context: &HashMap<String, String>,
) -> Result<String> {
    // TODO: Implement in Step 3
    let _ = (template, guides_dir, context);
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

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

    // ========== Step 1: Original Placeholder Test ==========

    #[test]
    fn test_basic_rendering_placeholder() {
        // Placeholder test for basic Handlebars rendering
        let temp_dir = TempDir::new().unwrap();
        let context = HashMap::new();
        let result = render_template_hbs("Hello {{name}}", temp_dir.path(), &context);
        assert!(result.is_ok());
    }
}
