# Phase 1 Implementation Plan: Metrics Collection & Analysis

**Goal**: Parse hook data and build metrics to feed cycle detection and budget enforcement

**Context**: Hook events are captured in `.hegel/hooks.jsonl`. This phase processes that data into actionable metrics and visualizes them in a TUI dashboard.

**Progress**: ✅ ALL STEPS COMPLETE (Steps 1-7) | Session: 2025-10-10

---

## Architecture Overview

### Data Streams

**Input Streams**:
1. `.hegel/hooks.jsonl` - Claude Code tool use events (already implemented)
2. `.hegel/states.jsonl` - Hegel workflow state transitions (to be implemented)
3. Claude transcript files - Token usage data via `transcript_path` field in hook events
   - Location: `.claude/projects/**/*.jsonl`
   - Token data at: `message.usage.{input_tokens, output_tokens, cache_*_input_tokens}`

**Processing Layer**:
- Unified event schema for both streams
- Metrics aggregator (in-memory + persistent)
- Real-time event parser with stream-based merge-sort (constant memory)

**Output Layer**:
1. `hegel metrics` - Text-based metrics summary
2. `hegel top` - Interactive TUI dashboard (ratatui/crossterm with notify file watching)

### Core Data Structures

```rust
// Unified event envelope
struct Event {
    timestamp: DateTime<Utc>,
    source: EventSource,           // Hegel | Claude
    workflow_id: Option<String>,   // ISO timestamp of `hegel start` (e.g., "2025-10-09T04:15:23Z")
    phase: Option<String>,         // Current node (spec, plan, code, etc.)
    event_type: EventType,
    payload: serde_json::Value,
}

// Metrics aggregates
struct PhaseMetrics {
    phase_name: String,
    duration_secs: u64,
    token_usage: TokenMetrics,     // From transcript message.usage
    file_edits: Vec<FileEdit>,
    bash_commands: Vec<BashCommand>,
}
```

---

## Implementation Steps (TDD)

### Step 1: State Transition Logging ✅ COMPLETE

**Goal**: Emit workflow state changes to `.hegel/states.jsonl`

**Test 1**: State transition writes to states.jsonl
```rust
#[test]
fn test_state_transition_logged() {
    // Start workflow
    // Transition to next node
    // Assert states.jsonl contains transition event
}
```

**Implementation**:
- Add logging to `engine/mod.rs::get_next_prompt()` after successful transition
- Create `StateTransitionEvent` struct
- Write atomically to `.hegel/states.jsonl` (append-only)

**Test 2**: State transition includes all required fields
```rust
#[test]
fn test_state_transition_event_schema() {
    // Assert event has: timestamp, workflow_id, from_node, to_node, phase, mode
}
```

**Files to modify**:
- `src/engine/mod.rs` - Add logging after line 56 (transition logic)
- `src/storage/mod.rs` - Add `log_state_transition()` method

---

### Step 2: Unified Event Parser ⚠️ COMPLETE (Alternative Approach)

**Goal**: Read and normalize both JSONL streams into unified `Event` type

**Note**: Instead of a unified Event type with stream-based merge-sort, implemented separate parsers (`hooks.rs`, `states.rs`, `transcript.rs`) with correlation at metrics aggregation level. Simpler and equally effective.

**Test 1**: Parse Claude hook event
```rust
#[test]
fn test_parse_claude_hook_event() {
    let json = r#"{"session_id":"...", "hook_event_name":"PostToolUse", ...}"#;
    let event = parse_event(json).unwrap();
    assert_eq!(event.source, EventSource::Claude);
}
```

**Test 2**: Parse Hegel state event
```rust
#[test]
fn test_parse_hegel_state_event() {
    let json = r#"{"timestamp":"...", "from_node":"spec", "to_node":"plan"}"#;
    let event = parse_event(json).unwrap();
    assert_eq!(event.source, EventSource::Hegel);
}
```

**Test 3**: Read both JSONL files with stream-based merge
```rust
#[test]
fn test_read_mixed_event_stream() {
    // Create test hooks.jsonl and states.jsonl
    // Parse both into unified event stream using merge-sort
    // Assert events are chronologically ordered
    // Assert constant memory usage (no full file load)
}
```

**Implementation**:
- Create `src/metrics/parser.rs`
- Implement `Event`, `EventSource`, `EventType` enums
- Implement `parse_event()` and `read_event_stream()`
- Stream-based merge: maintain two file readers, compare timestamps, emit in order
- Handle edge cases: exhausted streams, malformed lines (skip + log), missing timestamps

**Files to create**:
- `src/metrics/mod.rs`
- `src/metrics/parser.rs`

---

### Step 3: Metrics Aggregator ✅ COMPLETE

**Goal**: Extract structured metrics from event stream

**Test 1**: Count tool usage by type
```rust
#[test]
fn test_count_tool_usage() {
    let events = vec![
        hook_event("Bash", ...),
        hook_event("Read", ...),
        hook_event("Bash", ...),
    ];
    let metrics = aggregate_metrics(&events);
    assert_eq!(metrics.tool_counts.get("Bash"), Some(&2));
}
```

**Test 2**: Track file modifications
```rust
#[test]
fn test_track_file_edits() {
    let events = vec![
        edit_event("src/main.rs", ...),
        edit_event("src/main.rs", ...),
        edit_event("README.md", ...),
    ];
    let metrics = aggregate_metrics(&events);
    assert_eq!(metrics.file_edits.get("src/main.rs").count, 2);
}
```

**Test 3**: Correlate metrics per workflow phase
```rust
#[test]
fn test_phase_correlation() {
    let events = vec![
        state_transition("spec", "plan"),
        bash_event("cargo build"),
        bash_event("cargo test"),
        state_transition("plan", "code"),
    ];
    let phase_metrics = aggregate_by_phase(&events);
    assert_eq!(phase_metrics["plan"].bash_commands.len(), 2);
}
```

**Implementation**:
- Create `src/metrics/aggregator.rs`
- Implement `aggregate_metrics()`, `aggregate_by_phase()`
- Track: tool counts, file edits, bash commands, phase durations

**Files to create**:
- `src/metrics/aggregator.rs`

---

### Step 4: `hegel metrics` Command ✅ COMPLETE (as `hegel analyze`)

**Goal**: Display text-based metrics summary

**Note**: Implemented as `hegel analyze` instead of `hegel metrics`. Output includes all planned sections plus per-phase breakdown with token correlation.

