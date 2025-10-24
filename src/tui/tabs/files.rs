use crate::metrics::UnifiedMetrics;
use crate::tui::utils::scroll_indicators;
use ratatui::style::{Color, Modifier, Style};
use ratatui::{
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

/// Files tab: File modification frequency
pub fn render_files_tab(
    metrics: &UnifiedMetrics,
    scroll: usize,
    max_scroll: usize,
) -> List<'static> {
    let mut freq = metrics.hook_metrics.file_modification_frequency();
    let mut sorted: Vec<_> = freq.drain().collect();
    // Sort by count (desc), then by filename (asc) for stable ordering
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    // Apply scroll
    let items: Vec<ListItem> = sorted
        .iter()
        .skip(scroll)
        .map(|(file, count)| {
            // Color intensity based on modification count
            let count_color = if *count > 10 {
                Color::Red
            } else if *count > 5 {
                Color::Yellow
            } else {
                Color::Green
            };

            let line = Line::from(vec![
                Span::raw("  📝 "),
                Span::styled(
                    format!("{:>3}×", count),
                    Style::default()
                        .fg(count_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(file.to_string(), Style::default().fg(Color::White)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let (up_indicator, down_indicator) = scroll_indicators(scroll, max_scroll);
    let title = format!(" File Modifications {} {} ", up_indicator, down_indicator);

    if items.is_empty() {
        let empty_item = ListItem::new(Line::from(Span::styled(
            "  No file modifications",
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
    fn test_render_files_tab() {
        let metrics = UnifiedMetricsBuilder::new()
            .with_session("test")
            .with_events(0, 5)
            .build();

        let widget = render_files_tab(&metrics, 0, 0);

        // Verify widget renders
        assert!(format!("{:?}", widget).contains("List"));
    }
}
