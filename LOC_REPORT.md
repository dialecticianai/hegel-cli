# Lines of Code Report

**Last Updated**: 2025-10-15 17:30
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 8,248 | 14,229 | 22,477 |
| **Comments** | 1,085 | - | 1,085 |
| **Blank Lines** | 1,457 | - | 1,457 |
| **Total Lines** | 10,790 | 14,229 | 25,019 |
| **Files** | 40 | 92 | 132 |

**Documentation Ratio**: 1.73 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            40           1457           1085           8248
-------------------------------------------------------------------------------
SUM:                            40           1457           1085           8248
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `adapters/claude_code.rs` | 255 | 126 | 129 | 50.6% | ✅ |
| `adapters/codex.rs` | 499 | 318 | 181 | 36.3% | ⚠️ Large |
| `adapters/cursor.rs` | 308 | 157 | 151 | 49.0% | ✅ |
| `adapters/mod.rs` | 120 | 120 | 0 | 0.0% | ✅ |
| `commands/analyze/mod.rs` | 182 | 28 | 154 | 84.6% | ✅ |
| `commands/analyze/sections.rs` | 253 | 253 | 0 | 0.0% | ⚠️ Large |
| `commands/astq.rs` | 45 | 45 | 0 | 0.0% | ✅ |
| `commands/git.rs` | 10 | 10 | 0 | 0.0% | ✅ |
| `commands/hook.rs` | 236 | 108 | 128 | 54.2% | ✅ |
| `commands/mod.rs` | 18 | 18 | 0 | 0.0% | ✅ |
| `commands/reflect.rs` | 82 | 82 | 0 | 0.0% | ✅ |
| `commands/workflow.rs` | 703 | 256 | 447 | 63.6% | ⚠️ Large |
| `commands/wrapped.rs` | 128 | 64 | 64 | 50.0% | ✅ |
| `engine/mod.rs` | 728 | 136 | 592 | 81.3% | ✅ |
| `engine/template.rs` | 351 | 87 | 264 | 75.2% | ✅ |
| `guardrails/mod.rs` | 5 | 5 | 0 | 0.0% | ✅ |
| `guardrails/parser.rs` | 71 | 23 | 48 | 67.6% | ✅ |
| `guardrails/types.rs` | 171 | 98 | 73 | 42.7% | ✅ |
| `main.rs` | 172 | 9 | 163 | 94.8% | ✅ |
| `metrics/aggregation.rs` | 204 | 144 | 60 | 29.4% | ✅ |
| `metrics/graph.rs` | 370 | 222 | 148 | 40.0% | ⚠️ Large |
| `metrics/hooks.rs` | 326 | 216 | 110 | 33.7% | ⚠️ Large |
| `metrics/mod.rs` | 333 | 115 | 218 | 65.5% | ✅ |
| `metrics/states.rs` | 137 | 33 | 104 | 75.9% | ✅ |
| `metrics/transcript.rs` | 257 | 100 | 157 | 61.1% | ✅ |
| `rules/evaluator.rs` | 1,491 | 115 | 1,376 | 92.3% | ✅ |
| `rules/interrupt.rs` | 175 | 32 | 143 | 81.7% | ✅ |
| `rules/mod.rs` | 7 | 7 | 0 | 0.0% | ✅ |
| `rules/types.rs` | 297 | 72 | 225 | 75.8% | ✅ |
| `storage/mod.rs` | 522 | 224 | 298 | 57.1% | ⚠️ Large |
| `test_helpers.rs` | 816 | 530 | 286 | 35.0% | ✅ (infra) |
| `tui/app.rs` | 402 | 165 | 237 | 59.0% | ✅ |
| `tui/mod.rs` | 66 | 66 | 0 | 0.0% | ✅ |
| `tui/tabs/events.rs` | 100 | 81 | 19 | 19.0% | ✅ |
| `tui/tabs/files.rs` | 81 | 63 | 18 | 22.2% | ✅ |
| `tui/tabs/mod.rs` | 9 | 9 | 0 | 0.0% | ✅ |
| `tui/tabs/overview.rs` | 88 | 74 | 14 | 15.9% | ✅ |
| `tui/tabs/phases.rs` | 135 | 103 | 32 | 23.7% | ✅ |
| `tui/ui.rs` | 317 | 158 | 159 | 50.2% | ✅ |
| `tui/utils.rs` | 320 | 187 | 133 | 41.6% | ✅ |

