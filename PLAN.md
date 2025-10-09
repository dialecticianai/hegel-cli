# Phase 1 Implementation Plan: Metrics Collection & Analysis

**Goal**: Parse hook data and build metrics to feed cycle detection and budget enforcement

**Context**: Hook events are captured in `.hegel/hooks.jsonl`. This phase processes that data into actionable metrics and visualizes them in a TUI dashboard.

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

### Step 0: Prerequisites

**Goal**: Infrastructure needed for metrics collection

**Prerequisite 1**: Add `--state-dir` flag and `HEGEL_STATE_DIR` env var
```rust
#[test]
fn test_state_dir_flag_overrides_default() {
    // Run hegel command with --state-dir /tmp/test
    // Assert state written to /tmp/test/.hegel/state.json
}

#[test]
fn test_state_dir_env_var() {
    // Set HEGEL_STATE_DIR=/tmp/test
    // Run hegel command
    // Assert state written to /tmp/test/state.json
}

#[test]
fn test_state_dir_precedence() {
    // Set HEGEL_STATE_DIR=/tmp/env
    // Run with --state-dir /tmp/flag
    // Assert flag takes precedence
}
```

**Prerequisite 2**: Inject timestamp in `hegel hook` command
```rust
#[test]
fn test_hook_injects_timestamp() {
    // Pipe hook event JSON to `hegel hook PostToolUse`
    // Read hooks.jsonl
    // Assert written event has timestamp field
}
```

**Prerequisite 3**: Generate workflow_id in `hegel start`
```rust
#[test]
fn test_start_generates_workflow_id() {
    // Run `hegel start discovery`
    // Load state.json
    // Assert workflow_id is ISO timestamp
}
```

**Implementation**:
- Modify `src/main.rs` to accept global `--state-dir` flag
- Modify `FileStorage::new()` to check `HEGEL_STATE_DIR` env var if no flag provided
- Update `hegel hook` to inject `timestamp` field before appending to hooks.jsonl
- Update `hegel start` to generate workflow_id (ISO timestamp) and persist in state.json

**Files to modify**:
- `src/main.rs` - Add global `--state-dir` flag
- `src/storage/mod.rs` - Check env var in constructor
- `src/commands/mod.rs` - Update hook and start commands

---

### Step 1: State Transition Logging

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

### Step 2: Unified Event Parser

**Goal**: Read and normalize both JSONL streams into unified `Event` type

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

### Step 3: Metrics Aggregator

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

### Step 4: `hegel metrics` Command

**Goal**: Display text-based metrics summary

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

### Step 5: TUI Framework Setup

**Goal**: Basic TUI scaffold with ratatui/crossterm

**Dependencies** (add to Cargo.toml):
- `ratatui = "0.28"`
- `crossterm = "0.28"`
- `notify = "6.0"` (for file watching)
- `colored = "2.0"` (for terminal output)

**Test 1**: TUI renders without error
```rust
#[test]
fn test_tui_renders() {
    // Initialize TUI
    // Render frame
    // Assert no panics
}
```

**Test 2**: TUI accepts input and quits
```rust
#[test]
fn test_tui_quit_on_q() {
    // Simulate 'q' key press
    // Assert TUI exits cleanly
}
```

**Implementation**:
- Create `src/tui/mod.rs`
- Implement basic event loop (render + input)
- Add 'q' to quit, 'r' to refresh

**Files to create**:
- `src/tui/mod.rs`
- `src/tui/app.rs` - Application state
- `src/tui/ui.rs` - Rendering logic

---

### Step 6: `hegel top` Command

**Goal**: Interactive dashboard showing live metrics

**Test 1**: Display real-time event counts
```rust
#[test]
fn test_top_displays_event_counts() {
    // Create test events
    // Render top dashboard
    // Assert event counts visible
}
```

**Test 2**: Display per-phase gauges
```rust
#[test]
fn test_top_shows_phase_gauges() {
    // Create test phase metrics
    // Render top dashboard
    // Assert gauges display token/time budgets
}
```

