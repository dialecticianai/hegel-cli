# CLAUDE.md

**Hegel**: CLI orchestration for Dialectic-Driven Development. State-based workflow enforcement with no external dependencies.

---

## Architecture

**Core**: `src/{main,commands,engine,storage,metrics}` - CLI → workflow state machine → file-based persistence
**Workflows**: `workflows/*.yaml` - YAML definitions (discovery/execution modes)
**Guides**: `guides/*.md` - Template content injected via `{{GUIDE_NAME}}` placeholders
**State**: `.hegel/{state.json,hooks.jsonl,states.jsonl}` - Local state + event logs
**Metrics**: Submodules `metrics/{hooks,transcript,states}.rs` - Parse JSONL, extract telemetry

---

## Philosophy (Compressed from LEXICON.md)

**Context is king** - Line counts are physics, not style. Token overhead is immediate cost. Refactor on pattern, not pain.

**Artifacts disposable, clarity durable** - Code rewrites. Insights don't. Generation cheap, understanding valuable.

**Infrastructure compounds** - Helpers, submodules, test patterns save context forever. Build once, reuse infinitely.

**Test density is infrastructure** - Verbose patterns = compounding friction. Extract early, compress aggressively.

**Remember you're not human** - No cost to thoroughness. 18x token waste is real waste, not hypothetical debt.

**The human always knows best** - Execute instructions. Don't editorialize. Questions are literal, not criticism.

**Refactor early, not late** - Structure for reading efficiency, not writing comfort. 200+ line files trigger immediate split.

---

## HANDOFF.md Protocol

**CRITICAL: Only update at END OF SESSION**

**Purpose**: Session-to-session continuity. Gitignored ephemeral file.

**At session start:**
- Read `HANDOFF.md` if exists
- **Delete after reading**: `rm HANDOFF.md` (force explicit handoff, prevent drift)

**At session end:**
- Write fresh `HANDOFF.md` (old already deleted)
- Include: Status, learnings, next action, key files
- **NO CODE WORK AFTER WRITING** - signals session end
- Only housekeeping: docs updates, commits
- **NEVER commit HANDOFF.md**

**When to write:**
- User says "done for now"
- Tokens running low
- Natural stopping point

---

## Claude Code Hooks Integration

`hegel hook <event_name>` reads JSON from stdin, appends to `.hegel/hooks.jsonl`.

**Events**: `PostToolUse`, `PreToolUse`, `UserPromptSubmit`, `Stop`, `SessionStart`
**Configuration**: `.claude/settings.json` routes events to Hegel
**Schema**: `src/metrics/hooks.rs` - HookEvent struct, BashCommand, FileModification
**Transcripts**: Token usage at `.message.usage` (new format) or `.usage` (old format)

---

## Critical Patterns

**Atomic writes**: Temp file + rename (prevents corruption). See `src/storage/mod.rs`.

**File locking**: `fs2::FileExt` exclusive locks on JSONL appends (prevents concat corruption).

**Test helpers**: `src/test_helpers.rs` - `create_{hooks,transcript,states}_file()` compress boilerplate.

**Submodule organization**: `metrics/{hooks,transcript,states}.rs` - One parser per file. ~100-200 lines each.

**Documentation ordering**: Update README/ROADMAP **BEFORE** committing code changes.

**ROADMAP policy**: Future-only. Delete completed sections, don't mark them done.

---

## Session Continuity Vectors

**Coverage target**: ≥80% lines (current: ~85%)
**Commit format**: Conventional commits (`type(scope): subject`). Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`. Always include footer with Claude Code attribution.
**Pre-commit hooks**: Auto-format, update coverage/LOC reports, auto-stage
**Tests**: TDD discipline. `cargo test` before commits.

**Key files:**
- `LEXICON.md` - Full philosophy (reference, don't read every session)
- `ROADMAP.md` - Future-only development plan
- `COVERAGE_REPORT.md` - Auto-generated test metrics
- `LOC_REPORT.md` - Auto-generated line counts

---

## Workflow Execution

**State machine**: `src/engine/mod.rs` - Evaluate claims → transition nodes → update state
**Templates**: `{{GUIDE_NAME}}` required (error if missing), `{{?guide_name}}` optional
**Atomic state**: `.hegel/state.json` stores workflow definition + current node + history

**Commands:**
- `hegel start <workflow>` - Initialize from `workflows/<workflow>.yaml`
- `hegel next '{"claim":true}'` - Evaluate claim, transition if matched
- `hegel status` - Show current mode/node/history
- `hegel reset` - Clear state
- `hegel analyze` - Parse metrics from `.hegel/*.jsonl`

---

## Development Constraints

**Platform**: macOS Apple Silicon (M1)
**Language**: Rust stable
**Dependencies**: Minimal (serde, anyhow, clap, fs2, ratatui/crossterm for future TUI)

**Build**: `cargo build --release`
**Test**: `cargo test`
**Format**: `cargo fmt` (pre-commit auto-formats)

---

**Remember**: Hegel orchestrates workflows through state transitions. Keep it simple, transparent, local-first.
