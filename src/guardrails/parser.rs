use super::types::GuardRailsConfig;
use anyhow::{Context, Result};
use std::path::Path;

/// Load guardrails configuration from file
pub fn load_guardrails(state_dir: &Path) -> Result<GuardRailsConfig> {
    let guardrails_path = state_dir.join("guardrails.yaml");

    // If file doesn't exist, return empty config (all commands allowed)
    if !guardrails_path.exists() {
        return Ok(GuardRailsConfig::empty());
    }

    // Read and parse YAML
    let contents = std::fs::read_to_string(&guardrails_path)
        .with_context(|| format!("Failed to read guardrails file: {:?}", guardrails_path))?;

    let config: GuardRailsConfig = serde_yaml::from_str(&contents)
        .with_context(|| format!("Failed to parse guardrails YAML: {:?}", guardrails_path))?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_guardrails_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let config = load_guardrails(temp_dir.path()).unwrap();
        assert!(config.git.is_none());
        assert!(config.docker.is_none());
    }

    #[test]
    fn test_load_guardrails_valid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let guardrails_path = temp_dir.path().join("guardrails.yaml");

        let yaml = r#"
git:
  blocked:
    - pattern: "clean -fd"
      reason: "Destructive operation"
  allowed:
    - "status"
    - "log"
"#;
        std::fs::write(&guardrails_path, yaml).unwrap();

        let config = load_guardrails(temp_dir.path()).unwrap();
        assert!(config.git.is_some());

        let git_rules = config.git.as_ref().unwrap();
        assert_eq!(git_rules.blocked.len(), 1);
        assert_eq!(git_rules.allowed.len(), 2);
    }

    #[test]
    fn test_load_guardrails_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let guardrails_path = temp_dir.path().join("guardrails.yaml");

        std::fs::write(&guardrails_path, "invalid: [yaml: syntax").unwrap();

        let result = load_guardrails(temp_dir.path());
        assert!(result.is_err());
    }
}