**Test 1**: Display tool usage counts
```rust
#[test]
fn test_metrics_command_output() {
    // Create test events
    // Run metrics command
    // Assert output contains tool counts
}
```

**Implementation**:
- Add `metrics` subcommand to `src/main.rs`
- Implement `commands::show_metrics()`
- Format output with colored text

**Output format**:
```
=== Hegel Metrics ===

Event Summary:
  Total events: 1,234
  Claude hooks: 1,000
  Hegel transitions: 234

Tool Usage (last 24h):
  Bash:   150 (45%)
  Read:    80 (24%)
  Edit:    60 (18%)
  Write:   40 (12%)

File Modifications:
  src/main.rs:    12 edits
  src/engine.rs:   8 edits
  README.md:       3 edits

Bash Commands (top 5):
  cargo build:     25
  cargo test:      18
  git status:      10
```

**Files to modify**:
- `src/main.rs` - Add metrics subcommand
- `src/commands/mod.rs` - Add `show_metrics()`

---

## TUI Testing Strategy (Steps 5-7)

**Challenge**: TUI development requires visual validation, but we want to maximize autonomous progress.

**Staged Autonomy Approach**:

**Phase 1: Pure Logic (100% autonomous)**
- App state machine with full test coverage
- Event handlers (keyboard, file watching) - pure functions
- Data formatters (metrics → display strings)
- Mock rendering layer
- Goal: Every code path tested without visual validation

**Phase 2: Snapshot Rendering (95% autonomous)**
- Implement ratatui rendering to string buffers
- Snapshot tests for layout (golden file comparison)
- Use ratatui defaults for colors/themes
- Programmatic view exercising script
- Goal: Provably correct rendering, even if aesthetics need tuning

**Phase 3: First Human Run (5% human)**
- Single smoke test: launches without crash
- Basic sanity: arrow key navigation works
- Capture fundamental UX issues
- Goal: Minimize iteration count (one round ideal)

**Phase 4: Polish (human-guided)**
- Color/spacing/ergonomics tweaks from feedback
- Foundation already solid from Phases 1-2

**Testability Boundaries**:
- ✅ Fully testable: State transitions, layout math, keyboard handling, file watching triggers, render output (snapshots)
- ⚠️ Human required: Visual aesthetics, UX ergonomics, real-time feel, terminal compatibility edge cases

**Architecture for Testability**:
```
App State (pure) → Formatters → Rendering
     ↑                 ↑            ↑
   tested          tested    snapshot tested
```

---

### Step 5: TUI Framework Setup ✅ COMPLETE

**Goal**: Basic TUI scaffold with ratatui/crossterm

**Approach**: Phase 1 (Pure Logic) + Phase 2 (Snapshot Rendering) + Phase 4 (Colorful UI)

**Implementation**: `src/tui/app.rs` (360 lines), `src/tui/ui.rs` (420 lines), `src/tui/mod.rs` (66 lines)

**Dependencies** (add to Cargo.toml):
- `ratatui = "0.28"`
- `crossterm = "0.28"`
- `notify = "6.0"` (for file watching)

**App State Structure** (Pure Logic):
```rust
// src/tui/app.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,   // Session summary + token usage
    Phases,     // Per-phase breakdown
    Events,     // Event stream (last 100)
    Files,      // File modification frequency
}

pub struct AppState {
    pub metrics: UnifiedMetrics,
    pub selected_tab: Tab,
    pub scroll_offset: usize,
    pub should_quit: bool,
    pub needs_reload: bool,
}

impl AppState {
    pub fn new(metrics: UnifiedMetrics) -> Self { ... }
    pub fn handle_key(&mut self, code: KeyCode) { ... }
    pub fn next_tab(&mut self) { ... }
    pub fn prev_tab(&mut self) { ... }
    pub fn scroll_up(&mut self) { ... }
    pub fn scroll_down(&mut self) { ... }
    pub fn scroll_to_top(&mut self) { ... }
    pub fn scroll_to_bottom(&mut self) { ... }
    pub fn max_scroll(&self) -> usize { ... }  // Based on current tab content
}
```

**Phase 1 Tests - App State (Pure Logic)**:

**Test 1**: App state initialization
```rust
use crate::test_helpers::test_unified_metrics;

#[test]
fn test_app_state_init() {
    let metrics = test_unified_metrics();  // Helper from test_helpers
    let app = AppState::new(metrics);

    assert_eq!(app.selected_tab, Tab::Overview);
    assert_eq!(app.scroll_offset, 0);
    assert!(!app.should_quit);
    assert!(!app.needs_reload);
}
```

**Test 2**: Keyboard event handling - quit
```rust
#[test]
fn test_handle_key_quit() {
    let mut app = AppState::new(test_unified_metrics());
    app.handle_key(KeyCode::Char('q'));
    assert!(app.should_quit);
}
```

**Test 3**: Tab navigation (circular)
```rust
#[test]
fn test_tab_navigation() {
    let mut app = AppState::new(test_unified_metrics());

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
    assert_eq!(app.selected_tab, Tab::Overview);  // Wraps around
}

#[test]
fn test_back_tab_navigation() {
    let mut app = AppState::new(test_unified_metrics());

    // BackTab goes in reverse
    app.handle_key(KeyCode::BackTab);
    assert_eq!(app.selected_tab, Tab::Files);
}
```

**Test 4**: Scroll behavior with bounds
```rust
#[test]
fn test_scroll_down_within_bounds() {
    let mut app = AppState::new(test_unified_metrics());

    app.handle_key(KeyCode::Down);
    assert_eq!(app.scroll_offset, 1);

    app.handle_key(KeyCode::Down);
    assert_eq!(app.scroll_offset, 2);
}

#[test]
fn test_scroll_up_stops_at_zero() {
    let mut app = AppState::new(test_unified_metrics());

    // Can't scroll above 0
    app.handle_key(KeyCode::Up);
    assert_eq!(app.scroll_offset, 0);
}

#[test]
fn test_scroll_down_stops_at_max() {
    let mut app = AppState::new(test_unified_metrics());
    let max = app.max_scroll();

    // Scroll past max
    for _ in 0..max + 10 {
        app.handle_key(KeyCode::Down);
    }

    assert_eq!(app.scroll_offset, max);
}

#[test]
fn test_scroll_to_top_and_bottom() {
    let mut app = AppState::new(test_unified_metrics());

    app.handle_key(KeyCode::Char('g'));  // 'g' = top
    assert_eq!(app.scroll_offset, 0);

    app.handle_key(KeyCode::Char('G'));  // 'G' = bottom
    assert_eq!(app.scroll_offset, app.max_scroll());
}
```

