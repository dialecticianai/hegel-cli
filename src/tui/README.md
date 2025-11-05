# src/tui/

Layer 6: Terminal User Interface. Interactive dashboard for real-time workflow metrics visualization (`hegel top` command).

## Purpose

Provides a live, interactive TUI dashboard that displays workflow metrics with automatic file watching. Built with ratatui for cross-platform terminal rendering. Features tabbed navigation, scrolling, and real-time updates when JSONL files change.

## Structure

```
tui/
├── mod.rs               Event loop (keyboard polling, file watching integration, terminal setup/restore)
├── app.rs               AppState (file watching via notify, keyboard handling, scroll management, tab navigation)
├── ui.rs                Main rendering orchestrator (header, footer, tab routing)
├── utils.rs             Scroll utilities (visible_window, max_scroll, scroll_indicators), timeline builder (merge hooks+states)
│
└── tabs/                Tab rendering modules (separation of concerns)
    ├── mod.rs           Tab rendering exports
    ├── overview.rs      Overview tab (session summary, token usage, activity metrics)
    ├── phases.rs        Phases tab (per-phase breakdown with duration, tokens, activity)
    ├── events.rs        Events tab (unified timeline of hooks and states, scrollable)
    └── files.rs         Files tab (file modification frequency, color-coded by intensity)
```

## Key Features

**Live Updates**: Uses `notify` crate for non-blocking real-time updates (100ms poll, auto-reload on modify events)
**Tabbed Navigation**: 4 interactive tabs (Overview, Phases, Events, Files)
**Scrolling**: Arrow keys, vim bindings (j/k), jump to top/bottom (g/G)
**Keyboard Shortcuts**: q (quit), Tab/BackTab (switch tabs), r (manual reload)
**Colorful UI**: Emoji icons, syntax highlighting, status indicators via theme.rs
