# Claude Code Integration

Hegel integrates with [Claude Code](https://claude.com/claude-code) to capture development activity as you work. This enables metrics collection and workflow analysis.

## Hook Events

The `hegel hook` command processes Claude Code hook events:

```bash
# Typically configured in .claude/settings.json
hegel hook PostToolUse < event.json
```

Hook events are logged to `.hegel/hooks.jsonl` with timestamps. Each workflow session is assigned a unique `workflow_id` (ISO 8601 timestamp) when you run `hegel start`, enabling correlation between workflow phases and development activity.

## Analyzing Metrics

View captured development activity and metrics:

```bash
hegel analyze
```

**Default output** (brief summary):
- Session ID
- Token totals (input/output/cache)
- Activity counts (commands, files, commits)
- Workflow/phase counts
- Recent transitions

**Progressive disclosure** via flags:
```bash
# Show specific sections
hegel analyze --activity           # Session, tokens, commands, files
hegel analyze --workflow-transitions  # State transition history
hegel analyze --phase-breakdown      # Per-phase metrics
hegel analyze --workflow-graph       # ASCII visualization

# Show all sections (old default behavior)
hegel analyze --full

# Combine sections
hegel analyze --brief --phase-breakdown  # Brief + phases

# Export workflow graph to DOT format
hegel analyze --export-dot > workflow.dot
dot -Tpng workflow.dot -o workflow.png
```

**Section details:**

- **Brief** (default): Cross-section summary of key metrics
- **Activity** (`--activity`): Session, tokens, top bash commands, top file modifications
- **Workflow Transitions** (`--workflow-transitions`): Complete state transition history
- **Phase Breakdown** (`--phase-breakdown`): Per-phase metrics (duration, tokens, activity, git commits)
- **Workflow Graph** (`--workflow-graph`): ASCII visualization with node metrics and cycle detection

## Interactive Dashboard

Launch a real-time TUI dashboard:

```bash
hegel top
```

**Features:**
- **4 interactive tabs**: Overview, Phases, Events, Files
- **Live updates**: Auto-reloads when `.hegel/*.jsonl` files change
- **Scrolling**: Arrow keys, vim bindings (j/k), jump to top/bottom (g/G)
- **Navigation**: Tab/BackTab to switch tabs
- **Colorful UI**: Emoji icons, syntax highlighting, status indicators

**Keyboard shortcuts:**
- `q` - Quit
- `Tab` / `BackTab` - Navigate tabs
- `↑↓` / `j`/`k` - Scroll
- `g` / `G` - Jump to top/bottom
- `r` - Reload metrics manually

**What's tracked:**
- Tool usage (Bash, Read, Edit, Write, etc.)
- File modifications with frequency counts
- Workflow state transitions (logged to states.jsonl)
- Token usage from Claude Code transcripts
- Per-phase metrics (duration, tokens, commands, file edits)
  - Correlated via timestamps across hooks.jsonl, states.jsonl, and transcripts
  - Enables budget enforcement rules in Phase 2 (cycle detection)

## Configuration

See `.claude/settings.json` in this repository for an example hook configuration. Hook events are optional—Hegel works without them, but metrics features require hook data.