**Test 5**: Tab-specific scroll resets
```rust
#[test]
fn test_scroll_resets_on_tab_change() {
    let mut app = AppState::new(test_unified_metrics());

    // Scroll down in Overview
    app.scroll_offset = 5;

    // Switch tab
    app.handle_key(KeyCode::Tab);

    // Scroll should reset
    assert_eq!(app.scroll_offset, 0);
}
```

**Test 6**: Keyboard shortcuts reference
```rust
#[test]
fn test_all_keyboard_shortcuts() {
    let mut app = AppState::new(test_unified_metrics());

    // Navigation
    assert_eq!(handles_key!(app, KeyCode::Tab), "next_tab");
    assert_eq!(handles_key!(app, KeyCode::BackTab), "prev_tab");
    assert_eq!(handles_key!(app, KeyCode::Up), "scroll_up");
    assert_eq!(handles_key!(app, KeyCode::Down), "scroll_down");
    assert_eq!(handles_key!(app, KeyCode::Char('k')), "scroll_up");  // vim
    assert_eq!(handles_key!(app, KeyCode::Char('j')), "scroll_down");  // vim
    assert_eq!(handles_key!(app, KeyCode::Char('g')), "scroll_to_top");
    assert_eq!(handles_key!(app, KeyCode::Char('G')), "scroll_to_bottom");

    // Actions
    assert_eq!(handles_key!(app, KeyCode::Char('r')), "reload");
    assert_eq!(handles_key!(app, KeyCode::Char('q')), "quit");
}
```

**Phase 2 Tests - Snapshot Rendering**:

**Test 7**: Layout dimensions (deterministic)
```rust
#[test]
fn test_layout_main_areas() {
    use ratatui::layout::Rect;

    let area = Rect::new(0, 0, 80, 24);
    let chunks = ui::main_layout(area);

    // [Header(3), Main(Min), Footer(1)]
    assert_eq!(chunks[0].height, 3);   // Header (title + tab bar)
    assert_eq!(chunks[1].height, 20);  // Main content (fills)
    assert_eq!(chunks[2].height, 1);   // Footer (keybindings)
}

#[test]
fn test_layout_responsive() {
    use ratatui::layout::Rect;

    // Small terminal
    let small = Rect::new(0, 0, 40, 10);
    let chunks = ui::main_layout(small);
    assert_eq!(chunks[0].height, 3);
    assert_eq!(chunks[2].height, 1);
    assert_eq!(chunks[1].height, 6);  // Main fills remaining

    // Large terminal
    let large = Rect::new(0, 0, 120, 40);
    let chunks = ui::main_layout(large);
    assert_eq!(chunks[0].height, 3);
    assert_eq!(chunks[2].height, 1);
    assert_eq!(chunks[1].height, 36);  // Main fills remaining
}
```

**Test 8**: Render overview tab (snapshot)
```rust
use crate::test_helpers::{test_unified_metrics, tui::{test_terminal, buffer_to_string, MEDIUM_TERM}};

#[test]
fn test_render_overview_tab() {
    let metrics = test_unified_metrics();
    let app = AppState::new(metrics);
    let (width, height) = MEDIUM_TERM;  // Standard 80x24 terminal
    let mut terminal = test_terminal(width, height);

    terminal.draw(|f| ui::draw(f, &app)).unwrap();

    let buffer = terminal.backend().buffer();
    let output = buffer_to_string(buffer);  // Helper converts buffer to string

    // Golden file comparison
    let expected = include_str!("../snapshots/overview_tab.txt");
    assert_eq!(output, expected);
}
```

**Test 9**: Render phases tab (snapshot)
```rust
use crate::test_helpers::tui::{test_terminal, buffer_to_string, LARGE_TERM};

#[test]
fn test_render_phases_tab() {
    let mut app = AppState::new(test_unified_metrics());
    app.selected_tab = Tab::Phases;

    let (width, height) = LARGE_TERM;  // 120x40 terminal for more content
    let mut terminal = test_terminal(width, height);

    terminal.draw(|f| ui::draw(f, &app)).unwrap();

    let buffer = terminal.backend().buffer();
    let output = buffer_to_string(buffer);

    let expected = include_str!("../snapshots/phases_tab.txt");
    assert_eq!(output, expected);
}
```

**Test 10**: Render with scroll offset
```rust
#[test]
fn test_render_with_scroll() {
    let mut app = AppState::new(test_unified_metrics());
    app.scroll_offset = 5;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| ui::draw(f, &app)).unwrap();

    // Verify scroll indicator appears in output
    let buffer = terminal.backend().buffer();
    let output = buffer_to_string(buffer);

    assert!(output.contains("↓"));  // Scroll down indicator
}
```

**Test 11**: Widget rendering isolation
```rust
#[test]
fn test_render_header_widget() {
    let app = AppState::new(test_unified_metrics());
    let header = ui::render_header(&app);

    // Test widget properties without full terminal
    assert_eq!(header.title().unwrap(), "Hegel Dialectic Dashboard");
}

#[test]
fn test_render_footer_widget() {
    let footer = ui::render_footer();

    // Verify keybindings shown
    let text = extract_text(&footer);
    assert!(text.contains("[q] Quit"));
    assert!(text.contains("[Tab] Next"));
    assert!(text.contains("[↑↓] Scroll"));
}
```

**Implementation Details**:

**Phase 1 - Pure Logic** (`src/tui/app.rs`):

```rust
// Full keyboard mapping
impl AppState {
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

            _ => {},
        }
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = match self.selected_tab {
            Tab::Overview => Tab::Phases,
            Tab::Phases => Tab::Events,
            Tab::Events => Tab::Files,
            Tab::Files => Tab::Overview,
        };
        self.scroll_offset = 0;  // Reset scroll on tab change
    }

    pub fn max_scroll(&self) -> usize {
        use crate::tui::utils::{max_scroll, build_timeline};

        // Calculate based on current tab content height
        match self.selected_tab {
            Tab::Overview => 0,  // Fits on one screen
            Tab::Phases => max_scroll(self.metrics.phase_metrics.len(), 10),
            Tab::Events => {
                let timeline = build_timeline(&self.metrics);
                max_scroll(timeline.len(), 20)
            },
            Tab::Files => {
                let file_count = self.metrics.hook_metrics
                    .file_modification_frequency()
                    .len();
                max_scroll(file_count, 15)
            },
        }
    }
}
```

