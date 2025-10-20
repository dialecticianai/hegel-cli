//! Adapter test fixtures loader

/// Load JSON fixture from tests/fixtures/ directory
///
/// # Arguments
/// * `path` - Relative path from tests/fixtures/ (e.g., "adapters/codex_token_count.json")
///
/// # Returns
/// Parsed JSON value
///
/// # Panics
/// Panics if file doesn't exist or JSON is invalid
///
/// # Example
/// ```ignore
/// let event = load_fixture("adapters/codex_token_count.json");
/// let adapter = CodexAdapter::new();
/// let result = adapter.normalize(event).unwrap();
/// ```
pub fn load_fixture(path: &str) -> serde_json::Value {
    let project_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = project_root.join("tests/fixtures").join(path);

    let content = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", fixture_path.display(), e));

    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", fixture_path.display(), e))
}
