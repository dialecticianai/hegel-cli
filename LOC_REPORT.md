# Lines of Code Report

**Last Updated**: 2025-11-05 23:49
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 15,482 | 10,235 | 25,717 |
| **Comments** | 2,271 | - | 2,271 |
| **Blank Lines** | 2,885 | - | 2,885 |
| **Total Lines** | 20,638 | 10,235 | 30,873 |
| **Files** | 92 | 56 | 148 |

**Documentation Ratio**: 0.66 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            92           2885           2271          15482
Markdown                        15            144              0            429
-------------------------------------------------------------------------------
SUM:                           107           3029           2271          15911
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
| `analyze/cleanup/aborted.rs` | 162 | 64 | 98 | 60.5% | ✅ |
| `analyze/cleanup/duplicate_cowboy.rs` | 263 | 158 | 105 | 39.9% | ✅ |
| `analyze/cleanup/git.rs` | 90 | 90 | 0 | 0.0% | ✅ |
| `analyze/cleanup/mod.rs` | 75 | 75 | 0 | 0.0% | ✅ |
| `analyze/gap_detection.rs` | 293 | 293 | 0 | 0.0% | ⚠️ Large |
| `analyze/mod.rs` | 5 | 5 | 0 | 0.0% | ✅ |
| `analyze/repair.rs` | 362 | 362 | 0 | 0.0% | ⚠️ Large |
| `analyze/sections.rs` | 369 | 369 | 0 | 0.0% | ⚠️ Large |
| `analyze/totals.rs` | 32 | 32 | 0 | 0.0% | ✅ |
| `commands/analyze/mod.rs` | 337 | 87 | 250 | 74.2% | ✅ |
| `commands/archive.rs` | 384 | 295 | 89 | 23.2% | ⚠️ Large |
| `commands/astq.rs` | 83 | 63 | 20 | 24.1% | ✅ |
| `commands/config.rs` | 134 | 57 | 77 | 57.5% | ✅ |
| `commands/external_bin.rs` | 91 | 74 | 17 | 18.7% | ✅ |
| `commands/fork/amp.rs` | 43 | 20 | 23 | 53.5% | ✅ |
| `commands/fork/codex.rs` | 46 | 20 | 26 | 56.5% | ✅ |
| `commands/fork/cody.rs` | 48 | 21 | 27 | 56.2% | ✅ |
| `commands/fork/gemini.rs` | 43 | 20 | 23 | 53.5% | ✅ |
| `commands/fork/generic.rs` | 37 | 20 | 17 | 45.9% | ✅ |
| `commands/fork/mod.rs` | 304 | 248 | 56 | 18.4% | ⚠️ Large |
| `commands/fork/runtime.rs` | 251 | 193 | 58 | 23.1% | ✅ |
| `commands/git.rs` | 48 | 0 | 48 | 100.0% | ✅ |
| `commands/hook.rs` | 236 | 108 | 128 | 54.2% | ✅ |
| `commands/hooks_setup.rs` | 235 | 118 | 117 | 49.8% | ✅ |
| `commands/init.rs` | 223 | 97 | 126 | 56.5% | ✅ |
| `commands/meta.rs` | 254 | 166 | 88 | 34.6% | ✅ |
| `commands/mod.rs` | 35 | 35 | 0 | 0.0% | ✅ |
| `commands/pm.rs` | 30 | 15 | 15 | 50.0% | ✅ |
| `commands/reflect.rs` | 67 | 42 | 25 | 37.3% | ✅ |
| `commands/status.rs` | 131 | 131 | 0 | 0.0% | ✅ |
| `commands/workflow/claims.rs` | 24 | 24 | 0 | 0.0% | ✅ |
| `commands/workflow/context.rs` | 81 | 81 | 0 | 0.0% | ✅ |
| `commands/workflow/mod.rs` | 546 | 17 | 529 | 96.9% | ✅ |
| `commands/workflow/tests/commands.rs` | 460 | 460 | 0 | 0.0% | ✅ (infra) |
| `commands/workflow/tests/integration.rs` | 64 | 64 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/mod.rs` | 69 | 69 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/node_flow.rs` | 137 | 137 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/production.rs` | 65 | 65 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/transitions.rs` | 504 | 504 | 0 | 0.0% | ✅ (infra) |
| `commands/workflow/transitions.rs` | 557 | 557 | 0 | 0.0% | ✅ (infra) |
| `commands/wrapped.rs` | 129 | 68 | 61 | 47.3% | ✅ |
| `config.rs` | 147 | 102 | 45 | 30.6% | ✅ |
| `embedded.rs` | 145 | 125 | 20 | 13.8% | ✅ |
| `engine/handlebars.rs` | 512 | 160 | 352 | 68.8% | ✅ |
| `engine/mod.rs` | 1,088 | 250 | 838 | 77.0% | ⚠️ Large |
| `engine/template.rs` | 676 | 162 | 514 | 76.0% | ✅ |
| `guardrails/mod.rs` | 5 | 5 | 0 | 0.0% | ✅ |
| `guardrails/parser.rs` | 71 | 23 | 48 | 67.6% | ✅ |
| `guardrails/types.rs` | 170 | 97 | 73 | 42.9% | ✅ |
| `lib.rs` | 15 | 13 | 2 | 13.3% | ✅ |
| `main.rs` | 403 | 14 | 389 | 96.5% | ✅ |
| `metamodes/mod.rs` | 204 | 99 | 105 | 51.5% | ✅ |
| `metrics/aggregation.rs` | 206 | 147 | 59 | 28.6% | ✅ |
| `metrics/cowboy.rs` | 199 | 116 | 83 | 41.7% | ✅ |
| `metrics/git.rs` | 497 | 158 | 339 | 68.2% | ✅ |
| `metrics/graph.rs` | 459 | 287 | 172 | 37.5% | ✅ (infra) |
| `metrics/hooks.rs` | 326 | 216 | 110 | 33.7% | ✅ (infra) |
| `metrics/mod.rs` | 768 | 226 | 542 | 70.6% | ⚠️ Large |
| `metrics/states.rs` | 137 | 33 | 104 | 75.9% | ✅ |
| `metrics/transcript.rs` | 257 | 100 | 157 | 61.1% | ✅ |
| `rules/evaluator.rs` | 1,000 | 121 | 879 | 87.9% | ✅ |
| `rules/interrupt.rs` | 175 | 32 | 143 | 81.7% | ✅ |
| `rules/mod.rs` | 7 | 7 | 0 | 0.0% | ✅ |
| `rules/types.rs` | 299 | 74 | 225 | 75.3% | ✅ |
| `storage/archive/aggregation.rs` | 98 | 98 | 0 | 0.0% | ✅ |
| `storage/archive/builder.rs` | 151 | 87 | 64 | 42.4% | ✅ |
| `storage/archive/mod.rs` | 348 | 160 | 188 | 54.0% | ✅ |
| `storage/archive/validation.rs` | 41 | 22 | 19 | 46.3% | ✅ |
| `storage/log_cleanup.rs` | 26 | 26 | 0 | 0.0% | ✅ |
| `storage/mod.rs` | 794 | 403 | 391 | 49.2% | ✅ (infra) |
| `test_helpers/archive.rs` | 135 | 135 | 0 | 0.0% | ✅ |
| `test_helpers/fixtures.rs` | 29 | 29 | 0 | 0.0% | ✅ |
| `test_helpers/jsonl.rs` | 125 | 125 | 0 | 0.0% | ✅ |
| `test_helpers/metrics.rs` | 326 | 326 | 0 | 0.0% | ✅ (infra) |
| `test_helpers/mod.rs` | 22 | 11 | 11 | 50.0% | ✅ |
| `test_helpers/storage.rs` | 68 | 68 | 0 | 0.0% | ✅ |
| `test_helpers/tui.rs` | 81 | 81 | 0 | 0.0% | ✅ |
| `test_helpers/workflow.rs` | 299 | 299 | 0 | 0.0% | ✅ (infra) |
| `theme.rs` | 96 | 66 | 30 | 31.2% | ✅ |
| `tui/app.rs` | 403 | 166 | 237 | 58.8% | ✅ |
| `tui/mod.rs` | 84 | 67 | 17 | 20.2% | ✅ |
| `tui/tabs/events.rs` | 132 | 113 | 19 | 14.4% | ✅ |
| `tui/tabs/files.rs` | 90 | 72 | 18 | 20.0% | ✅ |
| `tui/tabs/mod.rs` | 9 | 9 | 0 | 0.0% | ✅ |
| `tui/tabs/overview.rs` | 88 | 74 | 14 | 15.9% | ✅ |
| `tui/tabs/phases.rs` | 143 | 111 | 32 | 22.4% | ✅ |
| `tui/ui.rs` | 319 | 160 | 159 | 49.8% | ✅ |
| `tui/utils.rs` | 364 | 211 | 153 | 42.0% | ✅ (infra) |