**Phase 2 - Rendering** (`src/tui/ui.rs`):

```rust
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Borders, Paragraph, List, ListItem, Gauge},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
};

/// Main draw function (called from terminal.draw())
pub fn draw(frame: &mut Frame, app: &AppState) {
    let layout = main_layout(frame.area());
    let [header_area, main_area, footer_area] = layout;

    frame.render_widget(render_header(app), header_area);
    frame.render_widget(render_main(app), main_area);
    frame.render_widget(render_footer(), footer_area);
}

/// Main layout: [Header(3), Main(Min), Footer(1)]
pub fn main_layout(area: Rect) -> [Rect; 3] {
    Layout::vertical([
        Constraint::Length(3),  // Header
        Constraint::Min(0),     // Main (fills)
        Constraint::Length(1),  // Footer
    ]).areas(area)
}

/// Render header: title + tab bar
fn render_header<'a>(app: &AppState) -> impl Widget + 'a {
    let title = Line::from(vec![
        " Hegel Dialectic Dashboard ".bold().cyan(),
        format!("Session: {} ", app.metrics.session_id.as_deref().unwrap_or("N/A"))
            .dark_gray(),
    ]);

    let tabs = Line::from(vec![
        tab_label("Overview", app.selected_tab == Tab::Overview),
        " │ ".dark_gray(),
        tab_label("Phases", app.selected_tab == Tab::Phases),
        " │ ".dark_gray(),
        tab_label("Events", app.selected_tab == Tab::Events),
        " │ ".dark_gray(),
        tab_label("Files", app.selected_tab == Tab::Files),
    ]);

    Paragraph::new(vec![title, tabs])
        .block(Block::default().borders(Borders::BOTTOM))
}

fn tab_label<'a>(name: &'a str, selected: bool) -> Span<'a> {
    if selected {
        Span::styled(name, Style::default().cyan().bold())
    } else {
        Span::raw(name)
    }
}

/// Render main content area (tab-specific)
fn render_main<'a>(app: &AppState) -> Box<dyn Widget + 'a> {
    match app.selected_tab {
        Tab::Overview => Box::new(render_overview_tab(&app.metrics)),
        Tab::Phases => Box::new(render_phases_tab(&app.metrics, app.scroll_offset)),
        Tab::Events => Box::new(render_events_tab(&app.metrics, app.scroll_offset)),
        Tab::Files => Box::new(render_files_tab(&app.metrics, app.scroll_offset)),
    }
}

/// Render footer: keybindings
fn render_footer<'a>() -> impl Widget + 'a {
    let keybindings = Line::from(vec![
        "[q]".cyan().bold(),
        " Quit  ".into(),
        "[Tab]".cyan().bold(),
        " Next  ".into(),
        "[↑↓]".cyan().bold(),
        " Scroll  ".into(),
        "[r]".cyan().bold(),
        " Reload".into(),
    ]);

    Paragraph::new(keybindings)
}

/// Overview tab: session summary + token usage
fn render_overview_tab<'a>(metrics: &UnifiedMetrics) -> impl Widget + 'a {
    let lines = vec![
        Line::from("Token Usage".bold()),
        Line::from(format!("  Input:  {:>10}", metrics.token_metrics.total_input_tokens)),
        Line::from(format!("  Output: {:>10}", metrics.token_metrics.total_output_tokens)),
        Line::from(""),
        Line::from("Activity".bold()),
        Line::from(format!("  Events: {:>10}", metrics.hook_metrics.total_events)),
        Line::from(format!("  Files:  {:>10}", metrics.hook_metrics.file_modifications.len())),
    ];

    Paragraph::new(lines).block(Block::bordered())
}

/// Phases tab: per-phase breakdown with gauges
fn render_phases_tab<'a>(metrics: &UnifiedMetrics, scroll: usize) -> impl Widget + 'a {
    // TODO: Implement with Gauge widgets for token budgets
    Paragraph::new("Phases tab").block(Block::bordered())
}

// ... similar for render_events_tab and render_files_tab
```

**Event Loop** (`src/tui/mod.rs`):

```rust
use crossterm::event::{self, Event, KeyCode, KeyEventKind, poll};
use ratatui::DefaultTerminal;
use std::time::Duration;

pub fn run_tui(state_dir: &Path) -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run_event_loop(&mut terminal, state_dir);
    ratatui::restore();
    result
}

fn run_event_loop(terminal: &mut DefaultTerminal, state_dir: &Path) -> std::io::Result<()> {
    let metrics = parse_unified_metrics(state_dir)?;
    let mut app = AppState::new(metrics);

    loop {
        // Render
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Handle events (100ms timeout for file watching)
        if poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    app.handle_key(key.code);

                    if app.should_quit {
                        break;
                    }

                    if app.needs_reload {
                        app.metrics = parse_unified_metrics(state_dir)?;
                        app.needs_reload = false;
                    }
                },
                _ => {},
            }
        }
    }

    Ok(())
}
```

**Helper Infrastructure** (already implemented ✅):
- `src/test_helpers.rs` - Extended with TUI test helpers + metrics builder
  - `test_unified_metrics()` - Create test metrics with 3 phases, 10 bash, 5 files
  - `UnifiedMetricsBuilder` - Fluent builder for custom test scenarios
  - `tui::test_terminal(w, h)` - Create ratatui TestBackend for snapshot tests
  - `tui::buffer_to_string(buffer)` - Convert terminal buffer to golden file format
  - `tui::SMALL_TERM`, `MEDIUM_TERM`, `LARGE_TERM` - Standard terminal sizes (40x10, 80x24, 120x40)
- `src/tui/utils.rs` - Scroll + timeline utilities (✅ 100% tested, 126/126 tests passing)
  - `visible_window(items, offset, height)` - Calculate scrollable slice
  - `max_scroll(content, visible)` - Calculate max scroll offset
  - `scroll_indicators(offset, max)` - Get ↑↓· symbols for UI
  - `build_timeline(metrics)` - Merge hooks + states into chronological stream
  - `TimelineEvent`, `EventSource` - Unified event types
  - `format_timestamp(iso)` - Extract HH:MM:SS from ISO 8601

