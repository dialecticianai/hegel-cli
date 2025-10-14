use super::{AgentAdapter, CanonicalHookEvent, EventType};
use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;

/// Codex adapter - normalizes OpenAI Codex events to canonical format
///
/// Codex emits events in `~/.codex/sessions/*.jsonl` with two key types:
/// - `turn_context`: Metadata about the current turn (includes model)
/// - `event_msg` with `payload.type === "token_count"`: Token usage data
///
/// Key challenges:
/// 1. Token counts are cumulative - we must compute deltas
/// 2. Model metadata scattered across multiple nested locations
/// 3. Legacy events may lack model metadata entirely (fallback to "gpt-5")
pub struct CodexAdapter {
    /// State tracking for cumulative → delta conversion
    /// This is NOT thread-safe, but adapters process one session at a time
    state: std::cell::RefCell<SessionState>,
}

#[derive(Debug, Clone)]
struct SessionState {
    current_model: Option<String>,
    current_model_is_fallback: bool,
    previous_totals: Option<RawUsage>,
}

#[derive(Debug, Clone)]
struct RawUsage {
    input_tokens: u64,
    cached_input_tokens: u64,
    output_tokens: u64,
    reasoning_output_tokens: u64,
    total_tokens: u64,
}

impl CodexAdapter {
    pub fn new() -> Self {
        Self {
            state: std::cell::RefCell::new(SessionState {
                current_model: None,
                current_model_is_fallback: false,
                previous_totals: None,
            }),
        }
    }

    /// Extract model name from nested payload structures
    /// Checks 9 locations in priority order (per ccusage data-loader.ts:109-158)
    fn extract_model(payload: &serde_json::Value) -> Option<String> {
        // Priority 1: payload.info.model
        if let Some(model) = payload
            .get("info")
            .and_then(|i| i.get("model"))
            .and_then(|m| m.as_str())
            .and_then(|s| if s.trim().is_empty() { None } else { Some(s) })
        {
            return Some(model.to_string());
        }

        // Priority 2: payload.info.model_name
        if let Some(model) = payload
            .get("info")
            .and_then(|i| i.get("model_name"))
            .and_then(|m| m.as_str())
            .and_then(|s| if s.trim().is_empty() { None } else { Some(s) })
        {
            return Some(model.to_string());
        }

        // Priority 3: payload.info.metadata.model
        if let Some(model) = payload
            .get("info")
            .and_then(|i| i.get("metadata"))
            .and_then(|m| m.get("model"))
            .and_then(|m| m.as_str())
            .and_then(|s| if s.trim().is_empty() { None } else { Some(s) })
        {
            return Some(model.to_string());
        }

        // Priority 4: payload.model
        if let Some(model) = payload.get("model").and_then(|m| m.as_str()).and_then(|s| {
            if s.trim().is_empty() {
                None
            } else {
                Some(s)
            }
        }) {
            return Some(model.to_string());
        }

        // Priority 5: payload.metadata.model
        if let Some(model) = payload
            .get("metadata")
            .and_then(|m| m.get("model"))
            .and_then(|m| m.as_str())
            .and_then(|s| if s.trim().is_empty() { None } else { Some(s) })
        {
            return Some(model.to_string());
        }

        None
    }

