//! TUI testing utilities for ratatui snapshot tests
//!
//! All exports reserved for future TUI testing (see test_helpers/README.md)

use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;

/// Standard terminal sizes for consistent testing
#[allow(dead_code)] // Reserved for TUI tests (see test_helpers/README.md)
pub const SMALL_TERM: (u16, u16) = (40, 10);
#[allow(dead_code)] // Reserved for TUI tests (see test_helpers/README.md)
pub const MEDIUM_TERM: (u16, u16) = (80, 24);
#[allow(dead_code)] // Reserved for TUI tests (see test_helpers/README.md)
pub const LARGE_TERM: (u16, u16) = (120, 40);

/// Create test terminal with specified size
///
/// # Arguments
/// * `width` - Terminal width in columns
/// * `height` - Terminal height in rows
///
/// # Example
/// ```ignore
/// let mut terminal = test_terminal(80, 24);
/// terminal.draw(|f| { ... }).unwrap();
/// ```
#[allow(dead_code)] // Reserved for TUI tests (see test_helpers/README.md)
pub fn test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

/// Convert buffer to string for snapshot comparison
///
/// Preserves exact layout including whitespace and newlines.
/// Useful for golden file testing.
///
/// # Example
/// ```ignore
/// let buffer = terminal.backend().buffer();
/// let output = buffer_to_string(buffer);
/// assert_eq!(output, expected_snapshot);
/// ```
#[allow(dead_code)] // Reserved for TUI tests (see test_helpers/README.md)
pub fn buffer_to_string(buffer: &Buffer) -> String {
    let mut output = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = &buffer[(x, y)];
            output.push_str(cell.symbol());
        }
        output.push('\n');
    }
    output
}

/// Render widget to buffer and return string representation
///
/// Convenience wrapper that creates a terminal, renders widget,
/// and returns string output in one call.
///
/// # Example
/// ```ignore
/// let widget = Paragraph::new("Hello");
/// let output = render_to_string(widget, 80, 24);
/// assert!(output.contains("Hello"));
/// ```
#[allow(dead_code)] // Reserved for TUI tests (see test_helpers/README.md)
pub fn render_to_string<W>(widget: W, width: u16, height: u16) -> String
where
    W: ratatui::widgets::Widget,
{
    let mut terminal = test_terminal(width, height);
    terminal
        .draw(|f| {
            f.render_widget(widget, f.area());
        })
        .unwrap();
    buffer_to_string(terminal.backend().buffer())
}
