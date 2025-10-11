use crate::metrics::UnifiedMetrics;
use crate::tui::utils::{build_timeline, visible_window, EventSource};
use ratatui::style::{Color, Modifier, Style};
use ratatui::{
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

/// Events tab: Recent hook events (scrollable)
pub fn render_events_tab(metrics: &UnifiedMetrics, scroll: usize) -> List<'static> {
    // Build unified timeline using helper (merges all sources)
    let timeline = build_timeline(metrics);

    // Apply scroll using helper (20 rows visible)
    let visible = visible_window(&timeline, scroll, 20);

    let items: Vec<ListItem<'static>> = visible
        .iter()
        .map(|event| {
            let (source_icon, source_style) = if event.source == EventSource::Claude {
                (
                    "🤖",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    "⚙️ ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            };

            let source_name = if event.source == EventSource::Claude {
                "Claude"
            } else {
                "Hegel "
            };

            // Clone all data to make it 'static
            let event_type = event.event_type.clone();
            let detail = event.detail.clone();

            let line = Line::from(vec![
                Span::raw(" "),
                Span::raw(source_icon),
                Span::raw(" "),
                Span::styled(source_name, source_style),
                Span::styled(" │ ", Style::default().fg(Color::Gray)),
                Span::styled(event_type, Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::styled(detail, Style::default().fg(Color::White)),
            ]);

            ListItem::new(line)
        })
        .collect();

    if items.is_empty() {
        let empty_item = ListItem::new(Line::from(Span::styled(
            "  No events available",
            Style::default().fg(Color::Gray),
        )));
        List::new(vec![empty_item]).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Event Stream "),
        )
    } else {
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Event Stream "),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::UnifiedMetricsBuilder;

    #[test]
    fn test_render_events_tab() {
        let metrics = UnifiedMetricsBuilder::new()
            .with_session("test")
            .with_phases(3)
            .with_events(10, 5)
            .build();

        let widget = render_events_tab(&metrics, 0);

        // Verify widget renders
        assert!(format!("{:?}", widget).contains("List"));
    }
}
