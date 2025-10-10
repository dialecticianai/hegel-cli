# CODE_MAP.md

## Architecture Overview

**Hegel** is a three-layer Rust CLI for orchestrating Dialectic-Driven Development workflows:

**Layer 1: Commands** (`src/commands/`) - User-facing CLI operations
**Layer 2: Engine** (`src/engine/`) - Workflow state machine with YAML parsing & template rendering
**Layer 3: Storage** (`src/storage/`) - Atomic file-based persistence (JSON state + JSONL event logs)

**Data Flow**: CLI → Load State → Evaluate Transitions → Render Templates → Save State → Display Prompt

**Key Patterns**:
- **State machine**: YAML workflows define nodes + transitions, engine evaluates claims to advance state
- **Template system**: Workflow prompts support guide injection ({{GUIDE_NAME}}) and context variables ({{var}})
- **Atomic writes**: State updates use temp file + rename to prevent corruption
- **File locking**: Exclusive locks on JSONL appends prevent concurrent write corruption (fs2 crate)
- **Hook integration**: Captures Claude Code events to `.hegel/hooks.jsonl`, parses transcripts for token metrics

**Event Stream Correlation** (Metrics Architecture):

Three independent event streams correlate via timestamps to provide unified metrics:

1. **hooks.jsonl** - Claude Code activity (tool usage, bash commands, file edits)
   - Written by: `hegel hook` (called from `.claude/settings.json` hooks)
   - Key fields: `session_id`, `hook_event_name`, `timestamp`, `tool_name`, `transcript_path`
   - Correlation key: `timestamp` (ISO 8601)

2. **states.jsonl** - Hegel workflow transitions (phase changes)
   - Written by: `hegel next` (workflow state machine transitions)
   - Key fields: `workflow_id`, `from_node`, `to_node`, `phase`, `mode`, `timestamp`
   - Correlation key: `workflow_id` (ISO 8601 timestamp from `hegel start`)

3. **Transcripts** - Token usage (input/output/cache metrics)
   - Location: `~/.claude/projects/<project>/<session_id>.jsonl`
   - Referenced by: `transcript_path` field in hooks.jsonl
   - Key fields: `message.usage.{input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens}`

**Correlation Strategy**:

- **Workflow membership**: `WHERE hook.timestamp >= state.workflow_id`
  - All hooks after workflow start belong to that workflow (workflow_id is start timestamp)

- **Per-phase attribution**: Join hooks to states.jsonl transitions by timestamp ranges
  - Hook belongs to phase X if: `state[X].timestamp <= hook.timestamp < state[X+1].timestamp`
  - Enables: "How many bash commands during SPEC phase?" or "Token usage in PLAN phase"

- **Token aggregation**: Parse transcript file, correlate to workflow phases via transcript timestamps
  - Each hook event includes `transcript_path` for transcript file location
  - Token metrics extracted from `message.usage` (new format) or root `usage` (old format)
  - TranscriptEvent includes `timestamp` field to bucket assistant turns by workflow phase

**Example Query Pattern** (pseudocode):
```rust
// Get all hooks for a workflow's SPEC phase
let workflow_start = state.workflow_id; // "2025-10-10T14:30:00Z"
let spec_end = states.find(to_node == "plan").timestamp; // "2025-10-10T14:45:00Z"
let spec_hooks = hooks.filter(|h| h.timestamp >= workflow_start && h.timestamp < spec_end);
```

---

## Project Structure

hegel-cli/
├── CLAUDE.md                    # Development guidelines for Claude Code
├── README.md                    # User-facing documentation
├── ROADMAP.md                   # Future development plans
├── LEXICON.md                   # Core philosophy and guidance vectors
├── COVERAGE_REPORT.md           # Test coverage metrics (auto-generated)
├── LOC_REPORT.md                # Lines of code metrics (auto-generated)
├── Cargo.toml                   # Rust package manifest
│
├── src/                         # Core implementation (three-layer architecture)
│   ├── main.rs                  # CLI entry point (clap parser, state directory resolution)
│   ├── test_helpers.rs          # Shared test utilities (builders, fixtures, JSONL readers)
│   │
│   ├── commands/                # Layer 1: User-facing command implementations
│   │   ├── mod.rs               # Public exports (start_workflow, next_prompt, show_status, reset_workflow, handle_hook, analyze_metrics)
│   │   ├── workflow.rs          # Workflow commands (start, next, status, reset)
│   │   ├── hook.rs              # Claude Code hook event capture (JSON stdin → hooks.jsonl, with file locking)
│   │   └── analyze.rs           # Metrics analysis and display (hegel analyze command)
│   │
│   ├── engine/                  # Layer 2: State machine and template rendering
│   │   ├── mod.rs               # Workflow/Node/Transition structs, load_workflow, init_state, get_next_prompt
│   │   └── template.rs          # Guide injection ({{UPPERCASE}}), context variables ({{lowercase}}, {{?optional}})
│   │
│   ├── metrics/                 # Metrics parsing and aggregation
│   │   ├── mod.rs               # Unified metrics aggregator, builds per-phase metrics from timestamp correlation
│   │   ├── hooks.rs             # Parses Claude Code hook events, extracts bash commands and file modifications
│   │   ├── states.rs            # Parses workflow state transition events
│   │   └── transcript.rs        # Parses Claude Code transcripts for token usage (handles old and new format, includes timestamp)
│   │
│   └── storage/                 # Layer 3: Atomic persistence and event logging
│       └── mod.rs               # FileStorage (load/save/clear state.json, log_state_transition → states.jsonl, with file locking)
│
├── workflows/                   # YAML workflow definitions
│   ├── discovery.yaml           # Learning-focused workflow (SPEC → PLAN → CODE → LEARNINGS → README)
│   ├── execution.yaml           # Production delivery workflow
│   └── minimal.yaml             # Simplified workflow for testing
│
├── guides/                      # Template content for workflow prompts
│   ├── SPEC_WRITING.md          # Behavioral contract guidance
│   ├── PLAN_WRITING.md          # TDD roadmap planning
│   ├── CODE_MAP_WRITING.md      # Code mapping guidelines
│   ├── LEARNINGS_WRITING.md     # Insight extraction guidance
│   ├── README_WRITING.md        # Summary documentation guidance
│   ├── HANDOFF_WRITING.md       # Session handoff protocol
│   └── KICKOFF_WRITING.md       # Project kickoff guidance
│
├── tests/                       # Unit tests are co-located in src/ modules (95.41% coverage)
│   └── (empty)                  # Future integration tests live here
│
├── scripts/                     # Development utilities
│   ├── generate-coverage-report.sh  # Update COVERAGE_REPORT.md
│   ├── generate-loc-report.sh       # Update LOC_REPORT.md
│   ├── check-transcript-tokens.sh   # Validate conversation token usage
│   ├── analyze-hook-schema.sh       # Explore hook event schema
│   ├── check-hook-fields.sh         # Verify hook field availability
│   ├── analyze-transcripts.sh       # Explore transcript structure
│   └── summarize-findings.sh        # Summary of hook/transcript analysis
│
├── .claude/                     # Claude Code configuration
│   └── settings.json            # Hook routing to `hegel hook` command
│
└── .hegel/                      # Runtime state (gitignored)
    ├── state.json               # Current workflow state (workflow + current_node + history + mode)
    ├── states.jsonl             # State transition event log (timestamped from→to transitions)
    └── hooks.jsonl              # Claude Code event log (PostToolUse, PreToolUse, SessionStart, etc.)
