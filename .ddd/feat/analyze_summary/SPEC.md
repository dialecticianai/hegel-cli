# Analyze Summary Default Specification

Make `hegel analyze` show a concise cross-section summary by default instead of 3000+ lines of detailed output.

---

## Overview

**What it does**: Changes the default output of `hegel analyze` from showing all detailed sections to showing a brief cross-section summary, while preserving progressive disclosure via flags.

**Key principles**:
- Default output should be scannable (20-30 lines)
- Progressive disclosure: users opt-in to detail via flags
- All existing detail levels remain accessible
- Backward compatible via explicit flags

**Scope**: CLI flag additions and output rendering logic changes only. No changes to metrics collection or storage.

**Integration context**: Modifies `src/commands/analyze/mod.rs` and `src/main.rs` CLI argument definitions. Adds new rendering function to `src/analyze/sections.rs`.

---

## Data Model

**MODIFIED**: `src/commands/analyze/mod.rs::AnalyzeOptions`
```rust
pub struct AnalyzeOptions {
    pub export_dot: bool,
    pub fix_archives: bool,
    pub dry_run: bool,
    pub json: bool,
    pub brief: bool,           // NEW: cross-section summary
    pub activity: bool,         // RENAMED from: summary
    pub workflow_transitions: bool,
    pub phase_breakdown: bool,
    pub workflow_graph: bool,
    pub full: bool,            // NEW: show all sections
}
```

**MODIFIED**: `src/main.rs::Commands::Analyze`
```rust
Analyze {
    // ... existing flags ...

    /// Display brief cross-section summary (default if no section flags provided)
    #[arg(long)]
    brief: bool,

    /// Display activity section (session, tokens, bash commands, files)
    #[arg(long)]
    activity: bool,  // RENAMED from: summary

    /// Display workflow transitions section
    #[arg(long)]
    workflow_transitions: bool,

    /// Display phase breakdown section
    #[arg(long)]
    phase_breakdown: bool,

    /// Display workflow graph section
    #[arg(long)]
    workflow_graph: bool,

    /// Display all sections in full detail
    #[arg(long)]
    full: bool,
}
```

**NEW**: `src/analyze/sections.rs::render_brief()`
Function that renders cross-section summary with top-level aggregates from all section types.

---

## Core Operations

### Operation: `hegel analyze` (no flags)

**Behavior**: Display brief cross-section summary only

**Output format**:
```
=== Hegel Metrics Analysis ===

Session: <session_id>
Tokens: <total_input> in, <total_output> out (<cache_hits> cache)
Activity: <bash_count> commands, <file_count> files, <commit_count> commits
Workflows: <transition_count> transitions, <phase_count> phases
Recent: <last_N_transitions>
```

**Example**:
```bash
$ hegel analyze
=== Hegel Metrics Analysis ===

Session: abc-123
Tokens: 50K in, 75K out (2M cache)
Activity: 250 commands, 180 files, 42 commits
Workflows: 45 transitions, 38 phases
Recent: spec->plan->code->code_review->done
```

### Operation: `hegel analyze --full`

**Behavior**: Display all sections in full detail (current default behavior)

**Output**: All sections rendered:
- Activity (session, tokens, commands, files)
- Workflow transitions (all transitions listed)
- Phase breakdown (per-phase metrics)
- Workflow graph (ASCII visualization)

### Operation: `hegel analyze --activity`

**Behavior**: Display only the activity section (renamed from `--summary`)

**Output**: Session info, token metrics, top bash commands, top file modifications

### Operation: `hegel analyze --brief --workflow-transitions`

**Behavior**: Display brief summary AND workflow transitions section

**Output**: Cross-section summary followed by full transition list

### Section Flag Logic

**Default (no flags)**: `show_brief = true`, all others false

**With `--full`**: All section flags become true (activity, workflow_transitions, phase_breakdown, workflow_graph)

**With individual flags**: Only requested sections shown

**Brief is additive**: `--brief` can combine with any other section flags

---

## Test Scenarios

### Simple: Default behavior shows brief summary
```bash
$ hegel analyze
# Output: ~20-30 lines of cross-section summary
# Includes: session, token totals, activity counts, transition count, recent activity
```

### Complex: Progressive disclosure
```bash
$ hegel analyze --activity
# Output: Only activity section (session, tokens, commands, files)

$ hegel analyze --brief --phase-breakdown
# Output: Brief summary + detailed phase breakdown

$ hegel analyze --full
# Output: All sections (activity, transitions, phases, graph)
```

### Error: Flag conflicts handled gracefully
```bash
$ hegel analyze --full --activity
# Behavior: --full takes precedence, shows all sections
# (or treat as additive - both are valid, specify which)
```

---

## Success Criteria

- `cargo test` passes with updated test cases
- `cargo build` succeeds
- Default `hegel analyze` outputs ≤50 lines for typical project
- All existing section flags still work (`--workflow-transitions`, `--phase-breakdown`, `--workflow-graph`)
- Renamed flag `--activity` produces same output as old `--summary`
- `--full` flag produces same output as old default (no flags)
- `--brief` flag produces new cross-section summary
- Flag combinations work as specified (brief + other sections)

---

## Out of Scope

- Changes to metrics collection or storage format
- Changes to individual section rendering logic (only adding new brief renderer)
- Performance optimization of rendering
- Output format changes beyond the new brief section
- Archive repair or git backfill features
- Export formats (DOT, JSON) - these remain unchanged

---

## Implementation Notes

**Files to modify**:
- `src/commands/analyze/mod.rs` - Add `brief` and `full` fields, rename `summary` to `activity`, update section display logic
- `src/main.rs` - Add CLI args for `--brief` and `--full`, rename `--summary` to `--activity`
- `src/analyze/sections.rs` - Add `render_brief()` function
- `src/commands/analyze/mod.rs` tests - Update tests for new default behavior

**Section display logic** (lines 35-44 in current `analyze/mod.rs`):
```rust
// Determine which sections to display
let show_brief = options.brief || (!options.full && !options.activity
    && !options.workflow_transitions && !options.phase_breakdown
    && !options.workflow_graph);

let show_activity = options.full || options.activity;
let show_workflow_transitions = options.full || options.workflow_transitions;
let show_phase_breakdown = options.full || options.phase_breakdown;
let show_workflow_graph = options.full || options.workflow_graph;
```

**Brief renderer**: Aggregates data from `UnifiedMetrics` without calling existing detailed renderers. Extract top-level stats and format concisely.
