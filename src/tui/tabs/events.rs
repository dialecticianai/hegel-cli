use crate::metrics::UnifiedMetrics;
use crate::tui::utils::{
    build_timeline, format_timestamp, relative_day_label, scroll_indicators, visible_window,
    EventSource,
};
use ratatui::style::{Color, Modifier, Style};
use ratatui::{
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

/// Events tab: Recent hook events (scrollable)
pub fn render_events_tab(
    metrics: &UnifiedMetrics,
    scroll: usize,
    max_scroll: usize,
) -> List<'static> {
    // Build unified timeline using helper (merges all sources)
    let timeline = build_timeline(metrics);

    // Apply scroll using helper (20 rows visible)
    let visible = visible_window(&timeline, scroll, 20);

    let mut items: Vec<ListItem<'static>> = Vec::new();
    let mut last_day_label: Option<String> = None;

    for event in visible.iter() {
        // Check if we need a day separator
        if let Some(day_label) = relative_day_label(&event.timestamp) {
            if last_day_label.as_ref() != Some(&day_label) {
                // Insert day separator
                items.push(ListItem::new(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(
                        format!("─── {} ", day_label),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("─".repeat(60), Style::default().fg(Color::DarkGray)),
                ])));
                last_day_label = Some(day_label);
            }
        }

        // Render event
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
        let time = format_timestamp(&event.timestamp);

        let line = Line::from(vec![
            Span::raw(" "),
            Span::styled(format!("[{}]", time), Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::raw(source_icon),
            Span::raw(" "),
            Span::styled(source_name, source_style),
            Span::styled(" │ ", Style::default().fg(Color::Gray)),
            Span::styled(event_type, Style::default().fg(Color::Yellow)),
            Span::raw(" "),
            Span::styled(detail, Style::default().fg(Color::White)),
        ]);

        items.push(ListItem::new(line));
    }

    let (up_indicator, down_indicator) = scroll_indicators(scroll, max_scroll);
    let title = format!(" Event Stream {} {} ", up_indicator, down_indicator);

    if items.is_empty() {
        let empty_item = ListItem::new(Line::from(Span::styled(
            "  No events available",
            Style::default().fg(Color::Gray),
        )));
        List::new(vec![empty_item]).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(title),
        )
    } else {
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(title),
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

        let widget = render_events_tab(&metrics, 0, 0);

        // Verify widget renders
        assert!(format!("{:?}", widget).contains("List"));
    }
}