**Files to create**:
- `src/tui/app.rs` - AppState (pure logic, ~200 lines)
- `src/tui/ui.rs` - Rendering functions (~400 lines with helpers)
- `tests/snapshots/*.txt` - Golden files for snapshot tests

---

### Step 6: `hegel top` Command ✅ COMPLETE

**Goal**: Interactive dashboard showing live metrics

**Approach**: Integrated file watching + colorful rendering into Step 5

**Implementation**: File watching integrated into `AppState` via `notify` crate (see Step 5 files)

**Phase 1 Tests - Data Formatting (Pure Logic)**:

**Test 1**: Format metrics for display
```rust
#[test]
fn test_format_event_count() {
    let metrics = test_unified_metrics();
    let formatted = format_event_summary(&metrics);
    assert_eq!(formatted.total_events, 150);
    assert_eq!(formatted.claude_hooks, 120);
    assert_eq!(formatted.hegel_transitions, 30);
}
```

**Test 2**: Format phase gauges
```rust
#[test]
fn test_format_phase_gauge() {
    let phase = test_phase_metrics("plan", 800, Some(2000));
    let gauge = format_gauge(&phase);
    assert_eq!(gauge.percentage, 40);
    assert_eq!(gauge.label, "PLAN: 800/2000 tokens (40%)");
}
```

**Test 3**: File watching integration (mocked)
```rust
#[test]
fn test_file_watcher_triggers_reload() {
    let mut app = AppState::new(metrics);
    app.handle_file_event(FileEvent::Modified("hooks.jsonl"));
    assert!(app.needs_reload);
}
```

**Phase 2 Tests - Full Rendering**:

**Test 4**: Render complete dashboard
```rust
#[test]
fn test_render_dashboard_snapshot() {
    let app = create_populated_app();
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| ui::render_dashboard(&app, f)).unwrap();

    let buffer = terminal.backend().buffer().clone();
    assert_snapshot!(buffer.content());
}
```

**Test 5**: Event stream view
```rust
#[test]
fn test_render_event_stream() {
    let events = vec![
        event("Claude", "PostToolUse", "Bash"),
        event("Hegel", "Transition", "spec → plan"),
    ];
    let view = render_event_stream(&events, 10);
    assert_eq!(view.len(), 2);
    assert!(view[0].contains("[Claude]"));
}
```

**Implementation Details**:

**Phase 1 - File Watching Integration**:

Add file watching to `AppState` (extends Step 5 implementation):

```rust
use notify::{Watcher, RecursiveMode, Event as NotifyEvent};
use std::sync::mpsc::{channel, Receiver};

pub struct AppState {
    // Existing fields...
    pub metrics: UnifiedMetrics,
    pub selected_tab: Tab,
    pub scroll_offset: usize,
    pub should_quit: bool,
    pub needs_reload: bool,

    // File watching (new)
    state_dir: PathBuf,
    file_rx: Receiver<Result<NotifyEvent, notify::Error>>,
    _watcher: notify::RecommendedWatcher,  // Keep alive
}

impl AppState {
    pub fn new(state_dir: impl AsRef<Path>) -> Result<Self> {
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
                if matches!(event.kind, notify::EventKind::Modify(_)) {
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
}
```

Update event loop in `src/tui/mod.rs`:

```rust
fn run_event_loop(terminal: &mut DefaultTerminal, state_dir: &Path) -> std::io::Result<()> {
    let mut app = AppState::new(state_dir)?;

    loop {
        // Check for file updates BEFORE rendering
        let _ = app.check_for_updates();

        // Render
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Handle keyboard events (100ms timeout)
        if poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    app.handle_key(key.code);
                    if app.should_quit { break; }
                },
                _ => {},
            }
        }
    }

    Ok(())
}
```

**Phase 2 - Dashboard Widgets**:

Implement tab-specific rendering in `src/tui/ui.rs`:

```rust
/// Events tab: Recent hook events (scrollable)
fn render_events_tab<'a>(metrics: &UnifiedMetrics, scroll: usize) -> impl Widget + 'a {
    use crate::tui::utils::{build_timeline, EventSource};

    // Build unified timeline using helper (merges all sources)
    let timeline = build_timeline(metrics);

    // Take last 100, apply scroll using helper
    let visible = visible_window(&timeline, scroll, 20);  // 20 rows visible

    let items: Vec<ListItem> = visible
        .iter()
        .map(|event| {
            let source_style = if event.source == EventSource::Claude {
                Style::default().cyan()
            } else {
                Style::default().green()
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("[{}]", if event.source == EventSource::Claude { "Claude" } else { "Hegel" }),
                    source_style
                ),
                " ".into(),
                Span::raw(&event.event_type).dark_gray(),
                " ".into(),
                Span::raw(&event.detail),
            ]);

            ListItem::new(line)
        })
        .collect();

    List::new(items).block(Block::bordered().title("Event Stream (Last 100)"))
}

/// Phases tab: Per-phase metrics with gauges
fn render_phases_tab<'a>(metrics: &UnifiedMetrics, scroll: usize) -> impl Widget + 'a {
    let mut lines = vec![];

    for (i, phase) in metrics.phase_metrics.iter().enumerate().skip(scroll) {
        // Phase header
        lines.push(Line::from(vec![
            Span::styled(&phase.phase_name.to_uppercase(), Style::default().cyan().bold()),
            format!(" ({} active)", if phase.end_time.is_none() { "active" } else { "completed" })
                .into(),
        ]));

        // Duration
        if phase.duration_seconds > 0 {
            let mins = phase.duration_seconds / 60;
            let secs = phase.duration_seconds % 60;
            lines.push(Line::from(format!("  Duration: {}m {:02}s", mins, secs)));
        }

        // Token gauge (if budget defined - Phase 2 feature)
        let total_tokens = phase.token_metrics.total_input_tokens
            + phase.token_metrics.total_output_tokens;

        if total_tokens > 0 {
            lines.push(Line::from(format!("  Tokens: {}", total_tokens)));

            // TODO: Add Gauge widget for budget visualization
            // let budget = 2000;  // Get from config
            // let percent = (total_tokens * 100 / budget) as u16;
            // Gauge::default().percent(percent).label(...)
        }

        // Activity counts
        lines.push(Line::from(format!("  Bash: {}  Files: {}",
            phase.bash_commands.len(),
            phase.file_modifications.len()
        )));

        lines.push(Line::from(""));  // Spacing
    }

    Paragraph::new(lines).block(Block::bordered().title("Phase Metrics"))
}

/// Files tab: File modification frequency
fn render_files_tab<'a>(metrics: &UnifiedMetrics, scroll: usize) -> impl Widget + 'a {
    let mut freq = metrics.hook_metrics.file_modification_frequency();
    let mut sorted: Vec<_> = freq.drain().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));  // Most modified first

    let items: Vec<ListItem> = sorted
        .iter()
        .skip(scroll)
        .map(|(file, count)| {
            let line = Line::from(vec![
                Span::styled(format!("{:>3}x", count), Style::default().green().bold()),
                "  ".into(),
                Span::raw(file),
            ]);
            ListItem::new(line)
        })
        .collect();

    List::new(items).block(Block::bordered().title("File Modifications"))
}

struct EventDisplay {
    timestamp: Option<String>,
    source: &'static str,  // "Claude" or "Hegel"
    event_type: &'static str,
    detail: String,
}
```

