# Lines of Code Report

**Last Updated**: 2025-11-02 22:09
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 12,630 | 7,846 | 20,476 |
| **Comments** | 1,749 | - | 1,749 |
| **Blank Lines** | 2,330 | - | 2,330 |
| **Total Lines** | 16,709 | 7,846 | 24,555 |
| **Files** | 74 | 36 | 110 |

**Documentation Ratio**: 0.62 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            74           2330           1749          12630
Markdown                         1             31              0             88
-------------------------------------------------------------------------------
SUM:                            75           2361           1749          12718
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `adapters/claude_code.rs` | 258 | 129 | 129 | 50.0% | ✅ |
| `adapters/codex.rs` | 466 | 285 | 181 | 38.8% | ✅ (infra) |
| `adapters/cursor.rs` | 307 | 157 | 150 | 48.9% | ✅ |
| `adapters/mod.rs` | 223 | 125 | 98 | 43.9% | ✅ |
| `commands/analyze/mod.rs` | 189 | 35 | 154 | 81.5% | ✅ |
| `commands/analyze/sections.rs` | 337 | 337 | 0 | 0.0% | ✅ (infra) |
| `commands/archive.rs` | 301 | 212 | 89 | 29.6% | ⚠️ Large |
| `commands/astq.rs` | 83 | 63 | 20 | 24.1% | ✅ |
| `commands/config.rs` | 134 | 57 | 77 | 57.5% | ✅ |
| `commands/fork/amp.rs` | 43 | 20 | 23 | 53.5% | ✅ |
| `commands/fork/codex.rs` | 46 | 20 | 26 | 56.5% | ✅ |
| `commands/fork/cody.rs` | 48 | 21 | 27 | 56.2% | ✅ |
| `commands/fork/gemini.rs` | 43 | 20 | 23 | 53.5% | ✅ |
| `commands/fork/generic.rs` | 37 | 20 | 17 | 45.9% | ✅ |
| `commands/fork/mod.rs` | 536 | 426 | 110 | 20.5% | ⚠️ Large |
| `commands/git.rs` | 48 | 0 | 48 | 100.0% | ✅ |
| `commands/hook.rs` | 236 | 108 | 128 | 54.2% | ✅ |
| `commands/hooks_setup.rs` | 235 | 118 | 117 | 49.8% | ✅ |
| `commands/init.rs` | 223 | 97 | 126 | 56.5% | ✅ |
| `commands/meta.rs` | 251 | 163 | 88 | 35.1% | ✅ |
| `commands/mod.rs` | 32 | 32 | 0 | 0.0% | ✅ |
| `commands/reflect.rs` | 132 | 83 | 49 | 37.1% | ✅ |
| `commands/status.rs` | 66 | 66 | 0 | 0.0% | ✅ |
| `commands/workflow/claims.rs` | 24 | 24 | 0 | 0.0% | ✅ |
| `commands/workflow/context.rs` | 76 | 76 | 0 | 0.0% | ✅ |
| `commands/workflow/mod.rs` | 386 | 16 | 370 | 95.9% | ✅ |
| `commands/workflow/tests/commands.rs` | 376 | 376 | 0 | 0.0% | ✅ (infra) |
| `commands/workflow/tests/integration.rs` | 64 | 64 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/mod.rs` | 69 | 69 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/node_flow.rs` | 137 | 137 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/production.rs` | 65 | 65 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/transitions.rs` | 345 | 345 | 0 | 0.0% | ✅ (infra) |
| `commands/workflow/transitions.rs` | 333 | 333 | 0 | 0.0% | ✅ (infra) |
| `commands/wrapped.rs` | 129 | 68 | 61 | 47.3% | ✅ |
| `config.rs` | 147 | 102 | 45 | 30.6% | ✅ |
| `embedded.rs` | 120 | 120 | 0 | 0.0% | ✅ |
| `engine/mod.rs` | 865 | 185 | 680 | 78.6% | ✅ |
| `engine/template.rs` | 676 | 162 | 514 | 76.0% | ✅ |
| `guardrails/mod.rs` | 5 | 5 | 0 | 0.0% | ✅ |
| `guardrails/parser.rs` | 71 | 23 | 48 | 67.6% | ✅ |
| `guardrails/types.rs` | 170 | 97 | 73 | 42.9% | ✅ |
| `lib.rs` | 14 | 12 | 2 | 14.3% | ✅ |
| `main.rs` | 322 | 13 | 309 | 96.0% | ✅ |
| `metamodes/mod.rs` | 204 | 99 | 105 | 51.5% | ✅ |
| `metrics/aggregation.rs` | 204 | 145 | 59 | 28.9% | ✅ |
| `metrics/git.rs` | 507 | 158 | 349 | 68.8% | ✅ |
| `metrics/graph.rs` | 372 | 222 | 150 | 40.3% | ✅ (infra) |
| `metrics/hooks.rs` | 326 | 216 | 110 | 33.7% | ✅ (infra) |
| `metrics/mod.rs` | 692 | 198 | 494 | 71.4% | ✅ |
| `metrics/states.rs` | 137 | 33 | 104 | 75.9% | ✅ |
| `metrics/transcript.rs` | 257 | 100 | 157 | 61.1% | ✅ |
| `rules/evaluator.rs` | 1,000 | 121 | 879 | 87.9% | ✅ |
| `rules/interrupt.rs` | 175 | 32 | 143 | 81.7% | ✅ |
| `rules/mod.rs` | 7 | 7 | 0 | 0.0% | ✅ |
| `rules/types.rs` | 299 | 74 | 225 | 75.3% | ✅ |
| `storage/archive.rs` | 537 | 306 | 231 | 43.0% | ⚠️ Large |
| `storage/mod.rs` | 702 | 313 | 389 | 55.4% | ✅ (infra) |
| `test_helpers/fixtures.rs` | 29 | 29 | 0 | 0.0% | ✅ |
| `test_helpers/jsonl.rs` | 125 | 125 | 0 | 0.0% | ✅ |
| `test_helpers/metrics.rs` | 277 | 277 | 0 | 0.0% | ✅ (infra) |
| `test_helpers/mod.rs` | 20 | 10 | 10 | 50.0% | ✅ |
| `test_helpers/storage.rs` | 65 | 65 | 0 | 0.0% | ✅ |
| `test_helpers/tui.rs` | 81 | 81 | 0 | 0.0% | ✅ |
| `test_helpers/workflow.rs` | 298 | 298 | 0 | 0.0% | ✅ (infra) |
| `theme.rs` | 96 | 66 | 30 | 31.2% | ✅ |
| `tui/app.rs` | 402 | 165 | 237 | 59.0% | ✅ |
| `tui/mod.rs` | 84 | 67 | 17 | 20.2% | ✅ |
| `tui/tabs/events.rs` | 132 | 113 | 19 | 14.4% | ✅ |
| `tui/tabs/files.rs` | 90 | 72 | 18 | 20.0% | ✅ |
| `tui/tabs/mod.rs` | 9 | 9 | 0 | 0.0% | ✅ |
| `tui/tabs/overview.rs` | 88 | 74 | 14 | 15.9% | ✅ |
| `tui/tabs/phases.rs` | 143 | 111 | 32 | 22.4% | ✅ |
| `tui/ui.rs` | 319 | 160 | 159 | 49.8% | ✅ |
| `tui/utils.rs` | 364 | 211 | 153 | 42.0% | ✅ (infra) |

