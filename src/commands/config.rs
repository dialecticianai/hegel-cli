use crate::config::HegelConfig;
use crate::storage::FileStorage;
use anyhow::Result;

/// Handle config command - get, set, or list configuration values
pub fn handle_config(
    action: Option<&str>,
    key: Option<&str>,
    value: Option<&str>,
    storage: &FileStorage,
) -> Result<()> {
    let state_dir = storage.state_dir();

    match action {
        Some("get") => {
            let key = key.ok_or_else(|| anyhow::anyhow!("Missing key for 'get' command"))?;
            let config = HegelConfig::load(state_dir)?;

            match config.get(key) {
                Some(val) => println!("{}", val),
                None => anyhow::bail!("Unknown config key: {}", key),
            }
        }
        Some("set") => {
            let key = key.ok_or_else(|| anyhow::anyhow!("Missing key for 'set' command"))?;
            let value = value.ok_or_else(|| anyhow::anyhow!("Missing value for 'set' command"))?;

            let mut config = HegelConfig::load(state_dir)?;
            let old_value = config.get(key).unwrap_or_else(|| "unset".to_string());
            config.set(key, value)?;
            config.save(state_dir)?;

            println!("{} is now {}", key, value);
            if old_value != value {
                println!("  (was: {})", old_value);
            }
        }
        Some("list") | None => {
            let config = HegelConfig::load(state_dir)?;

            println!("Hegel Configuration:");
            println!();
            for (key, value) in config.list() {
                println!("  {} = {}", key, value);
            }
        }
        Some(unknown) => {
            anyhow::bail!(
                "Unknown config action: {}. Use 'get', 'set', or 'list'",
                unknown
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test_storage;

    #[test]
    fn test_config_list() {
        let (temp_dir, storage) = test_storage();

        // Should work even without config file (uses defaults)
        let result = handle_config(Some("list"), None, None, &storage);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_get_default() {
        let (temp_dir, storage) = test_storage();

        let result = handle_config(Some("get"), Some("code_map_style"), None, &storage);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_set_and_get() {
        let (temp_dir, storage) = test_storage();

        // Set value
        handle_config(
            Some("set"),
            Some("code_map_style"),
            Some("monolithic"),
            &storage,
        )
        .unwrap();

        // Verify it was saved
        let config = HegelConfig::load(storage.state_dir()).unwrap();
        assert_eq!(config.code_map_style, "monolithic");
    }

    #[test]
    fn test_config_set_invalid_code_map_style() {
        let (temp_dir, storage) = test_storage();

        let result = handle_config(
            Some("set"),
            Some("code_map_style"),
            Some("invalid"),
            &storage,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_config_get_unknown_key() {
        let (temp_dir, storage) = test_storage();

        let result = handle_config(Some("get"), Some("nonexistent"), None, &storage);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_set_boolean() {
        let (temp_dir, storage) = test_storage();

        handle_config(
            Some("set"),
            Some("use_reflect_gui"),
            Some("false"),
            &storage,
        )
        .unwrap();

        let config = HegelConfig::load(storage.state_dir()).unwrap();
        assert!(!config.use_reflect_gui);
    }
}
