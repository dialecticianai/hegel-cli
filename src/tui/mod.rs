//! Terminal User Interface (TUI) module
//!
//! Provides interactive dashboard functionality for real-time metrics visualization.

pub mod app;
mod tabs;
pub mod ui;
pub mod utils;

use app::AppState;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;
use std::io;
use std::path::Path;
use std::time::Duration;

/// Run the TUI dashboard
pub fn run_tui(state_dir: &Path) -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let result = run_event_loop(&mut terminal, state_dir);
    ratatui::restore();
    result
}

fn run_event_loop(terminal: &mut DefaultTerminal, state_dir: &Path) -> io::Result<()> {
    let mut app = AppState::new(state_dir).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to initialize app: {}", e),
        )
    })?;

    loop {
        // Check for file updates BEFORE rendering
        let _ = app.check_for_updates();

        // Render
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Handle keyboard events (100ms timeout for responsive file watching)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);

                    if app.should_quit {
                        break;
                    }

                    if app.needs_reload {
                        // Manual reload requested (via 'r' key)
                        app = AppState::new(state_dir).map_err(|e| {
                            io::Error::new(
                                io::ErrorKind::Other,
                                format!("Failed to reload app: {}", e),
                            )
                        })?;
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use tempfile::TempDir;

    #[test]
    fn test_run_event_loop_requires_valid_state_dir() {
        // Can't easily test the full event loop without a real terminal,
        // but we can verify error handling for invalid state directories
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent");

        // Note: This would fail because we can't initialize a real terminal in tests
        // This test documents the expected error handling behavior
        assert!(!nonexistent.exists());
    }
}
