# Lines of Code Report

**Last Updated**: 2025-10-09 01:34
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 1,538 | 1,561 | 3,099 |
| **Comments** | 148 | - | 148 |
| **Blank Lines** | 318 | - | 318 |
| **Total Lines** | 2,004 | 1,561 | 3,565 |
| **Files** | 5 | 13 | 18 |

**Documentation Ratio**: 1.01 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                             5            318            148           1538
-------------------------------------------------------------------------------
SUM:                             5            318            148           1538
-------------------------------------------------------------------------------
```

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 275 |
| `COVERAGE_REPORT.md` | 66 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 142 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `LEXICON.md` | 91 |
| `LOC_REPORT.md` | 74 |
| `README.md` | 119 |
| `ROADMAP.md` | 121 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 1.01 | ✅ Excellent |
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