**⚠️ Warning:** 3 file(s) over 200 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `.ddd/feat/done-node-refactor/PLAN.md` | 84 |
| `.ddd/feat/done-node-refactor/SPEC.md` | 80 |
| `.ddd/feat/git-commit-metrics/PLAN.md` | 402 |
| `.ddd/feat/git-commit-metrics/SPEC.md` | 545 |
| `.ddd/feat/log_retention/PLAN.md` | 535 |
| `.ddd/feat/log_retention/SPEC.md` | 366 |
| `CLAUDE.md` | 155 |
| `CODE_MAP.md` | 215 |
| `commands/hegel.md` | 23 |
| `COVERAGE_REPORT.md` | 180 |
| `DEP_REVIEW.md` | 678 |
| `docs/astq_patterns/README.md` | 127 |
| `guides/ARCHITECTURE_WRITING.md` | 258 |
| `guides/CLAUDE_CUSTOMIZATION.md` | 312 |
| `guides/CODE_MAP_WRITING.md` | 108 |
| `guides/HANDOFF_WRITING.md` | 207 |
| `guides/KICKOFF_WRITING.md` | 96 |
| `guides/KNOWLEDGE_CAPTURE.md` | 345 |
| `guides/LEARNINGS_WRITING.md` | 96 |
| `guides/PLAN_WRITING.md` | 157 |
| `guides/QUESTION_TRACKING.md` | 397 |
| `guides/README_WRITING.md` | 142 |
| `guides/SPEC_WRITING.md` | 121 |
| `guides/STUDY_PLANNING.md` | 209 |
| `guides/templates/code_map_hierarchical.md` | 48 |
| `guides/templates/code_map_monolithic.md` | 40 |
| `guides/templates/mirror_workflow.md` | 11 |
| `guides/VISION_WRITING.md` | 176 |
| `HEGEL_CLAUDE.md` | 286 |
| `LEXICON.md` | 84 |
| `LOC_REPORT.md` | 179 |
| `README.md` | 575 |
| `ROADMAP.md` | 182 |
| `src/test_helpers/README.md` | 119 |
| `TESTING.md` | 41 |
| `workflows/ANALYSIS.md` | 267 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.62 | ✅ Excellent |
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