**Files to extend**:
- `src/tui/app.rs` - Add file watching fields
- `src/tui/ui.rs` - Add tab rendering functions (~150 lines each)
- `src/main.rs` - Add `top` subcommand that calls `tui::run_tui()`

**Layout**:
```
┌─ Hegel Dialectic Dashboard ────────────────────────────────┐
│ Session: 3712d8c3  │  Mode: discovery  │  Phase: PLAN      │
├────────────────────────────────────────────────────────────┤
│ Event Stream (last 100)                                    │
│ ◉ 04:26:15 [Claude] PostToolUse: Bash (cargo build)       │
│ ◉ 04:26:10 [Hegel]  Transition: spec → plan               │
│ ◉ 04:26:05 [Claude] PreToolUse: Edit (src/main.rs)        │
├────────────────────────────────────────────────────────────┤
│ Phase Metrics                                              │
│ SPEC  ████████████ 1,200 tokens / 2,000 budget (60%)      │
│ PLAN  ██████────── 800 tokens / 2,000 budget (40%)        │
├────────────────────────────────────────────────────────────┤
│ Tool Usage                │  File Edits                    │
│ Bash:   45%  ████████     │  src/main.rs:    12            │
│ Read:   24%  ████         │  src/engine.rs:   8            │
│ Edit:   18%  ███          │  README.md:       3            │
│ Write:  12%  ██           │                                │
└────────────────────────────────────────────────────────────┘
[q] Quit  [r] Refresh  [↑↓] Scroll
```

**Files to modify**:
- `src/main.rs` - Add top subcommand
- `src/commands/mod.rs` - Add `show_top()`
- `src/tui/ui.rs` - Implement dashboard widgets

---

### Step 7: Historical Graph Reconstruction ✅ COMPLETE

**Goal**: Visualize recursive workflow DAG with energy expenditure

**Approach**: Phase 1 (DAG logic) + Phase 2 (ASCII rendering + DOT export)

**Implementation**: `src/metrics/graph.rs` (320 lines), integrated into `hegel analyze` command

**Phase 1 Tests - DAG Construction (Pure Logic)**:

**Test 1**: Build workflow DAG from state transitions
```rust
#[test]
fn test_build_workflow_dag() {
    let transitions = vec![
        state_transition("spec", "plan"),
        state_transition("plan", "code"),
        state_transition("code", "spec"),  // Recursion
    ];
    let dag = build_dag(&transitions);
    assert_eq!(dag.nodes.len(), 3);
    assert!(dag.has_edge("code", "spec"));
}
```

**Test 2**: Detect cycles in DAG
```rust
#[test]
fn test_detect_cycles() {
    let dag = create_dag_with_cycle();
    let cycles = dag.find_cycles();
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0], vec!["code", "review", "refactor", "code"]);
}
```

**Test 3**: Annotate DAG nodes with metrics
```rust
#[test]
fn test_dag_annotation() {
    let dag = build_dag_with_metrics(&transitions, &phase_metrics);
    assert_eq!(dag.nodes["plan"].token_usage, 1200);
    assert_eq!(dag.nodes["plan"].duration_seconds, 900);
    assert_eq!(dag.nodes["plan"].file_edits, 5);
}
```

**Phase 2 Tests - Rendering**:

**Test 4**: Render ASCII DAG
```rust
#[test]
fn test_render_ascii_dag() {
    let dag = create_test_dag();
    let ascii = render_ascii(&dag);
    assert!(ascii.contains("spec → plan"));
    assert!(ascii.contains("├─"));
}
```

**Test 5**: Export DOT format
```rust
#[test]
fn test_export_dot() {
    let dag = create_test_dag();
    let dot = export_dot(&dag);
    assert!(dot.contains("digraph"));
    assert!(dot.contains("spec -> plan"));
    assert!(dot.contains("label=\"1200 tokens\""));
}
```

**Implementation Details**:

**Phase 1 - DAG Construction**:

Create `src/metrics/graph.rs`:

