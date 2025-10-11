use crate::metrics::UnifiedMetrics;
use ratatui::style::{Color, Modifier, Style};
use ratatui::{
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// Overview tab: session summary + token usage
pub fn render_overview_tab(metrics: &UnifiedMetrics) -> Paragraph<'static> {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  📊 Token Usage",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::raw("    Input:  "),
            Span::styled(
                format!("{:>10}", metrics.token_metrics.total_input_tokens),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("    Output: "),
            Span::styled(
                format!("{:>10}", metrics.token_metrics.total_output_tokens),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  ⚡ Activity",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::raw("    Events: "),
            Span::styled(
                format!("{:>10}", metrics.hook_metrics.total_events),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("    Files:  "),
            Span::styled(
                format!(
                    "{:>10}",
                    metrics.hook_metrics.file_modification_frequency().len()
                ),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Summary "),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test_unified_metrics;

    #[test]
    fn test_render_overview_tab() {
        let metrics = test_unified_metrics();
        let widget = render_overview_tab(&metrics);

        // Verify widget renders without panic
        assert!(format!("{:?}", widget).contains("Paragraph"));
    }
}
