# Dependency Review: TUI Stack

**Review Date**: 2025-10-10
**Target**: Phase 1-2 implementation of Steps 5-7 in PLAN.md
**Scope**: ratatui 0.29, crossterm 0.28, notify 6.0

---

## Executive Summary

All three dependencies are production-ready with the patterns we need:
- ✅ **ratatui**: Full TUI framework with TestBackend for snapshot testing
- ✅ **crossterm**: Event handling (keyboard/mouse) with blocking and non-blocking modes
- ✅ **notify**: File watching with debouncing support

Critical insight: ratatui's TestBackend enables 95% autonomous testing via golden file snapshots.

---

## ratatui 0.29 Patterns

### Core Architecture

**Immediate Mode Rendering**:
- Each frame renders complete UI from scratch (no retained state)
- Terminal performs diff and only redraws changed cells
- Pattern: `terminal.draw(|frame| { ... })` in loop

**Key Types**:
```rust
Terminal<B>              // Main entry point, generic over Backend
Frame                    // Rendering context (passed to draw closure)
TestBackend              // For snapshot testing (renders to in-memory buffer)
DefaultTerminal          // Type alias for Terminal<CrosstermBackend>
```

### Initialization Pattern (Modern - 0.29)

```rust
use ratatui;

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();  // Enters alternate screen + raw mode
    let result = run(&mut terminal);
    ratatui::restore();  // Restores terminal state
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(draw)?;
        if handle_events()? { break Ok(()); }
    }
}
```

**What `init()` does**:
- Enters alternate screen (main screen preserved)
- Enables raw mode (no line buffering, no echo)
- Sets up panic hook (auto-restores on panic)
- Returns `DefaultTerminal` (uses Crossterm backend)

### Testing Pattern (Phase 2)

```rust
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_render_snapshot() {
    let backend = TestBackend::new(80, 24);  // 80 cols, 24 rows
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| {
        // Your render logic here
        frame.render_widget(my_widget, frame.area());
    }).unwrap();

    // Get buffer contents as string
    let buffer = terminal.backend().buffer();

    // Snapshot comparison (manual or with insta crate)
    assert_eq!(buffer.content(), expected_output);
}
```

**Key TestBackend methods**:
- `new(width, height)` - Create backend with fixed size
- `buffer()` - Get current buffer (for assertions)
- `resize(width, height)` - Test responsive layouts

### Layout System

**Modern pattern (0.29)** - Uses array destructuring:

```rust
use ratatui::layout::{Constraint, Layout, Direction};
use Constraint::*;

fn draw(frame: &mut Frame) {
    // Vertical split
    let vertical = Layout::vertical([Length(3), Min(0), Length(1)]);
    let [header, main, footer] = vertical.areas(frame.area());

    // Horizontal split within main
    let horizontal = Layout::horizontal([Fill(1), Fill(1)]);
    let [left, right] = horizontal.areas(main);

    // Render widgets to areas
    frame.render_widget(Header::new(), header);
    frame.render_widget(Content::new(), left);
    frame.render_widget(Sidebar::new(), right);
    frame.render_widget(StatusBar::new(), footer);
}
```

**Constraint types**:
- `Length(n)` - Fixed size (n cells)
- `Min(n)` - Minimum size (expands to fill)
- `Max(n)` - Maximum size (shrinks if needed)
- `Percentage(n)` - Percentage of available space (0-100)
- `Fill(n)` - Proportional fill (like CSS flex)
- `Ratio(num, denom)` - Fractional ratio

### Widgets

**Stateless widgets** (implement `Widget` trait):
```rust
use ratatui::widgets::{Block, Paragraph, List, Gauge};

// Block - borders and titles
let block = Block::bordered()
    .title("Title")
    .border_style(Style::default().fg(Color::Cyan));

// Paragraph - multiline text
let paragraph = Paragraph::new("Hello\nWorld")
    .block(Block::bordered());

// List - selectable items
let items = vec!["Item 1", "Item 2"];
let list = List::new(items)
    .block(Block::bordered().title("List"));

// Gauge - progress bar
let gauge = Gauge::default()
    .block(Block::bordered().title("Progress"))
    .gauge_style(Style::default().fg(Color::Green))
    .percent(60);
```

