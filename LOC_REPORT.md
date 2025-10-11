# Lines of Code Report

**Last Updated**: 2025-10-11 01:59
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 6,603 | 4,299 | 10,902 |
| **Comments** | 863 | - | 863 |
| **Blank Lines** | 1,164 | - | 1,164 |
| **Total Lines** | 8,630 | 4,299 | 12,929 |
| **Files** | 29 | 17 | 46 |

**Documentation Ratio**: 0.65 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            29           1164            863           6603
-------------------------------------------------------------------------------
SUM:                            29           1164            863           6603
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `commands/analyze/mod.rs` | 182 | 28 | 154 | 84.6% | ✅ |
| `commands/analyze/sections.rs` | 253 | 253 | 0 | 0.0% | ⚠️ Large |
| `commands/hook.rs` | 230 | 110 | 120 | 52.2% | ✅ |
| `commands/mod.rs` | 8 | 8 | 0 | 0.0% | ✅ |
| `commands/workflow.rs` | 595 | 212 | 383 | 64.4% | ⚠️ Large |
| `engine/mod.rs` | 722 | 130 | 592 | 82.0% | ✅ |
| `engine/template.rs` | 351 | 87 | 264 | 75.2% | ✅ |
| `main.rs` | 115 | 7 | 108 | 93.9% | ✅ |
| `metrics/aggregation.rs` | 204 | 144 | 60 | 29.4% | ✅ |
| `metrics/graph.rs` | 370 | 222 | 148 | 40.0% | ⚠️ Large |
| `metrics/hooks.rs` | 286 | 176 | 110 | 38.5% | ✅ |
| `metrics/mod.rs` | 333 | 115 | 218 | 65.5% | ✅ |
| `metrics/states.rs` | 137 | 33 | 104 | 75.9% | ✅ |
| `metrics/transcript.rs` | 257 | 100 | 157 | 61.1% | ✅ |
| `rules/evaluator.rs` | 1,491 | 115 | 1,376 | 92.3% | ✅ |
| `rules/interrupt.rs` | 175 | 32 | 143 | 81.7% | ✅ |
| `rules/mod.rs` | 7 | 7 | 0 | 0.0% | ✅ |
| `rules/types.rs` | 297 | 72 | 225 | 75.8% | ✅ |
| `storage/mod.rs` | 472 | 184 | 288 | 61.0% | ✅ |
| `test_helpers.rs` | 786 | 530 | 256 | 32.6% | ✅ (infra) |
| `tui/app.rs` | 369 | 165 | 204 | 55.3% | ✅ |
| `tui/mod.rs` | 66 | 66 | 0 | 0.0% | ✅ |
| `tui/tabs/events.rs` | 100 | 81 | 19 | 19.0% | ✅ |
| `tui/tabs/files.rs` | 81 | 63 | 18 | 22.2% | ✅ |
| `tui/tabs/mod.rs` | 9 | 9 | 0 | 0.0% | ✅ |
| `tui/tabs/overview.rs` | 88 | 74 | 14 | 15.9% | ✅ |
| `tui/tabs/phases.rs` | 135 | 103 | 32 | 23.7% | ✅ |
| `tui/ui.rs` | 191 | 158 | 33 | 17.3% | ✅ |
| `tui/utils.rs` | 320 | 187 | 133 | 41.6% | ✅ |

**⚠️ Warning:** 3 file(s) over 200 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 120 |
| `CODE_MAP.md` | 159 |
| `COVERAGE_REPORT.md` | 108 |
| `DEP_REVIEW.md` | 678 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 207 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `LEXICON.md` | 84 |
| `LOC_REPORT.md` | 117 |
| `PLAN.md` | 1,348 |
| `README.md` | 236 |
| `ROADMAP.md` | 95 |
| `SPEC.md` | 474 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.65 | ✅ Excellent |
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
