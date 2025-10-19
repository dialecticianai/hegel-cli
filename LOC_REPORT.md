# Lines of Code Report

**Last Updated**: 2025-10-19 17:22
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 9,582 | 5,378 | 14,960 |
| **Comments** | 1,336 | - | 1,336 |
| **Blank Lines** | 1,773 | - | 1,773 |
| **Total Lines** | 12,691 | 5,378 | 18,069 |
| **Files** | 47 | 28 | 75 |

**Documentation Ratio**: 0.56 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            47           1773           1336           9582
-------------------------------------------------------------------------------
SUM:                            47           1773           1336           9582
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `adapters/claude_code.rs` | 255 | 126 | 129 | 50.6% | ✅ |
| `adapters/codex.rs` | 466 | 285 | 181 | 38.8% | ✅ (infra) |
| `adapters/cursor.rs` | 308 | 157 | 151 | 49.0% | ✅ |
| `adapters/mod.rs` | 120 | 120 | 0 | 0.0% | ✅ |
| `commands/analyze/mod.rs` | 182 | 28 | 154 | 84.6% | ✅ |
| `commands/analyze/sections.rs` | 267 | 267 | 0 | 0.0% | ✅ (infra) |
| `commands/astq.rs` | 82 | 82 | 0 | 0.0% | ✅ |
| `commands/config.rs` | 134 | 57 | 77 | 57.5% | ✅ |
| `commands/git.rs` | 49 | 9 | 40 | 81.6% | ✅ |
| `commands/hook.rs` | 236 | 108 | 128 | 54.2% | ✅ |
| `commands/init.rs` | 167 | 72 | 95 | 56.9% | ✅ |
| `commands/meta.rs` | 232 | 147 | 85 | 36.6% | ✅ |
| `commands/mod.rs` | 25 | 25 | 0 | 0.0% | ✅ |
| `commands/reflect.rs` | 82 | 82 | 0 | 0.0% | ✅ |
| `commands/workflow.rs` | 1,329 | 586 | 743 | 55.9% | ✅ (infra) |
| `commands/wrapped.rs` | 131 | 68 | 63 | 48.1% | ✅ |
| `config.rs` | 147 | 102 | 45 | 30.6% | ✅ |
| `embedded.rs` | 82 | 82 | 0 | 0.0% | ✅ |
| `engine/mod.rs` | 751 | 153 | 598 | 79.6% | ✅ |
| `engine/template.rs` | 512 | 117 | 395 | 77.1% | ✅ |
| `guardrails/mod.rs` | 5 | 5 | 0 | 0.0% | ✅ |
| `guardrails/parser.rs` | 71 | 23 | 48 | 67.6% | ✅ |
| `guardrails/types.rs` | 171 | 98 | 73 | 42.7% | ✅ |
| `main.rs` | 197 | 13 | 184 | 93.4% | ✅ |
| `metamodes/mod.rs` | 236 | 111 | 125 | 53.0% | ✅ |
| `metrics/aggregation.rs` | 204 | 144 | 60 | 29.4% | ✅ |
| `metrics/graph.rs` | 370 | 222 | 148 | 40.0% | ✅ (infra) |
| `metrics/hooks.rs` | 326 | 216 | 110 | 33.7% | ✅ (infra) |
| `metrics/mod.rs` | 333 | 115 | 218 | 65.5% | ✅ |
| `metrics/states.rs` | 137 | 33 | 104 | 75.9% | ✅ |
| `metrics/transcript.rs` | 257 | 100 | 157 | 61.1% | ✅ |
| `rules/evaluator.rs` | 998 | 121 | 877 | 87.9% | ✅ |
| `rules/interrupt.rs` | 175 | 32 | 143 | 81.7% | ✅ |
| `rules/mod.rs` | 7 | 7 | 0 | 0.0% | ✅ |
| `rules/types.rs` | 297 | 72 | 225 | 75.8% | ✅ |
| `storage/mod.rs` | 695 | 297 | 398 | 57.3% | ✅ (infra) |
| `test_helpers.rs` | 1,009 | 723 | 286 | 28.3% | ✅ (infra) |
| `theme.rs` | 128 | 98 | 30 | 23.4% | ✅ |
| `tui/app.rs` | 402 | 165 | 237 | 59.0% | ✅ |
| `tui/mod.rs` | 66 | 66 | 0 | 0.0% | ✅ |
| `tui/tabs/events.rs` | 100 | 81 | 19 | 19.0% | ✅ |
| `tui/tabs/files.rs` | 81 | 63 | 18 | 22.2% | ✅ |
| `tui/tabs/mod.rs` | 9 | 9 | 0 | 0.0% | ✅ |
| `tui/tabs/overview.rs` | 88 | 74 | 14 | 15.9% | ✅ |
| `tui/tabs/phases.rs` | 135 | 103 | 32 | 23.7% | ✅ |
| `tui/ui.rs` | 317 | 158 | 159 | 50.2% | ✅ |
| `tui/utils.rs` | 320 | 187 | 133 | 41.6% | ✅ |

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 137 |
| `CODE_MAP.md` | 189 |
| `commands/hegel.md` | 23 |
| `COVERAGE_REPORT.md` | 142 |
| `DEP_REVIEW.md` | 678 |
| `docs/astq_patterns/README.md` | 127 |
| `guides/ARCHITECTURE_WRITING.md` | 258 |
| `guides/CLAUDE_CUSTOMIZATION.md` | 312 |
| `guides/CODE_MAP_WRITING.md` | 114 |
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
| `HEGEL_CLAUDE.md` | 560 |
| `LEXICON.md` | 84 |
| `LOC_REPORT.md` | 144 |
| `README.md` | 392 |
| `ROADMAP.md` | 181 |
| `TESTING.md` | 41 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.56 | ✅ Excellent |
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
