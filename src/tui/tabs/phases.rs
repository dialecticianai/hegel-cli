use crate::metrics::{PhaseMetrics, UnifiedMetrics};
use crate::tui::utils::scroll_indicators;
use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::{
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
};

/// Column widths for the phases table.
/// PHASE (icon+name), WINDOW (widest — multi-day spans), DURATION, TOKENS, BASH, FILES.
const PHASE_COLUMNS: [Constraint; 6] = [
    Constraint::Length(10),
    Constraint::Length(41),
    Constraint::Length(10),
    Constraint::Length(8),
    Constraint::Length(5),
    Constraint::Length(6),
];

/// Phases tab: one row per phase with aligned columns.
pub fn render_phases_tab(
    metrics: &UnifiedMetrics,
    scroll: usize,
    max_scroll: usize,
) -> Table<'static> {
    let header = Row::new([
        Cell::from("PHASE"),
        Cell::from("WINDOW"),
        Cell::from(Line::from("DURATION").right_aligned()),
        Cell::from(Line::from("TOKENS").right_aligned()),
        Cell::from(Line::from("BASH").right_aligned()),
        Cell::from(Line::from("FILES").right_aligned()),
    ])
    .style(
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let mut rows: Vec<Row> = metrics
        .phase_metrics
        .iter()
        .filter(|p| is_displayed_phase(p))
        .skip(scroll)
        .map(|phase| {
            let icon = if phase.end_time.is_none() {
                "🔵"
            } else {
                "✅"
            };
            let total_tokens =
                phase.token_metrics.total_input_tokens + phase.token_metrics.total_output_tokens;

            Row::new(vec![
                Cell::from(Line::from(vec![
                    Span::raw(format!("{} ", icon)),
                    Span::styled(
                        phase.phase_name.to_uppercase(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])),
                Cell::from(Span::styled(
                    fmt_phase_window(&phase.start_time, phase.end_time.as_deref()),
                    Style::default().fg(Color::Gray),
                )),
                Cell::from(Line::from(fmt_duration(phase.duration_seconds)).right_aligned())
                    .style(Style::default().fg(Color::Cyan)),
                Cell::from(Line::from(total_tokens.to_string()).right_aligned())
                    .style(Style::default().fg(Color::Magenta)),
                Cell::from(Line::from(phase.bash_commands.len().to_string()).right_aligned())
                    .style(Style::default().fg(Color::Green)),
                Cell::from(Line::from(phase.file_modifications.len().to_string()).right_aligned())
                    .style(Style::default().fg(Color::Blue)),
            ])
        })
        .collect();

    if rows.is_empty() {
        rows.push(Row::new(vec![Cell::from(Span::styled(
            "No phase data available",
            Style::default().fg(Color::Gray),
        ))]));
    }

    let (up_indicator, down_indicator) = scroll_indicators(scroll, max_scroll);
    let title = format!(" Phase Metrics {} {} ", up_indicator, down_indicator);

    Table::new(rows, PHASE_COLUMNS)
        .header(header)
        .column_spacing(2)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(title),
        )
}

/// Whether a phase should appear in the phases pane. Zero-duration terminal
/// (done/aborted) phases are structural transitions with no activity, so they
/// are hidden from the metrics view.
pub(crate) fn is_displayed_phase(phase: &PhaseMetrics) -> bool {
    !(phase.duration_seconds == 0 && crate::engine::is_terminal(&phase.phase_name))
}

/// Format a duration in seconds as `{m}m {s:02}s`.
fn fmt_duration(secs: u64) -> String {
    format!("{}m {:02}s", secs / 60, secs % 60)
}

/// Format a phase's `[start, end]` window for display.
///
/// The start is always shown as full `YYYY-MM-DD HH:MM:SS`. The end shows just
/// `HH:MM:SS` when it falls on the same calendar day, or a full date-time when
/// the phase spans multiple days (so long/multi-day phases are unambiguous).
/// `None` end renders as "(active)". Unparseable timestamps fall back to the
/// raw string (never panics).
fn fmt_phase_window(start: &str, end: Option<&str>) -> String {
    use chrono::DateTime;

    let start_dt = DateTime::parse_from_rfc3339(start).ok();
    let start_str = start_dt
        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| start.to_string());

    let Some(end) = end else {
        return format!("{} → (active)", start_str);
    };

    let end_dt = DateTime::parse_from_rfc3339(end).ok();
    let end_str = match (start_dt, end_dt) {
        // Same calendar day: time only.
        (Some(s), Some(e)) if s.date_naive() == e.date_naive() => e.format("%H:%M:%S").to_string(),
        // Spans days (or no start to compare): full date-time.
        (_, Some(e)) => e.format("%Y-%m-%d %H:%M:%S").to_string(),
        // Unparseable end.
        (_, None) => end.to_string(),
    };
    format!("{} → {}", start_str, end_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::UnifiedMetricsBuilder;

    #[test]
    fn test_render_phases_tab() {
        let metrics = UnifiedMetricsBuilder::new()
            .with_session("test")
            .with_phases(3)
            .build();

        let widget = render_phases_tab(&metrics, 0, 0);

        // Verify widget renders, and that phase timestamps (with date) are shown
        // (builder starts the first phase at 2025-01-01 10:00:00).
        let rendered = format!("{:?}", widget);
        assert!(rendered.contains("Table"));
        assert!(rendered.contains("2025-01-01 10:00:00"));
    }

    #[test]
    fn test_fmt_phase_window() {
        // Same calendar day: end shows time only.
        assert_eq!(
            fmt_phase_window("2025-01-01T10:00:00Z", Some("2025-01-01T10:15:00Z")),
            "2025-01-01 10:00:00 → 10:15:00"
        );
        // Spans days: end shows full date-time.
        assert_eq!(
            fmt_phase_window("2025-01-01T23:00:00Z", Some("2025-01-03T01:00:00Z")),
            "2025-01-01 23:00:00 → 2025-01-03 01:00:00"
        );
        // Active phase.
        assert_eq!(
            fmt_phase_window("2025-01-01T10:00:00Z", None),
            "2025-01-01 10:00:00 → (active)"
        );
        // Unparseable timestamps fall back to raw, no panic.
        assert_eq!(
            fmt_phase_window("bogus", Some("also-bogus")),
            "bogus → also-bogus"
        );
    }

    #[test]
    fn test_is_displayed_phase_hides_terminal_zero_duration() {
        use crate::metrics::TokenMetrics;
        let mk = |name: &str, dur: u64| PhaseMetrics {
            phase_name: name.to_string(),
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: Some("2025-01-01T10:00:00Z".to_string()),
            duration_seconds: dur,
            token_metrics: TokenMetrics::default(),
            bash_commands: vec![],
            file_modifications: vec![],
            git_commits: vec![],
            is_synthetic: false,
            workflow_id: None,
        };
        assert!(!is_displayed_phase(&mk("done", 0))); // terminal + zero duration -> hidden
        assert!(!is_displayed_phase(&mk("aborted", 0))); // hidden
        assert!(is_displayed_phase(&mk("code", 0))); // non-terminal -> shown
        assert!(is_displayed_phase(&mk("done", 5))); // terminal but has duration -> shown
    }

    #[test]
    fn test_render_with_scroll() {
        let metrics = UnifiedMetricsBuilder::new()
            .with_session("test")
            .with_phases(15)
            .build();

        // Render with scroll offset
        let widget = render_phases_tab(&metrics, 5, 10);

        // Verify widget renders with scroll applied
        assert!(format!("{:?}", widget).contains("Table"));
    }
}
