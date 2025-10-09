mod hooks;
mod states;
mod transcript;

// Re-export public types from submodules
pub use hooks::{parse_hooks_file, BashCommand, FileModification, HookEvent, HookMetrics};
pub use states::{parse_states_file, StateTransitionEvent};
pub use transcript::{
    parse_transcript_file, MessageWrapper, TokenMetrics, TokenUsage, TranscriptEvent,
};

use anyhow::Result;
use std::fs;
use std::path::Path;

/// Unified metrics combining all data sources
#[derive(Debug, Default)]
pub struct UnifiedMetrics {
    pub hook_metrics: HookMetrics,
    pub token_metrics: TokenMetrics,
    pub state_transitions: Vec<StateTransitionEvent>,
    pub session_id: Option<String>,
}

/// Parse all available metrics from .hegel directory
pub fn parse_unified_metrics<P: AsRef<Path>>(state_dir: P) -> Result<UnifiedMetrics> {
    let state_dir = state_dir.as_ref();
    let hooks_path = state_dir.join("hooks.jsonl");
    let states_path = state_dir.join("states.jsonl");

    let mut unified = UnifiedMetrics::default();

    // Parse hooks if available
    if hooks_path.exists() {
        unified.hook_metrics = parse_hooks_file(&hooks_path)?;

        // Extract session_id and transcript_path from first hook event
        let content = fs::read_to_string(&hooks_path)?;
        if let Some(first_line) = content.lines().next() {
            let event: HookEvent = serde_json::from_str(first_line)?;
            unified.session_id = Some(event.session_id.clone());

            // Parse transcript if we have a path
            if let Some(transcript_path) = event.transcript_path {
                if Path::new(&transcript_path).exists() {
                    unified.token_metrics = parse_transcript_file(&transcript_path)?;
                }
            }
        }
    }

    // Parse states if available
    if states_path.exists() {
        unified.state_transitions = parse_states_file(&states_path)?;
    }

    Ok(unified)
}