    /// Normalize raw usage structure, handling field aliases
    fn normalize_raw_usage(value: Option<&serde_json::Value>) -> Option<RawUsage> {
        let obj = value?.as_object()?;

        let input_tokens = obj
            .get("input_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Handle alias: cached_input_tokens OR cache_read_input_tokens
        let cached_input_tokens = obj
            .get("cached_input_tokens")
            .or_else(|| obj.get("cache_read_input_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let output_tokens = obj
            .get("output_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let reasoning_output_tokens = obj
            .get("reasoning_output_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let total_tokens = obj
            .get("total_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        // Synthesize total if missing (legacy format)
        let total_tokens = if total_tokens > 0 {
            total_tokens
        } else {
            input_tokens + output_tokens
        };

        Some(RawUsage {
            input_tokens,
            cached_input_tokens,
            output_tokens,
            reasoning_output_tokens,
            total_tokens,
        })
    }

    /// Subtract previous totals from current to get delta
    fn subtract_raw_usage(current: &RawUsage, previous: Option<&RawUsage>) -> RawUsage {
        let prev = previous;
        RawUsage {
            input_tokens: current
                .input_tokens
                .saturating_sub(prev.map(|p| p.input_tokens).unwrap_or(0)),
            cached_input_tokens: current
                .cached_input_tokens
                .saturating_sub(prev.map(|p| p.cached_input_tokens).unwrap_or(0)),
            output_tokens: current
                .output_tokens
                .saturating_sub(prev.map(|p| p.output_tokens).unwrap_or(0)),
            reasoning_output_tokens: current
                .reasoning_output_tokens
                .saturating_sub(prev.map(|p| p.reasoning_output_tokens).unwrap_or(0)),
            total_tokens: current
                .total_tokens
                .saturating_sub(prev.map(|p| p.total_tokens).unwrap_or(0)),
        }
    }
}

impl AgentAdapter for CodexAdapter {
    fn name(&self) -> &str {
        "codex"
    }

    fn detect(&self) -> bool {
        // Check CODEX_HOME env var
        if std::env::var("CODEX_HOME").is_ok() {
            return true;
        }

        // Check default Codex directory
        if let Ok(home) = std::env::var("HOME") {
            let codex_dir = std::path::PathBuf::from(home).join(".codex/sessions");
            if codex_dir.exists() {
                return true;
            }
        }

        false
    }

    fn normalize(&self, input: serde_json::Value) -> Result<Option<CanonicalHookEvent>> {
        let event_type = input
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'type' field"))?;

        let timestamp = input
            .get("timestamp")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'timestamp' field"))?
            .to_string();

        let mut state = self.state.borrow_mut();

        match event_type {
            "turn_context" => {
                // Update model context, don't emit event
                if let Some(payload) = input.get("payload") {
                    if let Some(model) = Self::extract_model(payload) {
                        state.current_model = Some(model);
                        state.current_model_is_fallback = false;
                    }
                }
                Ok(None)
            }

            "event_msg" => {
                let payload = input
                    .get("payload")
                    .ok_or_else(|| anyhow::anyhow!("Missing 'payload' field"))?;

                // Only process token_count events
                if payload.get("type").and_then(|t| t.as_str()) != Some("token_count") {
                    return Ok(None);
                }

                let info = payload.get("info");

                // Extract token data
                let last_usage =
                    Self::normalize_raw_usage(info.and_then(|i| i.get("last_token_usage")));
                let total_usage =
                    Self::normalize_raw_usage(info.and_then(|i| i.get("total_token_usage")));

                // Compute delta
                let mut raw = last_usage;
                if raw.is_none() {
                    if let Some(ref total) = total_usage {
                        raw = Some(Self::subtract_raw_usage(
                            total,
                            state.previous_totals.as_ref(),
                        ));
                    }
                }

                // Update state for next iteration
                if let Some(ref total) = total_usage {
                    state.previous_totals = Some(total.clone());
                }

                let raw = match raw {
                    Some(r) => r,
                    None => return Ok(None),
                };

                // Skip zero-token events
                if raw.input_tokens == 0
                    && raw.cached_input_tokens == 0
                    && raw.output_tokens == 0
                    && raw.reasoning_output_tokens == 0
                {
                    return Ok(None);
                }

                // Determine model (with fallback)
                let extracted_model = Self::extract_model(payload);
                let (model, is_fallback) = if let Some(m) = extracted_model {
                    state.current_model = Some(m.clone());
                    state.current_model_is_fallback = false;
                    (m, false)
                } else if let Some(ref m) = state.current_model {
                    (m.clone(), state.current_model_is_fallback)
                } else {
                    // Legacy fallback
                    state.current_model = Some("gpt-5".to_string());
                    state.current_model_is_fallback = true;
                    ("gpt-5".to_string(), true)
                };

                // Build canonical event
                let mut extra = HashMap::new();
                extra.insert("model".to_string(), json!(model));

                Ok(Some(CanonicalHookEvent {
                    timestamp,
                    session_id: "codex-session".to_string(), // Will be set by caller from filename
                    event_type: EventType::PostToolUse,
                    tool_name: Some("Codex".to_string()),
                    tool_input: None,
                    tool_response: Some(json!({
                        "input_tokens": raw.input_tokens,
                        "cached_input_tokens": raw.cached_input_tokens,
                        "output_tokens": raw.output_tokens,
                        "reasoning_output_tokens": raw.reasoning_output_tokens,
                        "total_tokens": raw.total_tokens,
                        "model": model,
                    })),
                    cwd: None,
                    transcript_path: None, // Will be set by caller
                    adapter: Some("codex".to_string()),
                    fallback_used: if is_fallback { Some(true) } else { None },
                    extra,
                }))
            }

            _ => Ok(None), // Skip unknown event types
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_with_env_var() {
        std::env::set_var("CODEX_HOME", "/tmp/codex");
        let adapter = CodexAdapter::new();
        assert!(adapter.detect());
        std::env::remove_var("CODEX_HOME");
    }

    #[test]
    fn test_extract_model_priority() {
        // Priority 1: payload.info.model
        let payload = json!({
            "info": {
                "model": "gpt-5",
                "model_name": "other"
            },
            "model": "fallback"
        });
        assert_eq!(
            CodexAdapter::extract_model(&payload),
            Some("gpt-5".to_string())
        );

        // Priority 2: payload.info.model_name
        let payload = json!({
            "info": {
                "model_name": "gpt-5"
            },
            "model": "fallback"
        });
        assert_eq!(
            CodexAdapter::extract_model(&payload),
            Some("gpt-5".to_string())
        );

        // Priority 4: payload.model
        let payload = json!({
            "model": "gpt-5"
        });
        assert_eq!(
            CodexAdapter::extract_model(&payload),
            Some("gpt-5".to_string())
        );

        // No model found
        let payload = json!({});
        assert_eq!(CodexAdapter::extract_model(&payload), None);
    }

    #[test]
    fn test_normalize_turn_context() {
        use crate::test_helpers::load_fixture;

        let adapter = CodexAdapter::new();
        let input = load_fixture("adapters/codex_turn_context.json");

        let result = adapter.normalize(input).unwrap();
        assert!(result.is_none()); // turn_context doesn't emit event

        // Check state was updated
        let state = adapter.state.borrow();
        assert_eq!(state.current_model, Some("gpt-5".to_string()));
        assert!(!state.current_model_is_fallback);
    }

    #[test]
    fn test_normalize_token_count_with_model() {
        use crate::test_helpers::load_fixture;

        let adapter = CodexAdapter::new();
        let input = load_fixture("adapters/codex_token_count.json");

        let event = adapter.normalize(input).unwrap().unwrap();
        assert_eq!(event.tool_name, Some("Codex".to_string()));
        assert_eq!(event.adapter, Some("codex".to_string()));
        assert_eq!(event.event_type, EventType::PostToolUse);

        let response = event.tool_response.unwrap();
        assert_eq!(response.get("input_tokens").unwrap().as_u64(), Some(1200));
        assert_eq!(
            response.get("cached_input_tokens").unwrap().as_u64(),
            Some(200)
        );
        assert_eq!(response.get("output_tokens").unwrap().as_u64(), Some(500));
        assert_eq!(response.get("model").unwrap().as_str(), Some("gpt-5"));
    }

    #[test]
    fn test_cumulative_to_delta_conversion() {
        use crate::test_helpers::load_fixture;

        let adapter = CodexAdapter::new();

        // First event: cumulative totals
        let input1 = load_fixture("adapters/codex_cumulative_1.json");
        let event1 = adapter.normalize(input1).unwrap().unwrap();
        let response1 = event1.tool_response.unwrap();
        assert_eq!(response1.get("input_tokens").unwrap().as_u64(), Some(1200));
        assert_eq!(response1.get("output_tokens").unwrap().as_u64(), Some(500));

        // Second event: new cumulative totals (should compute delta)
        let input2 = load_fixture("adapters/codex_cumulative_2.json");
        let event2 = adapter.normalize(input2).unwrap().unwrap();
        let response2 = event2.tool_response.unwrap();

        // Deltas: 2000-1200=800, 300-200=100, 800-500=300
        assert_eq!(response2.get("input_tokens").unwrap().as_u64(), Some(800));
        assert_eq!(
            response2.get("cached_input_tokens").unwrap().as_u64(),
            Some(100)
        );
        assert_eq!(response2.get("output_tokens").unwrap().as_u64(), Some(300));
    }

    #[test]
    fn test_legacy_fallback_model() {
        use crate::test_helpers::load_fixture;

        let adapter = CodexAdapter::new();
        let input = load_fixture("adapters/codex_legacy_fallback.json");

        let event = adapter.normalize(input).unwrap().unwrap();
        assert_eq!(event.fallback_used, Some(true));
        assert_eq!(event.extra.get("model").unwrap().as_str(), Some("gpt-5"));
    }

    #[test]
    fn test_skip_zero_token_events() {
        let adapter = CodexAdapter::new();
        let input = json!({
            "timestamp": "2025-09-11T18:25:40Z",
            "type": "event_msg",
            "payload": {
                "type": "token_count",
                "info": {
                    "last_token_usage": {
                        "input_tokens": 0,
                        "output_tokens": 0,
                        "total_tokens": 0
                    },
                    "model": "gpt-5"
                }
            }
        });

        let result = adapter.normalize(input).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_field_alias_cache_read_input_tokens() {
        let adapter = CodexAdapter::new();
        let input = json!({
            "timestamp": "2025-09-11T18:25:40Z",
            "type": "event_msg",
            "payload": {
                "type": "token_count",
                "info": {
                    "last_token_usage": {
                        "input_tokens": 1000,
                        "cache_read_input_tokens": 300,  // Legacy field name
                        "output_tokens": 500,
                        "total_tokens": 1500
                    },
                    "model": "gpt-5"
                }
            }
        });

        let event = adapter.normalize(input).unwrap().unwrap();
        let response = event.tool_response.unwrap();
        assert_eq!(
            response.get("cached_input_tokens").unwrap().as_u64(),
            Some(300)
        );
    }
}
