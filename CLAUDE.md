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
- User says "done for now" AND work is incomplete/in-progress
- Tokens running low (<10% remaining) with work in progress

**When NOT to write:**
- All planned work completed and committed cleanly
- Session ending with no incomplete work (git history is the handoff)

---

## Claude Code Hooks Integration

`hegel hook <event_name>` reads JSON from stdin, appends to `.hegel/hooks.jsonl`.

**Events**: `PostToolUse`, `PreToolUse`, `UserPromptSubmit`, `Stop`, `SessionStart`
**Configuration**: `.claude/settings.json` routes events to Hegel
**Schema**: `src/metrics/hooks.rs` - HookEvent struct, BashCommand, FileModification
**Transcripts**: Token usage at `.message.usage` (new format) or `.usage` (old format)

---

## Critical Patterns

**Code search: hegel astq > grep**: ALWAYS use `hegel astq -l rust -p 'identifier' src/` for code search, NOT grep/rg. Why:
- AST-aware: finds only actual code usage, ignores comments/strings/docs
- Context-aware: understands Rust syntax structure
- Agent-friendly: provides "No matches found" feedback vs silent failure
- Example: `hegel astq -l rust -p 'green' src/` finds all `.green()` calls
- Only use grep for non-code searches (logs, markdown content, etc.)

**Scripts over inline commands**: NEVER write multi-line inline shell scripts. Always write to `scripts/` and execute. Check `scripts/` before writing - tool may already exist. Reusable scripts are infrastructure.

**Available scripts** (`scripts/`):
- `generate-coverage-report.sh` - Update COVERAGE_REPORT.md (auto-run by pre-commit)
- `generate-loc-report.sh` - Update LOC_REPORT.md (auto-run by pre-commit)
- `test-stability.sh [N]` - Run tests N times (default 20) to detect flaky tests
- `build.sh [--install] [--skip-bump]` - Build release binary (optionally install to ~/.cargo/bin)
- `check-transcript-tokens.sh` - Validate conversation token usage
- `analyze-hook-schema.sh` - Explore hook event schema
- `check-hook-fields.sh` - Verify hook field availability
- `analyze-transcripts.sh` - Explore transcript structure
- `summarize-findings.sh` - Summary of hook/transcript analysis
- `debug-analyze.sh` - Debug metrics analysis with sample data

**Test helpers**: `src/test_helpers.rs` - `create_{hooks,transcript,states}_file()` compress boilerplate.

**Submodule organization**: Split files >200 impl lines. One parser per file (~100-200 lines each).

**Documentation ordering**: Update README/ROADMAP **BEFORE** committing code changes.

**ROADMAP policy**: Future-only. Delete completed sections, don't mark them done.

---

## Session Continuity Vectors

**Coverage target**: ≥80% lines (current: ~85%)

**Commit format**: Conventional commits (`type(scope): subject`).

**Types**: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

**Message guidelines**:
- **Keep concise** - Short subject + body to minimize token usage in pre-commit hooks
- **Extended messages** - Only for extremely large/complex commits (subjective judgment)
- **Always include** - Claude Code attribution footer (required for all commits)
- **Format**:
  ```
  type(scope): brief subject line

  Optional brief body (1-3 sentences for larger commits)

  🤖 Generated with [Claude Code](https://claude.com/claude-code)

  Co-Authored-By: Claude <noreply@anthropic.com>
  ```

**Pre-commit hooks**: Auto-format, update coverage/LOC reports, auto-stage
**Tests**: TDD discipline. `cargo test` before commits.

**Key files:**
- `LEXICON.md` - Full philosophy (reference, don't read every session)
- `ROADMAP.md` - Future-only development plan
- `COVERAGE_REPORT.md` - Auto-generated test metrics
- `LOC_REPORT.md` - Auto-generated line counts

---

## Workflow Execution

**Commands**: `hegel start <workflow>` → `hegel next` → `hegel status` → `hegel restart|abort`

**CRITICAL - Workflow Advancement Protocol**:
- **NEVER run `hegel next` autonomously** - only the user decides when to advance
- When phase work is complete, **suggest** advancing: "Ready to run `hegel next`?"
- Wait for user approval (e.g., "SGTM", "yes", "go ahead")
- User controls workflow pacing, not the agent

**Templates**: `{{GUIDE_NAME}}` required (error if missing), `{{?guide_name}}` optional

**State**: `.hegel/state.json` (workflow + current node + history)

---

## Development Constraints

**Platform**: macOS Apple Silicon (M1)
**Language**: Rust stable
**Dependencies**: Minimal (serde, anyhow, clap, fs2, ratatui/crossterm for future TUI)

## External Documentation Cache

**.webcache/ protocol** (gitignored):

**Cache**: `curl -s <url> -o .webcache/<topic>/<file>.html`
**Read**: `lynx -dump -nolist .webcache/<path>.html` (HTML → clean text)
**Prefer**: `cargo doc --no-deps -p <crate>` for Rust crates (always version-correct)

Use for offline access, version stability, faster lookup. Reference original URLs in docs, not .webcache paths.

---

**Remember**: Hegel orchestrates workflows through state transitions. Keep it simple, transparent, local-first.
