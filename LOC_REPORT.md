# Lines of Code Report

**Last Updated**: 2025-10-09 19:19
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 2,193 | 2,197 | 4,390 |
| **Comments** | 299 | - | 299 |
| **Blank Lines** | 410 | - | 410 |
| **Total Lines** | 2,902 | 2,197 | 5,099 |
| **Files** | 10 | 15 | 25 |

**Documentation Ratio**: 1.00 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            10            410            299           2193
-------------------------------------------------------------------------------
SUM:                            10            410            299           2193
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
| `metrics/mod.rs` | 520 | 320 | 200 | 38.5% | ⚠️ Large |
| `storage/mod.rs` | 450 | 162 | 288 | 64.0% | ✅ |
| `test_helpers.rs` | 345 | 345 | 0 | 0.0% | ✅ (infra) |

**⚠️ Warning:** 1 file(s) over 200 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 282 |
| `CODE_MAP.md` | 76 |
| `COVERAGE_REPORT.md` | 74 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 142 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `LEXICON.md` | 81 |
| `LOC_REPORT.md` | 95 |
| `PLAN.md` | 436 |
| `README.md` | 190 |
| `ROADMAP.md` | 148 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 1.00 | ✅ Excellent |
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
