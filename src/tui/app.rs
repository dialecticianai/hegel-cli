use crate::metrics::{parse_unified_metrics, UnifiedMetrics};
use crossterm::event::KeyCode;
use notify::{Event as NotifyEvent, EventKind, RecursiveMode, Watcher};
use std::cell::Cell;
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

    /// Height (rows) of the main content area, captured each frame from
    /// `frame.area()` during `ui::draw`. `0` until the first draw. Interior
    /// mutability so `draw(&AppState)` can record it; read back by the scroll
    /// math so page jumps and bounds track the real terminal size.
    content_height: Cell<u16>,

    // File watching
    state_dir: PathBuf,
    file_rx: Receiver<Result<NotifyEvent, notify::Error>>,
    _watcher: notify::RecommendedWatcher,
}

impl AppState {
    /// Create new AppState with file watching enabled
    pub fn new(state_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let state_dir = state_dir.as_ref();
        // TUI shows full history including archives
        let metrics = parse_unified_metrics(state_dir, true, None)?;

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
            content_height: Cell::new(0),
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
            if let Ok(metrics) = parse_unified_metrics(&self.state_dir, true, None) {
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

            // Page scrolling
            KeyCode::Char(' ') | KeyCode::PageDown => self.page_down(),
            KeyCode::Char('b') | KeyCode::PageUp => self.page_up(),

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

    /// Record the main content area height for the current frame. Called from
    /// `ui::draw` (which holds `&AppState`) via interior mutability.
    pub(crate) fn set_content_height(&self, height: u16) {
        self.content_height.set(height);
    }

    /// Number of data rows the current tab can actually display, derived from
    /// the last-rendered content height minus the pane's chrome (borders, plus
    /// a header row for the Phases table). Before the first draw — and in tests
    /// that never render — `content_height` is 0, so we fall back to sane
    /// per-tab defaults. Overview fits on one screen, so it never scrolls.
    pub(crate) fn visible_rows(&self) -> usize {
        let height = self.content_height.get() as usize;
        if height == 0 {
            return match self.selected_tab {
                Tab::Overview => 0,
                Tab::Phases => 10,
                Tab::Events => 20,
                Tab::Files => 15,
            };
        }
        // All scrollable panes draw a full border (top+bottom = 2 rows); the
        // Phases table also reserves one row for its column header.
        let chrome = match self.selected_tab {
            Tab::Overview => return 0,
            Tab::Phases => 3,
            Tab::Events | Tab::Files => 2,
        };
        height.saturating_sub(chrome).max(1)
    }

    /// Rows to advance per page jump: a full page minus one row of overlap, so
    /// the last line of the old page becomes the first line of the new one
    /// (standard pager behavior). Always at least 1.
    fn page_step(&self) -> usize {
        self.visible_rows().saturating_sub(1).max(1)
    }

    /// Scroll forward one page (clamped to the bottom).
    pub fn page_down(&mut self) {
        self.scroll_offset = (self.scroll_offset + self.page_step()).min(self.max_scroll());
    }

    /// Scroll back one page (clamped to the top).
    pub fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.page_step());
    }

    pub fn max_scroll(&self) -> usize {
        use crate::tui::utils::{build_timeline, max_scroll};

        // Content height per tab; the visible page height comes from visible_rows().
        let content_len = match self.selected_tab {
            Tab::Overview => return 0, // Fits on one screen
            Tab::Phases => self
                .metrics
                .phase_metrics
                .iter()
                .filter(|p| crate::tui::tabs::is_displayed_phase(p))
                .count(),
            Tab::Events => build_timeline(&self.metrics).len(),
            Tab::Files => self
                .metrics
                .hook_metrics
                .file_modification_frequency()
                .len(),
        };
        max_scroll(content_len, self.visible_rows())
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
            content_height: Cell::new(0),
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
    fn test_space_pages_forward_and_back() {
        use crate::test_helpers::UnifiedMetricsBuilder;
        let metrics = UnifiedMetricsBuilder::new()
            .with_session("t")
            .with_phases(30)
            .build();
        let mut app = AppState::new_for_test(metrics);
        app.selected_tab = Tab::Phases;
        // Simulate a frame: a 13-row content area -> 10 visible rows
        // (2 borders + 1 header). Page step is one less (9) for the overlap.
        app.set_content_height(13);
        assert_eq!(app.visible_rows(), 10);
        assert_eq!(app.max_scroll(), 20); // 30 phases - 10 visible

        // Space pages forward by a page minus one overlap row (step = 9).
        app.handle_key(KeyCode::Char(' '));
        assert_eq!(app.scroll_offset, 9);
        app.handle_key(KeyCode::PageDown);
        assert_eq!(app.scroll_offset, 18);
        // Clamps at the bottom.
        app.handle_key(KeyCode::Char(' '));
        assert_eq!(app.scroll_offset, 20);
        app.handle_key(KeyCode::Char(' '));
        assert_eq!(app.scroll_offset, 20);

        // PageUp / 'b' jump back a page (step 9) and clamp at the top.
        app.scroll_offset = 15;
        app.handle_key(KeyCode::PageUp);
        assert_eq!(app.scroll_offset, 6);
        app.handle_key(KeyCode::Char('b'));
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn test_visible_rows_tracks_content_height_and_chrome() {
        use crate::test_helpers::UnifiedMetricsBuilder;
        let metrics = UnifiedMetricsBuilder::new()
            .with_session("t")
            .with_phases(5)
            .build();
        let mut app = AppState::new_for_test(metrics);

        // Before any draw: per-tab fallback defaults.
        app.selected_tab = Tab::Phases;
        assert_eq!(app.visible_rows(), 10);
        app.selected_tab = Tab::Events;
        assert_eq!(app.visible_rows(), 20);
        app.selected_tab = Tab::Files;
        assert_eq!(app.visible_rows(), 15);

        // After a frame records the area height, rows = height - chrome.
        app.set_content_height(25);
        app.selected_tab = Tab::Phases; // 2 borders + 1 header
        assert_eq!(app.visible_rows(), 22);
        app.selected_tab = Tab::Events; // 2 borders
        assert_eq!(app.visible_rows(), 23);
        app.selected_tab = Tab::Files; // 2 borders
        assert_eq!(app.visible_rows(), 23);

        // Overview never scrolls.
        app.selected_tab = Tab::Overview;
        assert_eq!(app.visible_rows(), 0);
    }

    #[test]
    fn test_page_down_puts_previous_bottom_row_on_top() {
        use crate::test_helpers::UnifiedMetricsBuilder;
        // With R visible rows, page 1 shows rows [0, R-1]; a page down should
        // land the bottom row (R-1) at the top (one-row overlap).
        let metrics = UnifiedMetricsBuilder::new()
            .with_session("t")
            .with_phases(50)
            .build();
        let mut app = AppState::new_for_test(metrics);
        app.selected_tab = Tab::Phases;
        app.set_content_height(17); // 14 visible rows
        assert_eq!(app.visible_rows(), 14);

        app.handle_key(KeyCode::PageDown);
        assert_eq!(app.scroll_offset, 13); // previous bottom row is the new top
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
