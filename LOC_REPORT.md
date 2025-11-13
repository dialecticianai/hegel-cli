# Lines of Code Report

**Last Updated**: 2025-11-12 20:12
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 19,224 | 14,893 | 34,117 |
| **Comments** | 2,846 | - | 2,846 |
| **Blank Lines** | 3,637 | - | 3,637 |
| **Total Lines** | 25,707 | 14,893 | 40,600 |
| **Files** | 119 | 77 | 196 |

**Documentation Ratio**: 0.77 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                           119           3637           2846          19224
Markdown                        15            147              0            469
-------------------------------------------------------------------------------
SUM:                           134           3784           2846          19693
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `adapters/claude_code.rs` | 321 | 192 | 129 | 40.2% | ✅ |
| `adapters/codex.rs` | 466 | 285 | 181 | 38.8% | ✅ |
| `adapters/cursor.rs` | 307 | 157 | 150 | 48.9% | ✅ |
| `adapters/mod.rs` | 223 | 125 | 98 | 43.9% | ✅ |
| `analyze/cleanup/aborted.rs` | 159 | 61 | 98 | 61.6% | ✅ |
| `analyze/cleanup/duplicate_cowboy.rs` | 263 | 158 | 105 | 39.9% | ✅ |
| `analyze/cleanup/git.rs` | 90 | 90 | 0 | 0.0% | ✅ |
| `analyze/cleanup/mod.rs` | 75 | 75 | 0 | 0.0% | ✅ |
| `analyze/gap_detection.rs` | 365 | 365 | 0 | 0.0% | ✅ |
| `analyze/mod.rs` | 8 | 6 | 2 | 25.0% | ✅ |
| `analyze/repair.rs` | 362 | 362 | 0 | 0.0% | ✅ |
| `analyze/sections.rs` | 370 | 370 | 0 | 0.0% | ✅ |
| `analyze/tests/gap_detection.rs` | 346 | 346 | 0 | 0.0% | ✅ |
| `analyze/tests/mod.rs` | 1 | 1 | 0 | 0.0% | ✅ |
| `analyze/totals.rs` | 32 | 32 | 0 | 0.0% | ✅ |
| `commands/analyze/mod.rs` | 363 | 111 | 252 | 69.4% | ✅ |
| `commands/archive.rs` | 385 | 296 | 89 | 23.1% | ✅ |
| `commands/astq.rs` | 83 | 63 | 20 | 24.1% | ✅ |
| `commands/config.rs` | 134 | 57 | 77 | 57.5% | ✅ |
| `commands/doctor/mod.rs` | 259 | 0 | 259 | 100.0% | ✅ |
| `commands/doctor/tests.rs` | 250 | 250 | 0 | 0.0% | ✅ |
| `commands/external_bin.rs` | 185 | 154 | 31 | 16.8% | ✅ |
| `commands/fork/amp.rs` | 43 | 20 | 23 | 53.5% | ✅ |
| `commands/fork/codex.rs` | 46 | 20 | 26 | 56.5% | ✅ |
| `commands/fork/cody.rs` | 48 | 21 | 27 | 56.2% | ✅ |
| `commands/fork/gemini.rs` | 43 | 20 | 23 | 53.5% | ✅ |
| `commands/fork/generic.rs` | 37 | 20 | 17 | 45.9% | ✅ |
| `commands/fork/mod.rs` | 304 | 248 | 56 | 18.4% | ✅ |
| `commands/fork/runtime.rs` | 251 | 193 | 58 | 23.1% | ✅ |
| `commands/git.rs` | 48 | 0 | 48 | 100.0% | ✅ |
| `commands/hook.rs` | 236 | 108 | 128 | 54.2% | ✅ |
| `commands/hooks_setup.rs` | 235 | 118 | 117 | 49.8% | ✅ |
| `commands/ide.rs` | 27 | 15 | 12 | 44.4% | ✅ |
| `commands/init.rs` | 223 | 97 | 126 | 56.5% | ✅ |
| `commands/markdown.rs` | 495 | 403 | 92 | 18.6% | ⚠️ Large |
| `commands/meta.rs` | 255 | 167 | 88 | 34.5% | ✅ |
| `commands/mod.rs` | 43 | 43 | 0 | 0.0% | ✅ |
| `commands/pm.rs` | 30 | 15 | 15 | 50.0% | ✅ |
| `commands/reflect.rs` | 120 | 95 | 25 | 20.8% | ✅ |
| `commands/review.rs` | 498 | 231 | 267 | 53.6% | ✅ |
| `commands/status.rs` | 140 | 140 | 0 | 0.0% | ✅ |
| `commands/workflow/claims.rs` | 24 | 24 | 0 | 0.0% | ✅ |
| `commands/workflow/context.rs` | 79 | 79 | 0 | 0.0% | ✅ |
| `commands/workflow/mod.rs` | 714 | 18 | 696 | 97.5% | ✅ |
| `commands/workflow/tests/archiving_bug_repro.rs` | 210 | 210 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/commands.rs` | 434 | 434 | 0 | 0.0% | ✅ (infra) |
| `commands/workflow/tests/integration.rs` | 60 | 60 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/mod.rs` | 83 | 83 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/node_flow.rs` | 135 | 135 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/production.rs` | 65 | 65 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/stash.rs` | 234 | 234 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/transitions.rs` | 518 | 518 | 0 | 0.0% | ✅ (infra) |
| `commands/workflow/transitions.rs` | 606 | 606 | 0 | 0.0% | ✅ (infra) |
| `commands/wrapped.rs` | 129 | 68 | 61 | 47.3% | ✅ |
| `config.rs` | 188 | 139 | 49 | 26.1% | ✅ |
| `doctor/migrations.rs` | 129 | 129 | 0 | 0.0% | ✅ |
| `doctor/mod.rs` | 5 | 5 | 0 | 0.0% | ✅ |
| `doctor/rescue.rs` | 67 | 67 | 0 | 0.0% | ✅ |
| `embedded.rs` | 145 | 125 | 20 | 13.8% | ✅ |
| `engine/handlebars.rs` | 159 | 159 | 0 | 0.0% | ✅ |
| `engine/mod.rs` | 335 | 326 | 9 | 2.7% | ✅ |
| `engine/template.rs` | 161 | 161 | 0 | 0.0% | ✅ |
| `engine/tests/handlebars.rs` | 351 | 351 | 0 | 0.0% | ✅ |
| `engine/tests/integration.rs` | 365 | 365 | 0 | 0.0% | ✅ |
| `engine/tests/mod.rs` | 6 | 6 | 0 | 0.0% | ✅ |
| `engine/tests/navigation.rs` | 204 | 204 | 0 | 0.0% | ✅ |
| `engine/tests/rules.rs` | 258 | 258 | 0 | 0.0% | ✅ |
| `engine/tests/template.rs` | 514 | 514 | 0 | 0.0% | ⚠️ Large |
| `engine/tests/workflow.rs` | 210 | 210 | 0 | 0.0% | ✅ |
| `guardrails/mod.rs` | 5 | 5 | 0 | 0.0% | ✅ |
| `guardrails/parser.rs` | 71 | 23 | 48 | 67.6% | ✅ |
| `guardrails/types.rs` | 170 | 97 | 73 | 42.9% | ✅ |
| `lib.rs` | 15 | 13 | 2 | 13.3% | ✅ |
| `main.rs` | 541 | 15 | 526 | 97.2% | ✅ |
| `metamodes/mod.rs` | 206 | 101 | 105 | 51.0% | ✅ |
| `metrics/aggregation.rs` | 408 | 348 | 60 | 14.7% | ✅ |
| `metrics/cowboy.rs` | 199 | 116 | 83 | 41.7% | ✅ |
| `metrics/git.rs` | 157 | 157 | 0 | 0.0% | ✅ |
| `metrics/graph.rs` | 459 | 287 | 172 | 37.5% | ✅ |
| `metrics/hooks.rs` | 326 | 216 | 110 | 33.7% | ✅ |
| `metrics/mod.rs` | 377 | 372 | 5 | 1.3% | ✅ |
| `metrics/states.rs` | 137 | 33 | 104 | 75.9% | ✅ |
| `metrics/tests/git.rs` | 337 | 337 | 0 | 0.0% | ✅ |
| `metrics/tests/mod.rs` | 2 | 2 | 0 | 0.0% | ✅ |
| `metrics/tests/unified.rs` | 538 | 538 | 0 | 0.0% | ⚠️ Large |
| `metrics/transcript.rs` | 259 | 102 | 157 | 60.6% | ✅ |
| `rules/evaluator.rs` | 390 | 390 | 0 | 0.0% | ✅ |
| `rules/interrupt.rs` | 175 | 32 | 143 | 81.7% | ✅ |
| `rules/mod.rs` | 10 | 4 | 6 | 60.0% | ✅ |
| `rules/tests/evaluator.rs` | 919 | 919 | 0 | 0.0% | ⚠️ Large |
| `rules/tests/mod.rs` | 1 | 1 | 0 | 0.0% | ✅ |
| `rules/types.rs` | 375 | 88 | 287 | 76.5% | ✅ |
| `storage/archive/aggregation.rs` | 98 | 98 | 0 | 0.0% | ✅ |
| `storage/archive/builder.rs` | 151 | 87 | 64 | 42.4% | ✅ |
| `storage/archive/mod.rs` | 348 | 160 | 188 | 54.0% | ✅ |
| `storage/archive/validation.rs` | 41 | 22 | 19 | 46.3% | ✅ |
| `storage/log_cleanup.rs` | 26 | 26 | 0 | 0.0% | ✅ |
| `storage/mod.rs` | 687 | 671 | 16 | 2.3% | ✅ (infra) |
| `storage/reviews.rs` | 288 | 141 | 147 | 51.0% | ✅ |
| `storage/tests/mod.rs` | 1 | 1 | 0 | 0.0% | ✅ |
| `storage/tests/storage.rs` | 624 | 624 | 0 | 0.0% | ⚠️ Large |
| `test_helpers/archive.rs` | 137 | 137 | 0 | 0.0% | ✅ |
| `test_helpers/fixtures.rs` | 29 | 29 | 0 | 0.0% | ✅ |
| `test_helpers/jsonl.rs` | 125 | 125 | 0 | 0.0% | ✅ |
| `test_helpers/metrics.rs` | 326 | 326 | 0 | 0.0% | ✅ |
| `test_helpers/mod.rs` | 22 | 11 | 11 | 50.0% | ✅ |
| `test_helpers/storage.rs` | 67 | 67 | 0 | 0.0% | ✅ |
| `test_helpers/tui.rs` | 81 | 81 | 0 | 0.0% | ✅ |
| `test_helpers/workflow.rs` | 294 | 294 | 0 | 0.0% | ✅ |
| `theme.rs` | 96 | 66 | 30 | 31.2% | ✅ |
| `tui/app.rs` | 403 | 166 | 237 | 58.8% | ✅ |
| `tui/mod.rs` | 84 | 67 | 17 | 20.2% | ✅ |
| `tui/tabs/events.rs` | 132 | 113 | 19 | 14.4% | ✅ |
| `tui/tabs/files.rs` | 90 | 72 | 18 | 20.0% | ✅ |
| `tui/tabs/mod.rs` | 9 | 9 | 0 | 0.0% | ✅ |
| `tui/tabs/overview.rs` | 88 | 74 | 14 | 15.9% | ✅ |
| `tui/tabs/phases.rs` | 143 | 111 | 32 | 22.4% | ✅ |
| `tui/ui.rs` | 319 | 160 | 159 | 49.8% | ✅ |
| `tui/utils.rs` | 364 | 211 | 153 | 42.0% | ✅ |

