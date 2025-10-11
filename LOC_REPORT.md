# Lines of Code Report

**Last Updated**: 2025-10-10 23:36
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 4,516 | 4,067 | 8,583 |
| **Comments** | 753 | - | 753 |
| **Blank Lines** | 839 | - | 839 |
| **Total Lines** | 6,108 | 4,067 | 10,175 |
| **Files** | 18 | 16 | 34 |

**Documentation Ratio**: 0.90 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            18            839            753           4516
-------------------------------------------------------------------------------
SUM:                            18            839            753           4516
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `commands/analyze.rs` | 404 | 250 | 154 | 38.1% | ⚠️ Large |
| `commands/hook.rs` | 230 | 107 | 123 | 53.5% | ✅ |
| `commands/mod.rs` | 8 | 8 | 0 | 0.0% | ✅ |
| `commands/workflow.rs` | 400 | 158 | 242 | 60.5% | ✅ |
| `engine/mod.rs` | 452 | 97 | 355 | 78.5% | ✅ |
| `engine/template.rs` | 351 | 87 | 264 | 75.2% | ✅ |
| `main.rs` | 109 | 6 | 103 | 94.5% | ✅ |
| `metrics/graph.rs` | 370 | 222 | 148 | 40.0% | ⚠️ Large |
| `metrics/hooks.rs` | 286 | 176 | 110 | 38.5% | ✅ |
| `metrics/mod.rs` | 458 | 245 | 213 | 46.5% | ⚠️ Large |
| `metrics/states.rs` | 137 | 33 | 104 | 75.9% | ✅ |
| `metrics/transcript.rs` | 257 | 100 | 157 | 61.1% | ✅ |
| `storage/mod.rs` | 596 | 220 | 376 | 63.1% | ⚠️ Large |
| `test_helpers.rs` | 749 | 493 | 256 | 34.2% | ✅ (infra) |
| `tui/app.rs` | 369 | 165 | 204 | 55.3% | ✅ |
| `tui/mod.rs` | 65 | 65 | 0 | 0.0% | ✅ |
| `tui/ui.rs` | 547 | 449 | 98 | 17.9% | ⚠️ Large |
| `tui/utils.rs` | 320 | 187 | 133 | 41.6% | ✅ |

**⚠️ Warning:** 5 file(s) over 200 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 117 |
| `CODE_MAP.md` | 143 |
| `COVERAGE_REPORT.md` | 90 |
| `DEP_REVIEW.md` | 678 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 142 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `LEXICON.md` | 84 |
| `LOC_REPORT.md` | 105 |
| `PLAN.md` | 1,683 |
| `README.md` | 233 |
| `ROADMAP.md` | 119 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.90 | ✅ Excellent |
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