```rust
use std::collections::{HashMap, HashSet};
use crate::metrics::{StateTransitionEvent, PhaseMetrics};

#[derive(Debug, Clone)]
pub struct DAGNode {
    pub phase_name: String,
    pub visits: usize,  // How many times this phase was visited
    pub total_tokens: u64,
    pub total_duration_secs: u64,
    pub file_modifications: usize,
    pub bash_commands: usize,
}

#[derive(Debug, Clone)]
pub struct DAGEdge {
    pub from: String,
    pub to: String,
    pub count: usize,  // How many times this transition occurred
}

#[derive(Debug)]
pub struct WorkflowDAG {
    pub nodes: HashMap<String, DAGNode>,
    pub edges: Vec<DAGEdge>,
}

impl WorkflowDAG {
    /// Build DAG from state transitions
    pub fn from_transitions(
        transitions: &[StateTransitionEvent],
        phase_metrics: &[PhaseMetrics],
    ) -> Self {
        let mut nodes = HashMap::new();
        let mut edges_map: HashMap<(String, String), usize> = HashMap::new();

        // Initialize nodes from phase metrics
        for phase in phase_metrics {
            let entry = nodes.entry(phase.phase_name.clone())
                .or_insert_with(|| DAGNode {
                    phase_name: phase.phase_name.clone(),
                    visits: 0,
                    total_tokens: 0,
                    total_duration_secs: 0,
                    file_modifications: 0,
                    bash_commands: 0,
                });

            entry.visits += 1;
            entry.total_tokens += phase.token_metrics.total_input_tokens
                + phase.token_metrics.total_output_tokens;
            entry.total_duration_secs += phase.duration_seconds;
            entry.file_modifications += phase.file_modifications.len();
            entry.bash_commands += phase.bash_commands.len();
        }

        // Build edges from transitions
        for i in 0..transitions.len().saturating_sub(1) {
            let from = &transitions[i].phase;
            let to = &transitions[i + 1].phase;

            *edges_map.entry((from.clone(), to.clone())).or_insert(0) += 1;
        }

        let edges: Vec<DAGEdge> = edges_map
            .into_iter()
            .map(|((from, to), count)| DAGEdge { from, to, count })
            .collect();

        Self { nodes, edges }
    }

    /// Detect cycles in the graph (indicates workflow loops)
    pub fn find_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in self.nodes.keys() {
            if !visited.contains(node) {
                self.dfs_cycle(node, &mut visited, &mut rec_stack, &mut vec![], &mut cycles);
            }
        }

        cycles
    }

    fn dfs_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        // Find outgoing edges
        for edge in &self.edges {
            if edge.from == node {
                if rec_stack.contains(&edge.to) {
                    // Found cycle
                    let cycle_start = path.iter().position(|n| n == &edge.to).unwrap();
                    cycles.push(path[cycle_start..].to_vec());
                } else if !visited.contains(&edge.to) {
                    self.dfs_cycle(&edge.to, visited, rec_stack, path, cycles);
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
    }

    /// Get longest path (indicates deepest recursion)
    pub fn longest_path(&self) -> Vec<String> {
        // Topological sort + dynamic programming
        // Implementation omitted for brevity
        vec![]
    }
}
```

**Phase 2 - Visualization**:

Add rendering functions (create `src/metrics/visualize.rs` if >200 lines):

```rust
impl WorkflowDAG {
    /// Render as ASCII art using box-drawing characters
    pub fn render_ascii(&self) -> String {
        let mut output = String::new();

        // Sort nodes by first appearance
        let mut sorted_nodes: Vec<_> = self.nodes.keys().collect();
        sorted_nodes.sort();

        for node_name in &sorted_nodes {
            let node = &self.nodes[node_name];

            // Node header
            output.push_str(&format!("┌─ {} ", node.phase_name.to_uppercase()));
            output.push_str(&"─".repeat(40));
            output.push_str("┐\n");

            // Node stats
            output.push_str(&format!("│ Visits: {}  ", node.visits));
            output.push_str(&format!("Tokens: {}  ", node.total_tokens));
            output.push_str(&format!("Duration: {}s\n", node.total_duration_secs));
            output.push_str(&format!("│ Bash: {}  Files: {}\n",
                node.bash_commands, node.file_modifications));
            output.push_str("└");
            output.push_str(&"─".repeat(57));
            output.push_str("┘\n");

            // Outgoing edges
            let outgoing: Vec<_> = self.edges
                .iter()
                .filter(|e| e.from == *node_name)
                .collect();

            for (i, edge) in outgoing.iter().enumerate() {
                let connector = if i == outgoing.len() - 1 { "└─" } else { "├─" };
                output.push_str(&format!("  {}→ {}  ({}x)\n",
                    connector, edge.to, edge.count));
            }

            output.push('\n');
        }

        output
    }

    /// Export as DOT format for Graphviz
    pub fn export_dot(&self) -> String {
        let mut dot = String::from("digraph workflow {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=rounded];\n\n");

        // Nodes
        for (name, node) in &self.nodes {
            let label = format!(
                "{}\\n{} tokens\\n{}s\\n{} visits",
                name,
                node.total_tokens,
                node.total_duration_secs,
                node.visits
            );
            dot.push_str(&format!("  \"{}\" [label=\"{}\"];\n", name, label));
        }

        dot.push('\n');

        // Edges
        for edge in &self.edges {
            let label = if edge.count > 1 {
                format!(" [label=\"{}x\"]", edge.count)
            } else {
                String::new()
            };
            dot.push_str(&format!("  \"{}\" -> \"{}\"{};\n",
                edge.from, edge.to, label));
        }

        dot.push_str("}\n");
        dot
    }
}
```

**Integration with `hegel analyze`**:

Add graph section to `src/commands/analyze.rs`:

```rust
pub fn analyze_metrics(storage: &FileStorage) -> Result<()> {
    // ... existing output ...

    // Workflow Graph
    if !metrics.state_transitions.is_empty() {
        println!("{}", "Workflow Graph".bold());

        let graph = WorkflowDAG::from_transitions(
            &metrics.state_transitions,
            &metrics.phase_metrics,
        );

        // ASCII visualization
        println!("{}", graph.render_ascii());

        // Cycle detection
        let cycles = graph.find_cycles();
        if !cycles.is_empty() {
            println!("{}", "⚠ Cycles Detected:".yellow().bold());
            for cycle in cycles {
                println!("  {}", cycle.join(" → "));
            }
        }

        // DOT export option
        println!("Run `hegel graph --dot > workflow.dot` to export for Graphviz");
        println!();
    }

    Ok(())
}
```

**Optional: Separate `hegel graph` command**:

```rust
// src/commands/graph.rs

pub fn show_graph(storage: &FileStorage, format: GraphFormat) -> Result<()> {
    let metrics = parse_unified_metrics(storage.state_dir())?;

    let graph = WorkflowDAG::from_transitions(
        &metrics.state_transitions,
        &metrics.phase_metrics,
    );

    match format {
        GraphFormat::Ascii => {
            println!("{}", graph.render_ascii());
        },
        GraphFormat::Dot => {
            println!("{}", graph.export_dot());
        },
    }

    Ok(())
}

pub enum GraphFormat {
    Ascii,
    Dot,
}
```

**Files to create/modify**:
- `src/metrics/graph.rs` - DAG construction and algorithms (~300 lines)
- `src/metrics/mod.rs` - Re-export `WorkflowDAG`
- `src/commands/analyze.rs` - Add graph visualization
- `src/commands/graph.rs` - Optional separate command (~50 lines)

---

## Unresolved Questions

~~**To resolve before/during Step 2 implementation:**~~