**Stateful widgets** (implement `StatefulWidget` trait):
```rust
use ratatui::widgets::{List, ListState};

// Widget maintains state separately
let mut state = ListState::default();
state.select(Some(0));

// Render with state
frame.render_stateful_widget(list, area, &mut state);
```

**Custom widgets**:
```rust
use ratatui::widgets::Widget;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

struct MyWidget {
    data: String,
}

impl Widget for MyWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Manually draw to buffer cells
        buf.set_string(area.x, area.y, &self.data, Style::default());
    }
}
```

### Styling

**Style builder pattern**:
```rust
use ratatui::style::{Color, Modifier, Style};

let style = Style::default()
    .fg(Color::Yellow)
    .bg(Color::Blue)
    .add_modifier(Modifier::BOLD | Modifier::ITALIC);
```

**Stylize trait** (short-hand):
```rust
use ratatui::style::Stylize;

let text = "Hello".cyan().on_white().bold();
let paragraph = Paragraph::new("World").red().italic();
```

**Colors**:
- Named: `Color::Black`, `Color::Red`, etc.
- RGB: `Color::Rgb(255, 128, 0)`
- Indexed: `Color::Indexed(42)` (256-color palette)

### Frame Methods

```rust
fn draw(frame: &mut Frame) {
    frame.area()                          // Get full terminal area (Rect)
    frame.render_widget(widget, area)     // Render stateless widget
    frame.render_stateful_widget(widget, area, &mut state)  // Render stateful
}
```

### Critical Gotchas

1. **Don't store Terminal in struct** - Pass by reference in event loop
2. **Draw entire frame each time** - No partial updates
3. **Widgets are consumed on render** - Clone or rebuild if needed
4. **TestBackend size is fixed** - Create new backend to test resize

---

## crossterm 0.28 Patterns

### Event Handling

**Blocking mode** (default):
```rust
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

fn handle_events() -> std::io::Result<bool> {
    match event::read()? {  // Blocks until event
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            match key.code {
                KeyCode::Char('q') => return Ok(true),  // Quit
                KeyCode::Up => { /* handle up */ },
                KeyCode::Down => { /* handle down */ },
                KeyCode::Tab => { /* handle tab */ },
                _ => {},
            }
        },
        Event::Resize(width, height) => {
            // Handle terminal resize
        },
        _ => {},
    }
    Ok(false)
}
```

**Non-blocking mode** (with timeout):
```rust
use std::time::Duration;
use crossterm::event::{self, poll};

fn handle_events() -> std::io::Result<bool> {
    if poll(Duration::from_millis(100))? {  // Poll with timeout
        match event::read()? {
            Event::Key(key) => { /* ... */ },
            _ => {},
        }
    }
    Ok(false)
}
```

**Key event types**:
```rust
Event::Key(KeyEvent {
    code: KeyCode,       // Which key: Char('a'), Up, Down, Enter, etc.
    modifiers: KeyModifiers,  // Ctrl, Alt, Shift
    kind: KeyEventKind,  // Press, Release, Repeat
    state: KeyEventState,
})
```

**Pattern for our use case**:
```rust
// Non-blocking for file watching responsiveness
use crossterm::event::{poll, read, Event, KeyCode};
use std::time::Duration;

loop {
    // Poll for events with 100ms timeout (allows file watching to trigger)
    if poll(Duration::from_millis(100))? {
        match read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Tab => app.next_tab(),
                        KeyCode::Up => app.scroll_up(),
                        KeyCode::Down => app.scroll_down(),
                        KeyCode::Char('r') => app.refresh(),
                        _ => {},
                    }
                }
            },
            _ => {},
        }
    }

    // Check file watcher events here
    // ...

    terminal.draw(|f| draw(f, &app))?;
}
```

### Terminal Control