**⚠️ Warning:** 5 file(s) over 400 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `.ddd/feat/20251104-1-non_phase_commits/PLAN.md` | 374 |
| `.ddd/feat/20251104-1-non_phase_commits/SPEC.md` | 383 |
| `.ddd/feat/20251104-2-done_node_refactor/PLAN.md` | 84 |
| `.ddd/feat/20251104-2-done_node_refactor/SPEC.md` | 80 |
| `.ddd/feat/20251104-3-git_commit_metrics/PLAN.md` | 402 |
| `.ddd/feat/20251104-3-git_commit_metrics/SPEC.md` | 545 |
| `.ddd/feat/20251104-4-log_retention/PLAN.md` | 535 |
| `.ddd/feat/20251104-4-log_retention/SPEC.md` | 366 |
| `.ddd/feat/20251105-handlebars_templates/PLAN.md` | 273 |
| `.ddd/feat/20251105-handlebars_templates/SPEC.md` | 366 |
| `.ddd/feat/20251106-analyze_summary/PLAN.md` | 175 |
| `.ddd/feat/20251106-analyze_summary/SPEC.md` | 215 |
| `.ddd/feat/commit-guardrails/PLAN.md` | 307 |
| `.ddd/feat/commit-guardrails/SPEC.md` | 375 |
| `.ddd/feat/markdown-tree/PLAN.md` | 289 |
| `.ddd/feat/markdown-tree/SPEC.md` | 308 |
| `.ddd/feat/review-cli/PLAN.md` | 121 |
| `.ddd/feat/review-cli/SPEC.md` | 249 |
| `.ddd/feat/reviews-module/PLAN.md` | 133 |
| `.ddd/feat/reviews-module/SPEC.md` | 231 |
| `.ddd/feat/workflow-stash/PLAN.md` | 298 |
| `.ddd/feat/workflow-stash/SPEC.md` | 293 |
| `.ddd/PLAN.md` | 210 |
| `.ddd/refactor/20251104-large_files.md` | 261 |
| `.ddd/refactor/20251105-workflow_graph_grouping.md` | 104 |
| `.ddd/refactor/20251106-multi_session_token_attribution.md` | 370 |
| `.ddd/refactor/20251110-test_extraction.md` | 193 |
| `.ddd/reports/20251010-tui_dep_review.md` | 678 |
| `CLAUDE.md` | 198 |
| `commands/hegel.md` | 24 |
| `COVERAGE_REPORT.md` | 230 |
| `docs/ADVANCED_TOOLS.md` | 86 |
| `docs/astq_patterns/README.md` | 127 |
| `docs/CUSTOMIZING.md` | 109 |
| `docs/GUARDRAILS.md` | 53 |
| `docs/integrations/CLAUDE_CODE.md` | 91 |
| `docs/MD_REVIEW.md` | 95 |
| `docs/ONBOARDING_STATE_METRICS.md` | 297 |
| `docs/STATE.md` | 33 |
| `guides/ARCHITECTURE_WRITING.md` | 258 |
| `guides/CLAUDE_CUSTOMIZATION.md` | 312 |
| `guides/CODE_MAP_WRITING.md` | 152 |
| `guides/HANDOFF_WRITING.md` | 207 |
| `guides/KICKOFF_WRITING.md` | 96 |
| `guides/KNOWLEDGE_CAPTURE.md` | 345 |
| `guides/LEARNINGS_WRITING.md` | 96 |
| `guides/LEXICON.md` | 90 |
| `guides/PLAN_WRITING.md` | 181 |
| `guides/QUESTION_TRACKING.md` | 397 |
| `guides/README_WRITING.md` | 55 |
| `guides/SPEC_WRITING.md` | 196 |
| `guides/STUDY_PLANNING.md` | 209 |
| `guides/templates/code_map_hierarchical.md` | 57 |
| `guides/templates/code_map_monolithic.md` | 41 |
| `guides/templates/mirror_workflow.md` | 11 |
| `guides/VISION_WRITING.md` | 176 |
| `HEGEL_CLAUDE.md` | 349 |
| `LOC_REPORT.md` | 267 |
| `METRICS.md` | 258 |
| `README.md` | 443 |
| `ROADMAP.md` | 450 |
| `src/adapters/README.md` | 25 |
| `src/analyze/cleanup/README.md` | 22 |
| `src/analyze/README.md` | 28 |
| `src/commands/fork/README.md` | 30 |
| `src/commands/README.md` | 46 |
| `src/commands/workflow/README.md` | 34 |
| `src/engine/README.md` | 49 |
| `src/guardrails/README.md` | 34 |
| `src/metamodes/README.md` | 25 |
| `src/metrics/README.md` | 34 |
| `src/README.md` | 35 |
| `src/rules/README.md` | 45 |
| `src/storage/README.md` | 55 |
| `src/test_helpers/README.md` | 122 |
| `src/tui/README.md` | 32 |
| `TESTING.md` | 70 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.77 | ✅ Excellent |
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
