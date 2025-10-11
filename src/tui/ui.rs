use crate::metrics::UnifiedMetrics;
use crate::tui::app::{AppState, Tab};
use crate::tui::utils::{build_timeline, visible_window, EventSource};
use ratatui::style::{Color, Modifier, Style};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Main layout: [Header(3), Main(Min), Footer(1)]
pub fn main_layout(area: Rect) -> [Rect; 3] {
    Layout::vertical([
        Constraint::Length(3), // Header
        Constraint::Min(0),    // Main (fills)
        Constraint::Length(1), // Footer
    ])
    .areas(area)
}

/// Main draw function (called from terminal.draw())
pub fn draw(frame: &mut Frame, app: &AppState) {
    let layout = main_layout(frame.area());
    let [header_area, main_area, footer_area] = layout;

    frame.render_widget(render_header(app), header_area);

    // Render main content based on selected tab
    match app.selected_tab {
        Tab::Overview => frame.render_widget(render_overview_tab(&app.metrics), main_area),
        Tab::Phases => frame.render_widget(
            render_phases_tab(&app.metrics, app.scroll_offset),
            main_area,
        ),
        Tab::Events => frame.render_widget(
            render_events_tab(&app.metrics, app.scroll_offset),
            main_area,
        ),
        Tab::Files => {
            frame.render_widget(render_files_tab(&app.metrics, app.scroll_offset), main_area)
        }
    }

    frame.render_widget(render_footer(), footer_area);
}

/// Render header: title + tab bar
fn render_header(app: &AppState) -> Paragraph<'static> {
    let session_id = app
        .metrics
        .session_id
        .as_deref()
        .unwrap_or("N/A")
        .to_string();

    let title = Line::from(vec![
        Span::styled(
            " ⚡ ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Hegel Dialectic Dashboard",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " ⚡ ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("Session: {} ", session_id),
            Style::default().fg(Color::Gray),
        ),
    ]);

    let tabs = Line::from(vec![
        Span::raw("  "),
        tab_label("Overview", app.selected_tab == Tab::Overview),
        Span::styled(" │ ", Style::default().fg(Color::Gray)),
        tab_label("Phases", app.selected_tab == Tab::Phases),
        Span::styled(" │ ", Style::default().fg(Color::Gray)),
        tab_label("Events", app.selected_tab == Tab::Events),
        Span::styled(" │ ", Style::default().fg(Color::Gray)),
        tab_label("Files", app.selected_tab == Tab::Files),
    ]);

    Paragraph::new(vec![title, tabs]).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Cyan)),
    )
}

fn tab_label(name: &str, selected: bool) -> Span<'static> {
    let name_string = format!(" {} ", name);
    if selected {
        Span::styled(
            name_string,
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(name_string, Style::default().fg(Color::White))
    }
}

/// Render footer: keybindings
fn render_footer() -> Paragraph<'static> {
    let keybindings = Line::from(vec![
        Span::raw(" "),
        Span::styled(
            "[q]",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Quit  ", Style::default().fg(Color::Gray)),
        Span::styled(
            "[Tab]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Next  ", Style::default().fg(Color::Gray)),
        Span::styled(
            "[↑↓/jk]",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Scroll  ", Style::default().fg(Color::Gray)),
        Span::styled(
            "[g/G]",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Top/Bottom  ", Style::default().fg(Color::Gray)),
        Span::styled(
            "[r]",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Reload", Style::default().fg(Color::Gray)),
    ]);

    Paragraph::new(keybindings).style(Style::default().bg(Color::Black))
}

/// Overview tab: session summary + token usage
fn render_overview_tab(metrics: &UnifiedMetrics) -> Paragraph<'static> {
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

/// Phases tab: per-phase breakdown with token counts
fn render_phases_tab(metrics: &UnifiedMetrics, scroll: usize) -> Paragraph<'static> {
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

/// Events tab: Recent hook events (scrollable)
fn render_events_tab(metrics: &UnifiedMetrics, scroll: usize) -> List<'static> {
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

/// Files tab: File modification frequency
fn render_files_tab(metrics: &UnifiedMetrics, scroll: usize) -> List<'static> {
    let mut freq = metrics.hook_metrics.file_modification_frequency();
    let mut sorted: Vec<_> = freq.drain().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1)); // Most modified first

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

    if items.is_empty() {
        let empty_item = ListItem::new(Line::from(Span::styled(
            "  No file modifications",
            Style::default().fg(Color::Gray),
        )));
        List::new(vec![empty_item]).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" File Modifications "),
        )
    } else {
        List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" File Modifications "),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{test_unified_metrics, UnifiedMetricsBuilder};
    use ratatui::layout::Rect;

    #[test]
    fn test_layout_main_areas() {
        let area = Rect::new(0, 0, 80, 24);
        let chunks = main_layout(area);

        // [Header(3), Main(Min), Footer(1)]
        assert_eq!(chunks[0].height, 3); // Header (title + tab bar)
        assert_eq!(chunks[1].height, 20); // Main content (fills)
        assert_eq!(chunks[2].height, 1); // Footer (keybindings)
    }

    #[test]
    fn test_layout_responsive() {
        // Small terminal
        let small = Rect::new(0, 0, 40, 10);
        let chunks = main_layout(small);
        assert_eq!(chunks[0].height, 3);
        assert_eq!(chunks[2].height, 1);
        assert_eq!(chunks[1].height, 6); // Main fills remaining

        // Large terminal
        let large = Rect::new(0, 0, 120, 40);
        let chunks = main_layout(large);
        assert_eq!(chunks[0].height, 3);
        assert_eq!(chunks[2].height, 1);
        assert_eq!(chunks[1].height, 36); // Main fills remaining
    }

    #[test]
    fn test_render_overview_tab() {
        let metrics = test_unified_metrics();
        let widget = render_overview_tab(&metrics);

        // Verify widget renders without panic (snapshot test would be better but requires terminal)
        // This at least validates the widget is constructed correctly
        assert!(format!("{:?}", widget).contains("Paragraph"));
    }

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

    #[test]
    fn test_render_files_tab() {
        let metrics = UnifiedMetricsBuilder::new()
            .with_session("test")
            .with_events(0, 5)
            .build();

        let widget = render_files_tab(&metrics, 0);

        // Verify widget renders
        assert!(format!("{:?}", widget).contains("List"));
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