**Implementation**:
- Add `top` subcommand to `src/main.rs`
- Implement dashboard with:
  - Event counts (live updating via `notify` file watching)
  - Tool usage breakdown
  - Phase gauges (token/time budgets)
  - Recent activity log
  - Color-coded Hegel vs Claude events
- File watching: use `notify` crate to watch `.hegel/hooks.jsonl` and `.hegel/states.jsonl` for modifications
- Near-instant updates on new events (event-driven, not polling)

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

### Step 7: Historical Graph Reconstruction

**Goal**: Visualize recursive workflow DAG with energy expenditure

**Test 1**: Build workflow DAG from events
```rust
#[test]
fn test_build_workflow_dag() {
    let events = vec![
        state_transition("spec", "plan"),
        state_transition("plan", "code"),
        state_transition("code", "spec"),  // Recursion
    ];
    let dag = build_dag(&events);
    assert_eq!(dag.nodes.len(), 3);
    assert!(dag.has_edge("code", "spec"));
}
```

**Test 2**: Annotate DAG with token usage
```rust
#[test]
fn test_dag_with_energy() {
    let events = vec![...];  // Include token metrics
    let dag = build_dag_with_metrics(&events);
    assert_eq!(dag.nodes["plan"].token_usage, 1200);
}
```

**Implementation**:
- Create `src/metrics/graph.rs`
- Implement DAG construction
- Annotate nodes with: token usage, duration, file edits
- Render ASCII graph visualization
- Support DOT format export for external visualization tools

**Files to create**:
- `src/metrics/graph.rs`

---

## Unresolved Questions

**To resolve before/during Step 2 implementation:**

- [ ] **Token metrics correlation strategy** - How to attach transcript `message.usage` data to events?
  - **Leading approach**: Attach to `Stop` hook events (when assistant response completes)
  - **Unknowns**:
    - Verify actual ratio of message.usage entries to Stop hooks (script reported 1,430 vs 8 - seems high)
    - Should we aggregate ALL usage between Stop hooks or just most recent?
    - How to handle missing Stop hooks (if hooks weren't enabled for part of session)?
    - Transcript I/O strategy: open on-demand per Stop, or cache file handle?
    - Timestamp alignment tolerance when looking backward from Stop hook
  - **Decision needed**: Investigate actual transcript structure before implementing

---

## Success Criteria (from ROADMAP)

- [x] Both event streams (hooks.jsonl, states.jsonl) feed unified metrics for rule evaluation
- [ ] `hegel top` displays correlated state and performance telemetry in real-time
- [ ] Reports show phase metrics correlating epistemic state with energetic usage
- [ ] Graph reconstruction visualizes branching and synthesis across workflows
- [ ] Everything is beautifully colorful enough for any MUD enthusiast 🎨


## Implementation Order (Red-Green-Refactor)

0. **Prerequisites** (Step 0) - Infrastructure setup (--state-dir flag, timestamp injection, workflow_id generation)
1. **State transition logging** (Step 1) - Foundational for correlation
2. **Unified event parser** (Step 2) - Required for all downstream metrics
3. **Metrics aggregator** (Step 3) - Core business logic
4. **Text metrics command** (Step 4) - Early validation of metrics logic
5. **TUI framework** (Step 5) - Scaffold for dashboard (add deps: ratatui, crossterm, notify, colored)
6. **Dashboard command** (Step 6) - Primary deliverable
7. **Graph reconstruction** (Step 7) - Advanced visualization

---

## Notes

- **Test coverage goal**: ≥80% for all new code
- **Pre-commit hook**: Auto-update coverage reports
- **Color palette**: Use `colored` crate for terminal, ratatui themes for TUI
- **Performance**: Stream processing for large JSONL files (avoid loading all into memory)
- **Error handling**: Gracefully handle malformed JSONL entries (log + skip)
- **Test isolation**: Add `--state-dir <PATH>` flag (or `HEGEL_STATE_DIR` env var) to override default `.hegel` directory
  - Precedence: CLI flag > env var > default (`.hegel`)
  - Tests use `tempfile` crate with `--state-dir` for automatic cleanup
  - Users can manage multiple workflow contexts with different state directories
