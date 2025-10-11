# Lines of Code Report

**Last Updated**: 2025-10-11 01:05
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 5,590 | 4,193 | 9,783 |
| **Comments** | 796 | - | 796 |
| **Blank Lines** | 967 | - | 967 |
| **Total Lines** | 7,353 | 4,193 | 11,546 |
| **Files** | 21 | 17 | 38 |

**Documentation Ratio**: 0.75 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            21            967            796           5590
-------------------------------------------------------------------------------
SUM:                            21            967            796           5590
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
| `rules/evaluator.rs` | 1,019 | 102 | 917 | 90.0% | ✅ |
| `rules/mod.rs` | 5 | 5 | 0 | 0.0% | ✅ |
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
| `COVERAGE_REPORT.md` | 94 |
| `DEP_REVIEW.md` | 678 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 142 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
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
| Docs/Code Ratio | ≥0.3 | 0.75 | ✅ Excellent |
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
