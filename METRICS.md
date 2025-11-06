# Metrics Architecture

## Overview

Hegel correlates three independent event streams to provide unified workflow metrics:

1. **hooks.jsonl** - Agent activity (bash commands, file edits, tool usage)
2. **states.jsonl** - Workflow transitions (phase changes)
3. **Transcripts** - Token usage (input/output/cache metrics from Claude Code sessions)

All correlation happens via **timestamps**.

## Event Stream Sources

### 1. hooks.jsonl

**Location**: `.hegel/hooks.jsonl`

**Written by**: `hegel hook` command (called via Claude Code hook configuration)

**Content**: Normalized agent activity events
- Tool usage (Bash, Edit, Write, Read, etc.)
- Session metadata (session_id, transcript_path, cwd)
- Timestamps for temporal correlation

**Schema**: See `src/adapters/mod.rs::CanonicalHookEvent`

**Key fields**:
- `session_id` - Claude Code session UUID
- `transcript_path` - Path to session's transcript JSONL file
- `timestamp` - RFC3339 timestamp
- `event_type` - pre_tool_use | post_tool_use

### 2. states.jsonl

**Location**: `.hegel/states.jsonl`

**Written by**: `hegel next/prev/restart/abort` commands

**Content**: Workflow state transitions
- Phase changes (from_node → to_node)
- Workflow membership (workflow_id)
- Mode tracking (discovery/execution)

**Schema**: See `src/metrics/states.rs::StateTransitionEvent`

**Key fields**:
- `workflow_id` - RFC3339 timestamp identifying workflow run
- `timestamp` - RFC3339 timestamp of transition
- `from_node` / `to_node` - Phase names
- `phase` - Active phase after transition

### 3. Transcripts

**Location**: Agent-specific (see adapters documentation)

**Written by**: Agent CLI (Claude Code, Cursor, Codex - external to Hegel)

**Content**: Conversation events with token usage
- User messages
- Assistant messages (with token metrics)
- Tool calls and responses

**Schema**: Agent-specific (adapters normalize to common format)

**Key fields** (after adapter normalization):
- `timestamp` - RFC3339 timestamp
- `type` - "user" | "assistant" | ...
- `usage.input_tokens` - Input tokens consumed
- `usage.output_tokens` - Output tokens generated
- `usage.cache_creation_input_tokens` - Cache write tokens
- `usage.cache_read_input_tokens` - Cache read tokens

**Agent-specific details**: See `src/adapters/<agent>/README.md`
- Claude Code: `~/.claude/projects/<normalized-path>/<session-id>.jsonl`
- Cursor: TBD
- Codex: TBD

## Correlation Strategy

### Workflow Membership

Events belong to a workflow if:
```
event.timestamp >= workflow.workflow_id
```

Where `workflow_id` is the RFC3339 timestamp when the workflow started.

### Per-Phase Attribution

Events belong to a phase if:
```
state[i].timestamp <= event.timestamp < state[i+1].timestamp
```

For the active (incomplete) phase:
```
state[last].timestamp <= event.timestamp < infinity
```

### Multi-Session Workflows

**Problem**: A single workflow may span multiple Claude Code sessions.

**Example scenario**:
1. User starts workflow, works for 2 hours → session A (transcript A)
2. User closes Claude Code, goes to lunch
3. User reopens Claude Code, continues same workflow → session B (transcript B)
4. Workflow completes and archives

**Challenge**: Transcript paths are session-specific. We must discover and scan ALL transcript files covering the workflow's time range, not just the current session.

## Archiving Flow

### Normal Archiving (Explicit Workflow)

**Trigger**: `hegel next done` → `archive_and_cleanup()`

**Flow**:
```
1. parse_unified_metrics(state_dir, include_archives=false, debug=None)
   ├─ Parse hooks.jsonl → bash_commands, file_modifications
   ├─ Get transcript_path from state.session_metadata (current session only!)
   ├─ Parse states.jsonl → state_transitions
   └─ build_phase_metrics(transitions, hooks, transcript_path, debug)
      └─ For each phase:
         └─ aggregate_tokens_for_phase(transcript_path, phase_start, phase_end)
            └─ Read transcript file, filter by timestamp range

2. Parse git commits, attribute to phases by timestamp

3. WorkflowArchive::from_metrics(&metrics, workflow_id, is_synthetic=false)

4. Write archive to .hegel/archive/<workflow_id>.json

5. Delete hooks.jsonl and states.jsonl
```

