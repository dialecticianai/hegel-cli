use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

/// Auto-install Claude Code hooks if running in Claude Code and hooks not configured
///
/// This runs automatically on any hegel command to ensure metrics are captured.
/// Only installs if:
/// - CLAUDECODE=1 environment variable is set (running in Claude Code)
/// - .claude/settings.json doesn't exist yet
pub fn auto_install_hooks() -> Result<()> {
    // Check if we're running in Claude Code
    if std::env::var("CLAUDECODE").unwrap_or_default() != "1" {
        return Ok(()); // Not in Claude Code, skip
    }

    let claude_dir = Path::new(".claude");
    let settings_path = claude_dir.join("settings.json");

    // Load existing settings or create new
    let mut settings: Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)
            .context("Failed to read existing .claude/settings.json")?;
        serde_json::from_str(&content).context("Failed to parse existing .claude/settings.json")?
    } else {
        // Create .claude directory if it doesn't exist
        if !claude_dir.exists() {
            fs::create_dir(claude_dir)
                .context("Failed to create .claude directory for hook installation")?;
        }
        json!({})
    };

    // Check if hegel hooks are already installed
    if is_hegel_hooks_installed(&settings) {
        return Ok(()); // Hegel hooks already present, skip
    }

    // Load hegel hooks configuration (embedded at compile time)
    const HEGEL_HOOKS_JSON: &str = include_str!("../../adapters/claude_code/hooks.json");
    let hegel_hooks: Value =
        serde_json::from_str(HEGEL_HOOKS_JSON).context("Failed to parse embedded hooks.json")?;

    // Merge hegel hooks with existing hooks
    merge_hooks(&mut settings, &hegel_hooks);

    // Write merged configuration
    let formatted =
        serde_json::to_string_pretty(&settings).context("Failed to serialize settings.json")?;
    fs::write(&settings_path, formatted).context("Failed to write .claude/settings.json")?;

    eprintln!("✅ Installed Hegel hooks to .claude/settings.json");
    eprintln!("   Metrics will now be captured automatically.");

    Ok(())
}

/// Check if hegel hooks are already installed in settings
fn is_hegel_hooks_installed(settings: &Value) -> bool {
    if let Some(hooks) = settings.get("hooks").and_then(|h| h.as_object()) {
        // Check if any hook event has a hegel hook command
        for (_event_name, event_hooks) in hooks {
            if let Some(arr) = event_hooks.as_array() {
                for hook_group in arr {
                    if let Some(hooks_list) = hook_group.get("hooks").and_then(|h| h.as_array()) {
                        for hook in hooks_list {
                            if let Some(cmd) = hook.get("command").and_then(|c| c.as_str()) {
                                if cmd.starts_with("hegel hook ") {
                                    return true; // Found at least one hegel hook
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Merge hegel hooks into existing settings
fn merge_hooks(settings: &mut Value, hegel_hooks: &Value) {
    // Ensure settings has a "hooks" object
    if !settings.is_object() {
        *settings = json!({});
    }
    let settings_obj = settings.as_object_mut().unwrap();

    if !settings_obj.contains_key("hooks") {
        settings_obj.insert("hooks".to_string(), json!({}));
    }

    let hooks_obj = settings_obj
        .get_mut("hooks")
        .unwrap()
        .as_object_mut()
        .unwrap();
    let hegel_hooks_obj = hegel_hooks.as_object().unwrap();

    // For each hegel hook event, append to existing or create new
    for (event_name, hegel_event_hooks) in hegel_hooks_obj {
        if !hooks_obj.contains_key(event_name) {
            // Event doesn't exist, add it
            hooks_obj.insert(event_name.clone(), hegel_event_hooks.clone());
        } else {
            // Event exists, append hegel hooks to the array
            let existing_array = hooks_obj
                .get_mut(event_name)
                .unwrap()
                .as_array_mut()
                .unwrap();
            let hegel_array = hegel_event_hooks.as_array().unwrap();
            existing_array.extend_from_slice(hegel_array);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hegel_hooks_installed_empty() {
        let settings = json!({});
        assert!(!is_hegel_hooks_installed(&settings));
    }

    #[test]
    fn test_is_hegel_hooks_installed_found() {
        let settings = json!({
            "hooks": {
                "PostToolUse": [{
                    "matcher": "*",
                    "hooks": [{
                        "type": "command",
                        "command": "hegel hook PostToolUse"
                    }]
                }]
            }
        });
        assert!(is_hegel_hooks_installed(&settings));
    }

    #[test]
    fn test_is_hegel_hooks_installed_other_hooks() {
        let settings = json!({
            "hooks": {
                "PostToolUse": [{
                    "matcher": "*",
                    "hooks": [{
                        "type": "command",
                        "command": "some-other-hook"
                    }]
                }]
            }
        });
        assert!(!is_hegel_hooks_installed(&settings));
    }

    #[test]
    fn test_merge_hooks_empty_settings() {
        let mut settings = json!({});
        let hegel_hooks = json!({
            "PostToolUse": [{
                "matcher": "*",
                "hooks": [{"type": "command", "command": "hegel hook PostToolUse"}]
            }]
        });

        merge_hooks(&mut settings, &hegel_hooks);

        assert!(settings["hooks"]["PostToolUse"].is_array());
        assert_eq!(
            settings["hooks"]["PostToolUse"].as_array().unwrap().len(),
            1
        );
    }

    #[test]
    fn test_merge_hooks_with_existing() {
        let mut settings = json!({
            "hooks": {
                "PostToolUse": [{
                    "matcher": "Bash",
                    "hooks": [{"type": "command", "command": "echo 'existing'"}]
                }]
            }
        });

        let hegel_hooks = json!({
            "PostToolUse": [{
                "matcher": "*",
                "hooks": [{"type": "command", "command": "hegel hook PostToolUse"}]
            }]
        });

        merge_hooks(&mut settings, &hegel_hooks);

        let post_tool_use = settings["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(post_tool_use.len(), 2); // Original + hegel hook
        assert_eq!(post_tool_use[0]["matcher"], "Bash");
        assert_eq!(post_tool_use[1]["matcher"], "*");
    }

    #[test]
    fn test_merge_hooks_new_event_type() {
        let mut settings = json!({
            "hooks": {
                "PostToolUse": [{
                    "hooks": [{"type": "command", "command": "existing"}]
                }]
            }
        });

        let hegel_hooks = json!({
            "SessionStart": [{
                "hooks": [{"type": "command", "command": "hegel hook SessionStart"}]
            }]
        });

        merge_hooks(&mut settings, &hegel_hooks);

        assert!(settings["hooks"]["PostToolUse"].is_array());
        assert!(settings["hooks"]["SessionStart"].is_array());
        assert_eq!(
            settings["hooks"]["PostToolUse"].as_array().unwrap().len(),
            1
        );
        assert_eq!(
            settings["hooks"]["SessionStart"].as_array().unwrap().len(),
            1
        );
    }
}
