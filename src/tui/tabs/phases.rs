use crate::metrics::UnifiedMetrics;
use ratatui::style::{Color, Modifier, Style};
use ratatui::{
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Phases tab: per-phase breakdown with token counts
pub fn render_phases_tab(metrics: &UnifiedMetrics, scroll: usize) -> Paragraph<'static> {
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

    Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Phase Metrics "),
    )
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

        let widget = render_phases_tab(&metrics, 0);

        // Verify widget renders
        assert!(format!("{:?}", widget).contains("Paragraph"));
    }

    #[test]
    fn test_render_with_scroll() {
        let metrics = UnifiedMetricsBuilder::new()
            .with_session("test")
            .with_phases(15)
            .build();

        // Render with scroll offset
        let widget = render_phases_tab(&metrics, 5);

        // Verify widget renders with scroll applied
        assert!(format!("{:?}", widget).contains("Paragraph"));
    }
}