**Current limitation**: Only scans ONE transcript file (current session).

### Gap Detection Archiving (Synthetic Cowboy)

**Trigger**: `hegel analyze --fix-archives` → gap detection

**Purpose**: Backfill gaps between workflows with synthetic "cowboy mode" archives

**Flow**:
```
1. Identify gaps between non-synthetic workflows
2. For each gap with git activity:
   create_cowboy_for_gap(gap_start, gap_end, state_dir)
   ├─ Create synthetic state_transitions for [gap_start, gap_end)
   ├─ Parse hooks.jsonl → bash_commands, file_modifications
   ├─ Get transcript_path from state.session_metadata (current session only!)
   └─ build_phase_metrics(transitions, hooks, transcript_path, debug)
      └─ aggregate_tokens_for_phase(transcript_path, gap_start, gap_end)

3. Mark phases as is_synthetic=true
4. Write archive
```

**Current limitation**: Only scans ONE transcript file (current session). Gap detection typically runs MUCH later than the gap period, so the current session's transcript has no overlap.

## Token Attribution Algorithm

**Function**: `aggregate_tokens_for_phase(transcript_path, start, end, phase, debug)`

**Location**: `src/metrics/aggregation.rs:133`

**Algorithm**:
```rust
1. Read transcript file line-by-line (streaming parse)
2. For each line:
   - Parse as TranscriptEvent
   - Skip if not type="assistant"
   - Skip if no timestamp
   - Check if timestamp in [start, end) range
   - Extract usage (try .message.usage, fallback to .usage)
   - Accumulate: input_tokens, output_tokens, cache_*_tokens, turns++
3. Return TokenMetrics
```

**Performance**: Streams file (constant memory), but reads ENTIRE file even if only a small time range matches.

**Limitation**: Only scans ONE file. Multi-session workflows lose tokens from earlier sessions.

## Discovery Problems

### Current Implementation

**Transcript discovery** (src/metrics/mod.rs:183-220):
```rust
// Try state.json first
if let Some(session) = state.session_metadata {
    transcript_path = Some(session.transcript_path);
}
// Fallback: scan hooks.jsonl for last SessionStart event
else {
    for line in hooks.jsonl {
        if event.hook_event_name == "SessionStart" {
            last_session_start = Some(event);
        }
    }
    transcript_path = last_session_start.transcript_path;
}
```

**Result**: Single transcript path (most recent session).

### Multi-Session Gap Example

**Nov 3-4 Gap** (discovered via Perl script):
- Gap: `2025-11-03T03:13:34Z` to `2025-11-04T22:11:56Z`
- Duration: ~43 hours
- Git commits: 20 commits
- Claude Code activity: 1625 assistant turns across MULTIPLE sessions
- Tokens found: 75K input, 132K output, 5.2M cache creation, 160M cache read
- Current implementation: 0 tokens (only scanned current session's transcript)

## Required Changes

### High-Level Goal

**Replace single-file scanning with multi-file discovery and aggregation.**

### Architecture Principles

**Adapter separation**: Agent-specific logic (transcript location, file discovery) belongs in `src/adapters/<agent>/`

**Metrics agnostic**: Token aggregation logic stays in `src/metrics/`, works with any list of transcript files

### Affected Components

1. **Adapter layer** (new functionality)
   - Each adapter implements transcript discovery for its agent
   - Claude Code: Find project directory, list `*.jsonl` files
   - Cursor/Codex: TBD when multi-agent support added
   - Returns: `Vec<PathBuf>` of transcript files for project

2. **Metrics layer** (signature changes)
   - Current: `aggregate_tokens_for_phase(transcript_path: &str, ...)`
   - New: `aggregate_tokens_for_range(transcript_files: &[PathBuf], ...)`
   - Streams each file, filters by timestamp, accumulates totals

3. **Phase metrics builder** (signature change)
   - Current: `build_phase_metrics(..., transcript_path: Option<&str>, ...)`
   - New: `build_phase_metrics(..., transcript_files: &[PathBuf], ...)`
   - Calls updated aggregation for each phase

### Design Questions

1. **API design**: Should adapters provide discovery API, or just document location patterns?
2. **Caching**: Should we cache transcript file timestamp ranges to avoid repeated scans?
3. **Performance**: Parallel file scanning? Optimization needed given file sizes?
4. **Adapter trait**: Should `AgentAdapter` trait include `discover_transcripts()` method?

See `.ddd/refactor/20251106-multi_session_token_attribution.md` for detailed refactoring plan.
