use crate::metrics::{parse_unified_metrics, UnifiedMetrics};
use crossterm::event::KeyCode;
use notify::{Event as NotifyEvent, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Phases,
    Events,
    Files,
}

pub struct AppState {
    pub metrics: UnifiedMetrics,
    pub selected_tab: Tab,
    pub scroll_offset: usize,
    pub should_quit: bool,
    pub needs_reload: bool,

    // File watching
    state_dir: PathBuf,
    file_rx: Receiver<Result<NotifyEvent, notify::Error>>,
    _watcher: notify::RecommendedWatcher,
}

impl AppState {
    /// Create new AppState with file watching enabled
    pub fn new(state_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let state_dir = state_dir.as_ref();
        let metrics = parse_unified_metrics(state_dir)?;

        // Setup file watcher
        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;

        watcher.watch(state_dir, RecursiveMode::NonRecursive)?;

        Ok(Self {
            metrics,
            selected_tab: Tab::Overview,
            scroll_offset: 0,
            should_quit: false,
            needs_reload: false,
            state_dir: state_dir.to_path_buf(),
            file_rx: rx,
            _watcher: watcher,
        })
    }

    /// Check for file updates (non-blocking)
    pub fn check_for_updates(&mut self) -> bool {
        let mut updated = false;

        // Drain all pending events
        while let Ok(res) = self.file_rx.try_recv() {
            if let Ok(event) = res {
                // Only reload on modify events (not create/remove)
                if matches!(event.kind, EventKind::Modify(_)) {
                    updated = true;
                }
            }
        }

        if updated {
            // Reload metrics
            if let Ok(metrics) = parse_unified_metrics(&self.state_dir) {
                self.metrics = metrics;
            }
        }

        updated
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        match code {
            // Quit
            KeyCode::Char('q') => self.should_quit = true,

            // Tab navigation (circular)
            KeyCode::Tab => self.next_tab(),
            KeyCode::BackTab => self.prev_tab(),

            // Scrolling (arrow keys)
            KeyCode::Up => self.scroll_up(),
            KeyCode::Down => self.scroll_down(),

            // Scrolling (vim bindings)
            KeyCode::Char('k') => self.scroll_up(),
            KeyCode::Char('j') => self.scroll_down(),
            KeyCode::Char('g') => self.scroll_to_top(),
            KeyCode::Char('G') => self.scroll_to_bottom(),

            // Reload metrics
            KeyCode::Char('r') => self.needs_reload = true,

            _ => {}
        }
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = match self.selected_tab {
            Tab::Overview => Tab::Phases,
            Tab::Phases => Tab::Events,
            Tab::Events => Tab::Files,
            Tab::Files => Tab::Overview,
        };
        self.scroll_offset = 0; // Reset scroll on tab change
    }

    pub fn prev_tab(&mut self) {
        self.selected_tab = match self.selected_tab {
            Tab::Overview => Tab::Files,
            Tab::Files => Tab::Events,
            Tab::Events => Tab::Phases,
            Tab::Phases => Tab::Overview,
        };
        self.scroll_offset = 0; // Reset scroll on tab change
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        let max = self.max_scroll();
        if self.scroll_offset < max {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll();
    }

    pub fn max_scroll(&self) -> usize {
        use crate::tui::utils::{build_timeline, max_scroll};

        // Calculate based on current tab content height
        match self.selected_tab {
            Tab::Overview => 0, // Fits on one screen
            Tab::Phases => max_scroll(self.metrics.phase_metrics.len(), 10),
            Tab::Events => {
                let timeline = build_timeline(&self.metrics);
                max_scroll(timeline.len(), 20)
            }
            Tab::Files => {
                let file_count = self
                    .metrics
                    .hook_metrics
                    .file_modification_frequency()
                    .len();
                max_scroll(file_count, 15)
            }
        }
    }
}

#[cfg(test)]
impl AppState {
    /// Create AppState for testing without file watching
    ///
    /// This is a minimal constructor for testing rendering without the complexity
    /// of file watching. For tests that need file watching, use AppState::new().
    pub fn new_for_test(metrics: UnifiedMetrics) -> Self {
        use std::sync::mpsc::channel;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let state_dir = temp_dir.path().to_path_buf();
        std::mem::forget(temp_dir); // Keep temp dir alive for the test

        let (tx, rx) = channel();
        let watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })
        .unwrap();

