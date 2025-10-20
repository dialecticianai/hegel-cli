//! Utility functions for TUI rendering and data manipulation
//!
//! This module provides reusable helpers for scrolling, event timeline
//! construction, and other common TUI operations.

use crate::metrics::UnifiedMetrics;

// ========== Scroll Utilities ==========

/// Calculate visible window for scrollable content
///
/// Returns a slice of items that should be visible given the scroll offset
/// and visible height. Handles edge cases where scroll offset exceeds content.
///
/// # Arguments
/// * `items` - Full list of items
/// * `scroll_offset` - Current scroll position (0 = top)
/// * `visible_height` - Number of items that fit on screen
///
/// # Example
/// ```ignore
/// let all_items = vec![1, 2, 3, 4, 5];
/// let visible = visible_window(&all_items, 2, 2);
/// assert_eq!(visible, &[3, 4]);
/// ```
pub fn visible_window<T>(items: &[T], scroll_offset: usize, visible_height: usize) -> &[T] {
    let start = scroll_offset.min(items.len().saturating_sub(1));
    let end = (start + visible_height).min(items.len());
    &items[start..end]
}

/// Calculate maximum scroll offset for given content
///
/// Returns the maximum scroll position before hitting bottom.
/// Result is always >= 0.
///
/// # Arguments
/// * `content_height` - Total number of items
/// * `visible_height` - Number of items visible at once
///
/// # Example
/// ```ignore
/// let max = max_scroll(100, 20);
/// assert_eq!(max, 80); // Can scroll down 80 positions
/// ```
pub fn max_scroll(content_height: usize, visible_height: usize) -> usize {
    content_height.saturating_sub(visible_height)
}

/// Get scroll indicator symbols based on current position
///
/// Returns (top_indicator, bottom_indicator) tuple.
/// Uses Unicode arrows: ↑ (can scroll up), ↓ (can scroll down), · (can't scroll)
///
/// # Arguments
/// * `scroll_offset` - Current scroll position
/// * `max_scroll` - Maximum scroll position
///
/// # Returns
/// Tuple of (top, bottom) indicators
///
/// # Example
/// ```ignore
/// let (top, bottom) = scroll_indicators(5, 10);
/// assert_eq!((top, bottom), ("↑", "↓")); // Can scroll both ways
///
/// let (top, bottom) = scroll_indicators(0, 10);
/// assert_eq!((top, bottom), ("·", "↓")); // Can only scroll down
/// ```
/// TODO: Investigate why this wasn't used in TUI implementation
#[allow(dead_code)]
pub fn scroll_indicators(scroll_offset: usize, max_scroll: usize) -> (&'static str, &'static str) {
    let can_scroll_up = scroll_offset > 0;
    let can_scroll_down = scroll_offset < max_scroll;

    match (can_scroll_up, can_scroll_down) {
        (true, true) => ("↑", "↓"),   // Both directions
        (true, false) => ("↑", "·"),  // Only up
        (false, true) => ("·", "↓"),  // Only down
        (false, false) => ("·", "·"), // No scroll
    }
}

// ========== Event Timeline ==========

/// Unified event for timeline display
///
/// Combines different event sources (Claude hooks, Hegel state transitions)
/// into a single type for chronological display.
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    pub timestamp: String,
    pub source: EventSource,
    pub event_type: String,
    pub detail: String,
}

/// Source of timeline event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSource {
    /// Event from Claude Code hooks
    Claude,
    /// Event from Hegel workflow transitions
    Hegel,
}

/// Build chronological timeline from all metric sources
///
/// Merges bash commands, file modifications, and state transitions
/// into a single timeline sorted by timestamp (most recent first).
///
/// # Arguments
/// * `metrics` - UnifiedMetrics containing all event data
///
/// # Returns
/// Vector of TimelineEvents sorted by timestamp descending
///
/// # Example
/// ```ignore
/// let timeline = build_timeline(&metrics);
/// for event in timeline.iter().take(10) {
///     println!("[{}] {}: {}", event.source, event.event_type, event.detail);
/// }
/// ```
pub fn build_timeline(metrics: &UnifiedMetrics) -> Vec<TimelineEvent> {
    let mut events = Vec::new();

    // Add bash commands from Claude hooks
    for bash in &metrics.hook_metrics.bash_commands {
        if let Some(ts) = &bash.timestamp {
            events.push(TimelineEvent {
                timestamp: ts.clone(),
                source: EventSource::Claude,
                event_type: "Bash".to_string(),
                detail: bash.command.clone(),
            });
        }
    }

    // Add file modifications from Claude hooks
    for file_mod in &metrics.hook_metrics.file_modifications {
        if let Some(ts) = &file_mod.timestamp {
            events.push(TimelineEvent {
                timestamp: ts.clone(),
                source: EventSource::Claude,
                event_type: file_mod.tool.clone(),
                detail: file_mod.file_path.clone(),
            });
        }
    }

    // Add state transitions from Hegel
    for transition in &metrics.state_transitions {
        events.push(TimelineEvent {
            timestamp: transition.timestamp.clone(),
            source: EventSource::Hegel,
            event_type: "Transition".to_string(),
            detail: format!("{} → {}", transition.from_node, transition.to_node),
        });
    }

    // Sort by timestamp (most recent first)
    events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    events
}

