use crate::metrics::UnifiedMetrics;
use crate::tui::utils::scroll_indicators;
use ratatui::style::{Color, Modifier, Style};
use ratatui::{
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Phases tab: per-phase breakdown with token counts
pub fn render_phases_tab(
    metrics: &UnifiedMetrics,
    scroll: usize,
    max_scroll: usize,
) -> Paragraph<'static> {
    let mut lines = vec![];

    // Apply scroll to phase list
    let visible_phases = metrics.phase_metrics.iter().skip(scroll);

    for phase in visible_phases {
        // Phase header with status indicator
        let (status_icon, status_color) = if phase.end_time.is_none() {
            ("🔵", Color::Green)
        } else {
            ("✅", Color::Gray)
        };

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::raw(status_icon),
            Span::raw(" "),
            Span::styled(
                phase.phase_name.to_uppercase(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if phase.end_time.is_none() {
                    " (active)"
                } else {
                    ""
                },
                Style::default().fg(status_color),
            ),
        ]));

        // Timestamps (start → end, or → active for an in-progress phase)
        let time_range = match &phase.end_time {
            Some(end) => format!("{} → {}", fmt_time(&phase.start_time), fmt_time(end)),
            None => format!("{} → (active)", fmt_time(&phase.start_time)),
        };
        lines.push(Line::from(vec![
            Span::raw("    🕐 Time:     "),
            Span::styled(time_range, Style::default().fg(Color::Gray)),
        ]));

        // Duration
        if phase.duration_seconds > 0 {
            let mins = phase.duration_seconds / 60;
            let secs = phase.duration_seconds % 60;
            lines.push(Line::from(vec![
                Span::raw("    ⏱  Duration: "),
                Span::styled(
                    format!("{}m {:02}s", mins, secs),
                    Style::default().fg(Color::Cyan),
                ),
            ]));
        }

        // Token usage
        let total_tokens =
            phase.token_metrics.total_input_tokens + phase.token_metrics.total_output_tokens;

        if total_tokens > 0 {
            lines.push(Line::from(vec![
                Span::raw("    📊 Tokens:   "),
                Span::styled(
                    format!("{}", total_tokens),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        // Activity counts
        lines.push(Line::from(vec![
            Span::raw("    ⚡ Activity:  Bash: "),
            Span::styled(
                format!("{}", phase.bash_commands.len()),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  Files: "),
            Span::styled(
                format!("{}", phase.file_modifications.len()),
                Style::default().fg(Color::Blue),
            ),
        ]));

        lines.push(Line::from("")); // Spacing
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No phase data available",
            Style::default().fg(Color::Gray),
        )));
    }

    let (up_indicator, down_indicator) = scroll_indicators(scroll, max_scroll);
    let title = format!(" Phase Metrics {} {} ", up_indicator, down_indicator);

    Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title),
    )
}

/// Format an RFC3339 timestamp as `HH:MM:SS`, falling back to the raw string
/// if it can't be parsed (never panics).
fn fmt_time(rfc3339: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(rfc3339)
        .map(|dt| dt.format("%H:%M:%S").to_string())
        .unwrap_or_else(|_| rfc3339.to_string())
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

        // Verify widget renders, and that phase timestamps are shown
        // (builder starts the first phase at 10:00:00).
        let rendered = format!("{:?}", widget);
        assert!(rendered.contains("Paragraph"));
        assert!(rendered.contains("10:00:00"));
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
        assert!(format!("{:?}", widget).contains("Paragraph"));
    }
}