- [x] **Token metrics correlation strategy** ✅ RESOLVED
  - **Solution**: Correlate transcript token data to workflow phases by timestamp
  - Parse transcript file referenced in hooks.jsonl `transcript_path` field
  - Bucket assistant events by phase timestamps (from states.jsonl)
  - Aggregate tokens per phase in `build_phase_metrics()` function
  - See `src/metrics/mod.rs:169-214` for implementation

---

## Success Criteria (from ROADMAP)

- [x] Both event streams (hooks.jsonl, states.jsonl) feed unified metrics for rule evaluation ✅
- [x] Reports show phase metrics correlating epistemic state with energetic usage ✅ (`hegel analyze`)
- [x] `hegel top` displays correlated state and performance telemetry in real-time ✅
- [x] Graph reconstruction visualizes branching and synthesis across workflows ✅
- [x] Everything is beautifully colorful enough for any MUD enthusiast 🎨 ✅ (vibrant TUI with emojis!)


## Implementation Order (Red-Green-Refactor)

1. ✅ **State transition logging** (Step 1) - Foundational for correlation
2. ✅ **Unified event parser** (Step 2) - Required for all downstream metrics
3. ✅ **Metrics aggregator** (Step 3) - Core business logic
4. ✅ **Text metrics command** (Step 4) - Early validation of metrics logic
5. ✅ **TUI framework** (Step 5) - Phases 1-4 (Pure Logic + Snapshot Rendering + Colorful UI)
6. ✅ **Dashboard command** (Step 6) - Integrated with Step 5 (File watching + Live updates)
7. ✅ **Graph reconstruction** (Step 7) - Phases 1-2 (DAG logic + ASCII/DOT rendering)

**Phase 3 Gate (Human Testing Required)**:

*Acceptance criteria for autonomous → human handoff:*

1. **Test Coverage**:
   - All Phase 1 tests passing (state management, keyboard handling)
   - All Phase 2 tests passing (snapshot rendering)
   - Coverage ≥80% on all TUI code (`src/tui/`)

2. **Smoke Test Script**:
   - Create `scripts/smoke-test-tui.sh` that programmatically:
     - Launches `hegel top` with test data
     - Simulates keyboard sequences (q for quit, Tab for navigation, etc.)
     - Executes without panics or errors
     - Validates exit codes

3. **Snapshot Validation**:
   - Golden files committed for all major views
   - Regression tests prevent unintended layout changes
   - All snapshot tests passing

4. **Documentation**:
   - Keyboard shortcuts documented in help text
   - Known limitations documented (if any)
   - README includes `hegel top` usage example

*At this point, TUI is provably correct without human eyes. Phase 3 = aesthetic validation only.*

**Phase 4 Polish** (after Phase 3 validation):
- Color/spacing adjustments based on human feedback
- Keyboard shortcut refinements if ergonomics issues found
- Performance tuning if lag detected
- Terminal compatibility fixes if edge cases discovered

---

## Notes

- **Test coverage goal**: ≥80% for all new code ✅ Achieved: 94%
- **Pre-commit hook**: Auto-update coverage reports ✅
- **Color palette**: Use `colored` crate for terminal, ratatui themes for TUI (partial - colored implemented)
- **Performance**: Stream processing for large JSONL files (avoid loading all into memory) ⚠️ Files loaded fully, but correlation uses timestamp filtering
- **Error handling**: Gracefully handle malformed JSONL entries (log + skip) ✅
- **Test isolation**: Tests use `tempfile` crate with `--state-dir` flag for automatic cleanup ✅

---

## Implementation Summary

**✅ PHASE 1 COMPLETE - All 7 Steps Implemented**

**Completed Work (Steps 1-7)**:
- ✅ State transition logging to `.hegel/states.jsonl` with file locking
- ✅ Three separate parsers: `hooks.rs`, `states.rs`, `transcript.rs` (alternative to unified Event)
- ✅ Metrics aggregation with per-phase correlation
- ✅ `hegel analyze` command with comprehensive output + workflow graph visualization
- ✅ Interactive TUI dashboard (`hegel top`) with real-time file watching
- ✅ Colorful UI with emojis (⚡📊🤖⚙️📝) across all tabs
- ✅ Workflow DAG reconstruction with cycle detection + ASCII/DOT export
- ✅ Token correlation strategy resolved and implemented
- ✅ 90.88% test coverage (148 tests passing, 0 failed)
- ✅ Comprehensive tests for all metrics + TUI functionality

**Phase 1 Deliverables**:
- `hegel analyze` - Text-based metrics with graph visualization
- `hegel top` - Real-time interactive dashboard (file watching, 4 tabs, scrolling, vim bindings)
- `src/metrics/graph.rs` - DAG construction, cycle detection, ASCII/DOT rendering
- `src/tui/*` - Complete TUI implementation (app, ui, utils modules)

**Testing Strategy for Autonomous Development**:
- **Phase 1**: Pure logic, 100% testable (state machines, event handlers, data structures)
- **Phase 2**: Snapshot rendering, 95% testable (ratatui TestBackend, golden files)
- **Phase 3**: Human validation, 5% manual (aesthetic check, UX ergonomics)
- **Phase 4**: Polish based on Phase 3 feedback

**Handoff Criteria** (Phase 2 → Phase 3):
- All tests passing (≥80% coverage on TUI code)
- Smoke test script executes without panics
- Golden file snapshots committed
- Documentation complete (keyboard shortcuts, usage examples)

**Key Files Implemented**:

**Metrics & Parsing**:
- `src/metrics/mod.rs` - Unified metrics aggregation and per-phase correlation
- `src/metrics/hooks.rs` - Hook event parsing (silent error handling)
- `src/metrics/states.rs` - State transition parsing
- `src/metrics/transcript.rs` - Transcript token parsing
- `src/metrics/graph.rs` - Workflow DAG construction, cycle detection, ASCII/DOT rendering

**Commands**:
- `src/commands/analyze.rs` - Analysis command with colored output + graph visualization
- `src/main.rs` - CLI with `analyze` and `top` subcommands

**TUI Implementation**:
- `src/tui/app.rs` - AppState with file watching, keyboard handling, scroll management
- `src/tui/ui.rs` - Colorful rendering for all 4 tabs (Overview, Phases, Events, Files)
- `src/tui/utils.rs` - Scroll utilities, timeline builder, event merging
- `src/tui/mod.rs` - Event loop with 100ms polling

**Test Infrastructure**:
- `src/test_helpers.rs` - JSONL file creators, TUI test helpers, metrics builders
