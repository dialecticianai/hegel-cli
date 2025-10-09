# CLAUDE.md

This file provides guidance to Claude Code when working with the Hegel CLI project.

## Project Overview

**Hegel** is a command-line tool for orchestrating Dialectic-Driven Development (DDD) workflows. It uses YAML-based workflow definitions and local state management to guide developers through structured development cycles.

**Philosophy**: Thesis. Antithesis. Synthesis. State-based workflow orchestration with no external dependencies.

---

## Project Structure

### Core Implementation (`src/`)

- **`main.rs`** - CLI entry point, command parsing
- **`commands/mod.rs`** - Command implementations (start, next, status, reset)
- **`engine/mod.rs`** - Workflow state machine, transition logic
- **`engine/template.rs`** - Template rendering ({{GUIDE_NAME}} placeholders)
- **`storage/mod.rs`** - File-based state persistence (.hegel/state.json)

### Workflows (`workflows/`)

YAML workflow definitions:
- **`discovery.yaml`** - Learning-focused workflow (SPEC → PLAN → CODE → LEARNINGS → README)
- **`execution.yaml`** - Production delivery workflow
- **`minimal.yaml`** - Simplified workflow for testing

### Guides (`guides/`)

Template content injected into workflow prompts via {{GUIDE_NAME}} placeholders. These are part of Hegel's workflow system, not instructions for working on Hegel itself.

### Documentation

- **`README.md`** - User-facing documentation
- **`ROADMAP.md`** - Future development plans (delete completed phases)
- **`LEXICON.md`** - Core philosophy and guidance vectors
- **`COVERAGE_REPORT.md`** - Test coverage metrics (auto-generated)
- **`LOC_REPORT.md`** - Lines of code metrics (auto-generated)

### Session Artifacts (Project Root)

**Gitignored ephemeral files**:
- **`HANDOFF.md`** - Session handoff document for context between Claude sessions (see protocol below)

---

## Core Methodology

Hegel implements **Dialectic-Driven Development (DDD)**:

1. **SPEC** - Write behavioral contract (what must hold true)
2. **PLAN** - Define TDD implementation roadmap
3. **CODE** - Build with Red → Green → Refactor discipline
4. **LEARNINGS** - Extract insights and architectural decisions
5. **README** - Summarize for context refresh

**Modes**:
- **Discovery** - Learning-driven, experimental (toys in separate projects)
- **Execution** - Production-focused, incremental (features in main codebase)

---

## Development Best Practices

### Testing

**Current coverage**: 95.41% lines (target: ≥80%)

- Write tests first (TDD discipline)
- Use `cargo test` for unit tests
- Integration tests in `tests/` directory
- Pre-commit hook runs coverage updates automatically

**Running tests**:
```bash
cargo test                    # Run all tests
cargo test --package hegel    # Package-specific tests
cargo llvm-cov --html         # Generate HTML coverage report
```

### Code Quality

**Pre-commit hooks** (`.git/hooks/pre-commit`):
- Auto-format Rust code with `rustfmt`
- Update coverage reports (only for .rs/.toml changes)
- Update LOC reports (only for .rs/.toml changes)
- Auto-stage updated reports

**Manual formatting**:
```bash
cargo fmt                     # Format all Rust code
cargo clippy                  # Run linter
```

### Commit Guidelines

**Format**: `type(scope): subject`

**Types**: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`

**Examples**:
- `feat(engine): add template rendering support`
- `fix(storage): handle missing state file gracefully`
- `docs(guides): make CODE_MAP guide project-agnostic`
- `chore(deps): update serde to 1.0.195`

**Commit footer** (add to all commits):
```
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

### Documentation Updates

**CRITICAL**: Update documentation BEFORE committing code changes

- Update `README.md` when changing user-facing behavior
- Update `ROADMAP.md` when completing phases (delete completed sections)
- Update `HANDOFF.md` per protocol below (end of session only)
- Coverage/LOC reports update automatically via pre-commit hook

### HANDOFF.md Protocol

**CRITICAL: Only update at END OF SESSION**

- `HANDOFF.md` is ephemeral (gitignored, session-to-session handoff only)
- **ONLY write when session is ending** (user says "done for now", tokens running low, etc.)
- **DO NOT write after completing a task** if continuing work in same session

**At start of session:**
- Read HANDOFF.md for context (if it exists)
- **Delete after reading**: `rm HANDOFF.md` (keep context clean, force explicit handoff)