**Manual setup** (if not using `ratatui::init()`):
```rust
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{Hide, Show},
};
use std::io::stdout;

// Enter alternate screen and raw mode
enable_raw_mode()?;
execute!(stdout(), EnterAlternateScreen, Hide)?;

// ... run app ...

// Restore
execute!(stdout(), LeaveAlternateScreen, Show)?;
disable_raw_mode()?;
```

### Testing Event Handling

**Mock events in tests**:
```rust
#[test]
fn test_handle_quit_key() {
    let mut app = AppState::new(test_metrics());

    // Simulate key press
    app.handle_key(KeyCode::Char('q'));

    assert!(app.should_quit);
}
```

---

## notify 6.0 Patterns

### File Watching

**Basic watcher setup**:
```rust
use notify::{Watcher, RecursiveMode, Result};
use std::path::Path;
use std::sync::mpsc::channel;

fn watch() -> Result<()> {
    let (tx, rx) = channel();

    let mut watcher = notify::recommended_watcher(move |res| {
        tx.send(res).unwrap();
    })?;

    // Watch .hegel directory
    watcher.watch(Path::new(".hegel"), RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                // Handle file event
                println!("Event: {:?}", event);
            },
            Err(e) => println!("Error: {:?}", e),
        }
    }

    Ok(())
}
```

**Event types**:
```rust
use notify::EventKind;

match event.kind {
    EventKind::Create(_) => { /* File created */ },
    EventKind::Modify(_) => { /* File modified */ },
    EventKind::Remove(_) => { /* File removed */ },
    _ => {},
}
```

**Pattern for TUI integration**:
```rust
use notify::{Watcher, RecursiveMode, Event as NotifyEvent};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

struct App {
    metrics: UnifiedMetrics,
    watcher: notify::RecommendedWatcher,
    rx: Receiver<Result<NotifyEvent, notify::Error>>,
}

impl App {
    fn new(state_dir: &Path) -> Result<Self> {
        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;

        watcher.watch(state_dir, RecursiveMode::NonRecursive)?;

        Ok(Self {
            metrics: parse_unified_metrics(state_dir)?,
            watcher,
            rx,
        })
    }

    fn check_for_updates(&mut self) -> bool {
        let mut updated = false;

        // Drain all pending events (non-blocking)
        while let Ok(res) = self.rx.try_recv() {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_)) {
                    updated = true;
                }
            }
        }

        if updated {
            // Reload metrics
            self.metrics = parse_unified_metrics(".hegel").unwrap();
        }

        updated
    }
}

// In main loop:
loop {
    if poll(Duration::from_millis(100))? {
        // Handle keyboard events
    }

    if app.check_for_updates() {
        // Force redraw on file change
    }

    terminal.draw(|f| draw(f, &app))?;
}
```

### Debouncing

**Problem**: File writes trigger multiple events (create → modify → modify)

**Solution**: Simple time-based debounce:
```rust
use std::time::{Duration, Instant};

struct DebouncedWatcher {
    last_update: Instant,
    debounce_duration: Duration,
}

impl DebouncedWatcher {
    fn should_reload(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_update) > self.debounce_duration {
            self.last_update = now;
            true
        } else {
            false
        }
    }
}
```

### Testing File Watching

**Mock file events in tests**:
```rust
#[test]
fn test_file_watcher_triggers_reload() {
    let mut app = AppState::new(test_metrics());

    // Simulate file event
    app.handle_file_event(FileEvent::Modified(".hegel/hooks.jsonl"));

    assert!(app.needs_reload);
}
```

---

## Integration Pattern: Complete TUI Event Loop

**Combining all three dependencies**:

