# Lines of Code Report

**Last Updated**: 2025-10-09 00:03
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 1,497 | 1,569 | 3,066 |
| **Comments** | 139 | - | 139 |
| **Blank Lines** | 310 | - | 310 |
| **Total Lines** | 1,946 | 1,569 | 3,515 |
| **Files** | 5 | 12 | 17 |

**Documentation Ratio**: 1.05 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                             5            310            139           1497
-------------------------------------------------------------------------------
SUM:                             5            310            139           1497
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
| Docs/Code Ratio | ≥0.3 | 1.05 | ✅ Excellent |
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
