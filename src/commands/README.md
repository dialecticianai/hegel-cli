# src/commands/

Layer 1: User-facing command implementations. Maps CLI subcommands to workflow operations, metrics analysis, and external tool integrations.

## Purpose

Translates user intent (CLI invocations) into actions on the workflow state machine, metrics collection, and external integrations. Each command module handles argument parsing, validation, and orchestration of lower layers.

## Structure

```
commands/
├── mod.rs               Public exports (command handlers, shared types)
│
├── workflow/            Workflow orchestration commands (See workflow/README.md)
├── fork/                External agent orchestration (See fork/README.md)
├── analyze/             Metrics analysis command entry point (delegates to src/analyze)
│
├── init.rs              Project initialization (greenfield vs retrofit workflow detection)
├── meta.rs              Meta-mode commands (declare, status, auto-start initial workflow)
├── hook.rs              Hook event capture (JSON stdin → adapter normalization → hooks.jsonl)
├── hooks_setup.rs       Hook configuration helper (generate .claude/settings.json)
├── status.rs            Workflow status display (current phase, history, meta-mode)
├── archive.rs           Workflow archive management (create/list archives from metrics)
│
├── astq.rs              AST-grep wrapper (builds from vendor/, LLM-friendly feedback)
├── reflect.rs           Mirror GUI launcher (finds binary, passes files for review)
├── pm.rs                Project manager dashboard launcher (wraps hegel-pm binary)
├── ide.rs               Hegel IDE launcher (wraps hegel-ide Electron app)
├── markdown.rs          Markdown file tree visualization (categorizes DDD vs regular docs)
│
├── git.rs               Git wrapper with guardrails (delegates to wrapped.rs)
├── wrapped.rs           Generic command wrapper (guardrails evaluation, audit logging)
├── config.rs            Configuration commands (get, set, list config values)
└── external_bin.rs      External binary/npm package discovery (find and execute companion tools)
```

## Command Categories

**Workflow Management**: `workflow/` (start, next, prev, repeat, restart, abort)
**Meta-Mode**: `meta.rs` (declare learning/standard patterns)
**Metrics**: `analyze/` (command entry - implementation lives in src/analyze)
**External Tools**: `astq.rs`, `reflect.rs`, `pm.rs`, `ide.rs`, `markdown.rs`, `fork/` (AST search, doc review, PM dashboard, IDE, markdown visualization, agent delegation)
**Safety**: `wrapped.rs`, `git.rs` (command guardrails and audit logging)
**Setup**: `init.rs`, `hooks_setup.rs` (project initialization, hook configuration)