```rust
use ratatui::{DefaultTerminal, Frame};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, poll};
use notify::{Watcher, RecursiveMode};
use std::time::Duration;
use std::sync::mpsc::channel;

struct App {
    metrics: UnifiedMetrics,
    selected_tab: Tab,
    scroll_offset: usize,
    should_quit: bool,
    file_rx: Receiver<Result<notify::Event, notify::Error>>,
    _watcher: notify::RecommendedWatcher,  // Keep alive
}

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let mut app = App::new(".hegel")?;

    loop {
        // Handle keyboard events (100ms timeout)
        if poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if handle_key(&mut app, key.code) {
                        break;  // Quit
                    }
                },
                Event::Resize(_, _) => {
                    // Terminal resized, will redraw automatically
                },
                _ => {},
            }
        }

        // Check for file updates
        app.check_file_updates();

        // Render
        terminal.draw(|frame| draw(frame, &app))?;
    }

    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode) -> bool {
    match code {
        KeyCode::Char('q') => return true,  // Quit
        KeyCode::Tab => app.next_tab(),
        KeyCode::Up => app.scroll_up(),
        KeyCode::Down => app.scroll_down(),
        KeyCode::Char('r') => app.reload_metrics(),
        _ => {},
    }
    false
}

fn draw(frame: &mut Frame, app: &App) {
    // Layout and widget rendering
    let layout = Layout::vertical([Length(3), Min(0), Length(1)]);
    let [header, main, footer] = layout.areas(frame.area());

    frame.render_widget(render_header(app), header);
    frame.render_widget(render_main(app), main);
    frame.render_widget(render_footer(), footer);
}
```

---

## Testing Strategy Summary

### Phase 1: Pure Logic (100% testable)

```rust
#[test]
fn test_app_state_transitions() {
    let mut app = AppState::new(test_metrics());

    // Keyboard handling
    app.handle_key(KeyCode::Char('q'));
    assert!(app.should_quit);

    app.handle_key(KeyCode::Tab);
    assert_eq!(app.selected_tab, Tab::Phases);

    // Scroll behavior
    app.handle_key(KeyCode::Down);
    assert_eq!(app.scroll_offset, 1);

    // File events
    app.handle_file_event(FileEvent::Modified("hooks.jsonl"));
    assert!(app.needs_reload);
}
```

### Phase 2: Snapshot Rendering (95% testable)

```rust
#[test]
fn test_render_dashboard() {
    let app = AppState::new(test_metrics());
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| draw(f, &app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Golden file comparison (manual or with `insta` crate)
    let expected = include_str!("../snapshots/dashboard.txt");
    assert_eq!(buffer.content(), expected);
}
```

---

## Dependency Decision Matrix

| Requirement | ratatui | crossterm | notify | Notes |
|-------------|---------|-----------|--------|-------|
| TUI rendering | ✅ | ❌ | ❌ | Primary |
| Keyboard input | ✅* | ✅ | ❌ | *via crossterm re-export |
| File watching | ❌ | ❌ | ✅ | Primary |
| Snapshot testing | ✅ | ❌ | ❌ | TestBackend |
| Cross-platform | ✅ | ✅ | ✅ | All support macOS/Linux/Windows |

---

## Anti-patterns to Avoid

1. **Storing Terminal in struct** → Pass by reference
2. **Partial frame updates** → Always render complete frame
3. **Blocking file I/O in draw()** → Pre-load data in event loop
4. **Ignoring KeyEventKind** → Filter to KeyEventKind::Press
5. **No debouncing on file events** → Multiple rapid events per write

---

## Files to Create (from PLAN.md)

**Phase 1 - Pure Logic**:
- `src/tui/app.rs` - AppState with pure methods (no rendering)
- `src/tui/formatters.rs` - Metrics → display structs

**Phase 2 - Rendering**:
- `src/tui/ui.rs` - Widget rendering functions
- `src/tui/mod.rs` - Event loop integration

**Testing**:
- `tests/tui/` - Snapshot test fixtures
- `scripts/smoke-test-tui.sh` - Automated smoke test

---

## Next Steps

1. Implement Phase 1 (pure logic) following patterns above
2. Add Phase 2 (rendering) with TestBackend snapshots
3. Create smoke test script
4. Handoff to human for Phase 3 aesthetic validation

**Coverage target**: ≥80% on all TUI code
**Success criteria**: All tests pass, no panics, snapshots committed
