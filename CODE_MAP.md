# CODE_MAP.md

## Architecture Overview

**Hegel** is a multi-layer Rust CLI for orchestrating Dialectic-Driven Development workflows:

**Layer 1: Commands** (`src/commands/`) - User-facing CLI operations (workflow, hook, analyze)
**Layer 2: Engine** (`src/engine/`) - Workflow state machine with YAML parsing & template rendering
**Layer 3: Rules** (`src/rules/`) - Deterministic workflow enforcement (evaluator, interrupt protocol, rule types)
**Layer 4: Storage** (`src/storage/`) - Atomic file-based persistence (JSON state + JSONL event logs)
**Layer 5: Metrics** (`src/metrics/`) - Event stream parsing, aggregation, and visualization (DAG reconstruction)
**Layer 6: TUI** (`src/tui/`) - Interactive dashboard with real-time file watching (`hegel top`)

**Data Flow**: CLI → Load State → Evaluate Transitions → Render Templates → Save State → Display Prompt
**Metrics Flow**: Hooks + States + Transcripts → Parse → Correlate by Timestamp → Aggregate by Phase → Visualize (analyze/top)

**Key Patterns**:
- **State machine**: YAML workflows define nodes + transitions, engine evaluates claims to advance state
- **Template system**: Workflow prompts support guide injection ({{GUIDE_NAME}}), template includes ({{templates/name}}), context variables ({{var}}, {{?optional}})
- **Parent directory discovery**: Finds `.hegel/` by walking up directory tree (like git), works from any subdirectory
- **Multi-agent support**: Adapter pattern normalizes events from Claude Code, Cursor, Codex to canonical schema
- **Meta-modes**: Learning (Research ↔ Discovery) vs Standard (Discovery ↔ Execution) development patterns
- **Atomic writes**: State updates use temp file + rename to prevent corruption
- **File locking**: Exclusive locks on JSONL appends prevent concurrent write corruption (fs2 crate)
- **Hook integration**: Captures agent events via adapters to `.hegel/hooks.jsonl`, parses transcripts for token metrics
- **Command guardrails**: Pre-execution safety checks for wrapped commands (git, docker, etc.) via guardrails.yaml
- **File watching**: TUI uses `notify` crate for non-blocking real-time updates (100ms poll, auto-reload on modify events)
- **Timestamp correlation**: Three event streams (hooks, states, transcripts) correlate via ISO 8601 timestamps for per-phase metrics

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

