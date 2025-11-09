# src/

Core implementation of Hegel CLI. Six-layer architecture for Dialectic-Driven Development workflow orchestration.

## Architecture Overview

**Layer 1: Commands** (commands/) - User-facing CLI operations
**Layer 2: Engine** (engine/) - Workflow state machine with YAML parsing & template rendering
**Layer 3: Rules** (rules/) - Deterministic workflow enforcement (evaluator, interrupt protocol)
**Layer 4: Storage** (storage/) - Atomic file-based persistence (JSON state + JSONL event logs)
**Layer 5: Metrics** (metrics/) - Event stream parsing, aggregation, and visualization
**Layer 6: TUI** (tui/) - Interactive dashboard with real-time file watching

## Structure

```
src/
├── main.rs              CLI entry point (clap parser with --force flag, state directory resolution, command routing)
├── lib.rs               Library interface (exposes modules for hegel-pm and external tools)
├── config.rs            User configuration (code_map_style, use_reflect_gui, commit_guard, use_git)
├── embedded.rs          Compile-time bundled resources (workflows, guides via include_str!)
├── theme.rs             Terminal color theme (semantic styling for success/error/warning, metrics)
│
├── adapters/            Multi-agent support (See adapters/README.md)
├── analyze/             Metrics analysis and visualization library (See analyze/README.md)
├── commands/            Layer 1: User-facing command implementations (See commands/README.md)
├── engine/              Layer 2: State machine and template rendering (See engine/README.md)
├── guardrails/          Command safety layer (See guardrails/README.md)
├── metamodes/           Meta-mode orchestration (See metamodes/README.md)
├── rules/               Layer 3: Deterministic workflow enforcement (See rules/README.md)
├── storage/             Layer 4: Atomic persistence and event logging (See storage/README.md)
├── metrics/             Layer 5: Event stream parsing and aggregation (See metrics/README.md)
├── tui/                 Layer 6: Terminal User Interface (See tui/README.md)
└── test_helpers/        Modular test utilities (See test_helpers/README.md)
```
