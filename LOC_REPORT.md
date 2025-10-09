# Lines of Code Report

**Last Updated**: 2025-10-09 19:39
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 2,241 | 2,206 | 4,447 |
| **Comments** | 351 | - | 351 |
| **Blank Lines** | 411 | - | 411 |
| **Total Lines** | 3,003 | 2,206 | 5,209 |
| **Files** | 10 | 15 | 25 |

**Documentation Ratio**: 0.98 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            10            411            351           2241
-------------------------------------------------------------------------------
SUM:                            10            411            351           2241
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `commands/analyze.rs` | 146 | 146 | 0 | 0.0% | ✅ |
| `commands/hook.rs` | 127 | 72 | 55 | 43.3% | ✅ |
| `commands/mod.rs` | 8 | 8 | 0 | 0.0% | ✅ |
| `commands/workflow.rs` | 400 | 158 | 242 | 60.5% | ✅ |
| `engine/mod.rs` | 452 | 97 | 355 | 78.5% | ✅ |
| `engine/template.rs` | 351 | 87 | 264 | 75.2% | ✅ |
| `main.rs` | 103 | 5 | 98 | 95.1% | ✅ |
| `metrics/mod.rs` | 553 | 343 | 210 | 38.0% | ⚠️ Large |
| `storage/mod.rs` | 461 | 173 | 288 | 62.5% | ✅ |
| `test_helpers.rs` | 402 | 402 | 0 | 0.0% | ✅ (infra) |

**⚠️ Warning:** 1 file(s) over 200 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 282 |
| `CODE_MAP.md` | 85 |
| `COVERAGE_REPORT.md` | 74 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 142 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `LEXICON.md` | 81 |
| `LOC_REPORT.md` | 96 |
| `PLAN.md` | 436 |
| `README.md` | 203 |
| `ROADMAP.md` | 134 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.98 | ✅ Excellent |
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