**⚠️ Warning:** 6 file(s) over 200 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 130 |
| `CODE_MAP.md` | 159 |
| `COVERAGE_REPORT.md` | 128 |
| `DEP_REVIEW.md` | 678 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 207 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `HEGEL_CLAUDE.md` | 522 |
| `LEXICON.md` | 84 |
| `LOC_REPORT.md` | 203 |
| `README.md` | 350 |
| `ROADMAP.md` | 182 |
| `TESTING.md` | 41 |
| `vendor/ast-grep/.github/CONTRIBUTING.md` | 8 |
| `vendor/ast-grep/.github/copilot-instructions.md` | 135 |
| `vendor/ast-grep/.github/ISSUE_TEMPLATE/feature_request.md` | 20 |
| `vendor/ast-grep/CHANGELOG.md` | 1,700 |
| `vendor/ast-grep/crates/napi/npm/darwin-arm64/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/darwin-x64/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/linux-arm64-gnu/README.md` | 2 |
| `vendor/ast-grep/crates/napi/npm/linux-arm64-musl/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/linux-x64-gnu/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/linux-x64-musl/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/win32-arm64-msvc/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/win32-ia32-msvc/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/win32-x64-msvc/README.md` | 3 |
| `vendor/ast-grep/crates/napi/README.md` | 34 |
| `vendor/ast-grep/crates/pyo3/README.md` | 73 |
| `vendor/ast-grep/npm/platforms/darwin-arm64/README.md` | 3 |
| `vendor/ast-grep/npm/platforms/darwin-x64/README.md` | 3 |
| `vendor/ast-grep/npm/platforms/linux-arm64-gnu/README.md` | 3 |
| `vendor/ast-grep/npm/platforms/linux-x64-gnu/README.md` | 3 |
| `vendor/ast-grep/npm/platforms/win32-arm64-msvc/README.md` | 3 |
| `vendor/ast-grep/npm/platforms/win32-ia32-msvc/README.md` | 3 |
| `vendor/ast-grep/npm/platforms/win32-x64-msvc/README.md` | 3 |
| `vendor/ast-grep/npm/README.md` | 11 |
| `vendor/ast-grep/README.md` | 118 |
| `vendor/ccusage/.claude/commands/analyze-code.md` | 268 |
| `vendor/ccusage/.claude/commands/lsmcp-onboarding.md` | 1 |
| `vendor/ccusage/.claude/commands/reduce-similarities.md` | 1 |
| `vendor/ccusage/.claude/commands/refactor.md` | 78 |
| `vendor/ccusage/.lsmcp/memories/symbol_index_info.md` | 39 |
| `vendor/ccusage/.lsmcp/memories/symbol_index_status.md` | 56 |
| `vendor/ccusage/AGENTS.md` | 351 |
| `vendor/ccusage/apps/ccusage/AGENTS.md` | 120 |
| `vendor/ccusage/apps/ccusage/CLAUDE.md` | 120 |
| `vendor/ccusage/apps/ccusage/README.md` | 182 |
| `vendor/ccusage/apps/codex/AGENTS.md` | 58 |
| `vendor/ccusage/apps/codex/CLAUDE.md` | 58 |
| `vendor/ccusage/apps/codex/README.md` | 116 |
| `vendor/ccusage/apps/mcp/AGENTS.md` | 120 |
| `vendor/ccusage/apps/mcp/CLAUDE.md` | 120 |
| `vendor/ccusage/apps/mcp/README.md` | 82 |
| `vendor/ccusage/CLAUDE.md` | 351 |
| `vendor/ccusage/docs/AGENTS.md` | 113 |
| `vendor/ccusage/docs/CLAUDE.md` | 113 |
| `vendor/ccusage/docs/guide/blocks-reports.md` | 357 |
| `vendor/ccusage/docs/guide/cli-options.md` | 344 |
| `vendor/ccusage/docs/guide/codex/daily.md` | 26 |
| `vendor/ccusage/docs/guide/codex/index.md` | 84 |
| `vendor/ccusage/docs/guide/codex/monthly.md` | 26 |
| `vendor/ccusage/docs/guide/codex/session.md` | 28 |
| `vendor/ccusage/docs/guide/config-files.md` | 432 |
| `vendor/ccusage/docs/guide/configuration.md` | 336 |
| `vendor/ccusage/docs/guide/cost-modes.md` | 348 |
| `vendor/ccusage/docs/guide/custom-paths.md` | 430 |
| `vendor/ccusage/docs/guide/daily-reports.md` | 284 |
| `vendor/ccusage/docs/guide/directory-detection.md` | 115 |
| `vendor/ccusage/docs/guide/environment-variables.md` | 243 |
| `vendor/ccusage/docs/guide/getting-started.md` | 141 |
| `vendor/ccusage/docs/guide/index.md` | 102 |
| `vendor/ccusage/docs/guide/installation.md` | 288 |
| `vendor/ccusage/docs/guide/json-output.md` | 447 |
| `vendor/ccusage/docs/guide/library-usage.md` | 150 |
| `vendor/ccusage/docs/guide/live-monitoring.md` | 275 |
| `vendor/ccusage/docs/guide/mcp-server.md` | 165 |
| `vendor/ccusage/docs/guide/monthly-reports.md` | 243 |
| `vendor/ccusage/docs/guide/related-projects.md` | 23 |
| `vendor/ccusage/docs/guide/session-reports.md` | 321 |
| `vendor/ccusage/docs/guide/sponsors.md` | 33 |
| `vendor/ccusage/docs/guide/statusline.md` | 284 |
| `vendor/ccusage/docs/guide/weekly-reports.md` | 230 |
| `vendor/ccusage/docs/index.md` | 84 |
| `vendor/ccusage/packages/internal/AGENTS.md` | 105 |
| `vendor/ccusage/packages/internal/CLAUDE.md` | 105 |
| `vendor/ccusage/packages/terminal/AGENTS.md` | 74 |
| `vendor/ccusage/packages/terminal/CLAUDE.md` | 74 |
| `vendor/ccusage/README.md` | 182 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 1.73 | ✅ Excellent |
| README exists | Yes | ✅ | Met |
| Architecture docs | Yes | ❌ | Optional |

---

## How to Update This Report

```bash
# Regenerate LOC report
./scripts/generate-loc-report.sh
```

---

*This report is auto-generated from `cloc` and `wc` output.*
*Updated automatically by pre-commit hook when source files change.*