        Self {
            metrics,
            selected_tab: Tab::Overview,
            scroll_offset: 0,
            should_quit: false,
            needs_reload: false,
            state_dir,
            file_rx: rx,
            _watcher: watcher,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::test_storage_with_files;

    fn test_app() -> AppState {
        let (_temp, storage) = test_storage_with_files(
            Some(&[
                r#"{"session_id":"test","hook_event_name":"SessionStart","timestamp":"2025-01-01T10:00:00Z"}"#,
            ]),
            Some(&[
                r#"{"timestamp":"2025-01-01T10:00:00Z","workflow_id":"test","from_node":"START","to_node":"spec","phase":"spec","mode":"discovery"}"#,
            ]),
        );
        AppState::new(storage.state_dir()).unwrap()
    }

    #[test]
    fn test_app_state_init() {
        let app = test_app();

        assert_eq!(app.selected_tab, Tab::Overview);
        assert_eq!(app.scroll_offset, 0);
        assert!(!app.should_quit);
        assert!(!app.needs_reload);
    }

    #[test]
    fn test_handle_key_quit() {
        let mut app = test_app();
        app.handle_key(KeyCode::Char('q'));
        assert!(app.should_quit);
    }

    #[test]
    fn test_tab_navigation() {
        let mut app = test_app();

        // Start at Overview
        assert_eq!(app.selected_tab, Tab::Overview);

        // Tab -> Phases -> Events -> Files -> Overview (circular)
        app.handle_key(KeyCode::Tab);
        assert_eq!(app.selected_tab, Tab::Phases);

        app.handle_key(KeyCode::Tab);
        assert_eq!(app.selected_tab, Tab::Events);

        app.handle_key(KeyCode::Tab);
        assert_eq!(app.selected_tab, Tab::Files);

        app.handle_key(KeyCode::Tab);
        assert_eq!(app.selected_tab, Tab::Overview); // Wraps around
    }

    #[test]
    fn test_back_tab_navigation() {
        let mut app = test_app();

        // BackTab goes in reverse
        app.handle_key(KeyCode::BackTab);
        assert_eq!(app.selected_tab, Tab::Files);

        app.handle_key(KeyCode::BackTab);
        assert_eq!(app.selected_tab, Tab::Events);

        app.handle_key(KeyCode::BackTab);
        assert_eq!(app.selected_tab, Tab::Phases);

        app.handle_key(KeyCode::BackTab);
        assert_eq!(app.selected_tab, Tab::Overview); // Wraps around
    }

    #[test]
    fn test_scroll_down_within_bounds() {
        let mut app = test_app();
        app.selected_tab = Tab::Phases;

        // First check if we have enough content to scroll
        let max = app.max_scroll();
        if max > 0 {
            app.handle_key(KeyCode::Down);
            assert_eq!(app.scroll_offset, 1);

            app.handle_key(KeyCode::Down);
            assert_eq!(app.scroll_offset, 2);
        } else {
            // If not enough content, scrolling should stay at 0
            app.handle_key(KeyCode::Down);
            assert_eq!(app.scroll_offset, 0);
        }
    }

    #[test]
    fn test_scroll_up_stops_at_zero() {
        let mut app = test_app();

        // Can't scroll above 0
        app.handle_key(KeyCode::Up);
        assert_eq!(app.scroll_offset, 0);

        // Try with vim binding too
        app.handle_key(KeyCode::Char('k'));
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_down_stops_at_max() {
        let mut app = test_app();
        app.selected_tab = Tab::Phases;
        let max = app.max_scroll();

        // Scroll past max
        for _ in 0..max + 10 {
            app.handle_key(KeyCode::Down);
        }

        assert_eq!(app.scroll_offset, max);
    }

    #[test]
    fn test_scroll_to_top_and_bottom() {
        let mut app = test_app();
        app.selected_tab = Tab::Phases;

        // First scroll down a bit
        app.scroll_offset = 5;

        app.handle_key(KeyCode::Char('g')); // 'g' = top
        assert_eq!(app.scroll_offset, 0);

        app.handle_key(KeyCode::Char('G')); // 'G' = bottom
        assert_eq!(app.scroll_offset, app.max_scroll());
    }

    #[test]
    fn test_scroll_resets_on_tab_change() {
        let mut app = test_app();
        app.selected_tab = Tab::Phases;

        // Scroll down
        app.scroll_offset = 5;

        // Switch tab
        app.handle_key(KeyCode::Tab);

        // Scroll should reset
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn test_all_keyboard_shortcuts() {
        let mut app = test_app();

        // Test quit
        app.handle_key(KeyCode::Char('q'));
        assert!(app.should_quit);

        // Reset for next tests
        app.should_quit = false;

        // Test reload
        app.handle_key(KeyCode::Char('r'));
        assert!(app.needs_reload);

        // Test navigation (already covered by other tests, just verify they don't panic)
        app.handle_key(KeyCode::Tab);
        assert_eq!(app.selected_tab, Tab::Phases);

        app.handle_key(KeyCode::BackTab);
        assert_eq!(app.selected_tab, Tab::Overview);

        // Test scrolling (both arrow keys and vim bindings)
        app.selected_tab = Tab::Phases;

        // Test basic scroll operations (don't assume specific scroll offset values)
        app.scroll_offset = 0;
        app.handle_key(KeyCode::Down);
        let after_down = app.scroll_offset;

        app.handle_key(KeyCode::Up);
        assert_eq!(app.scroll_offset, after_down.saturating_sub(1));

        app.handle_key(KeyCode::Char('k')); // vim up
        let after_k = app.scroll_offset;

        app.handle_key(KeyCode::Char('j')); // vim down
        assert_eq!(
            app.scroll_offset,
            after_k
                + if app.scroll_offset < app.max_scroll() {
                    1
                } else {
                    0
                }
        );

        app.handle_key(KeyCode::Char('g')); // top
        assert_eq!(app.scroll_offset, 0);

        app.handle_key(KeyCode::Char('G')); // bottom
        assert_eq!(app.scroll_offset, app.max_scroll());
    }
}