**⚠️ Warning:** 7 file(s) over 200 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `.ddd/feat/analyze_summary_default_PLAN.md` | 175 |
| `.ddd/feat/analyze_summary_default.md` | 215 |
| `.ddd/feat/done-node-refactor/PLAN.md` | 84 |
| `.ddd/feat/done-node-refactor/SPEC.md` | 80 |
| `.ddd/feat/git-commit-metrics/PLAN.md` | 402 |
| `.ddd/feat/git-commit-metrics/SPEC.md` | 545 |
| `.ddd/feat/handlebars_templates/PLAN.md` | 273 |
| `.ddd/feat/handlebars_templates/SPEC.md` | 366 |
| `.ddd/feat/log_retention/PLAN.md` | 535 |
| `.ddd/feat/log_retention/SPEC.md` | 366 |
| `.ddd/feat/non-phase-commits/PLAN.md` | 374 |
| `.ddd/feat/non-phase-commits/SPEC.md` | 383 |
| `.ddd/refactor/20251104-large_files.md` | 261 |
| `.ddd/refactor/20251105-workflow_graph_grouping.md` | 104 |
| `CLAUDE.md` | 185 |
| `commands/hegel.md` | 24 |
| `COVERAGE_REPORT.md` | 214 |
| `DEP_REVIEW.md` | 678 |
| `docs/astq_patterns/README.md` | 127 |
| `guides/ARCHITECTURE_WRITING.md` | 258 |
| `guides/CLAUDE_CUSTOMIZATION.md` | 312 |
| `guides/CODE_MAP_WRITING.md` | 152 |
| `guides/HANDOFF_WRITING.md` | 207 |
| `guides/KICKOFF_WRITING.md` | 96 |
| `guides/KNOWLEDGE_CAPTURE.md` | 345 |
| `guides/LEARNINGS_WRITING.md` | 96 |
| `guides/LEXICON.md` | 84 |
| `guides/PLAN_WRITING.md` | 165 |
| `guides/QUESTION_TRACKING.md` | 397 |
| `guides/README_WRITING.md` | 55 |
| `guides/SPEC_WRITING.md` | 209 |
| `guides/STUDY_PLANNING.md` | 209 |
| `guides/templates/code_map_hierarchical.md` | 57 |
| `guides/templates/code_map_monolithic.md` | 41 |
| `guides/templates/mirror_workflow.md` | 11 |
| `guides/VISION_WRITING.md` | 176 |
| `HEGEL_CLAUDE.md` | 306 |
| `LOC_REPORT.md` | 219 |
| `README.md` | 653 |
| `ROADMAP.md` | 182 |
| `src/adapters/README.md` | 25 |
| `src/analyze/cleanup/README.md` | 22 |
| `src/analyze/README.md` | 26 |
| `src/commands/fork/README.md` | 30 |
| `src/commands/README.md` | 43 |
| `src/commands/workflow/README.md` | 32 |
| `src/engine/README.md` | 38 |
| `src/guardrails/README.md` | 34 |
| `src/metamodes/README.md` | 25 |
| `src/metrics/README.md` | 34 |
| `src/README.md` | 35 |
| `src/rules/README.md` | 30 |
| `src/storage/README.md` | 45 |
| `src/test_helpers/README.md` | 122 |
| `src/tui/README.md` | 32 |
| `TESTING.md` | 41 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.66 | ✅ Excellent |
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
