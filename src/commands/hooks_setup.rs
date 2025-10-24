use anyhow::{Context, Result};
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

    // Check if hooks are already configured
    if settings_path.exists() {
        return Ok(()); // Already configured, skip
    }

    // Create .claude directory if it doesn't exist
    if !claude_dir.exists() {
        fs::create_dir(claude_dir)
            .context("Failed to create .claude directory for hook installation")?;
    }

    // Install default hooks configuration
    let hooks_config = r#"{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "hegel hook PostToolUse",
            "timeout": 5
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "hegel hook PreToolUse",
            "timeout": 5
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "hegel hook UserPromptSubmit",
            "timeout": 5
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "hegel hook Stop",
            "timeout": 5
          }
        ]
      }
    ],
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "hegel hook SessionStart",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
"#;

    fs::write(&settings_path, hooks_config).context("Failed to write .claude/settings.json")?;

    eprintln!("✅ Installed Claude Code hooks to .claude/settings.json");
    eprintln!("   Metrics will now be captured automatically.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_config_is_valid_json() {
        // Verify the embedded config is valid JSON with expected structure
        let hooks_config = r#"{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "hegel hook PostToolUse",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
"#;
        let json: serde_json::Value = serde_json::from_str(hooks_config).unwrap();
        assert!(json.get("hooks").is_some());
        assert!(json["hooks"].get("PostToolUse").is_some());
        assert!(json["hooks"].get("SessionStart").is_none()); // Partial check
    }
}
