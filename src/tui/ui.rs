use crate::tui::app::{AppState, Tab};
use crate::tui::tabs::{
    render_events_tab, render_files_tab, render_overview_tab, render_phases_tab,
};
use ratatui::style::{Color, Modifier, Style};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
