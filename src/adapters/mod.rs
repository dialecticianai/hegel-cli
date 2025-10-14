mod claude_code;
mod codex;
mod cursor;

pub use claude_code::ClaudeCodeAdapter;
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
