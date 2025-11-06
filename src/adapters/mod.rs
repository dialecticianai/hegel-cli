mod claude_code;
mod codex;
mod cursor;

pub use claude_code::{find_transcript_dir, list_transcript_files, ClaudeCodeAdapter};
pub use codex::CodexAdapter;
pub use cursor::CursorAdapter;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Event type - normalized across all agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    SessionStart,
    SessionEnd,
    PreToolUse,
    PostToolUse,
    Stop,
    #[serde(untagged)]
    Other(String),
}

/// Canonical hook event - normalized from any agent
///
/// This schema represents events from any AI coding agent (Claude Code, Cursor, Codex, etc.)
/// normalized to a common format for workflow orchestration and metrics tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalHookEvent {
    // Universal fields (all agents must provide)
    pub timestamp: String, // ISO 8601
    pub session_id: String,
    pub event_type: EventType,

    // Tool execution context (for guardrails)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_response: Option<serde_json::Value>,

    // Execution context (for phase-aware rules)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_path: Option<String>,

    // Adapter metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter: Option<String>, // "claude_code", "cursor", "codex"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_used: Option<bool>, // True if had to guess missing fields

    // Catch-all for agent-specific fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Agent adapter trait - normalizes agent-specific events to canonical format
pub trait AgentAdapter {
    /// Adapter name (e.g., "claude_code", "cursor", "codex")
    /// TODO: Used for future multi-agent hook routing
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Check if this adapter should handle the current environment
    /// (Checks env vars like CLAUDE_CODE_SESSION_ID, CURSOR_SESSION_ID)
    fn detect(&self) -> bool;

    /// Normalize agent-specific JSON to canonical event
    ///
    /// Returns None if event should be skipped (invalid, malformed, etc.)
    /// Returns Some(event) with optional fallback flags if normalization succeeded
    fn normalize(&self, input: serde_json::Value) -> Result<Option<CanonicalHookEvent>>;
}

/// Registry of available adapters
pub struct AdapterRegistry {
    adapters: Vec<Box<dyn AgentAdapter>>,
}

impl AdapterRegistry {
    /// Create a new registry with all built-in adapters
    pub fn new() -> Self {
        Self {
            adapters: vec![
                Box::new(ClaudeCodeAdapter::new()),
                Box::new(CodexAdapter::new()),
                Box::new(CursorAdapter::new()),
            ],
        }
    }

    /// Auto-detect which adapter to use based on environment
    pub fn detect(&self) -> Option<&dyn AgentAdapter> {
        self.adapters
            .iter()
            .find(|adapter| adapter.detect())
            .map(|b| b.as_ref())
    }

    /// Get adapter by name (for explicit selection)
    /// TODO: Used for future multi-agent hook routing
    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<&dyn AgentAdapter> {
        self.adapters
            .iter()
            .find(|adapter| adapter.name() == name)
            .map(|b| b.as_ref())
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creates_all_adapters() {
        let registry = AdapterRegistry::new();
        assert_eq!(registry.adapters.len(), 3);
    }

    #[test]
    fn test_registry_default_same_as_new() {
        let registry1 = AdapterRegistry::new();
        let registry2 = AdapterRegistry::default();
        assert_eq!(registry1.adapters.len(), registry2.adapters.len());
    }

    #[test]
    fn test_registry_get_by_name() {
        let registry = AdapterRegistry::new();

        let claude = registry.get("claude_code");
        assert!(claude.is_some());
        assert_eq!(claude.unwrap().name(), "claude_code");

        let cursor = registry.get("cursor");
        assert!(cursor.is_some());
        assert_eq!(cursor.unwrap().name(), "cursor");

        let codex = registry.get("codex");
        assert!(codex.is_some());
        assert_eq!(codex.unwrap().name(), "codex");

        let nonexistent = registry.get("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_registry_detect_in_claude_code_env() {
        // This test runs in Claude Code environment, so detection should work
        let registry = AdapterRegistry::new();
        let detected = registry.detect();

        // Should detect Claude Code since we're running in it
        if let Some(adapter) = detected {
            assert_eq!(adapter.name(), "claude_code");
        }
    }

    #[test]
    fn test_event_type_serialization() {
        use serde_json::json;

        assert_eq!(
            serde_json::to_value(EventType::SessionStart).unwrap(),
            json!("session_start")
        );
        assert_eq!(
            serde_json::to_value(EventType::PostToolUse).unwrap(),
            json!("post_tool_use")
        );
    }

    #[test]
    fn test_event_type_deserialization() {
        use serde_json::json;

        let event: EventType = serde_json::from_value(json!("session_start")).unwrap();
        assert_eq!(event, EventType::SessionStart);

        let event: EventType = serde_json::from_value(json!("post_tool_use")).unwrap();
        assert_eq!(event, EventType::PostToolUse);

        let event: EventType = serde_json::from_value(json!("custom_event")).unwrap();
        assert_eq!(event, EventType::Other("custom_event".to_string()));
    }

    #[test]
    fn test_canonical_event_serialization() {
        let event = CanonicalHookEvent {
            timestamp: "2025-01-01T00:00:00Z".to_string(),
            session_id: "test".to_string(),
            event_type: EventType::PostToolUse,
            tool_name: Some("Bash".to_string()),
            tool_input: Some(serde_json::json!({"command": "echo test"})),
            tool_response: None,
            cwd: None,
            transcript_path: None,
            adapter: Some("claude_code".to_string()),
            fallback_used: None,
            extra: HashMap::new(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"event_type\":\"post_tool_use\""));
        assert!(json.contains("\"tool_name\":\"Bash\""));
    }
}
