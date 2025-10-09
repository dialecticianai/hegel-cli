# Lines of Code Report

**Last Updated**: 2025-10-08 23:54
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 1,163 | 1,569 | 2,732 |
| **Comments** | 90 | - | 90 |
| **Blank Lines** | 218 | - | 218 |
| **Total Lines** | 1,471 | 1,569 | 3,040 |
| **Files** | 5 | 12 | 17 |

**Documentation Ratio**: 1.35 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                             5            218             90           1163
-------------------------------------------------------------------------------
SUM:                             5            218             90           1163
-------------------------------------------------------------------------------
```

---

## Documentation Files

| File | Lines |
|------|-------|
| `COVERAGE_REPORT.md` | 66 |
| `guides/CODE_MAP_WRITING.md` | 254 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/ORIENTATION_WRITING.md` | 259 |
| `guides/PLAN_WRITING.md` | 144 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `LEXICON.md` | 91 |
| `LOC_REPORT.md` | 74 |
| `README.md` | 113 |
| `ROADMAP.md` | 135 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 1.35 | ✅ Excellent |
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