```
hegel-cli/
├── CLAUDE.md                    # Development guidelines for Claude Code
├── README.md                    # User-facing documentation
├── ROADMAP.md                   # Future development plans
├── LEXICON.md                   # Core philosophy and guidance vectors
├── COVERAGE_REPORT.md           # Test coverage metrics (auto-generated)
├── LOC_REPORT.md                # Lines of code metrics (auto-generated)
├── Cargo.toml                   # Rust package manifest
│
├── src/                         # Core implementation (six-layer architecture)
│   ├── main.rs                  # CLI entry point (clap parser, state directory resolution, analyze + top subcommands)
│   ├── test_helpers.rs          # Shared test utilities (builders, fixtures, JSONL readers, TUI test helpers, metrics builders, production workflow setup)
│   ├── config.rs                # User configuration (load/save .hegel/config.toml, use_reflect_gui setting)
│   ├── embedded.rs              # Compile-time bundled resources (workflows, guides via include_str!)
│   ├── theme.rs                 # Terminal color theme (semantic styling: success/error/warning, metric values, headers)
│   │
│   ├── adapters/                # Multi-agent support (normalize events from Claude Code, Cursor, Codex to canonical format)
│   │   ├── claude_code.rs       # Claude Code adapter (env detection, event normalization)
│   │   ├── cursor.rs            # Cursor adapter (future multi-agent support)
│   │   ├── codex.rs             # Codex adapter (future multi-agent support)
│   │   └── mod.rs               # AgentAdapter trait, CanonicalHookEvent schema, AdapterRegistry
│   │
│   ├── commands/                # Layer 1: User-facing command implementations
│   │   ├── mod.rs               # Public exports (start_workflow, next_prompt, show_status, reset_workflow, handle_hook, analyze_metrics)
│   │   ├── meta.rs              # Meta-mode commands (declare, status, auto-start initial workflow)
│   │   ├── hook.rs              # Hook event capture (JSON stdin → adapter normalization → hooks.jsonl)
│   │   ├── astq.rs              # AST-grep wrapper (builds from vendor/, LLM-friendly feedback on no matches)
│   │   ├── git.rs               # Git wrapper with guardrails (delegates to wrapped.rs)
│   │   ├── reflect.rs           # Mirror GUI launcher (finds binary, passes files for review)
│   │   ├── wrapped.rs           # Generic command wrapper (guardrails evaluation, audit logging, exits on block)
│   │   ├── init.rs              # Project initialization (greenfield vs retrofit workflow detection)
│   │   ├── config.rs            # Configuration commands (get, set, list config values)
│   │   ├── workflow/            # Workflow orchestration (252 impl + 740 test lines, refactored for SoC)
│   │   │   ├── mod.rs           # Command handlers (start, next, status, reset, abort, repeat, restart)
│   │   │   ├── claims.rs        # ClaimAlias type (Next/Repeat/Restart/Custom claim transformations)
│   │   │   ├── context.rs       # WorkflowContext (loading, prompt rendering with guide injection)
│   │   │   ├── transitions.rs   # Transition evaluation and execution (Stay/IntraWorkflow/InterWorkflow/Ambiguous outcomes)
│   │   │   └── tests.rs         # All workflow tests (49 tests covering orchestration, transitions, meta-modes)
│   │   └── analyze/             # Metrics analysis and display (hegel analyze)
│   │       ├── mod.rs           # Main analyze command orchestrator
│   │       └── sections.rs      # Rendering sections (session, tokens, activity, top commands/files, transitions, phases, graph)
│   │
│   ├── engine/                  # Layer 2: State machine and template rendering
│   │   ├── mod.rs               # Workflow/Node/Transition structs, load_workflow, init_state, get_next_prompt (integrates rules)
│   │   └── template.rs          # Template rendering ({{UPPERCASE}} guides, {{templates/name}} includes, {{lowercase}} context vars, recursive expansion)
│   │
│   ├── guardrails/              # Command safety layer (pre-execution guardrails for wrapped commands)
│   │   ├── parser.rs            # Load guardrails.yaml, parse blocked/allowed patterns
│   │   └── types.rs             # GuardRailsConfig, CommandGuardrails, RuleMatch (Allowed/Blocked/NoMatch)
│   │
│   ├── metamodes/               # Meta-mode orchestration (learning vs standard development patterns)
│   │   └── mod.rs               # MetaModeDefinition (learning/standard), transition evaluation, workflow completion detection
│   │
│   ├── rules/                   # Layer 3: Deterministic workflow enforcement
│   │   ├── mod.rs               # Public exports (evaluate_rules, interrupt_if_violated)
│   │   ├── types.rs             # Rule definitions (require_files, max_tokens, phase_timeout, etc.)
│   │   ├── evaluator.rs         # Rule evaluation engine (stateless, context-based, phase_start_time support)
│   │   └── interrupt.rs         # Interrupt protocol (rule violation → prompt injection)
│   │
│   ├── storage/                 # Layer 4: Atomic persistence and event logging
│   │   └── mod.rs               # FileStorage (load/save/clear state.json, log_state_transition → states.jsonl, parent dir discovery, file locking)
│   │
│   ├── metrics/                 # Layer 5: Event stream parsing, aggregation, and visualization
│   │   ├── mod.rs               # Unified metrics orchestrator, parse_unified_metrics entry point
│   │   ├── aggregation.rs       # Phase metrics builder (timestamp correlation, token aggregation per phase)
│   │   ├── hooks.rs             # Parses Claude Code hook events, extracts bash commands and file modifications (silent error handling)
│   │   ├── states.rs            # Parses workflow state transition events
│   │   ├── transcript.rs        # Parses Claude Code transcripts for token usage (handles old and new format, includes timestamp)
│   │   └── graph.rs             # Workflow DAG reconstruction (build from transitions, cycle detection, ASCII/DOT rendering)
│   │
│   └── tui/                     # Layer 6: Terminal User Interface (hegel top command)
│       ├── mod.rs               # Event loop (keyboard polling, file watching integration, terminal setup/restore)
│       ├── app.rs               # AppState (file watching via notify, keyboard handling, scroll management, tab navigation)
│       ├── ui.rs                # Main rendering orchestrator (header, footer, tab routing)
│       ├── utils.rs             # Scroll utilities (visible_window, max_scroll, scroll_indicators), timeline builder (merge hooks+states)
│       └── tabs/                # Tab rendering modules (separation of concerns)
│           ├── mod.rs           # Tab rendering exports
│           ├── overview.rs      # Overview tab (session summary, token usage, activity metrics)
│           ├── phases.rs        # Phases tab (per-phase breakdown with duration, tokens, activity)
│           ├── events.rs        # Events tab (unified timeline of hooks and states, scrollable)
│           └── files.rs         # Files tab (file modification frequency, color-coded by intensity)
│
├── workflows/                   # YAML workflow definitions
│   ├── discovery.yaml           # Learning-focused workflow (SPEC → PLAN → CODE → LEARNINGS → README)
│   ├── execution.yaml           # Production delivery workflow
│   └── minimal.yaml             # Simplified workflow for testing
│
├── guides/                      # Template content for workflow prompts
│   ├── templates/               # Reusable template fragments (DRY via {{templates/name}} includes)
│   │   └── mirror_workflow.md   # File operations and Mirror review process (used by all guides)
│   ├── SPEC_WRITING.md          # Behavioral contract guidance
│   ├── PLAN_WRITING.md          # TDD roadmap planning
│   ├── KICKOFF_WRITING.md       # Binary-weave execution mode kickoff
│   ├── LEARNINGS_WRITING.md     # Insight extraction guidance
│   ├── README_WRITING.md        # Summary documentation guidance
│   ├── CODE_MAP_WRITING.md      # Code mapping guidelines
│   ├── HANDOFF_WRITING.md       # Session handoff protocol
│   ├── STUDY_PLANNING.md        # Research mode study planning
│   ├── KNOWLEDGE_CAPTURE.md     # Research mode synthesis guidance
│   └── QUESTION_TRACKING.md     # Research mode open questions
│
├── tests/                       # Unit tests are co-located in src/ modules (85%+ coverage, 228 tests passing)
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
└── .hegel/                      # Runtime state (gitignored, located in cwd)
    ├── state.json               # Current workflow state (workflow + workflow_state + session_metadata)
    ├── states.jsonl             # State transition event log (timestamped from→to transitions)
    └── hooks.jsonl              # Claude Code event log (PostToolUse, PreToolUse, SessionStart, etc.)
```