/// Format timestamp for display
///
/// Extracts time portion from ISO 8601 timestamp for compact display.
///
/// # Example
/// ```ignore
/// let formatted = format_timestamp("2025-01-01T10:30:45Z");
/// assert_eq!(formatted, "10:30:45");
/// ```
/// TODO: Investigate why this wasn't used in TUI implementation
#[allow(dead_code)]
pub fn format_timestamp(timestamp: &str) -> String {
    // Extract time portion (HH:MM:SS) from ISO 8601
    if let Some(time_start) = timestamp.find('T') {
        let time_part = &timestamp[time_start + 1..];
        if let Some(z_pos) = time_part.find('Z') {
            return time_part[..z_pos].to_string();
        }
        time_part.to_string()
    } else {
        timestamp.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test_unified_metrics;

    // ========== Scroll Tests ==========

    #[test]
    fn test_visible_window_normal() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let visible = visible_window(&items, 2, 3);
        assert_eq!(visible, &[3, 4, 5]);
    }

    #[test]
    fn test_visible_window_at_start() {
        let items = vec![1, 2, 3, 4, 5];
        let visible = visible_window(&items, 0, 3);
        assert_eq!(visible, &[1, 2, 3]);
    }

    #[test]
    fn test_visible_window_at_end() {
        let items = vec![1, 2, 3, 4, 5];
        let visible = visible_window(&items, 3, 3);
        assert_eq!(visible, &[4, 5]); // Only 2 items left
    }

    #[test]
    fn test_visible_window_offset_too_large() {
        let items = vec![1, 2, 3];
        let visible = visible_window(&items, 100, 2);
        assert_eq!(visible, &[3]); // Clamped to last valid position
    }

    #[test]
    fn test_visible_window_empty() {
        let items: Vec<i32> = vec![];
        let visible = visible_window(&items, 0, 5);
        assert!(visible.is_empty());
    }

    #[test]
    fn test_max_scroll() {
        assert_eq!(max_scroll(100, 20), 80);
        assert_eq!(max_scroll(10, 10), 0);
        assert_eq!(max_scroll(5, 10), 0); // Content smaller than window
        assert_eq!(max_scroll(0, 10), 0); // Empty content
    }

    #[test]
    fn test_scroll_indicators_both_ways() {
        let (top, bottom) = scroll_indicators(5, 10);
        assert_eq!((top, bottom), ("↑", "↓"));
    }

    #[test]
    fn test_scroll_indicators_only_down() {
        let (top, bottom) = scroll_indicators(0, 10);
        assert_eq!((top, bottom), ("·", "↓"));
    }

    #[test]
    fn test_scroll_indicators_only_up() {
        let (top, bottom) = scroll_indicators(10, 10);
        assert_eq!((top, bottom), ("↑", "·"));
    }

    #[test]
    fn test_scroll_indicators_none() {
        let (top, bottom) = scroll_indicators(0, 0);
        assert_eq!((top, bottom), ("·", "·"));
    }

    // ========== Timeline Tests ==========

    #[test]
    fn test_build_timeline_includes_all_sources() {
        let metrics = test_unified_metrics();
        let timeline = build_timeline(&metrics);

        // Should have bash commands + file mods + state transitions
        // From test_unified_metrics: 10 bash + 5 files + 3 states = 18
        assert_eq!(timeline.len(), 18);

        // Check we have events from both sources
        let has_claude = timeline.iter().any(|e| e.source == EventSource::Claude);
        let has_hegel = timeline.iter().any(|e| e.source == EventSource::Hegel);
        assert!(has_claude);
        assert!(has_hegel);
    }

    #[test]
    fn test_build_timeline_sorted_by_timestamp() {
        let metrics = test_unified_metrics();
        let timeline = build_timeline(&metrics);

        // Verify descending timestamp order (most recent first)
        for i in 0..timeline.len() - 1 {
            assert!(
                timeline[i].timestamp >= timeline[i + 1].timestamp,
                "Timeline not sorted: {} < {}",
                timeline[i].timestamp,
                timeline[i + 1].timestamp
            );
        }
    }

    #[test]
    fn test_build_timeline_event_types() {
        let metrics = test_unified_metrics();
        let timeline = build_timeline(&metrics);

        let bash_events: Vec<_> = timeline.iter().filter(|e| e.event_type == "Bash").collect();
        assert_eq!(bash_events.len(), 10);

        let edit_events: Vec<_> = timeline.iter().filter(|e| e.event_type == "Edit").collect();
        assert_eq!(edit_events.len(), 5);

        let transition_events: Vec<_> = timeline
            .iter()
            .filter(|e| e.event_type == "Transition")
            .collect();
        assert_eq!(transition_events.len(), 3);
    }

    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp("2025-01-01T10:30:45Z"), "10:30:45");
        assert_eq!(format_timestamp("2025-01-01T00:00:00Z"), "00:00:00");
        assert_eq!(format_timestamp("10:30:45"), "10:30:45"); // Already formatted
    }
}
