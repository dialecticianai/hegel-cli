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
- **`storage/mod.rs`** - File-based state persistence (~/.hegel/state.json)

### Workflows (`workflows/`)

YAML workflow definitions:
- **`discovery.yaml`** - Learning-focused workflow (SPEC → PLAN → CODE → LEARNINGS → README)
- **`execution.yaml`** - Production delivery workflow
- **`minimal.yaml`** - Simplified workflow for testing

### Guides (`guides/`)

Writing guides for DDD artifacts (project-agnostic, reusable):
- **`SPEC_WRITING.md`** - How to write specifications
- **`PLAN_WRITING.md`** - How to write implementation plans
- **`LEARNINGS_WRITING.md`** - How to write retrospectives
- **`README_WRITING.md`** - How to write library READMEs
- **`CODE_MAP_WRITING.md`** - How to write code maps
- **`KICKOFF_WRITING.md`** - How to write feature kickoffs
- **`HANDOFF_WRITING.md`** - How to write session handoffs

### Documentation

- **`README.md`** - User-facing documentation
- **`ROADMAP.md`** - Future development plans (delete completed phases)
- **`LEXICON.md`** - Core philosophy and guidance vectors
- **`COVERAGE_REPORT.md`** - Test coverage metrics (auto-generated)
- **`LOC_REPORT.md`** - Lines of code metrics (auto-generated)

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

- Update `guides/` when adding new artifact types
- Update `README.md` when changing user-facing behavior
- Update `ROADMAP.md` when completing phases (delete completed sections)
- Coverage/LOC reports update automatically via pre-commit hook

---

## Workflow Lifecycle

### State Storage

All state lives in `~/.hegel/state.json`:
- Current workflow definition (YAML serialized)
- Current node/phase
- Navigation history
- Workflow mode

**Atomic writes**: Uses temp file + rename pattern to prevent corruption

### Command Flow

1. **`hegel start <workflow>`**
   - Load YAML from `workflows/<workflow>.yaml`
   - Initialize state at start_node
   - Save to `~/.hegel/state.json`
   - Display first prompt

2. **`hegel next '{"claim": true}'`**
   - Load state from `~/.hegel/state.json`
   - Evaluate claims against current node's transitions
   - Transition to next node (or stay if no match)
   - Save updated state
   - Display next prompt

3. **`hegel status`**
   - Display current mode, node, history

4. **`hegel reset`**
   - Clear `~/.hegel/state.json`

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

## Project-Agnostic Guides

**CRITICAL**: All guides in `guides/` MUST be project and language-agnostic

- NO specific tech stacks (FastMCP, PyYAML, pytest, etc.)
- NO specific languages (Python, JavaScript, Rust examples)
- NO project-specific references (ddd-nes, hegel-cli internals)

**Why**: These guides are reusable across all DDD projects

**When updating**: Strip out specific examples, use generic patterns

---

## Next Steps Protocol

**After completing any task, propose specific next action**

Format: "Should I [specific action], or [alternative]?"

Examples:
- "Should I start implementing Phase 1 (Claude Code hooks), or review the roadmap?"
- "Should I add tests for the new command, or update the README first?"

**Wait for explicit approval before proceeding**

---

**Remember**: Hegel orchestrates workflows through state transitions. Keep it simple, transparent, and fully local.