**When updating for next session:**
- Write fresh content (file already deleted at session start)
- **DO NOT** read old content first (already consumed and deleted)
- Include: Current status, what was learned/completed, what to do next, key files to review
- Make it clear where we left off and what's the immediate next action

**After writing HANDOFF.md:**
- **NO FURTHER CODE WORK** - Writing HANDOFF.md signals session end
- **Clear todo list** - No pending implementation tasks should remain
- **Only housekeeping allowed**: Documentation updates (README, LEXICON, CLAUDE.md), committing those changes
- **NO**: New features, refactorings, test changes, implementation work
- **NEVER commit HANDOFF.md** - It is gitignored and ephemeral

---

## Claude Code Hooks Integration

Hegel integrates with Claude Code's hook system to capture development activity. All hook events are logged to `.hegel/hooks.jsonl` for future analysis.

**Hook command**: `hegel hook <event_name>` reads JSON from stdin and appends to JSONL log.

**Configuration**: `.claude/settings.json` routes Claude Code events (PostToolUse, PreToolUse, UserPromptSubmit, Stop, SessionStart) to Hegel.

**Note**: Currently just captures events. Metrics processing and workflow enforcement are future work.

---

## Engine & State Management

### State Storage

All state lives in `.hegel/state.json`:
- Current workflow definition (YAML serialized)
- Current node/phase
- Navigation history
- Workflow mode

**Atomic writes**: Uses temp file + rename pattern to prevent corruption

### Command Flow

1. **`hegel start <workflow>`**
   - Load YAML from `workflows/<workflow>.yaml`
   - Initialize state at start_node
   - Save to `.hegel/state.json`
   - Display first prompt

2. **`hegel next '{"claim": true}'`**
   - Load state from `.hegel/state.json`
   - Evaluate claims against current node's transitions
   - Transition to next node (or stay if no match)
   - Save updated state
   - Display next prompt

3. **`hegel status`**
   - Display current mode, node, history

4. **`hegel reset`**
   - Clear `.hegel/state.json`

---

## Template System

Workflow prompts can include template placeholders:

- **Required**: `{{GUIDE_NAME}}` - Must be provided, error if missing
- **Optional**: `{{?guide_name}}` - Replaced with empty string if not provided

**Example** (`workflows/discovery.yaml`):
```yaml
nodes:
  spec:
    prompt: |
      You are in the SPEC phase.

      {{SPEC_WRITING}}

      Your task: Write a behavioral contract.
      {{?context_note}}
```

**Guide resolution**: Looks for `guides/GUIDE_NAME.md` and inlines content

---

## Platform & Tooling

**Development machine**: macOS Apple Silicon (M1)

**Build requirements**:
- Rust (stable channel)
- Cargo (comes with Rust)

**Optional tools**:
- `cargo-llvm-cov` for coverage reports
- `cloc` for LOC counting

**Installation**:
```bash
cargo build --release          # Build optimized binary
./target/release/hegel --help  # Verify build
```

---

## Guiding Principles

From `LEXICON.md`:

**Context is king** - State determines what's possible

**Artifacts are disposable, clarity is durable** - Code can be rewritten, insights cannot

**Thesis, antithesis, synthesis** - Dialectical method as cognitive process

**CLI-first, local-first** - Terminal over GUI, files over databases

**No black boxes** - All rules visible, state always inspectable

**Remember you're not human** - Comprehensive is just complete, no cost to thoroughness

**The human always knows best** - Execute instructions, don't editorialize

---

## Next Steps Protocol

**Never just report what you did - always suggest what to do next:**
- After completing any task, propose the next logical action
- Don't say "done" or "ready for next step" - suggest a specific next move
- **Wait for explicit approval before proceeding**

**Format**: "Should I [specific action], or [alternative]?"
- ✅ Good: "Should I start implementing the hook subcommand, or design the metrics schema first?"
- ❌ Bad: "Continue, or wrap up?" (too vague - forces user to clarify what "continue" means)
- ❌ Bad: "Ready for next session." (declares stopping instead of proposing)

**Examples**:
- "Should I start implementing Phase 1 (Claude Code hooks), or review the roadmap?"
- "Should I add tests for the new command, or update the README first?"
- "Built the hook parser. Should I add metrics storage next, or test the JSON parsing first?"

---

**Remember**: Hegel orchestrates workflows through state transitions. Keep it simple, transparent, and fully local.
