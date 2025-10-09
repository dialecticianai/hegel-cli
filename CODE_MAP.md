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
- **Hook integration**: Captures Claude Code events to `.hegel/hooks.jsonl` for future metrics

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
│   │   ├── mod.rs               # Public exports (start_workflow, next_prompt, show_status, reset_workflow, handle_hook)
│   │   ├── workflow.rs          # Workflow commands (start, next, status, reset)
│   │   └── hook.rs              # Claude Code hook event capture (JSON stdin → hooks.jsonl)
│   │
│   ├── engine/                  # Layer 2: State machine and template rendering
│   │   ├── mod.rs               # Workflow/Node/Transition structs, load_workflow, init_state, get_next_prompt
│   │   └── template.rs          # Guide injection ({{UPPERCASE}}), context variables ({{lowercase}}, {{?optional}})
│   │
│   └── storage/                 # Layer 3: Atomic persistence and event logging
│       └── mod.rs               # FileStorage (load/save/clear state.json, log_state_transition → states.jsonl)
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
│   └── check-transcript-tokens.sh   # Validate conversation token usage
│
├── .claude/                     # Claude Code configuration
│   └── settings.json            # Hook routing to `hegel hook` command
│
└── .hegel/                      # Runtime state (gitignored)
    ├── state.json               # Current workflow state (workflow + current_node + history + mode)
    ├── states.jsonl             # State transition event log (timestamped from→to transitions)
    └── hooks.jsonl              # Claude Code event log (PostToolUse, PreToolUse, SessionStart, etc.)
