// User configuration for hegel behavior
// Loaded from .hegel/config.toml (or defaults)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// User configuration for hegel behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HegelConfig {
    /// Whether to automatically open GUI for document review after writing docs
    pub use_reflect_gui: bool,
}

impl Default for HegelConfig {
    fn default() -> Self {
        Self {
            use_reflect_gui: true, // Default to on
        }
    }
}

impl HegelConfig {
    /// Load config from .hegel/config.toml, or return defaults
    pub fn load(state_dir: &Path) -> Result<Self> {
        let config_path = state_dir.join("config.toml");

        if !config_path.exists() {
            // No config file - use defaults
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

        let config: HegelConfig =
            toml::from_str(&content).with_context(|| "Failed to parse config.toml")?;

        Ok(config)
    }

    /// Save config to .hegel/config.toml
    pub fn save(&self, state_dir: &Path) -> Result<()> {
        let config_path = state_dir.join("config.toml");

        let content = toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = HegelConfig::default();
        assert!(config.use_reflect_gui);
    }

    #[test]
    fn test_load_missing_config_returns_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let config = HegelConfig::load(temp_dir.path()).unwrap();
        assert!(config.use_reflect_gui); // Default
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();

        let original = HegelConfig {
            use_reflect_gui: false,
        };

        original.save(temp_dir.path()).unwrap();
        let loaded = HegelConfig::load(temp_dir.path()).unwrap();

        assert!(!loaded.use_reflect_gui);
    }

    #[test]
    fn test_load_invalid_toml_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        fs::write(&config_path, "invalid toml {{{").unwrap();

        let result = HegelConfig::load(temp_dir.path());
        assert!(result.is_err());
    }
}
