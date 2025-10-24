# Lines of Code Report

**Last Updated**: 2025-10-23 23:34
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 10,279 | 5,281 | 15,560 |
| **Comments** | 1,426 | - | 1,426 |
| **Blank Lines** | 1,885 | - | 1,885 |
| **Total Lines** | 13,590 | 5,281 | 18,871 |
| **Files** | 58 | 28 | 86 |

**Documentation Ratio**: 0.51 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            58           1885           1426          10279
-------------------------------------------------------------------------------
SUM:                            58           1885           1426          10279
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `adapters/claude_code.rs` | 255 | 126 | 129 | 50.6% | ✅ |
| `adapters/codex.rs` | 466 | 285 | 181 | 38.8% | ✅ (infra) |
| `adapters/cursor.rs` | 307 | 157 | 150 | 48.9% | ✅ |
| `adapters/mod.rs` | 124 | 124 | 0 | 0.0% | ✅ |
| `commands/analyze/mod.rs` | 182 | 28 | 154 | 84.6% | ✅ |
| `commands/analyze/sections.rs` | 267 | 267 | 0 | 0.0% | ✅ (infra) |
| `commands/astq.rs` | 113 | 88 | 25 | 22.1% | ✅ |
| `commands/config.rs` | 134 | 57 | 77 | 57.5% | ✅ |
| `commands/git.rs` | 48 | 0 | 48 | 100.0% | ✅ |
| `commands/hook.rs` | 236 | 108 | 128 | 54.2% | ✅ |
| `commands/hooks_setup.rs` | 235 | 118 | 117 | 49.8% | ✅ |
| `commands/init.rs` | 167 | 72 | 95 | 56.9% | ✅ |
| `commands/meta.rs` | 247 | 163 | 84 | 34.0% | ✅ |
| `commands/mod.rs` | 26 | 26 | 0 | 0.0% | ✅ |
| `commands/reflect.rs` | 132 | 83 | 49 | 37.1% | ✅ |
| `commands/workflow/claims.rs` | 28 | 28 | 0 | 0.0% | ✅ |
| `commands/workflow/context.rs` | 76 | 76 | 0 | 0.0% | ✅ |
| `commands/workflow/mod.rs` | 398 | 16 | 382 | 96.0% | ✅ |
| `commands/workflow/tests.rs` | 879 | 879 | 0 | 0.0% | ✅ (infra) |
| `commands/workflow/transitions.rs` | 276 | 276 | 0 | 0.0% | ✅ (infra) |
| `commands/wrapped.rs` | 131 | 68 | 63 | 48.1% | ✅ |
| `config.rs` | 147 | 102 | 45 | 30.6% | ✅ |
| `embedded.rs` | 90 | 90 | 0 | 0.0% | ✅ |
| `engine/mod.rs` | 819 | 169 | 650 | 79.4% | ✅ |
| `engine/template.rs` | 676 | 162 | 514 | 76.0% | ✅ |
| `guardrails/mod.rs` | 5 | 5 | 0 | 0.0% | ✅ |
| `guardrails/parser.rs` | 71 | 23 | 48 | 67.6% | ✅ |
| `guardrails/types.rs` | 170 | 97 | 73 | 42.9% | ✅ |
| `main.rs` | 265 | 13 | 252 | 95.1% | ✅ |
| `metamodes/mod.rs` | 204 | 99 | 105 | 51.5% | ✅ |
| `metrics/aggregation.rs` | 203 | 144 | 59 | 29.1% | ✅ |
| `metrics/graph.rs` | 372 | 224 | 148 | 39.8% | ✅ (infra) |
| `metrics/hooks.rs` | 330 | 220 | 110 | 33.3% | ✅ (infra) |
| `metrics/mod.rs` | 333 | 115 | 218 | 65.5% | ✅ |
| `metrics/states.rs` | 137 | 33 | 104 | 75.9% | ✅ |
| `metrics/transcript.rs` | 257 | 100 | 157 | 61.1% | ✅ |
| `rules/evaluator.rs` | 998 | 121 | 877 | 87.9% | ✅ |
| `rules/interrupt.rs` | 175 | 32 | 143 | 81.7% | ✅ |
| `rules/mod.rs` | 7 | 7 | 0 | 0.0% | ✅ |
| `rules/types.rs` | 299 | 74 | 225 | 75.3% | ✅ |
| `storage/mod.rs` | 698 | 311 | 387 | 55.4% | ✅ (infra) |
| `test_helpers/fixtures.rs` | 29 | 29 | 0 | 0.0% | ✅ |
| `test_helpers/jsonl.rs` | 124 | 124 | 0 | 0.0% | ✅ |
| `test_helpers/metrics.rs` | 272 | 272 | 0 | 0.0% | ✅ (infra) |
| `test_helpers/mod.rs` | 20 | 10 | 10 | 50.0% | ✅ |
| `test_helpers/storage.rs` | 66 | 66 | 0 | 0.0% | ✅ |
| `test_helpers/tui.rs` | 73 | 73 | 0 | 0.0% | ✅ |
| `test_helpers/workflow.rs` | 296 | 296 | 0 | 0.0% | ✅ (infra) |
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

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 149 |
| `CODE_MAP.md` | 204 |
| `commands/hegel.md` | 23 |
| `COVERAGE_REPORT.md` | 162 |
| `DEP_REVIEW.md` | 678 |
| `docs/astq_patterns/README.md` | 127 |
| `guides/ARCHITECTURE_WRITING.md` | 258 |
| `guides/CLAUDE_CUSTOMIZATION.md` | 312 |
| `guides/CODE_MAP_WRITING.md` | 108 |
| `guides/HANDOFF_WRITING.md` | 207 |
| `guides/KICKOFF_WRITING.md` | 96 |
| `guides/KNOWLEDGE_CAPTURE.md` | 345 |
| `guides/LEARNINGS_WRITING.md` | 96 |
| `guides/PLAN_WRITING.md` | 149 |
| `guides/QUESTION_TRACKING.md` | 362 |
| `guides/README_WRITING.md` | 142 |
| `guides/SPEC_WRITING.md` | 115 |
| `guides/STUDY_PLANNING.md` | 209 |
| `guides/templates/code_map_hierarchical.md` | 48 |
| `guides/templates/code_map_monolithic.md` | 40 |
| `guides/templates/mirror_workflow.md` | 11 |
| `guides/VISION_WRITING.md` | 176 |
| `HEGEL_CLAUDE.md` | 267 |
| `LEXICON.md` | 84 |
| `LOC_REPORT.md` | 155 |
| `README.md` | 535 |
| `ROADMAP.md` | 182 |
| `TESTING.md` | 41 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.51 | ✅ Excellent |
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
