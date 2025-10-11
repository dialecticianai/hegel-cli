# Lines of Code Report

**Last Updated**: 2025-10-11 01:10
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 6,120 | 4,408 | 10,528 |
| **Comments** | 815 | - | 815 |
| **Blank Lines** | 1,067 | - | 1,067 |
| **Total Lines** | 8,002 | 4,408 | 12,410 |
| **Files** | 22 | 18 | 40 |

**Documentation Ratio**: 0.72 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            22           1067            815           6120
-------------------------------------------------------------------------------
SUM:                            22           1067            815           6120
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `commands/analyze.rs` | 404 | 250 | 154 | 38.1% | ⚠️ Large |
| `commands/hook.rs` | 230 | 110 | 120 | 52.2% | ✅ |
| `commands/mod.rs` | 8 | 8 | 0 | 0.0% | ✅ |
| `commands/workflow.rs` | 439 | 170 | 269 | 61.3% | ✅ |
| `engine/mod.rs` | 452 | 97 | 355 | 78.5% | ✅ |
| `engine/template.rs` | 351 | 87 | 264 | 75.2% | ✅ |
| `main.rs` | 110 | 7 | 103 | 93.6% | ✅ |
| `metrics/graph.rs` | 370 | 222 | 148 | 40.0% | ⚠️ Large |
| `metrics/hooks.rs` | 286 | 176 | 110 | 38.5% | ✅ |
| `metrics/mod.rs` | 465 | 247 | 218 | 46.9% | ⚠️ Large |
| `metrics/states.rs` | 137 | 33 | 104 | 75.9% | ✅ |
| `metrics/transcript.rs` | 257 | 100 | 157 | 61.1% | ✅ |
| `rules/evaluator.rs` | 1,491 | 115 | 1,376 | 92.3% | ✅ |
| `rules/interrupt.rs` | 175 | 32 | 143 | 81.7% | ✅ |
| `rules/mod.rs` | 7 | 7 | 0 | 0.0% | ✅ |
| `rules/types.rs` | 297 | 72 | 225 | 75.8% | ✅ |
| `storage/mod.rs` | 472 | 184 | 288 | 61.0% | ✅ |
| `test_helpers.rs` | 750 | 494 | 256 | 34.1% | ✅ (infra) |
| `tui/app.rs` | 369 | 165 | 204 | 55.3% | ✅ |
| `tui/mod.rs` | 65 | 65 | 0 | 0.0% | ✅ |
| `tui/ui.rs` | 547 | 449 | 98 | 17.9% | ⚠️ Large |
| `tui/utils.rs` | 320 | 187 | 133 | 41.6% | ✅ |

**⚠️ Warning:** 4 file(s) over 200 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 117 |
| `CODE_MAP.md` | 143 |
| `COVERAGE_REPORT.md` | 96 |
| `DEP_REVIEW.md` | 678 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 142 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `HANDOFF.md` | 213 |
| `LEXICON.md` | 84 |
| `LOC_REPORT.md` | 109 |
| `PLAN.md` | 1,348 |
| `README.md` | 236 |
| `ROADMAP.md` | 95 |
| `SPEC.md` | 474 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.72 | ✅ Excellent |
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
